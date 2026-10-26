//! Dormant traditional fixed-source subset; no probe, retry or default activation.
//! Order derives from rust-websearch0.1.2 searcher.rs, VCS b30180a0.
//! MIT notice: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Talos owns preparation, terminal error policy, admitted results and one shared budget.

use super::{
    CompatibilityError, CompatibilityProtocol, PreparedCompatibilitySearch, SearchTransport,
    ToolNetworkManifest, TransportError,
    admission::{AdmissionError, AdmittedCompatibilityResults},
    bing_parser::BingParseError,
    ddg_parser::DdgParseError,
    headers::CompatibilityHeaders,
};
use thiserror::Error;

pub(super) const ORDER: [CompatibilityProtocol; 4] = [
    CompatibilityProtocol::Lite,
    CompatibilityProtocol::InstantAnswer,
    CompatibilityProtocol::Html,
    CompatibilityProtocol::Bing,
];

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
        let [lite, instant, html, bing] = [
            ORDER[0].prepare_payload(manifest, query)?,
            ORDER[1].prepare_payload(manifest, query)?,
            ORDER[2].prepare_payload(manifest, query)?,
            ORDER[3].prepare_payload(manifest, query)?,
        ];
        let freeze = |protocol, mut request: super::PreparedTransportRequest| {
            request.headers = Some(CompatibilityHeaders::capture(protocol, &select)?);
            Ok::<_, CompatibilityError>(PreparedCompatibilitySearch {
                request,
                protocol,
                query: query.to_owned(),
                max_results,
            })
        };
        Ok(Self {
            sources: [
                freeze(ORDER[0], lite)?,
                freeze(ORDER[1], instant)?,
                freeze(ORDER[2], html)?,
                freeze(ORDER[3], bing)?,
            ],
        })
    }

    pub(in crate::search_backend) async fn execute(
        self,
        transport: &SearchTransport,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityChainError> {
        // Sentinels cannot escape: exhaustion is reached only after all four slots
        // are replaced with observed typed availability or semantic failures in the fixed order.
        let mut failures = [CompatibilityError::Transport(TransportError::InvalidRequest); 4];
        for (index, source) in self.sources.into_iter().enumerate() {
            let protocol = source.protocol;
            let terminal = |error| CompatibilityChainError::Terminal { protocol, error };
            transport
                .budget
                .check_live()
                .map_err(|e| terminal(e.into()))?;
            let outcome = source.execute(transport).await;
            transport
                .budget
                .check_live()
                .map_err(|e| terminal(e.into()))?;
            match outcome {
                Ok(results) => return Ok(results),
                Err(error) if may_continue(error) => failures[index] = error,
                Err(error) => return Err(terminal(error)),
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

fn may_continue(error: CompatibilityError) -> bool {
    match error {
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
            // Only typed resolver failure or connection refusal permits the next already
            // prepared source, under the same authority and enclosing budget.
            TransportError::Dns | TransportError::ConnectionRefused => true,
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
pub(super) mod tests;
