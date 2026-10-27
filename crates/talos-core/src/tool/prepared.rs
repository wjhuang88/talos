//! Owned invocation boundary for tools that cannot use parsed-JSON/path authorization.

use async_trait::async_trait;

use super::{ToolExecutionOutput, ToolPermissionFacet};

/// Ephemeral identity of one prepared effect plan, independent of its arguments.
/// Clones identify the same plan; constructing a new value always creates a
/// different identity. This is not authorization or a durable replay identifier.
#[derive(Debug, Clone)]
pub struct PreparedPlanIdentity(std::sync::Arc<()>);

impl PreparedPlanIdentity {
    /// Creates an identity for one preparation.
    pub fn new() -> Self {
        Self(std::sync::Arc::new(()))
    }

    /// Tests whether both handles belong to the same preparation.
    pub fn same_plan(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Default for PreparedPlanIdentity {
    fn default() -> Self {
        Self::new()
    }
}

/// Runner-owned stage history for one normal prepared invocation.
/// This enforces a finite eight-stage bound, identity uniqueness and cumulative
/// authorization profile limits when runners use [`Self::record_profile`];
/// it grants no authority and does not replace proof consumption or cleanup.
#[derive(Default)]
pub struct PreparedStageHistory {
    identities: Vec<PreparedPlanIdentity>,
    facets: usize,
    profile_bytes: usize,
}

const MAX_PREPARED_STAGES: usize = 8;
const MAX_PREPARED_FACETS: usize = 65_536;
const MAX_PREPARED_PROFILE_BYTES: usize = 16 * 1024 * 1024;

/// Total wall-clock budget for a prepared normal-tool invocation, including
/// every discovery and mutation stage. Approval budgets can be shorter.
pub const PREPARED_INVOCATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

impl PreparedStageHistory {
    /// Reserves one stage and its complete authorization profile before cloning
    /// or admission. Text bytes and facet counts accumulate across continuations;
    /// refusal changes no counters and supplies no authority.
    pub fn record_profile(
        &mut self,
        identity: &PreparedPlanIdentity,
        profile: &[ToolPermissionFacet],
    ) -> bool {
        let Some(facets) = self.facets.checked_add(profile.len()) else {
            return false;
        };
        if facets > MAX_PREPARED_FACETS {
            return false;
        }
        let mut bytes = self.profile_bytes;
        for facet in profile {
            for text in [&facet.resource, &facet.description].into_iter().flatten() {
                let Some(next) = bytes.checked_add(text.len()) else {
                    return false;
                };
                if next > MAX_PREPARED_PROFILE_BYTES {
                    return false;
                }
                bytes = next;
            }
        }
        if !self.record(identity) {
            return false;
        }
        self.facets = facets;
        self.profile_bytes = bytes;
        true
    }

    /// Records a fresh stage before admission. Returns false for a repeated
    /// identity or an exhausted budget, without changing the retained history.
    pub fn record(&mut self, identity: &PreparedPlanIdentity) -> bool {
        if self.identities.len() >= MAX_PREPARED_STAGES
            || self
                .identities
                .iter()
                .any(|previous| previous.same_plan(identity))
        {
            return false;
        }
        self.identities.push(identity.clone());
        true
    }
}

/// An owned normal-tool operation prepared from validated, normalized input.
///
/// The same object supplies permission facets and is consumed at dispatch. A
/// tool using this route must revalidate mutable external state immediately
/// before its effects; preparation alone is not filesystem containment.
#[async_trait]
pub trait PreparedNormalToolInvocation: Send {
    /// Returns operation-local cleanup evidence before execution can start.
    /// Continuations share this receipt; it must not wait for unrelated calls.
    /// Registration precedes spawning, and owner sealing also blocks subsequent
    /// stages. None retains the legacy implementation's lifecycle contract.
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn super::ToolCleanupReceipt>> {
        None
    }

    /// Identifies this exact prepared plan. Missing identity fails closed at admission.
    fn plan_identity(&self) -> Option<&PreparedPlanIdentity> {
        None
    }

    /// Returns implementation-owned command evidence for this exact stage.
    /// Caller JSON and approval presentation notes are not substitutes. Legacy
    /// producers return None and retain their existing authorization behavior.
    fn prepared_command_evidence(&self) -> Option<super::PreparedCommandEvidence> {
        None
    }

    /// Returns complete private before/after evidence captured after read admission.
    /// Dispatch must verify native instance ownership and the exact plan binding;
    /// evidence alone cannot authorize a mutation. Legacy producers return None.
    fn prepared_file_mutation_evidence(&self) -> Option<super::PreparedFileMutationEvidence> {
        None
    }

    /// Returns implementation-owned review configuration for this exact stage.
    /// This is descriptive evidence, never authority. Dispatch must verify the
    /// registered built-in instance and bind admission to this stage's identity.
    /// Returning None preserves the existing prepared authorization route.
    fn invocation_review_configuration(&self) -> Option<serde_json::Value> {
        None
    }

    /// The complete set of effects to admit as one invocation.
    fn permission_profile(&self) -> &[ToolPermissionFacet];

    /// Executes one independently admitted stage. A continuation has no inherited
    /// authority: callers must admit its fresh identity and facets through all
    /// normal permission and final hook gates before dispatching it.
    async fn execute_stage(
        self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> PreparedNormalStage {
        PreparedNormalStage::Complete(self.execute(admission).await)
    }

    /// Consumes the reviewed operation after permission and final hook admission.
    /// Built-ins must downcast and consume the permission crate's exact
    /// invocation proof before any effect; an arbitrary boxed value is not
    /// authority. This erased carrier avoids a core -> permission cycle.
    async fn execute(
        self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> ToolExecutionOutput;
}

/// Result of one independently authorized normal-tool stage.
pub enum PreparedNormalStage {
    /// Terminal output, including failures. Never retry the consumed stage.
    Complete(ToolExecutionOutput),
    /// Newly prepared operation requiring fresh admission, never automatic execution.
    Continue(Box<dyn PreparedNormalToolInvocation>),
}

/// Closed failure vocabulary for the browser prepared-invocation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub enum PreparedFailureCode {
    /// Reference is unknown or no longer valid.
    InvalidReference,
    /// Frame generation changed.
    StaleFrameReference,
    /// Frame was detached.
    DetachedFrame,
    /// Element snapshot changed.
    StaleElementReference,
    /// Trusted context is unavailable.
    ContextUnavailable,
    /// Destination origin lacks authority.
    OriginNotAuthorized,
    /// Protocol version is unsupported.
    UnsupportedVersion,
    /// Exact operation cannot be performed safely.
    UnsupportedOperation,
    /// Bounded capacity was exceeded.
    ResourceLimit,
    /// Admission lifetime elapsed before execution.
    AdmissionExpired,
    /// Request failed schema or semantic validation.
    InvalidRequest,
    /// Exact authorization was not granted.
    PermissionDenied,
    /// Execution may have begun without a confirmed result.
    IndeterminateExecution,
}

/// Result of a prepared invocation with request-only provider attachments.
/// Kept separate from the existing exhaustive tool output structure for source compatibility.
pub struct PreparedExecutionOutput {
    /// Existing display and persistence projection.
    pub output: ToolExecutionOutput,
    /// Images owned by the immediately following provider request, never durable history.
    pub images: Vec<crate::provider::EphemeralImage>,
}

impl From<ToolExecutionOutput> for PreparedExecutionOutput {
    fn from(output: ToolExecutionOutput) -> Self {
        Self {
            output,
            images: Vec::new(),
        }
    }
}

/// Bounded failure without original arguments or backend diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PreparedInvocationError {
    /// No valid prepared route, current admission, or exact authorization is available.
    #[error("prepared invocation denied or unavailable")]
    Denied,
    /// A closed, content-free browser failure. Unknown operation strings are never projected.
    #[error("prepared invocation failed: {code:?}")]
    Failure {
        /// Stable failure category.
        code: PreparedFailureCode,
        /// Known discriminator; output independently validates the closed vocabulary.
        operation: Option<&'static str>,
    },
}

impl PreparedInvocationError {
    /// Recognizes only a closed browser discriminator without retaining arbitrary input.
    pub fn browser_operation(name: &str) -> Option<&'static str> {
        [
            "open",
            "read",
            "snapshot",
            "current-url",
            "screenshot",
            "click",
            "fill",
            "select",
            "hover",
            "check",
            "uncheck",
            "press",
            "scroll",
            "wait-for-element",
            "wait-milliseconds",
            "tab-new",
            "tab-list",
            "tab-close",
            "tab-switch",
            "window-close",
            "frame-tree",
        ]
        .into_iter()
        .find(|operation| *operation == name)
    }
    /// Renders a closed browser failure without arguments or backend diagnostics.
    /// Only indeterminate execution claims an unknown outcome; all other errors from this
    /// boundary establish that execution was not performed.
    pub fn into_output(self) -> ToolExecutionOutput {
        let (code, operation) = match self {
            Self::Denied => (PreparedFailureCode::PermissionDenied, None),
            Self::Failure { code, operation } => (code, operation),
        };
        let operation = operation.and_then(Self::browser_operation);
        ToolExecutionOutput::error(serde_json::json!({
            "kind": "Failure", "code": code, "operation": operation,
            "outcome": if code == PreparedFailureCode::IndeterminateExecution { "unknown" } else { "notExecuted" },
        }).to_string())
    }
}

/// An admitted invocation owned by one active request, never cloned or serialized.
///
/// Implementations retain the typed resource and dedicated permission resolver together.
/// Generic path/network grants must not be consulted. Dropping this object revokes admission.
#[async_trait]
pub trait PreparedToolInvocation: Send {
    /// Evaluates the exact admitted resource once and consumes this preparation.
    /// Must not execute a tool. The caller retains its final hook gate before dispatch.
    async fn authorize(
        self: Box<Self>,
    ) -> Result<Box<dyn AuthorizedToolInvocation>, PreparedInvocationError>;
}

/// One exact authorized invocation, still subject to final lifecycle validation.
#[async_trait]
pub trait AuthorizedToolInvocation: Send {
    /// Executes once and transfers any ephemeral attachments to the active request.
    /// Legacy implementers retain their existing behavior without attachments.
    async fn execute_with_attachments(self: Box<Self>) -> PreparedExecutionOutput {
        PreparedExecutionOutput {
            output: self.execute().await,
            images: Vec::new(),
        }
    }

    /// Consumes the authorization and dispatches at most once, without fallback or replay.
    /// Dropping the future must propagate cancellation to the owned execution.
    async fn execute(self: Box<Self>) -> ToolExecutionOutput;
}

#[cfg(test)]
mod failure_tests {
    use super::*;

    #[test]
    fn stage_profiles_share_count_and_byte_budgets() {
        let mut counts = PreparedStageHistory::default();
        let profile =
            vec![ToolPermissionFacet::new(super::super::ToolNature::Read); MAX_PREPARED_FACETS / 2];
        assert!(counts.record_profile(&PreparedPlanIdentity::new(), &profile));
        assert!(counts.record_profile(&PreparedPlanIdentity::new(), &profile));
        let refused = PreparedPlanIdentity::new();
        assert!(!counts.record_profile(&refused, &profile[..1]));
        assert_eq!(counts.identities.len(), 2);
        assert!(counts.record_profile(&refused, &[]));

        let mut bytes = PreparedStageHistory::default();
        let profile = [ToolPermissionFacet::new(super::super::ToolNature::Read)
            .with_description("x".repeat(MAX_PREPARED_PROFILE_BYTES / 2))];
        assert!(bytes.record_profile(&PreparedPlanIdentity::new(), &profile));
        assert!(bytes.record_profile(&PreparedPlanIdentity::new(), &profile));
        assert!(!bytes.record_profile(&PreparedPlanIdentity::new(), &profile));
        assert_eq!(bytes.profile_bytes, MAX_PREPARED_PROFILE_BYTES);
        assert_eq!(bytes.identities.len(), 2);
    }

    #[test]
    fn stage_history_retains_identity_and_counts_only_fresh_stages() {
        let mut history = PreparedStageHistory::default();
        let first = PreparedPlanIdentity::new();
        assert!(history.record(&first));
        assert!(!history.record(&first.clone()));
        let later: Vec<_> = (1..MAX_PREPARED_STAGES)
            .map(|_| PreparedPlanIdentity::new())
            .collect();
        for identity in &later {
            assert!(history.record(identity));
        }
        assert!(!history.record(&first));
        assert!(!history.record(&later[2]));
        assert!(!history.record(&PreparedPlanIdentity::new()));
    }

    #[test]
    fn failure_projection_is_closed_and_effect_honest() {
        for (code, outcome) in [
            (PreparedFailureCode::InvalidRequest, "notExecuted"),
            (PreparedFailureCode::IndeterminateExecution, "unknown"),
        ] {
            let output = PreparedInvocationError::Failure {
                code,
                operation: Some("fill"),
            }
            .into_output();
            assert!(output.result.is_error);
            let value: serde_json::Value =
                serde_json::from_str(&output.result.content).expect("failure JSON");
            assert_eq!(value["operation"], "fill");
            assert_eq!(value["outcome"], outcome);
            assert_eq!(value["kind"], "Failure");
        }
        let output = PreparedInvocationError::Failure {
            code: PreparedFailureCode::InvalidRequest,
            operation: Some("secret-driver-error"),
        }
        .into_output();
        assert!(!output.result.content.contains("secret"));
        assert_eq!(
            PreparedInvocationError::browser_operation("hover"),
            Some("hover")
        );
    }
}
