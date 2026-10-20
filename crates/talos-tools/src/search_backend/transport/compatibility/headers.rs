//! Dormant fixed compatibility headers. No caller header map or executable override.
//!
//! Reuses rust-websearch 0.1.2's public anti_detection::random_user_agent helper.
//! Fixed Bing language derives from VCS b30180a0, crates/websearch/src/bing.rs;
//! client/UA provenance: src/anti_detection/{mod,user_agent}.rs in that package.
//! MIT notice: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Talos freezes once before admission, bounds/validates values and closes errors.

use super::{CompatibilityProtocol, TransportError};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::search_backend::transport) struct CompatibilityHeaders {
    user_agent: &'static str,
    bing: bool,
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

impl CompatibilityHeaders {
    // Only compatibility preparation can construct the type. The sole production suppliers
    // are the pinned helper; a private supplier enables deterministic error/order fixtures.
    pub(super) fn capture(
        protocol: CompatibilityProtocol,
        select: impl FnOnce() -> &'static str + std::panic::UnwindSafe,
    ) -> Result<Self, TransportError> {
        let user_agent =
            std::panic::catch_unwind(select).map_err(|_| TransportError::InvalidRequest)?;
        validate_user_agent(user_agent)?;
        Ok(Self {
            user_agent,
            bing: matches!(protocol, CompatibilityProtocol::Bing),
        })
    }

    pub(in crate::search_backend::transport) fn apply(
        self,
        request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        let request = request.header(reqwest::header::USER_AGENT, self.user_agent);
        if self.bing {
            request.header(reqwest::header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8")
        } else {
            request
        }
    }

    #[cfg(test)]
    pub(in crate::search_backend::transport) fn user_agent(self) -> &'static str {
        self.user_agent
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
}
