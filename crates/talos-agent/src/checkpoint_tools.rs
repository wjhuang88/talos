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
    cursor: Option<(String, usize)>,
    receipt: Option<(u64, oneshot::Receiver<CompactOutcome>)>,
}
#[derive(Clone)]
struct Snapshot {
    token: String,
    source: HistorySource,
    end: usize,
    refs: BTreeMap<String, String>,
    order: Vec<String>,
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
            let mut order = Vec::new();
            for (index, message) in messages[..end].iter().enumerate() {
                if matches!(message, Message::System { .. }) {
                    continue;
                }
                if let Some(id) =
                    source.resolve(source.generation(), source.revision(), index, message)
                {
                    let reference = uuid::Uuid::new_v4().to_string();
                    refs.insert(reference.clone(), id.to_owned());
                    order.push(reference);
                }
            }
            (!refs.is_empty()).then(|| Snapshot {
                token: uuid::Uuid::new_v4().to_string(),
                source: source.clone(),
                end,
                refs,
                order,
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
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    snapshot: Option<String>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    reference: Option<String>,
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
        "Discover a checkpoint snapshot, search permitted durable history, or read a returned reference. Search/detail require the discovered snapshot and a prepared durable host index; unsupported storage returns Unavailable."
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
        let Ok(execution) = EXECUTION.try_with(Clone::clone) else {
            return unavailable();
        };
        if matches!(input.mode.as_str(), "search" | "detail") {
            if input
                .cursor
                .as_ref()
                .is_some_and(|cursor| cursor.len() > 128)
                || input.query.as_ref().is_some_and(|query| query.len() > 1024)
                || input
                    .reference
                    .as_ref()
                    .is_some_and(|reference| reference.len() > 128)
                || (input.mode == "search" && (input.query.is_none() || input.reference.is_some()))
                || (input.mode == "detail"
                    && (input.reference.is_none()
                        || input.query.is_some()
                        || input.cursor.is_some()))
            {
                return ToolResult::error("invalid history query bounds");
            }
            let authority = {
                let Ok(state) = execution.state.lock() else {
                    return unavailable();
                };
                let Some(snapshot) = &state.issued else {
                    return unavailable();
                };
                let Some(current) = &state.current else {
                    return unavailable();
                };
                if input.snapshot.as_deref() != Some(snapshot.token.as_str())
                    || current.end < snapshot.end
                    || !(0..snapshot.end).all(|i| snapshot.source.same_entry(&current.source, i))
                {
                    return ToolResult::error("stale history snapshot");
                }
                snapshot.authority.clone()
            };
            if !authority.valid() {
                return unavailable();
            }
            let result = crate::checkpoint_history::HISTORY
                .try_with(|history| {
                    history.as_ref()?.query(
                        input.query.as_deref(),
                        input.reference.as_deref(),
                        input.cursor.as_deref(),
                    )
                })
                .ok()
                .flatten();
            return authority
                .commit(|| {
                    if !crate::checkpoint_history::HISTORY
                        .try_with(|history| {
                            history.as_ref().is_some_and(|history| !history.cancelled())
                        })
                        .unwrap_or(false)
                    {
                        return unavailable();
                    }
                    result
                        .map(|result| ToolResult::success(result.to_string()))
                        .unwrap_or_else(|| ToolResult::error("history recovery Unavailable"))
                })
                .unwrap_or_else(unavailable);
        }
        if input.mode != "discover"
            || input.query.is_some()
            || input.reference.is_some()
            || input.snapshot.is_some()
        {
            return ToolResult::error("invalid discovery input");
        }
        let Ok(mut state) = execution.state.lock() else {
            return unavailable();
        };
        let rediscover = input.cursor.is_none();
        let (snapshot, start) = if let Some(cursor) = input.cursor {
            let Some((expected, start)) = state.cursor.as_ref() else {
                return ToolResult::error("stale discovery cursor");
            };
            if cursor != *expected {
                return ToolResult::error("stale discovery cursor");
            }
            let Some(snapshot) = state.issued.clone() else {
                return unavailable();
            };
            let Some(current) = &state.current else {
                return unavailable();
            };
            if current.end < snapshot.end
                || !(0..snapshot.end).all(|i| snapshot.source.same_entry(&current.source, i))
            {
                return ToolResult::error("stale discovery source");
            }
            (snapshot, *start)
        } else {
            let Some(snapshot) = state.current.clone() else {
                return unavailable();
            };
            (snapshot, 0)
        };
        if !snapshot.authority.valid() {
            return unavailable();
        }
        if rediscover {
            let reset = crate::checkpoint_history::HISTORY
                .try_with(|history| history.as_ref().map(|history| history.reset_queries()))
                .ok()
                .flatten();
            if matches!(reset, Some(None)) {
                return unavailable();
            }
        }
        let refs: Vec<_> = snapshot
            .order
            .iter()
            .skip(start)
            .take(20)
            .cloned()
            .collect();
        let end = start + refs.len();
        let next = (end < snapshot.order.len()).then(|| uuid::Uuid::new_v4().to_string());
        let result = ToolResult::success(
            json!({"snapshot":snapshot.token,"sources":refs,
            "complete":next.is_none(),"next_cursor":next})
            .to_string(),
        );
        state.cursor = next.map(|cursor| (cursor, end));
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
    #[cfg(unix)]
    #[tokio::test]
    async fn real_history_tool_search_detail_uses_host_prepared_durable_source() {
        let (execution, _, raw) = fixture();
        let root = tempfile::tempdir().expect("root");
        let durable =
            talos_session::DurableSession::open_or_create(root.path(), "query").expect("session");
        durable
            .commit_turn("prior", &raw, &Default::default())
            .expect("commit");
        let cancel = tokio_util::sync::CancellationToken::new();
        let history = crate::checkpoint_history::History::prepare(&durable, &cancel)
            .await
            .expect("host context");
        crate::checkpoint_history::HISTORY.scope(Some(history),EXECUTION.scope(execution.clone(),async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            let search = HistorySearchTool.execute(json!({"mode":"search","snapshot":discovery["snapshot"],"query":"authorized"})).await;
            assert!(!search.is_error,"{}",search.content);
            let search: Value = serde_json::from_str(&search.content).expect("search");
            let reference = &search["results"][0]["reference"];
            assert_eq!(search["results"][0]["text"],"authorized source 0");
            let detail = HistorySearchTool.execute(json!({"mode":"detail","snapshot":discovery["snapshot"],"reference":reference})).await;
            assert!(!detail.is_error,"{}",detail.content);
            assert_eq!(serde_json::from_str::<Value>(&detail.content).expect("detail")["text"],"authorized source 0");
            let old_cursor = search["next_cursor"].as_str().expect("search cursor");
            let rediscovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let rediscovery: Value = serde_json::from_str(&rediscovery.content).expect("new snapshot");
            assert!(HistorySearchTool.execute(json!({"mode":"search","snapshot":rediscovery["snapshot"],"query":"authorized","cursor":old_cursor})).await.is_error);
            assert!(HistorySearchTool.execute(json!({"mode":"detail","snapshot":rediscovery["snapshot"],"reference":reference})).await.is_error);
            execution.permission.as_ref().expect("permission").clear().expect("reset");
            assert!(HistorySearchTool.execute(json!({"mode":"detail","snapshot":discovery["snapshot"],"reference":reference})).await.is_error);
        })).await;
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
    async fn discovery_pages_preserve_snapshot_source_order_and_reject_replayed_cursor() {
        let (execution, _, mut raw) = fixture();
        raw.push(Message::User {
            content: "later input".into(),
        });
        let source = HistorySource::bind(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
        )
        .expect("source");
        execution.refresh(Some(&source), &raw);
        let first = EXECUTION
            .scope(
                execution.clone(),
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        let first: Value = serde_json::from_str(&first.content).expect("first page");
        let cursor = first["next_cursor"].as_str().expect("next page");
        let (other, _, _) = fixture();
        assert!(
            EXECUTION
                .scope(
                    other,
                    HistorySearchTool.execute(json!({"mode":"discover","cursor":cursor}))
                )
                .await
                .is_error
        );
        // Appending history cannot expand or replace the issued snapshot.
        raw.push(Message::User {
            content: "appended after discovery".into(),
        });
        let source = HistorySource::bind(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
        )
        .expect("source");
        execution.refresh(Some(&source), &raw);
        let second = EXECUTION
            .scope(
                execution.clone(),
                HistorySearchTool.execute(json!({"mode":"discover","cursor":cursor})),
            )
            .await;
        assert!(!second.is_error);
        let second: Value = serde_json::from_str(&second.content).expect("second page");
        assert_eq!(second["snapshot"], first["snapshot"]);
        assert_eq!(second["complete"], true);
        assert_eq!(second["sources"].as_array().expect("sources").len(), 1);
        {
            let state = execution.state.lock().expect("state");
            let issued = state.issued.as_ref().expect("issued");
            let refs: Vec<_> = first["sources"]
                .as_array()
                .expect("first")
                .iter()
                .chain(second["sources"].as_array().expect("second"))
                .collect();
            for (i, reference) in refs.iter().enumerate() {
                assert_eq!(
                    issued.refs.get(reference.as_str().expect("ref")),
                    Some(&format!("raw-{i}"))
                );
            }
        }
        assert!(
            EXECUTION
                .scope(
                    execution,
                    HistorySearchTool.execute(json!({"mode":"discover","cursor":cursor}))
                )
                .await
                .is_error
        );
    }

    #[tokio::test]
    async fn discovery_cursor_revalidates_permission_generation_and_prefix() {
        for change in ["permission", "generation", "prefix", "rediscovery"] {
            let (execution, _, mut raw) = fixture();
            raw.push(Message::User {
                content: "extra".into(),
            });
            let bind = |generation, raw: &[Message]| {
                HistorySource::bind(
                    generation,
                    raw.len() as u64,
                    raw,
                    (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
                )
                .expect("source")
            };
            execution.refresh(Some(&bind(0, &raw)), &raw);
            let first = EXECUTION
                .scope(
                    execution.clone(),
                    HistorySearchTool.execute(json!({"mode":"discover"})),
                )
                .await;
            let first: Value = serde_json::from_str(&first.content).expect("page");
            let cursor = first["next_cursor"].as_str().expect("cursor");
            match change {
                "permission" => execution
                    .permission
                    .as_ref()
                    .expect("permission")
                    .clear()
                    .expect("reset"),
                "generation" => execution.refresh(Some(&bind(1, &raw)), &raw),
                "prefix" => {
                    raw[0] = Message::User {
                        content: "changed".into(),
                    };
                    execution.refresh(Some(&bind(0, &raw)), &raw);
                }
                _ => {
                    let replacement = EXECUTION
                        .scope(
                            execution.clone(),
                            HistorySearchTool.execute(json!({"mode":"discover"})),
                        )
                        .await;
                    assert!(!replacement.is_error);
                }
            }
            let refused = EXECUTION
                .scope(
                    execution,
                    HistorySearchTool.execute(json!({"mode":"discover","cursor":cursor})),
                )
                .await;
            assert!(refused.is_error, "{change} must invalidate cursor");
        }
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
