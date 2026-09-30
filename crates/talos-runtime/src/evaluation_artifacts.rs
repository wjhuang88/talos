//! Session-owned evidence from successful permission-gated file operations.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use sha2::Digest;
use talos_agent::evaluator::ArtifactObservation;
use talos_core::evaluation::{EvaluationSubject, EvidenceRef};
use talos_core::message::ToolCall;
use talos_permission::PermissionDecision;
use talos_plugin::{HookContext, HookEvent, HookEventKind, HookHandler, HookResult};

const MAX_ARTIFACTS: usize = 32;
const MAX_FILE_BYTES: u64 = 64 * 1024;
const MAX_TOTAL_BYTES: usize = 256 * 1024;

#[derive(Clone, PartialEq, Eq)]
struct Operation {
    turn_id: u64,
    call_id: String,
    name: String,
    path: String,
}

#[derive(Default)]
struct RegistryState {
    proposed: BTreeMap<(u64, String), (Operation, bool)>,
    completed: BTreeMap<String, Operation>,
    unavailable: bool,
}

/// Ephemeral Runtime evidence registry for permission-gated successful file operations.
///
/// Only matching write/edit/delete hook sequences register artifacts. Capture reads those
/// paths afresh through a pinned workspace capability; it never uploads unrelated files.
///
/// The embedding host must register the trusted built-in implementations under these names
/// and authorize disclosure to its evaluator. Tool names are not an authentication boundary:
/// do not attach this registry to a composition that lets plugins replace those implementations.
pub struct RuntimeArtifactEvidenceRegistry {
    session_id: uuid::Uuid,
    root_path: PathBuf,
    root: Result<cap_std::fs::Dir, String>,
    state: Mutex<RegistryState>,
    capture_slot: Arc<tokio::sync::Semaphore>,
    pub(crate) authority: uuid::Uuid,
}

impl RuntimeArtifactEvidenceRegistry {
    /// Pin the workspace authority and bind future observations to a Runtime session.
    pub fn new(session_id: uuid::Uuid, root: impl Into<PathBuf>) -> Self {
        let root_path = root.into();
        let root = cap_std::fs::Dir::open_ambient_dir(&root_path, cap_std::ambient_authority())
            .map_err(|error| format!("artifact workspace unavailable: {error}"));
        Self {
            session_id,
            root_path: std::fs::canonicalize(&root_path).unwrap_or(root_path),
            root,
            state: Mutex::new(RegistryState::default()),
            capture_slot: Arc::new(tokio::sync::Semaphore::new(1)),
            authority: uuid::Uuid::new_v4(),
        }
    }

    pub(crate) fn from_source(source: &super::RuntimeEvaluationEvidenceSource) -> Self {
        Self {
            session_id: source.session_id,
            root_path: source.workspace_root.clone(),
            root: source
                .workspace_directory
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|root| root.try_clone().map_err(|error| error.to_string())),
            state: Mutex::new(RegistryState::default()),
            capture_slot: source.capture_slots.clone(),
            authority: source.authority,
        }
    }

    fn operation(&self, ctx: &HookContext, call: &ToolCall) -> Option<Operation> {
        if !matches!(call.name.as_str(), "write" | "edit" | "delete")
            || call.id.is_empty()
            || call.id.len() > 256
        {
            return None;
        }
        let requested = Path::new(call.input.get("path")?.as_str()?);
        let relative = if requested.is_absolute() {
            requested.strip_prefix(&self.root_path).ok()?
        } else {
            requested
        };
        let mut path = PathBuf::new();
        for component in relative.components() {
            match component {
                Component::Normal(name) => {
                    if matches!(name.to_str(), Some(".git" | "target" | "node_modules")) {
                        return None;
                    }
                    path.push(name);
                }
                Component::CurDir => {}
                _ => return None,
            }
        }
        let path = path.to_str()?.replace(std::path::MAIN_SEPARATOR, "/");
        if path.is_empty() || path.len() > 4096 {
            return None;
        }
        Some(Operation {
            turn_id: ctx.turn_id.0,
            call_id: call.id.clone(),
            name: call.name.clone(),
            path,
        })
    }

    /// Capture bounded UTF-8 observations of successfully executed artifact paths.
    ///
    /// Missing files are accepted only for successful delete operations. Errors reject
    /// the whole capture; callers must not reinterpret missing evidence as success.
    pub fn capture(&self, subject: EvaluationSubject) -> Result<Vec<ArtifactObservation>, String> {
        subject.validate().map_err(|error| error.to_string())?;
        let operations = {
            let state = self
                .state
                .lock()
                .map_err(|_| "artifact registry poisoned")?;
            if state.unavailable {
                return Err("artifact registry bound exceeded".into());
            }
            state.completed.values().cloned().collect::<Vec<_>>()
        };
        let root = self.root.as_ref().map_err(Clone::clone)?;
        super::verify_workspace_root(root, &self.root_path)?;
        let mut total = 0usize;
        let mut observations = Vec::new();
        for operation in operations {
            let content = read_artifact(root, &operation.path, operation.name == "delete")?;
            total = total.saturating_add(content.as_ref().map_or(0, String::len));
            if total > MAX_TOTAL_BYTES {
                return Err("artifact aggregate byte bound exceeded".into());
            }
            let encoded = serde_json::to_vec(&(
                subject,
                self.session_id,
                operation.turn_id,
                &operation.call_id,
                &operation.name,
                &operation.path,
                &content,
            ))
            .map_err(|error| error.to_string())?;
            let digest = sha2::Sha256::digest(encoded);
            let record_digest = digest.iter().map(|byte| format!("{byte:02x}")).collect();
            let id = uuid::Uuid::from_bytes(digest[..16].try_into().map_err(|_| "invalid digest")?);
            observations.push(ArtifactObservation {
                evidence: EvidenceRef {
                    id,
                    kind: "runtime-artifact-observation-v1".into(),
                },
                subject,
                session_id: self.session_id,
                turn_id: operation.turn_id,
                call_id: operation.call_id,
                relative_path: operation.path,
                content,
                record_digest,
            });
        }
        super::verify_workspace_root(root, &self.root_path)?;
        Ok(observations)
    }

    /// Isolate filesystem work with a deadline and at most one retained blocking worker.
    pub async fn capture_async(
        self: &Arc<Self>,
        subject: EvaluationSubject,
    ) -> Result<Vec<ArtifactObservation>, String> {
        let permit = self
            .capture_slot
            .clone()
            .try_acquire_owned()
            .map_err(|_| "artifact capture already running")?;
        let registry = self.clone();
        let worker = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            registry.capture(subject)
        });
        tokio::time::timeout(Duration::from_secs(5), worker)
            .await
            .map_err(|_| "artifact capture deadline exceeded")?
            .map_err(|error| format!("artifact capture failed: {error}"))?
    }
}

fn read_artifact(
    root: &cap_std::fs::Dir,
    path: &str,
    deleted: bool,
) -> Result<Option<String>, String> {
    let path = Path::new(path);
    let mut directory = root.try_clone().map_err(|error| error.to_string())?;
    for component in path.parent().ok_or("artifact has no parent")?.components() {
        let Component::Normal(name) = component else {
            return Err("invalid artifact path".into());
        };
        directory = directory
            .open_dir_nofollow(name)
            .map_err(|error| error.to_string())?;
    }
    let name = path.file_name().ok_or("artifact has no file name")?;
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let file = match directory.open_with(name, &options) {
        Ok(file) => file,
        Err(error) if deleted && error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("artifact cannot be opened: {error}")),
    };
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err("artifact is not a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("artifact byte bound exceeded".into());
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "artifact is not UTF-8".into())
}

#[async_trait]
impl HookHandler for RuntimeArtifactEvidenceRegistry {
    fn name(&self) -> &str {
        "runtime-artifact-evidence"
    }
    fn subscribed(&self) -> &'static [HookEventKind] {
        &[
            HookEventKind::BeforeToolCall,
            HookEventKind::AfterPermissionCheck,
            HookEventKind::AfterToolCall,
            HookEventKind::TurnComplete,
        ]
    }
    async fn on_event(&self, ctx: &HookContext, event: &mut HookEvent<'_>) -> HookResult {
        let Ok(mut state) = self.state.lock() else {
            return HookResult::Continue;
        };
        match event {
            HookEvent::BeforeToolCall { call } => {
                if let Some(operation) = self.operation(ctx, call) {
                    if state.proposed.len() >= MAX_ARTIFACTS {
                        state.unavailable = true;
                    } else {
                        state
                            .proposed
                            .insert((ctx.turn_id.0, call.id.clone()), (operation, false));
                    }
                }
            }
            HookEvent::AfterPermissionCheck { call, decision } => {
                let key = (ctx.turn_id.0, call.id.clone());
                let matches = state
                    .proposed
                    .get(&key)
                    // Permission hooks deliberately redact input values. Correlate only
                    // immutable call identity here; the successful execution event below
                    // must still match the original operation and path in full.
                    .is_some_and(|(old, _)| old.name == call.name);
                if matches && *decision == PermissionDecision::Allow {
                    if let Some((_, allowed)) = state.proposed.get_mut(&key) {
                        *allowed = true;
                    }
                } else {
                    state.proposed.remove(&key);
                }
            }
            HookEvent::AfterToolCall { call, result } => {
                if let Some((operation, allowed)) =
                    state.proposed.remove(&(ctx.turn_id.0, call.id.clone()))
                    && allowed
                    && !result.is_error
                    && self.operation(ctx, call).as_ref() == Some(&operation)
                {
                    if state.completed.len() >= MAX_ARTIFACTS
                        && !state.completed.contains_key(&operation.path)
                    {
                        state.unavailable = true;
                    } else {
                        state.completed.insert(operation.path.clone(), operation);
                    }
                }
            }
            HookEvent::TurnComplete { .. } => {
                state.proposed.retain(|(turn, _), _| *turn != ctx.turn_id.0);
            }
            _ => {}
        }
        HookResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::ToolResult;

    fn subject() -> EvaluationSubject {
        use talos_core::work::{WorkIdentity, WorkKind};
        EvaluationSubject {
            mission: WorkIdentity {
                id: uuid::Uuid::new_v4(),
                kind: WorkKind::Mission,
                revision: 1,
            },
            goal: WorkIdentity {
                id: uuid::Uuid::new_v4(),
                kind: WorkKind::Goal,
                revision: 1,
            },
            workspace: talos_core::evaluation::WorkspaceRevision {
                id: uuid::Uuid::new_v4(),
                revision: 1,
            },
        }
    }

    fn call(name: &str, path: &str) -> ToolCall {
        ToolCall {
            id: "call-1".into(),
            name: name.into(),
            input: serde_json::json!({"path":path}),
        }
    }

    async fn sequence(
        registry: &RuntimeArtifactEvidenceRegistry,
        root: &Path,
        before: &ToolCall,
        after: &ToolCall,
        decision: PermissionDecision,
        failed: bool,
    ) {
        let ctx = HookContext::new(talos_plugin::TurnId(1), root.to_path_buf());
        registry
            .on_event(&ctx, &mut HookEvent::BeforeToolCall { call: before })
            .await;
        registry
            .on_event(
                &ctx,
                &mut HookEvent::AfterPermissionCheck {
                    call: before,
                    decision,
                },
            )
            .await;
        let result = if failed {
            ToolResult::error("failed")
        } else {
            ToolResult::success("ok")
        };
        registry
            .on_event(
                &ctx,
                &mut HookEvent::AfterToolCall {
                    call: after,
                    result: &result,
                },
            )
            .await;
    }

    #[tokio::test]
    async fn only_matching_allowed_successes_register() {
        for (decision, failed, changed) in [
            (PermissionDecision::Deny("denied".into()), false, false),
            (PermissionDecision::Allow, true, false),
            (PermissionDecision::Allow, false, true),
        ] {
            let root = tempfile::tempdir().expect("root");
            let registry = RuntimeArtifactEvidenceRegistry::new(uuid::Uuid::new_v4(), root.path());
            sequence(
                &registry,
                root.path(),
                &call("write", "a"),
                &call("write", if changed { "b" } else { "a" }),
                decision,
                failed,
            )
            .await;
            assert!(registry.capture(subject()).expect("capture").is_empty());
        }
    }

    #[tokio::test]
    async fn successful_write_is_fresh_and_delete_proves_absence() {
        let root = tempfile::tempdir().expect("root");
        let registry = RuntimeArtifactEvidenceRegistry::new(uuid::Uuid::new_v4(), root.path());
        std::fs::write(root.path().join("a"), "first").expect("fixture");
        let write = call("write", "a");
        sequence(
            &registry,
            root.path(),
            &write,
            &write,
            PermissionDecision::Allow,
            false,
        )
        .await;
        std::fs::write(root.path().join("a"), "fresh").expect("updated fixture");
        assert_eq!(
            registry.capture(subject()).expect("capture")[0]
                .content
                .as_deref(),
            Some("fresh")
        );
        for content in [
            "",
            "单行",
            "single\n",
            "single\r\n",
            "first\nsecond\n",
            " trailing \n",
        ] {
            std::fs::write(root.path().join("a"), content).expect("exact content fixture");
            let observations = registry.capture(subject()).expect("complete capture");
            assert_eq!(observations[0].content.as_deref(), Some(content));
        }
        std::fs::remove_file(root.path().join("a")).expect("remove fixture");
        assert!(registry.capture(subject()).is_err());
        let delete = call("delete", "a");
        sequence(
            &registry,
            root.path(),
            &delete,
            &delete,
            PermissionDecision::Allow,
            false,
        )
        .await;
        assert!(
            registry.capture(subject()).expect("absence")[0]
                .content
                .is_none()
        );
    }

    #[tokio::test]
    async fn oversized_and_non_utf8_artifacts_fail_closed() {
        let root = tempfile::tempdir().expect("root");
        let registry = RuntimeArtifactEvidenceRegistry::new(uuid::Uuid::new_v4(), root.path());
        let write = call("write", "a");
        sequence(
            &registry,
            root.path(),
            &write,
            &write,
            PermissionDecision::Allow,
            false,
        )
        .await;
        std::fs::write(root.path().join("a"), vec![0; MAX_FILE_BYTES as usize + 1])
            .expect("fixture");
        assert!(registry.capture(subject()).is_err());
        std::fs::write(root.path().join("a"), [0xff]).expect("fixture");
        assert!(registry.capture(subject()).is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn symlink_substitution_never_reads_outside() {
        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::tempdir().expect("outside");
        std::fs::write(outside.path().join("a"), "private").expect("fixture");
        let registry = RuntimeArtifactEvidenceRegistry::new(uuid::Uuid::new_v4(), root.path());
        let write = call("write", "directory/a");
        sequence(
            &registry,
            root.path(),
            &write,
            &write,
            PermissionDecision::Allow,
            false,
        )
        .await;
        std::os::unix::fs::symlink(outside.path(), root.path().join("directory")).expect("symlink");
        assert!(registry.capture(subject()).is_err());
    }
}
