//! Browser-specific invocation authorization. Legacy path/network grants are intentionally absent.

use std::sync::atomic::{AtomicBool, Ordering};

use super::{
    BrowserOperation, BrowserPermissionTarget, BrowserRequest, PreparedBrowserInvocation,
    PreparedBrowserObservation,
};

/// Exact browser permission resource derived only from trusted host state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserPermissionResource {
    binding: super::ticket::InvocationBinding,
    /// Exact operation being authorized.
    pub operation: BrowserOperation,
    /// Exact operation-family target, including independently verified origins where required.
    pub target: BrowserPermissionTarget,
    /// Canonical request digest binding sensitive values to this invocation.
    pub request_digest: [u8; 32],
}

/// One-shot browser authorization issued by a trusted browser permission evaluator.
#[derive(Debug)]
pub struct BrowserInvocationAuthorization {
    resource: BrowserPermissionResource,
    consumed: AtomicBool,
}

/// Browser-specific permission evaluator; default behavior is deny.
pub trait BrowserPermissionEvaluator: Send + Sync {
    /// Evaluates one exact resource. Implementations must never return reusable grants.
    fn evaluate(
        &self,
        resource: &BrowserPermissionResource,
    ) -> Result<BrowserInvocationAuthorization, BrowserAuthorizationError>;
}

/// Browser-aware asynchronous approval boundary for trusted embedded hosts.
/// Implementations may obtain per-invocation human approval but must never mint session grants.
#[async_trait::async_trait]
pub trait BrowserPermissionResolver: Send + Sync {
    /// Resolves only this exact host-derived resource; no execution occurs here.
    async fn resolve(
        &self,
        resource: &BrowserPermissionResource,
    ) -> Result<BrowserInvocationAuthorization, BrowserAuthorizationError>;
}

#[async_trait::async_trait]
impl<T: BrowserPermissionEvaluator> BrowserPermissionResolver for T {
    async fn resolve(
        &self,
        resource: &BrowserPermissionResource,
    ) -> Result<BrowserInvocationAuthorization, BrowserAuthorizationError> {
        self.evaluate(resource)
    }
}

/// Default browser policy: no invocation is authorized implicitly.
///
/// Hosts must explicitly replace this evaluator with a browser-aware policy or
/// per-invocation approval. Legacy network and workspace grants are not consulted.
#[derive(Debug, Default)]
pub struct DenyBrowserPermissionEvaluator;

impl BrowserPermissionEvaluator for DenyBrowserPermissionEvaluator {
    fn evaluate(
        &self,
        _resource: &BrowserPermissionResource,
    ) -> Result<BrowserInvocationAuthorization, BrowserAuthorizationError> {
        Err(BrowserAuthorizationError::PermissionDenied)
    }
}

/// Stable failure without request content or driver diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BrowserAuthorizationError {
    /// No browser-aware policy approved this exact invocation.
    #[error("browser permission denied")]
    PermissionDenied,
    /// The evaluator returned an authorization bound to a different request.
    #[error("browser authorization mismatch")]
    Mismatch,
    /// The canonical request could not be encoded; no authorization is issued.
    #[error("browser authorization request encoding failed")]
    InvalidRequest,
}

impl BrowserPermissionResource {
    /// Derives a resource from a prepared observation; no caller-supplied scope is accepted.
    pub(crate) fn from_observation(
        prepared: &PreparedBrowserObservation,
    ) -> Result<Self, BrowserAuthorizationError> {
        let request = prepared.request();
        Ok(Self {
            binding: prepared.binding(),
            operation: request.operation(),
            target: BrowserPermissionTarget::Observe {
                scope: prepared.scope().clone(),
                operation: request.operation(),
                element: None,
            },
            request_digest: request_digest(request)?,
        })
    }

    pub(crate) fn from_invocation(
        prepared: &PreparedBrowserInvocation,
    ) -> Result<Self, BrowserAuthorizationError> {
        Ok(Self {
            binding: prepared.ticket.binding(),
            operation: prepared.request().operation(),
            target: prepared.target().clone(),
            request_digest: request_digest(prepared.request())?,
        })
    }
}

impl BrowserInvocationAuthorization {
    /// Returns the exact resource covered by this one-shot authorization.
    pub fn resource(&self) -> &BrowserPermissionResource {
        &self.resource
    }

    pub(crate) fn consume(
        &self,
        resource: &BrowserPermissionResource,
    ) -> Result<(), BrowserAuthorizationError> {
        if !self.matches(resource)
            || self
                .consumed
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return Err(BrowserAuthorizationError::Mismatch);
        }
        Ok(())
    }

    pub(crate) fn matches(&self, resource: &BrowserPermissionResource) -> bool {
        self.resource == *resource
            && self.resource.binding.is_live()
            && !self.consumed.load(Ordering::Acquire)
    }
}

fn request_digest(request: &BrowserRequest) -> Result<[u8; 32], BrowserAuthorizationError> {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    let canonical = serde_json::to_vec(request.fields())
        .map_err(|_| BrowserAuthorizationError::InvalidRequest)?;
    digest.update(b"talos.browser.executor/v2/request\0");
    digest.update(canonical);
    Ok(digest.finalize().into())
}

/// Minimal evaluator useful for deterministic host tests; it only approves an exact resource.
pub struct ExactBrowserPermissionEvaluator {
    resource: BrowserPermissionResource,
    used: AtomicBool,
}

impl ExactBrowserPermissionEvaluator {
    /// Creates an evaluator that approves one exact resource once.
    pub fn for_resource(resource: BrowserPermissionResource) -> Self {
        Self {
            resource,
            used: AtomicBool::new(false),
        }
    }
}

impl BrowserPermissionEvaluator for ExactBrowserPermissionEvaluator {
    fn evaluate(
        &self,
        resource: &BrowserPermissionResource,
    ) -> Result<BrowserInvocationAuthorization, BrowserAuthorizationError> {
        if resource != &self.resource
            || !resource.binding.is_live()
            || self
                .used
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return Err(BrowserAuthorizationError::PermissionDenied);
        }
        Ok(BrowserInvocationAuthorization {
            resource: resource.clone(),
            consumed: AtomicBool::new(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser_executor::{BrowserContext, BrowserOrigin};

    #[test]
    fn exact_evaluator_denies_scope_or_payload_swap() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let frame = host
            .insert_frame(
                &tab,
                None,
                Some(BrowserOrigin::from_effective_url("https://example.com").expect("origin")),
                true,
            )
            .expect("frame");
        let request = BrowserRequest::parse_raw(&serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":tab,"frameRef":frame}).to_string()).expect("request");
        let prepared = host.prepare_observation(request).expect("prepared");
        let resource = BrowserPermissionResource::from_observation(&prepared).expect("resource");
        assert!(matches!(
            DenyBrowserPermissionEvaluator.evaluate(&resource),
            Err(BrowserAuthorizationError::PermissionDenied)
        ));
        let evaluator = ExactBrowserPermissionEvaluator::for_resource(resource.clone());
        let authorization = evaluator.evaluate(&resource).expect("approved");
        assert!(authorization.matches(&resource));
        assert!(evaluator.evaluate(&resource).is_err());
        let second = host.prepare_observation(BrowserRequest::parse_raw(&serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":tab,"frameRef":frame}).to_string()).expect("request")).expect("second invocation");
        let second_resource =
            BrowserPermissionResource::from_observation(&second).expect("resource");
        assert!(!authorization.matches(&second_resource));
        let mut changed = resource.clone();
        changed.operation = BrowserOperation::Snapshot;
        assert!(!authorization.matches(&changed));
        assert!(authorization.consume(&changed).is_err());
        assert!(authorization.consume(&resource).is_ok());
        assert!(authorization.consume(&resource).is_err());
        assert!(!authorization.matches(&resource));
    }

    #[test]
    fn digest_binds_normalized_payload_independent_of_wire_order() {
        let implicit = BrowserRequest::parse_raw(r#"{"protocolVersion":2,"operation":"scroll","tabRef":"t","frameRef":"f","direction":"down"}"#).expect("request");
        let explicit = BrowserRequest::parse_raw(r#"{"amount":500,"direction":"down","frameRef":"f","tabRef":"t","operation":"scroll","protocolVersion":2.0}"#).expect("request");
        let changed = BrowserRequest::parse_raw(r#"{"protocolVersion":2,"operation":"scroll","tabRef":"t","frameRef":"f","direction":"up"}"#).expect("request");
        assert_eq!(request_digest(&implicit), request_digest(&explicit));
        assert_ne!(request_digest(&implicit), request_digest(&changed));
    }
}
