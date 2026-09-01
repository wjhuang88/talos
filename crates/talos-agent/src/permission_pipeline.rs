//! Agent-owned permission evaluation, approval and admission pipeline.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::FutureExt;
use talos_core::ApprovalChoice;
use talos_core::tool::{ToolExecutionAuthorization, ToolPermissionFacet, ToolProvenance};
use talos_permission::{
    GrantPreview, GrantSource, InteractionCapability, PermissionContext, PermissionDecision,
    PermissionInvocation, PermissionMode, PermissionRequest, PermissionSessionState,
};
use thiserror::Error;
use tokio::time::{Instant, timeout_at};

/// Normalizes the exact value shared by permission evaluation and execution.
#[must_use]
pub fn normalize_permission_input(tool_name: &str, input: serde_json::Value) -> serde_json::Value {
    crate::helpers::normalize_tool_input(tool_name, input)
}

/// Produces a structure-only projection for permission hooks and logs.
///
/// Authorization and execution retain the exact normalized input. This projection deliberately
/// removes every caller-provided value so hooks and logs cannot observe raw arguments, secrets, or
/// concrete resource paths. Approval resolvers instead receive the tool-defined safe presentation
/// projection supplied in [`PermissionAuthorizationRequest::presentation_input`].
#[must_use]
pub fn project_permission_input(input: &serde_json::Value) -> serde_json::Value {
    match input {
        serde_json::Value::Object(fields) => serde_json::Value::Object(
            fields
                .keys()
                .map(|key| {
                    (
                        key.clone(),
                        serde_json::Value::String("<redacted>".to_owned()),
                    )
                })
                .collect(),
        ),
        serde_json::Value::Array(values) => {
            serde_json::json!({"kind": "array", "length": values.len()})
        }
        _ => serde_json::Value::String("<redacted>".to_owned()),
    }
}

/// Approval request projected from an evaluated, normalized tool request.
#[derive(Debug, Clone)]
pub struct PermissionApprovalRequest {
    /// Tool name being approved.
    pub tool_name: String,
    /// Trusted tool provenance captured by the authoritative pipeline.
    pub provenance: ToolProvenance,
    /// Safe presentation projection; not used for authorization identity.
    pub arguments: serde_json::Value,
    /// Bounded summary fields for a local approval surface.
    pub summary_fields: Vec<String>,
    /// Bounded current-turn user intent, never prior history or tool output.
    pub user_intent: String,
    /// Exact capability-relative Session preview.
    pub preview: GrantPreview,
    /// Redaction-safe state binding captured at evaluation time.
    pub binding: PermissionBinding,
}

/// Opaque permission state identity supplied to bounded approval adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionBinding {
    /// In-memory Session identity.
    pub session_id: String,
    /// Monotonic state generations in policy/mode/workspace order.
    pub revisions: [u64; 6],
    /// Evaluation mode.
    pub mode: PermissionMode,
    /// Whether a human approval surface is available.
    pub interaction: InteractionCapability,
}

/// Exact authorization inputs owned by the Agent permission pipeline.
pub struct PermissionAuthorizationRequest<'a> {
    /// Registered tool name.
    pub tool_name: &'a str,
    /// Tool implementation provenance.
    pub provenance: ToolProvenance,
    /// Permission facets derived from the normalized input.
    pub profile: &'a [ToolPermissionFacet],
    /// Exact normalized input shared with execution.
    pub input: &'a serde_json::Value,
    /// Safe presentation projection for an approval surface.
    pub presentation_input: serde_json::Value,
    /// Bounded fields highlighted by an approval surface.
    pub summary_fields: Vec<String>,
    /// Current-turn user instruction used only as bounded classifier context.
    pub user_intent: Option<&'a str>,
    /// Total remaining permission-pipeline budget.
    pub deadline: Duration,
}

/// Surface adapter that resolves an already-evaluated `Ask`.
#[async_trait]
pub trait ApprovalResolver: Send + Sync {
    /// Resolves only the bounded approval scope; policy and execution remain Agent-owned.
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        remaining: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError>;

    /// Identifies the bounded source recorded for a Session grant.
    fn grant_source(&self) -> GrantSource {
        GrantSource::InteractiveHuman
    }
}

/// A bounded approval adapter failed without producing an approval choice.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct ApprovalResolverError {
    message: String,
}

impl ApprovalResolverError {
    /// Creates a redaction-safe resolver failure.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Fail-closed errors from the Agent-owned permission pipeline.
#[derive(Debug, Error)]
pub enum PermissionPipelineError {
    /// Permission state could not be read or committed.
    #[error("permission state error: {0}")]
    State(String),
    /// No resolver was available for an Ask decision.
    #[error("approval required but no resolver is configured")]
    ResolverUnavailable,
    /// The resolver exceeded the remaining invocation budget.
    #[error("approval deadline exceeded")]
    DeadlineExceeded,
    /// The approval adapter failed or was cancelled.
    #[error("approval resolver failed closed: {0}")]
    Resolver(String),
    /// The resolver returned a denial or the approval channel closed.
    #[error("approval denied")]
    Denied(PermissionDecision),
}

impl PermissionPipelineError {
    /// Returns the final decision available to the final execution hook.
    #[must_use]
    pub fn decision(&self) -> Option<&PermissionDecision> {
        match self {
            Self::Denied(decision) => Some(decision),
            _ => None,
        }
    }

    /// Projects every pipeline failure to the final fail-closed hook decision.
    #[must_use]
    pub fn final_decision(&self) -> PermissionDecision {
        self.decision()
            .cloned()
            .unwrap_or_else(|| PermissionDecision::Deny("permission pipeline failed closed".into()))
    }
}

/// One Agent-owned authorization pipeline for a shared permission Session.
pub struct PermissionPipeline {
    state: Arc<PermissionSessionState>,
    context: PermissionContext,
    resolver: Option<Arc<dyn ApprovalResolver>>,
}

impl PermissionPipeline {
    /// Creates a pipeline around a composition-root-owned Session state.
    #[must_use]
    pub fn new(
        state: Arc<PermissionSessionState>,
        context: PermissionContext,
        resolver: Option<Arc<dyn ApprovalResolver>>,
    ) -> Self {
        Self {
            state,
            context,
            resolver,
        }
    }

    /// Returns the shared Session state used by this pipeline.
    #[must_use]
    pub fn state(&self) -> Arc<PermissionSessionState> {
        self.state.clone()
    }

    /// Attaches a surface resolver before the pipeline is shared.
    pub fn set_resolver(&mut self, resolver: Arc<dyn ApprovalResolver>) {
        self.resolver = Some(resolver);
        self.context =
            PermissionContext::new(self.context.mode(), InteractionCapability::Available);
    }

    /// Evaluates, resolves and admits one exact normalized request.
    pub async fn authorize(
        &self,
        authorization: PermissionAuthorizationRequest<'_>,
    ) -> Result<Vec<ToolExecutionAuthorization>, PermissionPipelineError> {
        let PermissionAuthorizationRequest {
            tool_name,
            provenance,
            profile,
            input,
            presentation_input,
            summary_fields,
            user_intent,
            deadline,
        } = authorization;
        let deadline_at = Instant::now() + deadline;
        ensure_before(deadline_at)?;
        let binding_snapshot = self
            .state
            .state_snapshot()
            .map_err(|error| PermissionPipelineError::State(error.to_string()))?;
        let request = PermissionRequest::new(tool_name, provenance.clone(), profile, input);
        let invocation = self
            .state
            .try_begin_invocation(&request, &self.context)
            .map_err(|error| PermissionPipelineError::State(error.to_string()))?;
        ensure_before(deadline_at)?;

        let pending = match invocation {
            PermissionInvocation::Allow(pending) => *pending,
            PermissionInvocation::Deny(decision) => {
                return Err(PermissionPipelineError::Denied(decision));
            }
            PermissionInvocation::Ask { once, session } => {
                let resolver = self
                    .resolver
                    .as_ref()
                    .ok_or(PermissionPipelineError::ResolverUnavailable)?;
                let approval = PermissionApprovalRequest {
                    tool_name: tool_name.to_owned(),
                    provenance,
                    arguments: presentation_input,
                    summary_fields,
                    user_intent: user_intent
                        .map(|intent| intent.chars().take(4096).collect())
                        .unwrap_or_default(),
                    preview: session.preview().clone(),
                    binding: PermissionBinding {
                        session_id: binding_snapshot.session_id.stable_id(),
                        revisions: binding_snapshot.revisions.as_array(),
                        mode: self.context.mode(),
                        interaction: self.context.interaction(),
                    },
                };
                let remaining = deadline_at.saturating_duration_since(Instant::now());
                let resolver_future =
                    std::panic::AssertUnwindSafe(resolver.resolve(approval, remaining))
                        .catch_unwind();
                let choice = timeout_at(deadline_at, resolver_future)
                    .await
                    .map_err(|_| PermissionPipelineError::DeadlineExceeded)?
                    .map_err(|_| PermissionPipelineError::Resolver("resolver panicked".to_owned()))?
                    .map_err(|error| PermissionPipelineError::Resolver(error.to_string()))?;
                ensure_before(deadline_at)?;
                match choice {
                    ApprovalChoice::ApproveOnce => self
                        .state
                        .try_approve_once(*once, &request, &self.context)
                        .map_err(|error| PermissionPipelineError::State(error.to_string()))?,
                    ApprovalChoice::AlwaysApprove => self
                        .state
                        .try_approve_session(
                            *session,
                            &request,
                            &self.context,
                            resolver.grant_source(),
                        )
                        .map_err(|error| PermissionPipelineError::State(error.to_string()))?,
                    ApprovalChoice::Deny => {
                        return Err(PermissionPipelineError::Denied(PermissionDecision::Deny(
                            "User denied".into(),
                        )));
                    }
                }
            }
        };

        ensure_before(deadline_at)?;

        self.state
            .try_admit(pending, &request, &self.context)
            .map_err(|error| PermissionPipelineError::State(error.to_string()))
            .and_then(|authorization| {
                ensure_before(deadline_at)?;
                Ok(authorization)
            })
    }
}

fn ensure_before(deadline: Instant) -> Result<(), PermissionPipelineError> {
    if Instant::now() >= deadline {
        Err(PermissionPipelineError::DeadlineExceeded)
    } else {
        Ok(())
    }
}

/// Creates the default headless pipeline state for an engine.
#[must_use]
pub fn headless_pipeline(engine: talos_permission::PermissionEngine) -> PermissionPipeline {
    PermissionPipeline::new(
        Arc::new(PermissionSessionState::new(engine)),
        PermissionContext::new(PermissionMode::Headless, InteractionCapability::Unavailable),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auto_resolver::{
        AutoPermissionAssessor, AutoPermissionControl, AutoPermissionRequest,
        AutoPermissionResolver, ManagedWorkspaceLease,
    };
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use talos_core::tool::{ToolNature, ToolResourceKind};
    use talos_permission::{PermissionEngine, PermissionRule, ResourceKind};

    struct StaticResolver(ApprovalChoice);

    struct MatrixCapability;

    impl talos_core::tool::AtomicCreateCapability for MatrixCapability {
        fn create_new(&self, _relative_path: &Path, _contents: &[u8]) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct MatrixAssessor {
        calls: Arc<AtomicUsize>,
    }

    fn matrix_approval_request(
        root: &Path,
        state: &PermissionSessionState,
        path: &str,
    ) -> PermissionApprovalRequest {
        let input = serde_json::json!({"path": path, "content": "hello"});
        let profile = [ToolPermissionFacet::with_resource(
            ToolNature::Write,
            root.join(path).display().to_string(),
            ToolResourceKind::Path,
        )];
        let request = PermissionRequest::new("write", ToolProvenance::Native, &profile, &input);
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let PermissionInvocation::Ask {
            session: proposal, ..
        } = state
            .begin_invocation(&request, &context)
            .expect("approval proposal")
        else {
            panic!("write should require approval");
        };
        PermissionApprovalRequest {
            tool_name: "write".to_owned(),
            provenance: ToolProvenance::Native,
            arguments: input,
            summary_fields: vec!["path".to_owned()],
            user_intent: String::new(),
            preview: proposal.preview().clone(),
            binding: PermissionBinding {
                session_id: state.session_id().expect("session id").stable_id(),
                revisions: state
                    .state_snapshot()
                    .expect("snapshot")
                    .revisions
                    .as_array(),
                mode: context.mode(),
                interaction: context.interaction(),
            },
        }
    }

    #[async_trait]
    impl AutoPermissionAssessor for MatrixAssessor {
        async fn assess(
            &self,
            request: AutoPermissionRequest,
            _remaining: Duration,
        ) -> Result<String, String> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            Ok(format!(
                "{{\"schema_version\":1,\"request_digest\":\"{}\",\"decision\":\"allow_once\",\"effect\":\"read_only\",\"reason_code\":\"bounded_workspace_text_create\",\"confidence\":\"high\"}}",
                request.request_digest
            ))
        }
    }

    #[async_trait]
    impl ApprovalResolver for StaticResolver {
        async fn resolve(
            &self,
            _request: PermissionApprovalRequest,
            _remaining: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            Ok(self.0.clone())
        }
    }

    struct SlowResolver;

    #[async_trait]
    impl ApprovalResolver for SlowResolver {
        async fn resolve(
            &self,
            _request: PermissionApprovalRequest,
            _remaining: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            tokio::time::sleep(Duration::from_secs(60)).await;
            Ok(ApprovalChoice::ApproveOnce)
        }
    }

    struct InvalidatingResolver {
        state: Arc<PermissionSessionState>,
    }

    struct FailingResolver;

    #[async_trait]
    impl ApprovalResolver for FailingResolver {
        async fn resolve(
            &self,
            request: PermissionApprovalRequest,
            remaining: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            assert_eq!(request.arguments["path"], "target.txt");
            assert!(request.arguments.get("secret").is_none());
            assert!(remaining <= Duration::from_secs(1));
            Err(ApprovalResolverError::new("approval channel closed"))
        }
    }

    struct PanickingResolver;

    #[async_trait]
    impl ApprovalResolver for PanickingResolver {
        async fn resolve(
            &self,
            _request: PermissionApprovalRequest,
            _remaining: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            panic!("resolver panic")
        }
    }

    #[async_trait]
    impl ApprovalResolver for InvalidatingResolver {
        async fn resolve(
            &self,
            _request: PermissionApprovalRequest,
            _remaining: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            self.state.clear().expect("invalidate permission revision");
            Ok(ApprovalChoice::ApproveOnce)
        }
    }

    fn write_profile(path: &Path) -> Vec<ToolPermissionFacet> {
        vec![ToolPermissionFacet::with_resource(
            ToolNature::Write,
            path.display().to_string(),
            ToolResourceKind::Path,
        )]
    }

    #[tokio::test]
    async fn ask_without_resolver_fails_closed() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target.txt");
        std::fs::write(&target, b"test").expect("fixture");
        let state = Arc::new(PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().to_path_buf()),
        ));
        let pipeline = PermissionPipeline::new(
            state,
            PermissionContext::new(PermissionMode::Headless, InteractionCapability::Unavailable),
            None,
        );
        let error = pipeline
            .authorize(PermissionAuthorizationRequest {
                tool_name: "write",
                provenance: ToolProvenance::Native,
                profile: &write_profile(root.path()),
                input: &serde_json::json!({"path": target}),
                presentation_input: serde_json::json!({"path": root.path()}),
                summary_fields: Vec::new(),
                user_intent: None,
                deadline: Duration::from_secs(1),
            })
            .await
            .expect_err("Ask must deny without resolver");
        assert!(matches!(
            error,
            PermissionPipelineError::ResolverUnavailable
        ));
    }

    #[tokio::test]
    async fn resolver_error_and_panic_fail_closed_with_no_grant() {
        for resolver in [
            Arc::new(FailingResolver) as Arc<dyn ApprovalResolver>,
            Arc::new(PanickingResolver) as Arc<dyn ApprovalResolver>,
        ] {
            let root = tempfile::tempdir().expect("tempdir");
            let target = root.path().join("target.txt");
            let state = Arc::new(PermissionSessionState::new(
                PermissionEngine::with_workspace_root(root.path().to_path_buf()),
            ));
            let pipeline = PermissionPipeline::new(
                state.clone(),
                PermissionContext::new(
                    PermissionMode::Interactive,
                    InteractionCapability::Available,
                ),
                Some(resolver),
            );
            let input = serde_json::json!({"path": target, "secret": "sentinel"});
            let error = pipeline
                .authorize(PermissionAuthorizationRequest {
                    tool_name: "write",
                    provenance: ToolProvenance::Native,
                    profile: &write_profile(&target),
                    input: &input,
                    presentation_input: serde_json::json!({"path": "target.txt"}),
                    summary_fields: vec!["path".to_owned()],
                    user_intent: None,
                    deadline: Duration::from_secs(1),
                })
                .await
                .expect_err("resolver failure must deny");
            assert!(matches!(error, PermissionPipelineError::Resolver(_)));
            assert_eq!(state.grant_count().expect("grant count"), 0);
        }
    }

    #[tokio::test]
    async fn session_approval_is_admitted_once_and_reused() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target.txt");
        std::fs::write(&target, b"test").expect("fixture");
        let state = Arc::new(PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().to_path_buf()),
        ));
        let pipeline = PermissionPipeline::new(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(Arc::new(StaticResolver(ApprovalChoice::AlwaysApprove))),
        );
        let profile = write_profile(&target);
        let input = serde_json::json!({"path": target});
        pipeline
            .authorize(PermissionAuthorizationRequest {
                tool_name: "write",
                provenance: ToolProvenance::Native,
                profile: &profile,
                input: &input,
                presentation_input: input.clone(),
                summary_fields: Vec::new(),
                user_intent: None,
                deadline: Duration::from_secs(1),
            })
            .await
            .expect("first approval");
        assert_eq!(state.grant_count().expect("grant count"), 1);
        pipeline
            .authorize(PermissionAuthorizationRequest {
                tool_name: "write",
                provenance: ToolProvenance::Native,
                profile: &profile,
                input: &input,
                presentation_input: input.clone(),
                summary_fields: Vec::new(),
                user_intent: None,
                deadline: Duration::from_secs(1),
            })
            .await
            .expect("matching Session grant");
        assert_eq!(state.grant_count().expect("grant count"), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn resolver_timeout_fails_closed_without_resetting_budget() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target.txt");
        std::fs::write(&target, b"test").expect("fixture");
        let pipeline = PermissionPipeline::new(
            Arc::new(PermissionSessionState::new(
                PermissionEngine::with_workspace_root(root.path().to_path_buf()),
            )),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(Arc::new(SlowResolver)),
        );

        let error = pipeline
            .authorize(PermissionAuthorizationRequest {
                tool_name: "write",
                provenance: ToolProvenance::Native,
                profile: &write_profile(&target),
                input: &serde_json::json!({"path": target}),
                presentation_input: serde_json::json!({"path": "target.txt"}),
                summary_fields: Vec::new(),
                user_intent: None,
                deadline: Duration::from_millis(10),
            })
            .await
            .expect_err("timeout must deny");
        assert!(matches!(error, PermissionPipelineError::DeadlineExceeded));
        assert!(matches!(
            error.final_decision(),
            PermissionDecision::Deny(_)
        ));
    }

    #[tokio::test]
    async fn revision_change_during_resolution_fails_closed() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target.txt");
        std::fs::write(&target, b"test").expect("fixture");
        let state = Arc::new(PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().to_path_buf()),
        ));
        let pipeline = PermissionPipeline::new(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(Arc::new(InvalidatingResolver { state })),
        );

        let error = pipeline
            .authorize(PermissionAuthorizationRequest {
                tool_name: "write",
                provenance: ToolProvenance::Native,
                profile: &write_profile(&target),
                input: &serde_json::json!({"path": target}),
                presentation_input: serde_json::json!({"path": "target.txt"}),
                summary_fields: Vec::new(),
                user_intent: None,
                deadline: Duration::from_secs(1),
            })
            .await
            .expect_err("stale approval must deny");
        assert!(matches!(error, PermissionPipelineError::State(_)));
        assert!(matches!(
            error.final_decision(),
            PermissionDecision::Deny(_)
        ));
    }

    #[derive(Clone, Copy)]
    enum SurfaceProfile {
        Goal,
        InteractiveCli,
        InteractiveTui,
        Runtime,
        Mcp,
    }

    impl SurfaceProfile {
        fn context(self) -> PermissionContext {
            match self {
                Self::Goal | Self::InteractiveCli | Self::InteractiveTui => PermissionContext::new(
                    PermissionMode::Interactive,
                    InteractionCapability::Available,
                ),
                Self::Runtime | Self::Mcp => PermissionContext::new(
                    PermissionMode::Headless,
                    InteractionCapability::Unavailable,
                ),
            }
        }

        fn has_interaction(self) -> bool {
            matches!(
                self,
                Self::Goal | Self::InteractiveCli | Self::InteractiveTui
            )
        }

        fn provenance(self) -> ToolProvenance {
            match self {
                Self::Mcp => ToolProvenance::McpRemote {
                    server: "standalone-mcp".to_owned(),
                },
                _ => ToolProvenance::Native,
            }
        }
    }

    #[tokio::test]
    async fn i236_surface_matrix_preserves_approval_and_headless_fallback() {
        let profiles = [
            SurfaceProfile::Goal,
            SurfaceProfile::InteractiveCli,
            SurfaceProfile::InteractiveTui,
            SurfaceProfile::Runtime,
            SurfaceProfile::Mcp,
        ];

        for surface in profiles {
            let root = tempfile::tempdir().expect("tempdir");
            let target = root.path().join("new.txt");
            let state = Arc::new(PermissionSessionState::new(
                PermissionEngine::with_workspace_root(root.path().to_path_buf()),
            ));
            let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let resolver = if surface.has_interaction() {
                let lease = ManagedWorkspaceLease::new(
                    root.path(),
                    state.session_id().expect("session id").stable_id(),
                )
                .expect("lease")
                .with_atomic_create_capability(Arc::new(MatrixCapability));
                Some(Arc::new(AutoPermissionResolver::new(
                    Arc::new(MatrixAssessor {
                        calls: calls.clone(),
                    }),
                    Arc::new(StaticResolver(ApprovalChoice::Deny)),
                    lease,
                    Duration::from_secs(8),
                    AutoPermissionControl::new(true),
                )) as Arc<dyn ApprovalResolver>)
            } else {
                None
            };
            let pipeline = PermissionPipeline::new(state.clone(), surface.context(), resolver);
            let profile = write_profile(&target);
            let result = pipeline
                .authorize(PermissionAuthorizationRequest {
                    tool_name: "write",
                    provenance: surface.provenance(),
                    profile: &profile,
                    input: &serde_json::json!({
                        "path": target.display().to_string(),
                        "content": "hello"
                    }),
                    presentation_input: serde_json::json!({
                        "path": "new.txt",
                        "content": "hello"
                    }),
                    summary_fields: vec!["path".to_owned()],
                    user_intent: None,
                    deadline: Duration::from_secs(1),
                })
                .await;

            if surface.has_interaction() {
                assert!(result.is_ok(), "interactive surface must resolve Ask");
                assert_eq!(calls.load(std::sync::atomic::Ordering::Acquire), 1);
            } else {
                assert!(matches!(
                    result,
                    Err(PermissionPipelineError::ResolverUnavailable)
                ));
                assert_eq!(state.grant_count().expect("grant count"), 0);
            }
        }
    }

    #[tokio::test]
    async fn i236_hard_deny_wins_over_every_surface_resolver() {
        let profiles = [
            SurfaceProfile::Goal,
            SurfaceProfile::InteractiveCli,
            SurfaceProfile::InteractiveTui,
            SurfaceProfile::Runtime,
            SurfaceProfile::Mcp,
        ];

        for surface in profiles {
            let root = tempfile::tempdir().expect("tempdir");
            let target = root.path().join("blocked.txt");
            let engine = PermissionEngine::from_rules(vec![PermissionRule::new_nature(
                ToolNature::Write,
                Some("**/blocked.txt".to_owned()),
                Some(ResourceKind::Path),
                PermissionDecision::Deny("policy deny".to_owned()),
            )]);
            let state = Arc::new(PermissionSessionState::new(engine));
            let resolver = surface.has_interaction().then(|| {
                Arc::new(StaticResolver(ApprovalChoice::ApproveOnce)) as Arc<dyn ApprovalResolver>
            });
            let pipeline = PermissionPipeline::new(state.clone(), surface.context(), resolver);
            let profile = write_profile(&target);
            let result = pipeline
                .authorize(PermissionAuthorizationRequest {
                    tool_name: "write",
                    provenance: surface.provenance(),
                    profile: &profile,
                    input: &serde_json::json!({"path": "blocked.txt", "content": "hello"}),
                    presentation_input: serde_json::json!({"path": "blocked.txt"}),
                    summary_fields: Vec::new(),
                    user_intent: None,
                    deadline: Duration::from_secs(1),
                })
                .await;
            assert!(matches!(
                result,
                Err(PermissionPipelineError::Denied(PermissionDecision::Deny(_)))
            ));
            assert_eq!(state.grant_count().expect("grant count"), 0);
        }
    }

    #[tokio::test]
    async fn i236_auto_off_rolls_back_to_human_fallback_without_assessor_call() {
        let root = tempfile::tempdir().expect("tempdir");
        let state = Arc::new(PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().to_path_buf()),
        ));
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let control = AutoPermissionControl::new(false);
        let lease = ManagedWorkspaceLease::new(
            root.path(),
            state.session_id().expect("session id").stable_id(),
        )
        .expect("lease")
        .with_atomic_create_capability(Arc::new(MatrixCapability));
        let resolver = AutoPermissionResolver::new(
            Arc::new(MatrixAssessor {
                calls: calls.clone(),
            }),
            Arc::new(StaticResolver(ApprovalChoice::Deny)),
            lease,
            Duration::from_secs(8),
            control.clone(),
        );
        let request = matrix_approval_request(root.path(), &state, "rollback.txt");
        assert_eq!(
            resolver
                .resolve(request, Duration::from_secs(1))
                .await
                .expect("fallback"),
            ApprovalChoice::Deny
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::Acquire), 0);
        assert!(resolver.last_report().is_none());
        control.set_enabled(true);
        let request = matrix_approval_request(root.path(), &state, "enabled.txt");
        assert_eq!(
            resolver
                .resolve(request, Duration::from_secs(1))
                .await
                .expect("assessor"),
            ApprovalChoice::ApproveOnce
        );
        let report = resolver.last_report().expect("redacted report");
        assert_eq!(report.outcome, "allow_once");
        assert_eq!(calls.load(std::sync::atomic::Ordering::Acquire), 1);
    }
}
