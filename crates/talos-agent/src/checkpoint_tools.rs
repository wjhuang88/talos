//! Execution-scoped model checkpoint admission. Durable history query follows separately.

use crate::compaction::{Checkpoint, HistorySource};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use talos_core::message::Message;
use talos_core::session::CompactOutcome;
use talos_core::tool::{AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResult};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub(crate) struct Execution {
    sender: mpsc::UnboundedSender<crate::AgentActorRequest>,
    state: Arc<Mutex<State>>,
    permission: Option<Arc<talos_permission::PermissionSessionState>>,
}
#[derive(Default)]
struct State {
    current: Option<Snapshot>,
    issued: Option<Snapshot>,
    receipt: Option<(u64, oneshot::Receiver<CompactOutcome>)>,
}
#[derive(Clone)]
struct Snapshot {
    token: String,
    source: HistorySource,
    end: usize,
    refs: BTreeMap<String, String>,
    authority: PermissionLease,
}
#[derive(Clone)]
pub(crate) struct PermissionLease {
    state: Arc<talos_permission::PermissionSessionState>,
    snapshot: talos_permission::PermissionStateSnapshot,
}
impl PermissionLease {
    pub(crate) fn commit<T>(&self, commit: impl FnOnce() -> T) -> Option<T> {
        self.state
            .with_current_snapshot(self.snapshot, commit)
            .ok()
            .flatten()
    }
    pub(crate) fn valid(&self) -> bool {
        self.state
            .state_snapshot()
            .is_ok_and(|snapshot| snapshot == self.snapshot)
    }
}
tokio::task_local! {
    pub(crate) static EXECUTION: Execution;
}
impl Execution {
    pub(crate) fn new(
        sender: mpsc::UnboundedSender<crate::AgentActorRequest>,
        permission: Option<Arc<talos_permission::PermissionSessionState>>,
    ) -> Self {
        Self {
            sender,
            state: Arc::new(Mutex::new(State::default())),
            permission,
        }
    }
    pub(crate) fn refresh(&self, source: Option<&HistorySource>, messages: &[Message]) {
        let snapshot = source.and_then(|source| {
            let permission = self.permission.as_ref()?;
            let authority = PermissionLease {
                state: permission.clone(),
                snapshot: permission.state_snapshot().ok()?,
            };
            let end = crate::checkpoint_request::eligible_prefix_end(messages)?;
            if end == 0 || end > 8192 {
                return None;
            }
            let mut refs = BTreeMap::new();
            for (index, message) in messages[..end].iter().enumerate() {
                if matches!(message, Message::System { .. }) {
                    continue;
                }
                if let Some(id) =
                    source.resolve(source.generation(), source.revision(), index, message)
                {
                    refs.insert(uuid::Uuid::new_v4().to_string(), id.to_owned());
                }
            }
            (!refs.is_empty()).then(|| Snapshot {
                token: uuid::Uuid::new_v4().to_string(),
                source: source.clone(),
                end,
                refs,
                authority,
            })
        });
        if let Ok(mut state) = self.state.lock() {
            state.current = snapshot;
        }
    }
    pub(crate) async fn terminal(&self) -> Option<String> {
        let pending = self.state.lock().ok()?.receipt.take();
        let (id, receipt) = pending?;
        let outcome = receipt.await.ok();
        Some(json!({"checkpoint_operation":id,"outcome":outcome.map(|value| format!("{value:?}")).unwrap_or_else(|| "Unavailable".into())}).to_string())
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CompactInput {
    snapshot: String,
    checkpoint: Checkpoint,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct HistoryInput {
    mode: String,
}

pub(crate) struct CompactContextTool;
pub(crate) struct HistorySearchTool;

fn unavailable() -> ToolResult {
    ToolResult::error("history/checkpoint execution unavailable")
}

#[async_trait]
impl AgentTool for HistorySearchTool {
    fn name(&self) -> &str {
        "history_search"
    }
    fn description(&self) -> &str {
        "Discover a current execution checkpoint snapshot and source references. Durable search and detail are not available in this implementation stage."
    }
    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(HistoryInput)).unwrap_or_else(|_| json!({}))
    }
    fn permission_profile(&self, _: &Value) -> Vec<ToolPermissionFacet> {
        vec![ToolPermissionFacet::new(ToolNature::Read)]
    }
    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
    fn is_always_on(&self) -> bool {
        true
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let Ok(input) = serde_json::from_value::<HistoryInput>(input) else {
            return ToolResult::error("invalid history input");
        };
        if input.mode != "discover" {
            return unavailable();
        }
        let Ok(execution) = EXECUTION.try_with(Clone::clone) else {
            return unavailable();
        };
        let Ok(mut state) = execution.state.lock() else {
            return unavailable();
        };
        let Some(snapshot) = state.current.clone() else {
            return unavailable();
        };
        // This bounded discovery emits opaque references, never source contents.
        if !snapshot.authority.valid() {
            return unavailable();
        }
        let refs: Vec<_> = snapshot.refs.keys().take(20).cloned().collect();
        let result = ToolResult::success(
            json!({"snapshot":snapshot.token,"sources":refs,"complete":snapshot.refs.len() <= 20})
                .to_string(),
        );
        state.issued = Some(snapshot);
        result
    }
}

#[async_trait]
impl AgentTool for CompactContextTool {
    fn name(&self) -> &str {
        "compact_context"
    }
    fn description(&self) -> &str {
        "Submit a structured checkpoint using a discovered snapshot and sources. Admitted reserves an operation; only a later Applied receipt confirms installation after the complete tool batch."
    }
    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(CompactInput)).unwrap_or_else(|_| json!({}))
    }
    fn permission_profile(&self, _: &Value) -> Vec<ToolPermissionFacet> {
        vec![ToolPermissionFacet::new(ToolNature::Write)]
    }
    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
    fn is_always_on(&self) -> bool {
        true
    }
    async fn execute(&self, input: Value) -> ToolResult {
        if input.to_string().len() > crate::compaction::MAX_CHECKPOINT_BYTES + 256 {
            return ToolResult::error("checkpoint payload exceeds byte limit");
        }
        let Ok(input) = serde_json::from_value::<CompactInput>(input) else {
            return ToolResult::error("invalid checkpoint input");
        };
        let Ok(execution) = EXECUTION.try_with(Clone::clone) else {
            return unavailable();
        };
        let snapshot = {
            let Ok(mut state) = execution.state.lock() else {
                return unavailable();
            };
            if state.receipt.is_some() {
                return ToolResult::error("checkpoint operation busy");
            }
            let Some(issued) = state
                .issued
                .take()
                .filter(|issued| issued.token == input.snapshot)
            else {
                return ToolResult::error("stale checkpoint snapshot");
            };
            let Some(current) = &state.current else {
                return unavailable();
            };
            if !issued.authority.valid()
                || current.end < issued.end
                || !(0..issued.end).all(|i| issued.source.same_entry(&current.source, i))
            {
                return ToolResult::error("stale checkpoint source");
            }
            issued
        };
        let Ok(mut payload) = serde_json::to_value(input.checkpoint) else {
            return ToolResult::error("invalid checkpoint payload");
        };
        fn translate(value: &mut Value, refs: &BTreeMap<String, String>) -> bool {
            match value {
                Value::Object(fields) => fields.iter_mut().all(|(key, value)| {
                    if key == "sources" {
                        value.as_array_mut().is_some_and(|sources| {
                            sources.iter_mut().all(|source| {
                                let Some(id) = source.as_str().and_then(|id| refs.get(id)) else {
                                    return false;
                                };
                                *source = Value::String(id.clone());
                                true
                            })
                        })
                    } else {
                        translate(value, refs)
                    }
                }),
                Value::Array(items) => items.iter_mut().all(|item| translate(item, refs)),
                _ => true,
            }
        }
        if !translate(&mut payload, &snapshot.refs) {
            return ToolResult::error("unknown checkpoint source");
        }
        let (receipt, terminal) = oneshot::channel();
        let (response, admitted) = oneshot::channel();
        if execution
            .sender
            .send(crate::AgentActorRequest::AdmitModel {
                submission: crate::ModelBoundarySubmission {
                    checkpoint: crate::SubmittedCheckpoint {
                        payload: payload.to_string(),
                        source: snapshot.source,
                        covered_end: snapshot.end,
                    },
                    receipt,
                    authority: Some(snapshot.authority),
                },
                response,
            })
            .is_err()
        {
            return unavailable();
        }
        match tokio::time::timeout(std::time::Duration::from_secs(2), admitted).await {
            Ok(Ok(Ok(id))) => {
                let Ok(mut state) = execution.state.lock() else {
                    return unavailable();
                };
                state.receipt = Some((id, terminal));
                ToolResult::success(json!({"status":"Admitted","operation_id":id}).to_string())
            }
            _ => ToolResult::error("checkpoint admission rejected or unavailable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (
        Execution,
        mpsc::UnboundedReceiver<crate::AgentActorRequest>,
        Vec<Message>,
    ) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let permission = Arc::new(talos_permission::PermissionSessionState::new(
            talos_permission::PermissionEngine::empty(),
        ));
        let execution = Execution::new(sender, Some(permission));
        let raw: Vec<_> = (0..30)
            .map(|i| Message::User {
                content: format!("authorized source {i}"),
            })
            .collect();
        let source = HistorySource::bind(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
        )
        .expect("source");
        execution.refresh(Some(&source), &raw);
        (execution, receiver, raw)
    }
    fn submission(discovery: &Value) -> Value {
        let item = json!({"text":"retain goal", "sources":[discovery["sources"][0]]});
        json!({"snapshot":discovery["snapshot"],"checkpoint":{
            "schema_version":1,"summary":item,"goal":item,"next_step":item,
            "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
            "facts":[],"assumptions":[],"resources":[],"open_loops":[]
        }})
    }
    #[tokio::test]
    async fn discovery_is_scoped_and_permission_reset_invalidates_submission() {
        let (execution, mut receiver, _) = fixture();
        let discovered = EXECUTION
            .scope(
                execution.clone(),
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        assert!(!discovered.is_error);
        assert!(!discovered.content.contains("authorized source"));
        assert!(!discovered.content.contains("raw-"));
        let discovery: Value = serde_json::from_str(&discovered.content).expect("discovery");
        assert_eq!(discovery["sources"].as_array().expect("sources").len(), 20);
        let (other, _, _) = fixture();
        assert!(
            EXECUTION
                .scope(other, CompactContextTool.execute(submission(&discovery)))
                .await
                .is_error
        );
        execution
            .permission
            .as_ref()
            .expect("permission")
            .clear()
            .expect("reset");
        assert!(
            EXECUTION
                .scope(
                    execution,
                    CompactContextTool.execute(submission(&discovery))
                )
                .await
                .is_error
        );
        assert!(
            receiver.try_recv().is_err(),
            "stale snapshot never reaches Actor admission"
        );
        assert!(
            HistorySearchTool
                .execute(json!({"mode":"discover"}))
                .await
                .is_error
        );
    }
    #[tokio::test]
    async fn prefix_mutation_invalidates_discovered_checkpoint_before_admission() {
        let (execution, mut receiver, mut raw) = fixture();
        let discovered = EXECUTION
            .scope(
                execution.clone(),
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        let discovery: Value = serde_json::from_str(&discovered.content).expect("discovery");
        raw[0] = Message::User {
            content: "replaced source".into(),
        };
        let changed = HistorySource::bind(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
        )
        .expect("changed");
        execution.refresh(Some(&changed), &raw);
        assert!(
            EXECUTION
                .scope(
                    execution,
                    CompactContextTool.execute(submission(&discovery))
                )
                .await
                .is_error
        );
        assert!(receiver.try_recv().is_err());
    }
}
