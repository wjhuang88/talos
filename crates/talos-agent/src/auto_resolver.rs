//! Bounded, fail-closed model assistance for permission prompts.
//!
//! This module deliberately sits behind [`ApprovalResolver`].  The model can only
//! suggest an invocation-local `ApproveOnce`; policy evaluation and grant admission
//! remain owned by [`crate::permission_pipeline::PermissionPipeline`].

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use talos_core::ApprovalChoice;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::LanguageModel;
use talos_core::tool::{SharedAtomicCreateCapability, ToolNature, ToolResourceKind};

use crate::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest,
};
use talos_permission::PermissionMode;

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
    /// Permission Session identity bound to this assessment.
    pub session_id: String,
    /// Monotonic policy/mode/workspace generations bound to this assessment.
    pub revisions: [u64; 6],
    /// Permission mode at assessment time.
    pub mode: PermissionMode,
    /// Digest binding the response to this exact request.
    pub request_digest: String,
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
    ) -> Self {
        Self {
            assessor,
            fallback,
            lease,
            state: Mutex::new(CircuitState::default()),
            deadline: deadline.clamp(Duration::from_millis(1), Duration::from_secs(30)),
        }
    }

    /// Explicitly resets the circuit, equivalent to `/auto on`.
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = CircuitState::default();
        }
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
            if s.technical_failures >= 2 {
                s.open = true;
            }
        }
    }
    fn record_human(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.human_required = s.human_required.saturating_add(1);
            if s.human_required >= 3 {
                s.open = true;
            }
        }
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

fn eligible(
    request: &PermissionApprovalRequest,
    lease: &ManagedWorkspaceLease,
) -> Option<AutoPermissionRequest> {
    if request.tool_name != "write" || request.preview.facets().len() != 1 {
        return None;
    }
    let facet = &request.preview.facets()[0];
    if facet.nature != ToolNature::Write || facet.resource_kind != ToolResourceKind::Path {
        return None;
    }
    let path = request.arguments.get("path")?.as_str()?;
    let path = Path::new(path);
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
        session_id: request.binding.session_id.clone(),
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
        if self.circuit_open() {
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
            Ok(ApprovalChoice::ApproveOnce)
        } else {
            self.record_human();
            self.fallback
                .resolve(request, remaining.saturating_sub(started.elapsed()))
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCapability;

    impl talos_core::tool::AtomicCreateCapability for TestCapability {
        fn create_new(&self, _relative_path: &Path, _contents: &[u8]) -> std::io::Result<()> {
            Ok(())
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
            session_id: "session".into(),
            revisions: [0; 6],
            mode: talos_permission::PermissionMode::Interactive,
            request_digest: String::new(),
        };
        let first = digest(&request);
        request.target_label = "other.txt".into();
        assert_ne!(first, digest(&request));
    }
}
