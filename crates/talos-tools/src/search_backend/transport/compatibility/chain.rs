//! Dormant traditional fixed-source subset; no startup probe or default activation.
//! Order/retry behavior derives from rust-websearch 0.1.2, VCS b30180a0.
//! MIT notice: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Talos owns preparation, terminal error policy, admitted results and one shared budget.

use std::time::Duration;

use super::{
    CompatibilityError, CompatibilityProtocol, PreparedCompatibilitySearch, SearchTransport,
    ToolNetworkManifest, TransportError,
    admission::{AdmissionError, AdmittedCompatibilityResults},
    bing_parser::BingParseError,
    ddg_parser::DdgParseError,
    headers::CompatibilityHeaders,
    probe::Reachability,
};
use thiserror::Error;
use tokio::time::{Instant, sleep, timeout_at};

pub(super) const ORDER: [CompatibilityProtocol; 4] = [
    CompatibilityProtocol::Lite,
    CompatibilityProtocol::InstantAnswer,
    CompatibilityProtocol::Html,
    CompatibilityProtocol::Bing,
];

// The legacy compatibility dependency applies fifteen seconds to each request and retries one
// reqwest Timeout/Network failure after 500ms for Lite, HTML and Bing. Instant Answer does not
// retry. Preserve that same-destination recovery window before disclosing the query to a later
// compatibility destination.
const LEGACY_COMPATIBILITY_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const LEGACY_COMPATIBILITY_RETRY_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(in crate::search_backend) enum CompatibilityChainError {
    #[error("search_compatibility_chain_terminal")]
    Terminal {
        protocol: CompatibilityProtocol,
        error: CompatibilityError,
    },
    #[error("search_compatibility_chain_exhausted")]
    Exhausted([CompatibilityError; 4]),
}

/// Same immutable sources survive future permission admission; not an approval seal.
pub(in crate::search_backend) struct PreparedCompatibilityChain {
    sources: [PreparedCompatibilitySearch; 4],
}

impl std::fmt::Debug for PreparedCompatibilityChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedCompatibilityChain(<redacted>)")
    }
}

#[derive(Debug)]
enum AttemptFailure {
    SourceTimeout,
    Compatibility(CompatibilityError),
}

impl PreparedCompatibilityChain {
    pub(in crate::search_backend) fn prepare(
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
    ) -> Result<Self, CompatibilityError> {
        Self::prepare_with(
            manifest,
            query,
            max_results,
            rust_websearch::anti_detection::random_user_agent,
        )
    }

    fn prepare_with(
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
        select: impl Fn() -> &'static str + std::panic::RefUnwindSafe,
    ) -> Result<Self, CompatibilityError> {
        if !(1..=20).contains(&max_results) {
            return Err(TransportError::InvalidRequest.into());
        }

        // Validate all legacy query-bearing attempts before drawing any headers. Retry objects are
        // frozen now, before approval, rather than cloned/reconstructed after failure.
        let [lite, instant, html, bing] = [
            ORDER[0].prepare_payload(manifest, query)?,
            ORDER[1].prepare_payload(manifest, query)?,
            ORDER[2].prepare_payload(manifest, query)?,
            ORDER[3].prepare_payload(manifest, query)?,
        ];
        let retry_for = |protocol: CompatibilityProtocol| {
            if protocol.retries_network_failure() {
                Ok::<_, CompatibilityError>(Some(protocol.prepare_payload(manifest, query)?))
            } else {
                Ok(None)
            }
        };
        let lite_retry = retry_for(ORDER[0])?;
        let instant_retry = retry_for(ORDER[1])?;
        let html_retry = retry_for(ORDER[2])?;
        let bing_retry = retry_for(ORDER[3])?;

        let freeze = |protocol,
                      mut request: super::PreparedTransportRequest,
                      mut retry: Option<super::PreparedTransportRequest>| {
            let headers = CompatibilityHeaders::capture(protocol, &select)?;
            request.headers = Some(headers);
            if let Some(retry) = retry.as_mut() {
                retry.headers = Some(headers);
            }
            Ok::<_, CompatibilityError>(PreparedCompatibilitySearch {
                request,
                retry,
                protocol,
                query: query.to_owned(),
                max_results,
            })
        };
        Ok(Self {
            sources: [
                freeze(ORDER[0], lite, lite_retry)?,
                freeze(ORDER[1], instant, instant_retry)?,
                freeze(ORDER[2], html, html_retry)?,
                freeze(ORDER[3], bing, bing_retry)?,
            ],
        })
    }

    pub(in crate::search_backend) async fn execute(
        self,
        transport: &SearchTransport,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityChainError> {
        self.execute_with_reachability(
            transport,
            Reachability {
                duckduckgo: true,
                bing: true,
            },
        )
        .await
    }

    pub(super) async fn execute_with_reachability(
        self,
        transport: &SearchTransport,
        reachability: Reachability,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityChainError> {
        self.execute_with_policy_and_reachability(
            transport,
            reachability,
            LEGACY_COMPATIBILITY_REQUEST_TIMEOUT,
            LEGACY_COMPATIBILITY_RETRY_DELAY,
        )
        .await
    }

    #[cfg(test)]
    async fn execute_with_source_timeout(
        self,
        transport: &SearchTransport,
        source_timeout: Duration,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityChainError> {
        self.execute_with_policy(transport, source_timeout, LEGACY_COMPATIBILITY_RETRY_DELAY)
            .await
    }

    #[cfg(test)]
    async fn execute_with_policy(
        self,
        transport: &SearchTransport,
        source_timeout: Duration,
        retry_delay: Duration,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityChainError> {
        self.execute_with_policy_and_reachability(
            transport,
            Reachability {
                duckduckgo: true,
                bing: true,
            },
            source_timeout,
            retry_delay,
        )
        .await
    }

    async fn execute_with_policy_and_reachability(
        self,
        transport: &SearchTransport,
        reachability: Reachability,
        source_timeout: Duration,
        retry_delay: Duration,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityChainError> {
        // Sentinels cannot escape: exhaustion is reached only after all four slots are replaced
        // with an observed typed result or an explicit query-free probe Unreachable decision.
        let mut failures = [CompatibilityError::Transport(TransportError::InvalidRequest); 4];
        for (index, source) in self.sources.into_iter().enumerate() {
            let PreparedCompatibilitySearch {
                request,
                retry,
                protocol,
                query,
                max_results,
            } = source;
            let terminal = |error| CompatibilityChainError::Terminal { protocol, error };
            transport
                .budget
                .check_live()
                .map_err(|e| terminal(e.into()))?;

            let source_reachable = match protocol {
                CompatibilityProtocol::Lite
                | CompatibilityProtocol::InstantAnswer
                | CompatibilityProtocol::Html => reachability.duckduckgo,
                CompatibilityProtocol::Bing => reachability.bing,
            };
            if !source_reachable {
                failures[index] = CompatibilityError::Unreachable;
                continue;
            }

            let first = execute_attempt(
                protocol,
                &query,
                max_results,
                request,
                transport,
                source_timeout,
            )
            .await;

            let final_outcome = match (first, retry) {
                (Ok(results), _) => return Ok(results),
                (Err(AttemptFailure::SourceTimeout), Some(retry)) => {
                    wait_before_retry(transport, retry_delay)
                        .await
                        .map_err(terminal)?;
                    execute_attempt(
                        protocol,
                        &query,
                        max_results,
                        retry,
                        transport,
                        source_timeout,
                    )
                    .await
                }
                (Err(AttemptFailure::Compatibility(error)), Some(retry))
                    if may_retry_network(error) =>
                {
                    wait_before_retry(transport, retry_delay)
                        .await
                        .map_err(terminal)?;
                    execute_attempt(
                        protocol,
                        &query,
                        max_results,
                        retry,
                        transport,
                        source_timeout,
                    )
                    .await
                }
                (other, _) => other,
            };

            transport
                .budget
                .check_live()
                .map_err(|e| terminal(e.into()))?;
            match final_outcome {
                Ok(results) => return Ok(results),
                Err(AttemptFailure::SourceTimeout) => {
                    // This is a source-local timeout, not the enclosing execution deadline.
                    failures[index] = CompatibilityError::Transport(TransportError::Deadline);
                }
                Err(AttemptFailure::Compatibility(error)) if may_continue(error) => {
                    failures[index] = error;
                }
                Err(AttemptFailure::Compatibility(error)) => return Err(terminal(error)),
            }
        }
        transport
            .budget
            .check_live()
            .map_err(|e| CompatibilityChainError::Terminal {
                protocol: CompatibilityProtocol::Bing,
                error: e.into(),
            })?;
        Err(CompatibilityChainError::Exhausted(failures))
    }
}

async fn execute_attempt(
    protocol: CompatibilityProtocol,
    query: &str,
    max_results: u32,
    request: super::PreparedTransportRequest,
    transport: &SearchTransport,
    source_timeout: Duration,
) -> Result<AdmittedCompatibilityResults, AttemptFailure> {
    transport
        .budget
        .check_live()
        .map_err(|error| AttemptFailure::Compatibility(error.into()))?;
    let now = Instant::now();
    let source_deadline = std::cmp::min(now + source_timeout, transport.budget.deadline);
    match timeout_at(
        source_deadline,
        super::execute_request(protocol, query, max_results, request, transport),
    )
    .await
    {
        Ok(result) => result.map_err(AttemptFailure::Compatibility),
        Err(_) => match transport.budget.check_live() {
            Ok(()) => Err(AttemptFailure::SourceTimeout),
            Err(error) => Err(AttemptFailure::Compatibility(error.into())),
        },
    }
}

async fn wait_before_retry(
    transport: &SearchTransport,
    delay: Duration,
) -> Result<(), CompatibilityError> {
    transport.budget.check_live()?;
    timeout_at(transport.budget.deadline, sleep(delay))
        .await
        .map_err(|_| CompatibilityError::Transport(TransportError::Deadline))?;
    transport.budget.check_live()?;
    Ok(())
}

fn may_retry_network(error: CompatibilityError) -> bool {
    matches!(
        error,
        CompatibilityError::Transport(
            TransportError::Dns | TransportError::ConnectionRefused | TransportError::Network
        )
    )
}

fn may_continue(error: CompatibilityError) -> bool {
    match error {
        CompatibilityError::Unreachable => true,
        CompatibilityError::Transport(error) => match error {
            TransportError::InvalidRequest
            | TransportError::Unauthorized
            | TransportError::UnsupportedProxy
            | TransportError::Address
            | TransportError::Redirect
            | TransportError::RequestLimit
            | TransportError::ResponseLimit
            | TransportError::ResponseEncoding
            | TransportError::Deadline
            | TransportError::Request => false,
            // After the protocol-specific retry budget is exhausted, legacy availability/network
            // failures permit the next already-prepared source under the same authority.
            TransportError::Dns | TransportError::ConnectionRefused | TransportError::Network => {
                true
            }
        },
        CompatibilityError::Ddg(error) => match error {
            DdgParseError::Empty | DdgParseError::Status | DdgParseError::Challenge => true,
            DdgParseError::Limit | DdgParseError::Encoding | DdgParseError::Parse => false,
        },
        CompatibilityError::Bing(error) => match error {
            BingParseError::Empty
            | BingParseError::Status
            | BingParseError::RateLimited
            | BingParseError::Challenge => true,
            BingParseError::Limit | BingParseError::Encoding | BingParseError::Parse => false,
        },
        CompatibilityError::Admission(error) => match error {
            AdmissionError::Empty | AdmissionError::Invalid => true,
            AdmissionError::Limit | AdmissionError::Identity | AdmissionError::Parse => false,
        },
    }
}

#[cfg(test)]
mod legacy_parity_tests;
#[cfg(test)]
mod reachability_tests;
#[cfg(test)]
pub(super) mod tests;
#[cfg(test)]
mod timeout_tests;
