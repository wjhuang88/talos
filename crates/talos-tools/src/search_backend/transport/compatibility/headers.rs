//! Dormant fixed compatibility headers and prepared credential headers.
//!
//! Reuses rust-websearch 0.1.2's public anti_detection::random_user_agent helper.
//! Fixed Bing language derives from VCS b30180a0, crates/websearch/src/bing.rs;
//! client/UA provenance: src/anti_detection/{mod,user_agent}.rs in that package.
//! MIT notice: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Talos freezes once before admission, bounds/validates values and closes errors.
//! Credential-bearing header values are prepared locally, marked sensitive, never exposed through
//! Debug, and remain dormant until an explicitly selected provider owns the prepared request.

use super::{CompatibilityProtocol, PreparedTransportRequest, TransportError};

const MAX_AUTHORIZATION_BYTES: usize = 4096;
const BEARER_PREFIX: &str = "Bearer ";

#[derive(Clone)]
enum PreparedHeaderKind {
    Compatibility {
        user_agent: &'static str,
        bing: bool,
    },
    Bearer(reqwest::header::HeaderValue),
}

#[derive(Clone)]
pub(in crate::search_backend::transport) struct CompatibilityHeaders {
    kind: PreparedHeaderKind,
}

impl std::fmt::Debug for CompatibilityHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CompatibilityHeaders(<redacted>)")
    }
}

fn validate_user_agent(value: &str) -> Result<(), TransportError> {
    if value.is_empty()
        || value.len() > 512
        || !value.bytes().all(|b| b.is_ascii_graphic() || b == b' ')
        || value.trim().is_empty()
    {
        return Err(TransportError::InvalidRequest);
    }
    reqwest::header::HeaderValue::from_str(value)
        .map(|_| ())
        .map_err(|_| TransportError::InvalidRequest)
}

fn prepare_bearer(value: &str) -> Result<reqwest::header::HeaderValue, TransportError> {
    let length = BEARER_PREFIX
        .len()
        .checked_add(value.len())
        .filter(|length| *length <= MAX_AUTHORIZATION_BYTES)
        .ok_or(TransportError::InvalidRequest)?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(TransportError::InvalidRequest);
    }
    debug_assert!(length <= MAX_AUTHORIZATION_BYTES);

    let mut header = reqwest::header::HeaderValue::from_str(&format!("{BEARER_PREFIX}{value}"))
        .map_err(|_| TransportError::InvalidRequest)?;
    header.set_sensitive(true);
    Ok(header)
}

impl CompatibilityHeaders {
    // Only compatibility preparation can construct this variant. The sole production suppliers
    // are the pinned helper; a private supplier enables deterministic error/order fixtures.
    pub(super) fn capture(
        protocol: CompatibilityProtocol,
        select: impl FnOnce() -> &'static str + std::panic::UnwindSafe,
    ) -> Result<Self, TransportError> {
        let user_agent =
            std::panic::catch_unwind(select).map_err(|_| TransportError::InvalidRequest)?;
        validate_user_agent(user_agent)?;
        Ok(Self {
            kind: PreparedHeaderKind::Compatibility {
                user_agent,
                bing: matches!(protocol, CompatibilityProtocol::Bing),
            },
        })
    }

    fn bearer(value: &str) -> Result<Self, TransportError> {
        Ok(Self {
            kind: PreparedHeaderKind::Bearer(prepare_bearer(value)?),
        })
    }

    pub(in crate::search_backend::transport) fn apply(
        &self,
        request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        match &self.kind {
            PreparedHeaderKind::Compatibility { user_agent, bing } => {
                let request = request.header(reqwest::header::USER_AGENT, *user_agent);
                if *bing {
                    request.header(reqwest::header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8")
                } else {
                    request
                }
            }
            PreparedHeaderKind::Bearer(value) => {
                request.header(reqwest::header::AUTHORIZATION, value.clone())
            }
        }
    }

    #[cfg(test)]
    pub(in crate::search_backend::transport) fn user_agent(&self) -> Option<&'static str> {
        match &self.kind {
            PreparedHeaderKind::Compatibility { user_agent, .. } => Some(*user_agent),
            PreparedHeaderKind::Bearer(_) => None,
        }
    }
}

impl PreparedTransportRequest {
    /// Freeze one explicit bearer credential into this already-authorized request shape.
    ///
    /// This does not discover credentials, grant network authority, or select a provider. The
    /// caller must already own explicit provider intent and retain this prepared request across
    /// the existing approval boundary. Credentialed provider scopes must reject redirects so the
    /// authorization value is never replayed to a redirected destination.
    pub(in crate::search_backend) fn with_bearer_authorization(
        mut self,
        token: &str,
    ) -> Result<Self, TransportError> {
        self.headers = Some(CompatibilityHeaders::bearer(token)?);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_accepts_visible_ascii_with_spaces_at_exact_limit_only() {
        assert!(validate_user_agent("Mozilla/5.0 (Synthetic Fixture)").is_ok());
        assert!(validate_user_agent(&"a".repeat(512)).is_ok());
        for invalid in [
            "".to_owned(),
            " ".to_owned(),
            "a".repeat(513),
            "中".to_owned(),
        ] {
            assert_eq!(
                validate_user_agent(&invalid),
                Err(TransportError::InvalidRequest)
            );
        }
        for byte in (0..=31).chain(127..=255) {
            let invalid = format!("fixture{}secret", char::from(byte));
            assert_eq!(
                validate_user_agent(&invalid),
                Err(TransportError::InvalidRequest)
            );
        }
    }

    #[test]
    fn supplier_panic_and_injection_are_closed_without_retry_or_fallback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        for panic in [false, true] {
            let calls = AtomicUsize::new(0);
            let error = CompatibilityHeaders::capture(CompatibilityProtocol::Bing, || {
                calls.fetch_add(1, Ordering::Relaxed);
                assert!(!panic, "synthetic supplier panic");
                "toy-secret\r\nAuthorization: injected"
            })
            .expect_err("closed supplier failure");
            assert_eq!(error, TransportError::InvalidRequest);
            assert_eq!(calls.load(Ordering::Relaxed), 1);
            assert!(!format!("{error:?} {error}").contains("toy-secret"));
        }
    }

    #[test]
    fn snapshot_debug_is_redacted_and_only_bing_has_language() {
        for protocol in [
            CompatibilityProtocol::Lite,
            CompatibilityProtocol::Html,
            CompatibilityProtocol::InstantAnswer,
            CompatibilityProtocol::Bing,
        ] {
            let headers = CompatibilityHeaders::capture(protocol, || "Synthetic Fixture/1.0")
                .expect("snapshot");
            assert_eq!(format!("{headers:?}"), "CompatibilityHeaders(<redacted>)");
            assert_eq!(headers.user_agent(), Some("Synthetic Fixture/1.0"));
            let request = headers
                .apply(reqwest::Client::new().get("https://fixture.invalid/"))
                .build()
                .expect("build only, no send");
            assert_eq!(
                request.headers()[reqwest::header::USER_AGENT],
                "Synthetic Fixture/1.0"
            );
            assert_eq!(
                request
                    .headers()
                    .get(reqwest::header::ACCEPT_LANGUAGE)
                    .map(|v| v.to_str().expect("ASCII")),
                matches!(protocol, CompatibilityProtocol::Bing)
                    .then_some("zh-CN,zh;q=0.9,en;q=0.8")
            );
        }
    }

    #[test]
    fn bearer_snapshot_is_bounded_sensitive_and_debug_redacted() {
        let secret = "toy-secret-123";
        let headers = CompatibilityHeaders::bearer(secret).expect("bearer snapshot");
        let debug = format!("{headers:?}");
        assert!(!debug.contains(secret));
        assert_eq!(headers.user_agent(), None);

        let request = headers
            .apply(reqwest::Client::new().post("https://fixture.invalid/search"))
            .build()
            .expect("build only, no send");
        let authorization = request
            .headers()
            .get(reqwest::header::AUTHORIZATION)
            .expect("authorization header");
        assert_eq!(authorization.to_str().expect("ASCII"), "Bearer toy-secret-123");
        assert!(authorization.is_sensitive());
    }

    #[test]
    fn bearer_snapshot_rejects_empty_control_whitespace_and_oversize_values() {
        for invalid in [
            String::new(),
            "contains space".to_owned(),
            "line\nfeed".to_owned(),
            "中".to_owned(),
            "a".repeat(MAX_AUTHORIZATION_BYTES),
        ] {
            let error = CompatibilityHeaders::bearer(&invalid).expect_err("invalid bearer");
            assert_eq!(error, TransportError::InvalidRequest);
            if !invalid.is_empty() {
                assert!(!format!("{error:?} {error}").contains(&invalid));
            }
        }
    }
}
