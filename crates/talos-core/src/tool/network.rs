//! Owned, non-serializable network invocation contracts (ADR-090).

use std::{fmt, net::IpAddr, sync::Arc};

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::{ToolExecutionOutput, ToolPermissionFacet};

/// Content-free preparation failures. Never attach upstream/configuration diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ToolNetworkPreparationError {
    /// Input or scope is invalid or exceeds its storage bound.
    #[error("network_preparation_invalid")]
    Invalid,
    /// The registered adapter cannot prepare a protected invocation.
    #[error("network_preparation_unavailable")]
    Unavailable,
}

/// Content-free execution authority failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ToolNetworkAuthorizationError {
    /// The invocation or destination differs from the admitted plan.
    #[error("network_authority_mismatch")]
    Mismatch,
    /// State, policy or grant authority has been invalidated.
    #[error("network_authority_invalidated")]
    Invalidated,
}

/// An exact HTTP destination, constructed from the actual parsed request URL.
/// Query values and credentials must not be supplied here. This is scope, not authority.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ToolNetworkEndpoint {
    scheme: String,
    host: String,
    port: u16,
    method: String,
    path: String,
}

impl ToolNetworkEndpoint {
    /// Validates an already canonical ASCII host and encoded path; performs no URL/DNS lookup.
    /// Callers must first parse the actual URL, including effective port, using their HTTP stack.
    pub fn new(
        scheme: &str,
        host: &str,
        port: u16,
        method: &str,
        path: &str,
    ) -> Result<Self, ToolNetworkPreparationError> {
        let host_valid = if host.starts_with('[') && host.ends_with(']') {
            host[1..host.len() - 1]
                .parse::<std::net::Ipv6Addr>()
                .is_ok_and(|ip| format!("[{ip}]") == host)
        } else if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
            ip.to_string() == host
        } else {
            host.len() <= 253
                && host.bytes().any(|b| b.is_ascii_lowercase())
                && host.split('.').all(|label| {
                    !label.is_empty()
                        && label.len() <= 63
                        && !label.starts_with('-')
                        && !label.ends_with('-')
                        && label
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                })
        };
        if !matches!(scheme, "http" | "https")
            || !host_valid
            || port == 0
            || !matches!(method, "GET" | "HEAD" | "POST" | "CONNECT")
            || !path.starts_with('/')
            || path.len() > 2048
            || path.bytes().any(|b| {
                !b.is_ascii()
                    || b.is_ascii_control()
                    || b == b' '
                    || matches!(b, b'?' | b'#' | b'\\')
            })
        {
            return Err(ToolNetworkPreparationError::Invalid);
        }
        let bytes = path.as_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            if *byte == b'%'
                && (i + 2 >= bytes.len()
                    || !bytes[i + 1].is_ascii_hexdigit()
                    || !bytes[i + 2].is_ascii_hexdigit())
            {
                return Err(ToolNetworkPreparationError::Invalid);
            }
        }
        if path.split('/').any(|part| {
            matches!(
                part.to_ascii_lowercase().replace("%2e", ".").as_str(),
                "." | ".."
            )
        }) {
            return Err(ToolNetworkPreparationError::Invalid);
        }
        Ok(Self {
            scheme: scheme.into(),
            host: host.into(),
            port,
            method: method.into(),
            path: path.into(),
        })
    }

    /// Returns the validated scheme.
    pub fn scheme(&self) -> &str {
        &self.scheme
    }
    /// Returns the canonical host used by Domain policy projection.
    pub fn host(&self) -> &str {
        &self.host
    }
    /// Returns the effective port, including default ports.
    pub const fn port(&self) -> u16 {
        self.port
    }
    /// Returns the exact HTTP method.
    pub fn method(&self) -> &str {
        &self.method
    }
    /// Returns the exact encoded path without query/fragment.
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Debug for ToolNetworkEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ToolNetworkEndpoint(<redacted>)")
    }
}

/// Immutable, query-independent Session scope. Equality covers the entire configuration.
#[derive(Clone, PartialEq, Eq)]
pub struct ToolNetworkScope {
    configuration_id: Uuid,
    configuration_revision: u64,
    endpoints: Vec<ToolNetworkEndpoint>,
    proxy: Option<ToolNetworkEndpoint>,
    private_addresses: Vec<IpAddr>,
    redirect_limit: u8,
}

impl ToolNetworkScope {
    /// Captures a new immutable tool configuration with a fresh identity and revision zero.
    /// Store this scope on the tool instance and clone it for each prepared manifest; do not
    /// construct a new configuration on every query if Session reuse is desired.
    pub fn new_configuration(
        endpoints: Vec<ToolNetworkEndpoint>,
        proxy: Option<ToolNetworkEndpoint>,
        private_addresses: Vec<IpAddr>,
        redirect_limit: u8,
    ) -> Result<Self, ToolNetworkPreparationError> {
        Self::new(
            Uuid::new_v4(),
            0,
            endpoints,
            proxy,
            private_addresses,
            redirect_limit,
        )
    }

    /// Builds a bounded scope. Private addresses and proxy are explicit trusted-host choices;
    /// implicit environment proxies are not represented or authorized. Transport must enforce
    /// these choices at connection time; this value alone does not prove SSRF protection.
    pub fn new(
        configuration_id: Uuid,
        configuration_revision: u64,
        mut endpoints: Vec<ToolNetworkEndpoint>,
        proxy: Option<ToolNetworkEndpoint>,
        mut private_addresses: Vec<IpAddr>,
        redirect_limit: u8,
    ) -> Result<Self, ToolNetworkPreparationError> {
        if configuration_id.is_nil()
            || endpoints.is_empty()
            || endpoints.len() > 64
            || private_addresses.len() > 16
            || redirect_limit > 5
        {
            return Err(ToolNetworkPreparationError::Invalid);
        }
        endpoints.sort();
        endpoints.dedup();
        private_addresses.sort();
        private_addresses.dedup();
        Ok(Self {
            configuration_id,
            configuration_revision,
            endpoints,
            proxy,
            private_addresses,
            redirect_limit,
        })
    }
    /// Returns the immutable tool instance/configuration identity.
    pub const fn configuration_id(&self) -> Uuid {
        self.configuration_id
    }
    /// Returns its revision.
    pub const fn configuration_revision(&self) -> u64 {
        self.configuration_revision
    }
    /// Returns all predeclared request/probe/fallback/redirect destinations.
    pub fn endpoints(&self) -> &[ToolNetworkEndpoint] {
        &self.endpoints
    }
    /// Returns the explicit proxy destination, if any.
    pub const fn proxy(&self) -> Option<&ToolNetworkEndpoint> {
        self.proxy.as_ref()
    }
    /// Returns the exact trusted-host opt-in address exceptions.
    pub fn private_addresses(&self) -> &[IpAddr] {
        &self.private_addresses
    }
    /// Returns the maximum manually followed redirects; implicit redirects are forbidden.
    pub const fn redirect_limit(&self) -> u8 {
        self.redirect_limit
    }
    /// Checks exact destination membership, including the declared proxy.
    pub fn contains(&self, endpoint: &ToolNetworkEndpoint) -> bool {
        self.endpoints.contains(endpoint) || self.proxy.as_ref() == Some(endpoint)
    }
}

impl fmt::Debug for ToolNetworkScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ToolNetworkScope(<redacted>)")
    }
}

/// One freshly identified prepared plan. Cloning data does not create execution authority.
#[derive(Clone, PartialEq, Eq)]
pub struct ToolNetworkManifest {
    invocation_id: Uuid,
    scope: ToolNetworkScope,
}

impl ToolNetworkManifest {
    /// Creates a fresh identity independent of query values.
    pub fn new(scope: ToolNetworkScope) -> Self {
        Self {
            invocation_id: Uuid::new_v4(),
            scope,
        }
    }
    /// Returns the per-invocation identity.
    pub const fn invocation_id(&self) -> Uuid {
        self.invocation_id
    }
    /// Returns the complete query-independent scope.
    pub const fn scope(&self) -> &ToolNetworkScope {
        &self.scope
    }
}

impl fmt::Debug for ToolNetworkManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ToolNetworkManifest(<redacted>)")
    }
}

/// Live permission-state checker injected by the Agent, never minted by the prepared object.
pub trait ToolNetworkAuthority: Send + Sync {
    /// Fences a logical request start. Revocation after this check cannot unsend bytes.
    fn check(
        &self,
        invocation_id: Uuid,
        destination: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError>;
}

/// One owned prepared execution. Input/configuration are captured before approval.
#[async_trait]
pub trait PreparedNetworkToolInvocation: Send {
    /// Returns the immutable, complete request plan.
    fn manifest(&self) -> &ToolNetworkManifest;
    /// Returns the matching Domain policy facets.
    fn permission_profile(&self) -> &[ToolPermissionFacet];
    /// Consumes this object, enforcing the supplied checker before every logical request start.
    async fn execute(
        self: Box<Self>,
        authority: Arc<dyn ToolNetworkAuthority>,
    ) -> ToolExecutionOutput;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_rejects_ambiguous_urls_credentials_and_control_characters() {
        for (scheme, host, port, method, path) in [
            ("ftp", "search.example", 443, "GET", "/search"),
            ("https", "SEARCH.example", 443, "GET", "/search"),
            ("https", "search.example.", 443, "GET", "/search"),
            ("https", "2130706433", 443, "GET", "/search"),
            ("https", "user@search.example", 443, "GET", "/search"),
            ("https", "search.example", 0, "GET", "/search"),
            ("https", "search.example", 443, "get", "/search"),
            ("https", "search.example", 443, "GET", "/search?q=secret"),
            ("https", "search.example", 443, "GET", "/search#secret"),
            ("https", "search.example", 443, "GET", "/%2e%2e/search"),
            ("https", "search.example", 443, "GET", "/bad%"),
            ("https", "search.example", 443, "GET", "/bad\u{7f}"),
        ] {
            assert_eq!(
                ToolNetworkEndpoint::new(scheme, host, port, method, path),
                Err(ToolNetworkPreparationError::Invalid)
            );
        }
        for host in ["xn--bcher-kva.example", "127.0.0.1", "[::1]"] {
            assert!(ToolNetworkEndpoint::new("https", host, 443, "GET", "/search").is_ok());
        }
    }

    #[test]
    fn scope_is_bounded_canonical_and_debug_redacted() {
        let endpoint =
            ToolNetworkEndpoint::new("https", "private.example", 443, "GET", "/private-path")
                .expect("endpoint");
        let id = Uuid::new_v4();
        let one =
            ToolNetworkScope::new(id, 0, vec![endpoint.clone()], None, vec![], 0).expect("scope");
        let duplicate = ToolNetworkScope::new(
            id,
            0,
            vec![endpoint.clone(), endpoint.clone()],
            None,
            vec![],
            0,
        )
        .expect("dedup");
        assert_eq!(one, duplicate);
        assert!(ToolNetworkScope::new(id, 0, vec![endpoint.clone(); 64], None, vec![], 5).is_ok());
        let addresses = vec!["127.0.0.1".parse().expect("ip"); 16];
        assert!(ToolNetworkScope::new(id, 0, vec![endpoint.clone()], None, addresses, 0).is_ok());
        let addresses = vec!["127.0.0.1".parse().expect("ip"); 17];
        assert!(ToolNetworkScope::new(id, 0, vec![endpoint.clone()], None, addresses, 0).is_err());
        assert!(
            ToolNetworkEndpoint::new(
                "https",
                "private.example",
                443,
                "GET",
                &format!("/{}", "x".repeat(2047))
            )
            .is_ok()
        );
        assert!(ToolNetworkScope::new(id, 0, vec![endpoint.clone(); 65], None, vec![], 0).is_err());
        assert!(ToolNetworkScope::new(id, 0, vec![endpoint.clone()], None, vec![], 6).is_err());
        assert!(
            ToolNetworkScope::new(Uuid::nil(), 0, vec![endpoint.clone()], None, vec![], 0).is_err()
        );
        assert!(ToolNetworkScope::new(id, 0, vec![], None, vec![], 0).is_err());
        assert!(
            ToolNetworkEndpoint::new(
                "https",
                "private.example",
                443,
                "GET",
                &format!("/{}", "x".repeat(2048))
            )
            .is_err()
        );
        let first = ToolNetworkManifest::new(one.clone());
        let second = ToolNetworkManifest::new(one);
        assert_ne!(first.invocation_id(), second.invocation_id());
        assert_eq!(first.scope(), second.scope());
        for debug in [
            format!("{endpoint:?}"),
            format!("{first:?}"),
            format!("{:?}", first.scope()),
        ] {
            assert!(!debug.contains("private.example"));
            assert!(!debug.contains("private-path"));
        }
    }
}
