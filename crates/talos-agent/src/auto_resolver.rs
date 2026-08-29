//! Bounded, fail-closed model assistance for permission prompts.
//!
//! This module deliberately sits behind [`ApprovalResolver`].  The model can only
//! suggest an invocation-local `ApproveOnce`; policy evaluation and grant admission
//! remain owned by [`crate::permission_pipeline::PermissionPipeline`].

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use talos_core::ApprovalChoice;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::LanguageModel;
use talos_core::tool::{
    SharedAtomicCreateCapability, ToolNature, ToolProvenance, ToolResourceKind,
};

use crate::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest,
};
use talos_permission::{InteractionCapability, PermissionMode};

/// Version of the closed evaluator wire format.
pub const AUTO_EVALUATOR_SCHEMA_VERSION: u8 = 1;

/// A typed lease proving that automatic creation is confined to one managed workspace.
#[derive(Clone)]
pub struct ManagedWorkspaceLease {
    root: PathBuf,
    session_id: String,
    atomic_create: Option<SharedAtomicCreateCapability>,
}

impl ManagedWorkspaceLease {
    /// Creates a lease. The root must be an existing directory.
    pub fn new(root: impl Into<PathBuf>, session_id: impl Into<String>) -> std::io::Result<Self> {
        let root = root.into().canonicalize()?;
        if !root.is_dir() {
            return Err(std::io::Error::other(
                "managed workspace root is not a directory",
            ));
        }
        Ok(Self {
            root,
            session_id: session_id.into(),
            atomic_create: None,
        })
    }

    /// Attaches the same host capability used by the authorized write tool.
    #[must_use]
    pub fn with_atomic_create_capability(
        mut self,
        capability: SharedAtomicCreateCapability,
    ) -> Self {
        self.atomic_create = Some(capability);
        self
    }

    /// Returns the opaque session identity (never sent to the evaluator).
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    fn allows(&self, path: &Path) -> bool {
        if self.atomic_create.is_none() {
            return false;
        }
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let Some(parent) = path.parent() else {
            return false;
        };
        let Ok(parent) = parent.canonicalize() else {
            return false;
        };
        parent.starts_with(&self.root) && !path.exists()
    }

    fn relative_label(&self, path: &Path) -> Option<String> {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let parent = absolute.parent()?.canonicalize().ok()?;
        let absolute = parent.join(absolute.file_name()?);
        absolute
            .strip_prefix(&self.root)
            .ok()
            .filter(|relative| !relative.as_os_str().is_empty())
            .map(|relative| relative.to_string_lossy().replace('\\', "/"))
    }
}

/// Closed, redacted request sent to a model assessor.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AutoPermissionRequest {
    /// Schema version.
    pub schema_version: u8,
    /// Stable tool identifier.
    pub tool: String,
    /// Stable provenance class.
    pub provenance: AutoProvenance,
    /// Fixed risk class.
    pub risk_class: AutoRiskClass,
    /// Workspace-relative target label, never an absolute path.
    pub target_label: String,
    /// Operation subtype.
    pub operation: AutoOperation,
    /// Bounded, non-secret content shape used for risk assessment.
    pub content_shape: AutoContentShape,
    /// Opaque digest binding this assessment to one Permission Session.
    pub session_binding: String,
    /// Monotonic policy/mode/workspace generations bound to this assessment.
    pub revisions: [u64; 6],
    /// Permission mode at assessment time.
    pub mode: PermissionMode,
    /// Digest binding the response to this exact request.
    pub request_digest: String,
}

/// Redacted shape of the proposed new text content.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AutoContentShape {
    /// Lowercase allowlisted file extension.
    pub extension: String,
    /// UTF-8 byte length of the proposed content.
    pub bytes: usize,
    /// Number of newline-delimited lines.
    pub lines: usize,
}

/// Closed provenance projection.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutoProvenance {
    Native,
}

/// Closed risk classes understood by the evaluator.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutoRiskClass {
    BoundedWorkspaceTextCreate,
}

/// Closed operation subtype.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutoOperation {
    CreateTextFile,
}

/// Closed model output. Unknown JSON fields are rejected during deserialization.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AutoPermissionResponse {
    /// Schema version.
    pub schema_version: u8,
    /// Request digest echoed by the model.
    pub request_digest: String,
    /// Model suggestion.
    pub decision: AutoDecision,
    /// Closed reason code.
    pub reason_code: AutoReasonCode,
    /// Confidence; only high can allow.
    pub confidence: AutoConfidence,
}

/// Model decision.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutoDecision {
    AllowOnce,
    HumanRequired,
}

/// Closed reason codes.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutoReasonCode {
    BoundedWorkspaceTextCreate,
    Uncertain,
    Malformed,
    InjectionDetected,
}

/// Closed confidence values.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutoConfidence {
    High,
    Low,
}

/// Provider-independent model assessor. Implementations must not call tools recursively.
#[async_trait]
pub trait AutoPermissionAssessor: Send + Sync {
    /// Returns one JSON response within the caller's deadline.
    async fn assess(
        &self,
        request: AutoPermissionRequest,
        remaining: Duration,
    ) -> Result<String, String>;
    /// Stable evaluator identity for audit/status surfaces.
    fn identity(&self) -> &str {
        "configured-model"
    }
}

/// Provider-backed assessor that performs one tool-free model request.
pub struct ProviderAutoPermissionAssessor {
    provider: Arc<dyn LanguageModel>,
    identity: String,
}

impl ProviderAutoPermissionAssessor {
    /// Creates an assessor using the supplied configured model.
    #[must_use]
    pub fn new(provider: Arc<dyn LanguageModel>) -> Self {
        Self {
            provider,
            identity: "configured-model".to_owned(),
        }
    }

    /// Overrides the redacted identity reported in audit/status surfaces.
    #[must_use]
    pub fn with_identity(mut self, identity: impl Into<String>) -> Self {
        self.identity = identity.into();
        self
    }
}

#[async_trait]
impl AutoPermissionAssessor for ProviderAutoPermissionAssessor {
    async fn assess(
        &self,
        request: AutoPermissionRequest,
        remaining: Duration,
    ) -> Result<String, String> {
        let payload = serde_json::to_string(&request).map_err(|error| error.to_string())?;
        let messages = vec![
            Message::System {
                content: "You are a permission risk assessor. Return only the closed JSON response schema; never request tools, infer missing authority, or include explanation.".to_owned(),
                cache_markers: Vec::new(),
            },
            Message::User {
                content: format!("Assess this redacted request and return JSON only:\n{payload}"),
            },
        ];
        let mut events = self
            .provider
            .stream(&messages)
            .await
            .map_err(|error| error.to_string())?;
        let deadline = tokio::time::sleep(remaining);
        tokio::pin!(deadline);
        let mut output = String::new();
        loop {
            tokio::select! {
                _ = &mut deadline => return Err("model assessment deadline exceeded".to_owned()),
                event = events.recv() => match event {
                    Some(AgentEvent::TextDelta { delta }) => output.push_str(&delta),
                    Some(AgentEvent::ToolCall { .. }) => return Err("tool use is forbidden in auto assessment".to_owned()),
                    Some(AgentEvent::Error { message }) => return Err(message),
                    Some(AgentEvent::TurnEnd { .. }) | None => break,
                    Some(_) => {}
                }
            }
        }
        if output.trim().is_empty() {
            return Err("model assessment returned no JSON".to_owned());
        }
        Ok(output)
    }

    fn identity(&self) -> &str {
        &self.identity
    }
}

#[derive(Debug, Default)]
struct CircuitState {
    technical_failures: u8,
    human_required: u8,
    open: bool,
}

/// Redacted outcome record retained for status and host-owned audit sinks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoDecisionReport {
    /// Final bounded outcome.
    pub outcome: String,
    /// Stable reason classification.
    pub reason: String,
    /// Evaluator identity.
    pub evaluator: String,
    /// Request digest, never raw arguments.
    pub request_digest: String,
}

/// Session-scoped control shared by the conversation UI and its permission resolver.
///
/// The control carries a reset epoch so enabling auto assistance also clears any
/// circuit state accumulated before it was disabled.
#[derive(Clone)]
pub struct AutoPermissionControl {
    enabled: Arc<AtomicBool>,
    reset_epoch: Arc<AtomicU64>,
}

impl AutoPermissionControl {
    /// Creates a control with the supplied configuration default.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(enabled)),
            reset_epoch: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Returns whether model assistance is enabled for this session.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Changes the session mode. Enabling starts a fresh circuit epoch.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
        if enabled {
            self.reset_epoch.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn reset_epoch(&self) -> u64 {
        self.reset_epoch.load(Ordering::Acquire)
    }
}

/// Redacted circuit snapshot suitable for status/audit surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoCircuitStatus {
    /// Whether evaluation is currently bypassed.
    pub open: bool,
    /// Consecutive technical/validation failures.
    pub technical_failures: u8,
    /// Consecutive human-required outcomes.
    pub human_required: u8,
}

/// Model-assisted resolver with deterministic eligibility, timeout and circuit breaker.
pub struct AutoPermissionResolver {
    assessor: Arc<dyn AutoPermissionAssessor>,
    fallback: Arc<dyn ApprovalResolver>,
    lease: ManagedWorkspaceLease,
    state: Mutex<CircuitState>,
    last_report: Mutex<Option<AutoDecisionReport>>,
    control: AutoPermissionControl,
    observed_reset_epoch: AtomicU64,
    deadline: Duration,
}

impl AutoPermissionResolver {
    /// Builds a resolver. Deadlines are clamped to ADR-064's eight/ thirty second bounds.
    #[must_use]
    pub fn new(
        assessor: Arc<dyn AutoPermissionAssessor>,
        fallback: Arc<dyn ApprovalResolver>,
        lease: ManagedWorkspaceLease,
        deadline: Duration,
        control: AutoPermissionControl,
    ) -> Self {
        Self {
            assessor,
            fallback,
            lease,
            state: Mutex::new(CircuitState::default()),
            last_report: Mutex::new(None),
            observed_reset_epoch: AtomicU64::new(control.reset_epoch()),
            control,
            deadline: deadline.clamp(Duration::from_millis(1), Duration::from_secs(30)),
        }
    }

    /// Explicitly resets the circuit, equivalent to `/auto on`.
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = CircuitState::default();
        }
        self.observed_reset_epoch
            .store(self.control.reset_epoch(), Ordering::Release);
    }

    /// Returns a redacted snapshot of circuit state.
    #[must_use]
    pub fn circuit_status(&self) -> AutoCircuitStatus {
        self.state
            .lock()
            .map(|state| AutoCircuitStatus {
                open: state.open,
                technical_failures: state.technical_failures,
                human_required: state.human_required,
            })
            .unwrap_or(AutoCircuitStatus {
                open: true,
                technical_failures: 0,
                human_required: 0,
            })
    }

    fn circuit_open(&self) -> bool {
        self.state.lock().map(|s| s.open).unwrap_or(true)
    }
    fn record_failure(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.technical_failures = s.technical_failures.saturating_add(1);
            s.human_required = 0;
            if s.technical_failures >= 2 {
                s.open = true;
            }
        }
    }
    fn record_human(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.human_required = s.human_required.saturating_add(1);
            s.technical_failures = 0;
            if s.human_required >= 3 {
                s.open = true;
            }
        }
    }

    fn record_success(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.technical_failures = 0;
            s.human_required = 0;
        }
    }

    fn report(&self, report: AutoDecisionReport) {
        if let Ok(mut slot) = self.last_report.lock() {
            *slot = Some(report);
        }
    }

    /// Returns the latest redacted decision report, if one was produced.
    #[must_use]
    pub fn last_report(&self) -> Option<AutoDecisionReport> {
        self.last_report
            .lock()
            .ok()
            .and_then(|report| report.clone())
    }

    /// Enables or disables model assistance for the current session.
    pub fn set_enabled(&self, enabled: bool) {
        self.control.set_enabled(enabled);
    }

    fn sync_reset(&self) {
        let epoch = self.control.reset_epoch();
        if self.observed_reset_epoch.load(Ordering::Acquire) == epoch {
            return;
        }
        if let Ok(mut state) = self.state.lock() {
            *state = CircuitState::default();
        }
        self.observed_reset_epoch.store(epoch, Ordering::Release);
    }
}

fn digest(request: &AutoPermissionRequest) -> String {
    let mut value = serde_json::to_value(request).unwrap_or_else(|_| serde_json::json!({}));
    if let Some(object) = value.as_object_mut() {
        object.remove("request_digest");
    }
    let encoded = serde_json::to_vec(&value).unwrap_or_default();
    let digest = Sha256::digest(encoded);
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("sha256:{hex}")
}

fn session_binding(session_id: &str) -> String {
    let digest = Sha256::digest(session_id.as_bytes());
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("sha256:{hex}")
}

fn eligible(
    request: &PermissionApprovalRequest,
    lease: &ManagedWorkspaceLease,
) -> Option<AutoPermissionRequest> {
    if request.tool_name != "write"
        || request.provenance != ToolProvenance::Native
        || request.binding.mode != PermissionMode::Interactive
        || request.binding.interaction != InteractionCapability::Available
        || request.preview.facets().len() != 1
    {
        return None;
    }
    let facet = &request.preview.facets()[0];
    if facet.nature != ToolNature::Write || facet.resource_kind != ToolResourceKind::Path {
        return None;
    }
    let path = request.arguments.get("path")?.as_str()?;
    let path = Path::new(path);
    if path.is_absolute()
        || path.components().count() != 1
        || !path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return None;
    }
    if lease.session_id() != request.binding.session_id {
        return None;
    }
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "txt" | "md" | "markdown" | "json" | "toml" | "yaml" | "yml"
    ) {
        return None;
    }
    let content = request.arguments.get("content")?.as_str()?;
    if content.len() > 64 * 1024 || content.as_bytes().contains(&0) {
        return None;
    }
    if !lease.allows(path) {
        return None;
    }
    let target_label = lease.relative_label(path)?;
    let mut result = AutoPermissionRequest {
        schema_version: AUTO_EVALUATOR_SCHEMA_VERSION,
        tool: "write".into(),
        provenance: AutoProvenance::Native,
        risk_class: AutoRiskClass::BoundedWorkspaceTextCreate,
        target_label,
        operation: AutoOperation::CreateTextFile,
        content_shape: AutoContentShape {
            extension,
            bytes: content.len(),
            lines: content.lines().count().max(1),
        },
        session_binding: session_binding(&request.binding.session_id),
        revisions: request.binding.revisions,
        mode: request.binding.mode,
        request_digest: String::new(),
    };
    result.request_digest = digest(&result);
    Some(result)
}

#[async_trait]
impl ApprovalResolver for AutoPermissionResolver {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        remaining: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        self.sync_reset();
        if !self.control.is_enabled() || self.circuit_open() {
            return self.fallback.resolve(request, remaining).await;
        }
        let Some(evaluator_request) = eligible(&request, &self.lease) else {
            return self.fallback.resolve(request, remaining).await;
        };
        let budget = remaining.min(self.deadline);
        let started = Instant::now();
        let raw = match tokio::time::timeout(
            budget,
            self.assessor.assess(evaluator_request.clone(), budget),
        )
        .await
        {
            Ok(Ok(raw)) => raw,
            _ => {
                self.record_failure();
                self.report(AutoDecisionReport {
                    outcome: "human_required".into(),
                    reason: "technical_failure".into(),
                    evaluator: self.assessor.identity().into(),
                    request_digest: evaluator_request.request_digest.clone(),
                });
                return self
                    .fallback
                    .resolve(request, remaining.saturating_sub(started.elapsed()))
                    .await;
            }
        };
        let response: AutoPermissionResponse = match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(_) => {
                self.record_failure();
                self.report(AutoDecisionReport {
                    outcome: "human_required".into(),
                    reason: "malformed_output".into(),
                    evaluator: self.assessor.identity().into(),
                    request_digest: evaluator_request.request_digest.clone(),
                });
                return self
                    .fallback
                    .resolve(request, remaining.saturating_sub(started.elapsed()))
                    .await;
            }
        };
        let valid = response.schema_version == AUTO_EVALUATOR_SCHEMA_VERSION
            && response.request_digest == evaluator_request.request_digest
            && response.decision == AutoDecision::AllowOnce
            && response.reason_code == AutoReasonCode::BoundedWorkspaceTextCreate
            && response.confidence == AutoConfidence::High;
        if valid {
            self.record_success();
            self.report(AutoDecisionReport {
                outcome: "allow_once".into(),
                reason: "bounded_workspace_text_create".into(),
                evaluator: self.assessor.identity().into(),
                request_digest: evaluator_request.request_digest.clone(),
            });
            Ok(ApprovalChoice::ApproveOnce)
        } else {
            self.record_human();
            self.report(AutoDecisionReport {
                outcome: "human_required".into(),
                reason: "validation_failed".into(),
                evaluator: self.assessor.identity().into(),
                request_digest: evaluator_request.request_digest.clone(),
            });
            self.fallback
                .resolve(request, remaining.saturating_sub(started.elapsed()))
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    use crate::permission_pipeline::PermissionBinding;
    use async_trait::async_trait;
    use talos_core::tool::ToolPermissionFacet;
    use talos_permission::{
        PermissionContext, PermissionEngine, PermissionInvocation, PermissionRequest,
        PermissionSessionState,
    };

    struct TestCapability;

    impl talos_core::tool::AtomicCreateCapability for TestCapability {
        fn create_new(&self, _relative_path: &Path, _contents: &[u8]) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct CountingAssessor {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl AutoPermissionAssessor for CountingAssessor {
        async fn assess(
            &self,
            request: AutoPermissionRequest,
            _remaining: Duration,
        ) -> Result<String, String> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            Ok(format!(
                "{{\"schema_version\":1,\"request_digest\":\"{}\",\"decision\":\"allow_once\",\"reason_code\":\"bounded_workspace_text_create\",\"confidence\":\"high\"}}",
                request.request_digest
            ))
        }
    }

    struct DenyFallback;

    #[async_trait]
    impl ApprovalResolver for DenyFallback {
        async fn resolve(
            &self,
            _request: PermissionApprovalRequest,
            _remaining: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            Ok(ApprovalChoice::Deny)
        }
    }

    fn approval_request(
        root: &std::path::Path,
        state: &PermissionSessionState,
        path: &str,
    ) -> PermissionApprovalRequest {
        let input = serde_json::json!({"path": path, "content": "hello"});
        let target_text = root.join(path).display().to_string();
        let profile = [ToolPermissionFacet::with_resource(
            ToolNature::Write,
            target_text,
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
            panic!("write should require approval")
        };
        PermissionApprovalRequest {
            tool_name: "write".to_owned(),
            provenance: ToolProvenance::Native,
            arguments: input,
            summary_fields: vec!["path".to_owned()],
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

    #[test]
    fn response_schema_rejects_unknown_fields() {
        let result = serde_json::from_str::<AutoPermissionResponse>(
            r#"{"schema_version":1,"request_digest":"sha256:x","decision":"allow_once","reason_code":"bounded_workspace_text_create","confidence":"high","extra":true}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn lease_only_accepts_absent_descendants_and_redacts_absolute_paths() {
        let root = tempfile::tempdir().expect("root");
        let lease = ManagedWorkspaceLease::new(root.path(), "session")
            .expect("lease")
            .with_atomic_create_capability(Arc::new(TestCapability));
        let absolute = root.path().join("new.txt");
        assert!(lease.allows(&absolute));
        assert_eq!(lease.relative_label(&absolute).as_deref(), Some("new.txt"));
        std::fs::write(&absolute, b"existing").expect("fixture");
        assert!(!lease.allows(&absolute));
    }

    #[test]
    fn request_digest_is_stable_and_binds_payload() {
        let mut request = AutoPermissionRequest {
            schema_version: AUTO_EVALUATOR_SCHEMA_VERSION,
            tool: "write".into(),
            provenance: AutoProvenance::Native,
            risk_class: AutoRiskClass::BoundedWorkspaceTextCreate,
            target_label: "new.txt".into(),
            operation: AutoOperation::CreateTextFile,
            content_shape: AutoContentShape {
                extension: "txt".into(),
                bytes: 0,
                lines: 1,
            },
            session_binding: session_binding("session"),
            revisions: [0; 6],
            mode: talos_permission::PermissionMode::Interactive,
            request_digest: String::new(),
        };
        let first = digest(&request);
        request.target_label = "other.txt".into();
        assert_ne!(first, digest(&request));
    }

    #[test]
    fn enabling_control_advances_reset_epoch() {
        let control = AutoPermissionControl::new(false);
        assert!(!control.is_enabled());
        let first = control.reset_epoch();
        control.set_enabled(true);
        assert!(control.is_enabled());
        assert!(control.reset_epoch() > first);
        control.set_enabled(false);
        assert!(!control.is_enabled());
    }

    #[tokio::test]
    async fn disabled_control_bypasses_assessor_and_enabling_resets_circuit() {
        let root = tempfile::tempdir().expect("root");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let calls = Arc::new(AtomicUsize::new(0));
        let control = AutoPermissionControl::new(false);
        let lease =
            ManagedWorkspaceLease::new(root.path(), state.session_id().unwrap().stable_id())
                .expect("lease")
                .with_atomic_create_capability(Arc::new(TestCapability));
        let resolver = AutoPermissionResolver::new(
            Arc::new(CountingAssessor {
                calls: calls.clone(),
            }),
            Arc::new(DenyFallback),
            lease,
            Duration::from_secs(8),
            control.clone(),
        );

        let request = approval_request(root.path(), &state, "new.txt");
        assert_eq!(
            resolver
                .resolve(request, Duration::from_secs(1))
                .await
                .unwrap(),
            ApprovalChoice::Deny
        );
        assert_eq!(calls.load(Ordering::Acquire), 0);

        control.set_enabled(true);
        let request = approval_request(root.path(), &state, "other.txt");
        assert_eq!(
            resolver
                .resolve(request, Duration::from_secs(1))
                .await
                .unwrap(),
            ApprovalChoice::ApproveOnce
        );
        assert_eq!(calls.load(Ordering::Acquire), 1);
        assert_eq!(resolver.circuit_status().technical_failures, 0);
    }
}
