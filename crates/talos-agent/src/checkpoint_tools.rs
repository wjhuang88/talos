//! Execution-scoped model checkpoint admission. Durable history query follows separately.

mod source_reads;

const SOURCE_TEXT_BUDGET: usize = 2 * 1024 * 1024;
const SOURCE_READ_BUDGET: usize = 16_384;

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

#[cfg(test)]
type QueryCompletionHook =
    Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send>;
#[cfg(test)]
type SourceDeliveryHook = Box<dyn FnOnce() + Send>;

#[derive(Clone)]
pub(crate) struct Execution {
    sender: mpsc::UnboundedSender<crate::AgentActorRequest>,
    state: Arc<Mutex<State>>,
    permission: Option<Arc<talos_permission::PermissionSessionState>>,
    #[cfg(test)]
    after_history_query: Arc<Mutex<Option<QueryCompletionHook>>>,
    #[cfg(test)]
    before_source_delivery: Arc<Mutex<Option<SourceDeliveryHook>>>,
}
#[derive(Default)]
struct State {
    refresh_epoch: Option<uuid::Uuid>,
    generation: Option<u64>,
    current: Option<Snapshot>,
    recovery: Option<(String, String, PermissionLease)>,
    issued: Option<Snapshot>,
    cursor: Option<(String, usize)>,
    receipt: Option<(u64, oneshot::Receiver<CompactOutcome>)>,
    admitting: bool,
    source_reads: BTreeMap<String, source_reads::PreparedRead>,
    source_cursors: BTreeMap<String, (String, usize)>,
}
#[derive(Clone)]
struct Snapshot {
    token: String,
    source: HistorySource,
    end: usize,
    refs: BTreeMap<String, String>,
    order: Vec<String>,
    details: Arc<BTreeMap<String, talos_session::history_projection::RecoveryText>>,
    authority: PermissionLease,
}

// A reservation covers the asynchronous Actor admission window, including a
// rediscovery during that window. Cancellation must release it as well.
struct AdmissionReservation(Arc<Mutex<State>>);
impl Drop for AdmissionReservation {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0.lock() {
            state.admitting = false;
        }
    }
}

fn checkpoint_source_text(
    message: &Message,
) -> Option<talos_session::history_projection::RecoveryText> {
    let (role, content) = match message {
        Message::User { content } => ("user", content),
        Message::Assistant { content, .. } => ("assistant", content),
        // Runtime tool messages have no durable receipt proving redisclosure.
        // Never expose reasoning, tool arguments, system text or raw fallbacks.
        _ => return None,
    };
    if content.len() > talos_session::history_page::HISTORY_PAGE_BYTES {
        return None;
    }
    let entry = talos_session::SessionEntry {
        id: String::new(),
        parent_id: None,
        timestamp: Default::default(),
        role: role.into(),
        content: content.clone(),
        metadata: Default::default(),
    };
    talos_session::history_projection::recovery_response_text(
        &entry,
        None,
        talos_session::history_projection::RecoveryTextKind::Detail,
    )
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
            #[cfg(test)]
            after_history_query: Arc::new(Mutex::new(None)),
            #[cfg(test)]
            before_source_delivery: Arc::new(Mutex::new(None)),
        }
    }
    pub(crate) async fn refresh_cooperative(
        &self,
        source: Option<&HistorySource>,
        messages: &[Message],
    ) {
        // Withdraw the previous live snapshot before the first yield. Dropping
        // this preparation future must not leave stale compaction authority.
        let epoch = uuid::Uuid::new_v4();
        if let Ok(mut state) = self.state.lock() {
            state.refresh_epoch = Some(epoch);
            state.source_reads.clear();
            state.current = None;
            let generation = source.map(HistorySource::generation);
            if generation.is_none() || state.generation != generation {
                state.issued = None;
                state.recovery = None;
                state.cursor = None;
                state.source_cursors.clear();
            }
            state.generation = generation;
        }
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        let snapshot = async {
            let source = source?;
            let permission = self.permission.as_ref()?;
            let authority = PermissionLease {
                state: permission.clone(),
                snapshot: permission.state_snapshot().ok()?,
            };
            let end = crate::checkpoint_request::eligible_prefix_end(messages)?;
            if end == 0 {
                return None;
            }
            let mut refs = BTreeMap::new();
            let mut order = Vec::new();
            let mut details = BTreeMap::new();
            // Text payload budget, in addition to the existing source metadata.
            let mut remaining = SOURCE_TEXT_BUDGET - SOURCE_READ_BUDGET;
            for (index, message) in messages[..end].iter().enumerate() {
                tokio::task::yield_now().await;
                if tokio::time::Instant::now() >= deadline {
                    tracing::warn!(
                        reason = "discovery_reference_deadline",
                        "history/checkpoint preparation unavailable"
                    );
                    return None;
                }
                if matches!(message, Message::System { .. }) {
                    continue;
                }
                if let Some(id) =
                    source.resolve(source.generation(), source.revision(), index, message)
                {
                    let reference = uuid::Uuid::new_v4().to_string();
                    if remaining > 0
                        && let Some(detail) = checkpoint_source_text(message)
                        && detail.text.len() <= remaining
                    {
                        remaining -= detail.text.len();
                        details.insert(reference.clone(), detail);
                    }
                    refs.insert(reference.clone(), id.to_owned());
                    order.push(reference);
                }
            }
            if tokio::time::Instant::now() >= deadline {
                tracing::warn!(
                    reason = "discovery_scan_deadline",
                    "history/checkpoint preparation unavailable"
                );
                return None;
            }
            // Cloning metadata is synchronous and O(history), not hard real-time.
            let source = source.clone();
            if tokio::time::Instant::now() >= deadline {
                tracing::warn!(
                    reason = "discovery_clone_deadline",
                    "history/checkpoint preparation unavailable"
                );
                return None;
            }
            (!refs.is_empty()).then(|| Snapshot {
                token: uuid::Uuid::new_v4().to_string(),
                source,
                end,
                refs,
                order,
                details: Arc::new(details),
                authority,
            })
        }
        .await;
        if let Ok(mut state) = self.state.lock()
            && state.refresh_epoch == Some(epoch)
        {
            state.current = snapshot;
        }
    }
    pub(crate) async fn terminal(&self) -> Option<(String, bool)> {
        let pending = self.state.lock().ok()?.receipt.take();
        let (id, receipt) = pending?;
        let outcome = receipt.await.ok();
        let applied = matches!(outcome, Some(CompactOutcome::Applied { .. }));
        Some((json!({"checkpoint_operation":id,"outcome":outcome.map(|value| format!("{value:?}")).unwrap_or_else(|| "Unavailable".into())}).to_string(), applied))
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
    /// discover lists checkpoint sources; source_search searches their safe text
    /// using a case-sensitive literal substring; source_detail reads one of those
    /// sources. search/detail use separate durable-history references, which
    /// cannot be submitted as checkpoint sources.
    mode: String,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    snapshot: Option<String>,
    #[serde(default)]
    /// Plain text search; FTS operators are not exposed. Matching depends on the storage backend.
    query: Option<String>,
    #[serde(default)]
    reference: Option<String>,
}

pub(crate) struct CompactContextTool;
pub(crate) struct HistorySearchTool;

fn unavailable() -> ToolResult {
    ToolResult::error("history/checkpoint execution unavailable")
}

fn unavailable_reason(reason: &'static str) -> ToolResult {
    tracing::warn!(reason, "history/checkpoint execution unavailable");
    unavailable()
}

#[async_trait]
impl AgentTool for HistorySearchTool {
    fn name(&self) -> &str {
        "history_search"
    }
    fn description(&self) -> &str {
        "Discover a checkpoint snapshot for the older prefix. The suffix starting at the tenth most recent user message is retained unchanged and excluded from these sources; do not seek older citations solely to duplicate that retained suffix. source_search finds eligible checkpoint sources by case-sensitive literal text in safe projections (up to 8192 bytes per source, five bounded previews per page; repeat the same query with next_cursor). source_detail reads an eligible checkpoint source UUID. Search/detail read permitted durable history using separate references that cannot be submitted to compact_context. All reads require the discovered snapshot; durable search/detail require a prepared host index."
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
            return unavailable_reason("execution_scope_missing");
        };
        // CLI persists terminal diagnostics between provider responses. Renew
        // only the seal for the identical captured prefix, never its cutoff or
        // index scope. Current-source checks after query still detect races.
        let _ = crate::checkpoint_history::HISTORY.try_with(|history| {
            history
                .as_ref()
                .and_then(|history| history.refresh_prefix())
        });
        if matches!(input.mode.as_str(), "source_detail" | "source_search") {
            if !source_reads::valid(&input) {
                return ToolResult::error("invalid checkpoint source read input");
            }
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            let durable = crate::checkpoint_history::HISTORY
                .try_with(|history| history.is_some())
                .unwrap_or(false);
            let _identity = if durable {
                let identity = crate::checkpoint_history::HISTORY
                    .try_with(|history| history.as_ref()?.lock_identity_for_delivery(deadline))
                    .ok()
                    .flatten();
                if identity.is_none() {
                    return unavailable_reason("delivery_source_or_snapshot_unavailable");
                }
                identity
            } else {
                None
            };
            let Ok(mut state) = execution.state.lock() else {
                return unavailable_reason("execution_state_poisoned");
            };
            let (Some(issued), Some(current)) = (&state.issued, &state.current) else {
                return ToolResult::error("stale checkpoint snapshot; discover again");
            };
            if input.snapshot.as_deref() != Some(issued.token.as_str())
                || current.end < issued.end
                || !(0..issued.end).all(|i| issued.source.same_entry(&current.source, i))
            {
                return ToolResult::error("stale checkpoint source");
            }
            let issued = issued.clone();
            let read_key = source_reads::key(&input);
            let prepared = state.source_reads.get(&read_key).cloned();
            if input.mode == "source_search"
                && input.cursor.as_ref().is_some_and(|cursor| {
                    !state
                        .source_cursors
                        .get(cursor)
                        .is_some_and(|(query, _)| input.query.as_ref() == Some(query))
                })
            {
                return ToolResult::error("stale checkpoint source cursor");
            }
            let value = prepared
                .as_ref()
                .map(|read| read.value.clone())
                .or_else(|| {
                    let reference = input.reference.as_deref()?;
                    let detail = issued.details.get(reference)?;
                    Some(json!({"reference":reference,"text":detail.text,
                    "truncated":detail.truncated,"checkpoint_source":true}))
                });
            let Some(value) = value else {
                return ToolResult::error(
                    "checkpoint source text unavailable: unsupported projection, text budget, or unknown source",
                );
            };
            if durable
                && !crate::checkpoint_history::HISTORY
                    .try_with(|history| {
                        history
                            .as_ref()
                            .is_some_and(|h| h.available_until(deadline))
                    })
                    .unwrap_or(false)
            {
                return unavailable_reason("delivery_source_or_snapshot_unavailable");
            }
            #[cfg(test)]
            if let Some(hook) = execution
                .before_source_delivery
                .lock()
                .expect("hook")
                .take()
            {
                hook();
            }
            return issued
                .authority
                .commit(|| {
                    if std::time::Instant::now() >= deadline
                        || (durable
                            && !crate::checkpoint_history::HISTORY
                                .try_with(|history| {
                                    history.as_ref().is_some_and(|history| !history.cancelled())
                                })
                                .unwrap_or(false))
                    {
                        return unavailable_reason("delivery_cancelled_or_expired");
                    }
                    if input.mode == "source_search" {
                        let next = prepared.as_ref().and_then(|read| read.next_cursor.clone());
                        if next.is_some() && input.cursor.is_none() && state.source_cursors.len() >= 16 {
                            return ToolResult::error("checkpoint source cursor capacity reached; finish an outstanding search or discover again");
                        }
                        if let Some(cursor) = &input.cursor {
                            state.source_cursors.remove(cursor);
                        }
                        if let Some((token, query, index)) = next {
                            state.source_cursors.insert(token, (query, index));
                        }
                    }
                    state.source_reads.remove(&read_key);
                    ToolResult::success(value.to_string())
                })
                .unwrap_or_else(|| unavailable_reason("permission_commit_rejected"));
        }
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
                    return unavailable_reason("execution_state_poisoned");
                };
                if let Some((token, scope, authority)) = &state.recovery {
                    let same_scope = crate::checkpoint_history::HISTORY
                        .try_with(|history| {
                            history
                                .as_ref()
                                .is_some_and(|history| history.scope_identity() == scope)
                        })
                        .unwrap_or(false);
                    if !same_scope || input.snapshot.as_deref() != Some(token.as_str()) {
                        return ToolResult::error("stale history snapshot");
                    }
                    authority.clone()
                } else {
                    let Some(snapshot) = &state.issued else {
                        return unavailable_reason("issued_snapshot_missing");
                    };
                    let Some(current) = &state.current else {
                        return unavailable_reason("current_snapshot_missing");
                    };
                    if input.snapshot.as_deref() != Some(snapshot.token.as_str())
                        || current.end < snapshot.end
                        || !(0..snapshot.end)
                            .all(|i| snapshot.source.same_entry(&current.source, i))
                    {
                        return ToolResult::error("stale history snapshot");
                    }
                    snapshot.authority.clone()
                }
            };
            if !authority.valid() {
                return unavailable_reason("permission_lease_revoked");
            }
            let delivery_deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
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
            #[cfg(test)]
            {
                let hook = execution
                    .after_history_query
                    .lock()
                    .expect("query hook")
                    .take();
                if let Some(hook) = hook {
                    hook().await;
                }
            }
            // Revalidate after query construction. Keep execution state stable
            // through permission commit, using the same state -> permission lock
            // order as discovery. Acquire identity custody before the state lock;
            // the final availability check below still runs under the state lock.
            let Some(_identity) = crate::checkpoint_history::HISTORY
                .try_with(|history| {
                    history
                        .as_ref()?
                        .lock_identity_for_delivery(delivery_deadline)
                })
                .ok()
                .flatten()
            else {
                return unavailable_reason("delivery_source_or_snapshot_unavailable");
            };
            let Ok(state) = execution.state.lock() else {
                return unavailable_reason("execution_state_poisoned");
            };
            let snapshot_current = if let Some((token, scope, _)) = &state.recovery {
                input.snapshot.as_deref() == Some(token.as_str())
                    && crate::checkpoint_history::HISTORY
                        .try_with(|history| {
                            history
                                .as_ref()
                                .is_some_and(|history| history.scope_identity() == scope)
                        })
                        .unwrap_or(false)
            } else if let (Some(issued), Some(current)) = (&state.issued, &state.current) {
                input.snapshot.as_deref() == Some(issued.token.as_str())
                    && current.end >= issued.end
                    && (0..issued.end).all(|i| issued.source.same_entry(&current.source, i))
            } else {
                false
            };
            if !snapshot_current
                || !crate::checkpoint_history::HISTORY
                    .try_with(|history| {
                        history
                            .as_ref()
                            .is_some_and(|history| history.available_until(delivery_deadline))
                    })
                    .unwrap_or(false)
            {
                return unavailable_reason("delivery_source_or_snapshot_unavailable");
            }
            return authority
                .commit(|| {
                    if std::time::Instant::now() >= delivery_deadline
                        || !crate::checkpoint_history::HISTORY
                            .try_with(|history| {
                                history.as_ref().is_some_and(|history| !history.cancelled())
                            })
                            .unwrap_or(false)
                    {
                        return unavailable_reason("delivery_cancelled_or_expired");
                    }
                    result
                        .map(|result| ToolResult::success(result.to_string()))
                        .unwrap_or_else(|| ToolResult::error("history recovery Unavailable"))
                })
                .unwrap_or_else(|| unavailable_reason("permission_commit_rejected"));
        }
        if input.mode != "discover"
            || input.query.is_some()
            || input.reference.is_some()
            || input.snapshot.is_some()
        {
            return ToolResult::error("invalid discovery input");
        }
        let Ok(mut state) = execution.state.lock() else {
            return unavailable_reason("execution_state_poisoned");
        };
        let rediscover = input.cursor.is_none();
        // Durable recovery does not require a materialized compaction source.
        // HISTORY is scoped to this Actor execution and revalidates its sealed
        // source and owner scope on every query. Never widen compaction eligibility.
        if state.current.is_none() && rediscover {
            let ready = crate::checkpoint_history::HISTORY
                .try_with(|history| {
                    let history = history.as_ref()?;
                    history.reset_queries()?;
                    history
                        .available()
                        .then(|| history.scope_identity().to_owned())
                })
                .ok()
                .flatten();
            let Some(scope) = ready else {
                return unavailable_reason("snapshot_missing_and_history_not_ready");
            };
            let Some(permission) = &execution.permission else {
                return unavailable_reason("permission_state_missing");
            };
            let Ok(permission_snapshot) = permission.state_snapshot() else {
                return unavailable_reason("permission_snapshot_unavailable");
            };
            let authority = PermissionLease {
                state: permission.clone(),
                snapshot: permission_snapshot,
            };
            let token = uuid::Uuid::new_v4().to_string();
            let result = ToolResult::success(
                json!({
                    "snapshot":token, "sources":[], "complete":true,
                    "next_cursor":null, "compact_available":false,
                    "history_available":true
                })
                .to_string(),
            );
            state.issued = None;
            state.cursor = None;
            state.recovery = Some((token, scope, authority.clone()));
            return authority
                .commit(|| result)
                .unwrap_or_else(|| unavailable_reason("permission_commit_rejected"));
        }
        state.recovery = None;
        let (snapshot, start) = if let Some(cursor) = input.cursor {
            let Some((expected, start)) = state.cursor.as_ref() else {
                return ToolResult::error("stale discovery cursor");
            };
            if cursor != *expected {
                return ToolResult::error("stale discovery cursor");
            }
            let Some(snapshot) = state.issued.clone() else {
                return unavailable_reason("issued_snapshot_missing");
            };
            let Some(current) = &state.current else {
                return unavailable_reason("current_snapshot_missing");
            };
            if current.end < snapshot.end
                || !(0..snapshot.end).all(|i| snapshot.source.same_entry(&current.source, i))
            {
                return ToolResult::error("stale discovery source");
            }
            (snapshot, *start)
        } else {
            let Some(snapshot) = state.current.clone() else {
                return unavailable_reason("current_snapshot_missing");
            };
            (snapshot, 0)
        };
        if !snapshot.authority.valid() {
            return unavailable_reason("permission_lease_revoked");
        }
        if rediscover {
            state.source_reads.clear();
            state.source_cursors.clear();
            let reset = crate::checkpoint_history::HISTORY
                .try_with(|history| history.as_ref().map(|history| history.reset_queries()))
                .ok()
                .flatten();
            if matches!(reset, Some(None)) {
                return unavailable_reason("query_state_reset_failed");
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
        "Submit a structured checkpoint replacing the discovered older prefix, using sources read with history_search source_detail. Preserve significant goals, constraints, decisions, unresolved work and risks from that prefix; the runtime retains the recent ten-user-message suffix unchanged. Do not cite an old source for newer facts or discard significant older state merely to obtain Applied. Durable search/detail references are not checkpoint sources. Admitted reserves an operation; only a later Applied receipt confirms installation after the complete tool batch."
    }
    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(CompactInput)).unwrap_or_else(|_| json!({}))
    }
    fn permission_profile(&self, _: &Value) -> Vec<ToolPermissionFacet> {
        // Session-owned checkpoint submission, not filesystem write authority.
        // Execution still requires the live snapshot and Actor installation gates.
        vec![ToolPermissionFacet::new(ToolNature::Internal)]
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
            return unavailable_reason("execution_scope_missing");
        };
        let snapshot = {
            let Ok(state) = execution.state.lock() else {
                return unavailable_reason("execution_state_poisoned");
            };
            if state.receipt.is_some() || state.admitting {
                return ToolResult::error("checkpoint operation busy");
            }
            let Some(issued) = state
                .issued
                .as_ref()
                .filter(|issued| issued.token == input.snapshot)
            else {
                return ToolResult::error("stale checkpoint snapshot");
            };
            let Some(current) = &state.current else {
                return unavailable_reason("current_snapshot_missing");
            };
            if !issued.authority.valid()
                || current.end < issued.end
                || !(0..issued.end).all(|i| issued.source.same_entry(&current.source, i))
            {
                return ToolResult::error("stale checkpoint source");
            }
            issued.clone()
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
        let _reservation = {
            let Ok(mut state) = execution.state.lock() else {
                return unavailable_reason("execution_state_poisoned");
            };
            if state.receipt.is_some() || state.admitting {
                return ToolResult::error("checkpoint operation busy");
            }
            if !state
                .issued
                .as_ref()
                .is_some_and(|issued| issued.token == snapshot.token)
                || !state.current.as_ref().is_some_and(|current| {
                    current.end >= snapshot.end
                        && (0..snapshot.end).all(|i| snapshot.source.same_entry(&current.source, i))
                })
                || !snapshot.authority.valid()
            {
                return ToolResult::error("stale checkpoint source");
            }
            state.issued = None;
            state.admitting = true;
            AdmissionReservation(execution.state.clone())
        };
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
            return unavailable_reason("checkpoint_actor_channel_closed");
        }
        match tokio::time::timeout(std::time::Duration::from_secs(2), admitted).await {
            Ok(Ok(Ok(id))) => {
                let Ok(mut state) = execution.state.lock() else {
                    return unavailable_reason("execution_state_poisoned");
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

    #[tokio::test]
    async fn compact_internal_admission_preserves_execution_scope() {
        use talos_permission::{
            PermissionContext, PermissionEngine, PermissionInvocation, PermissionRequest,
            PermissionSessionState,
        };
        let tool = CompactContextTool;
        let input = json!({});
        let facets = tool.permission_profile(&input);
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].nature, ToolNature::Internal);
        let state = PermissionSessionState::new(PermissionEngine::empty());
        let request = PermissionRequest::native("compact_context", &facets, &input);
        assert!(matches!(
            state
                .begin_invocation(&request, &PermissionContext::compatibility())
                .expect("internal admission"),
            PermissionInvocation::Allow(_)
        ));
        assert!(
            tool.execute(input).await.is_error,
            "admission does not bypass payload or execution scope validation"
        );
    }

    #[tokio::test]
    async fn only_applied_terminal_receipts_report_discovery_progress() {
        for (outcome, expected) in [
            (
                Some(CompactOutcome::Applied {
                    before: 100,
                    after: 50,
                }),
                true,
            ),
            (Some(CompactOutcome::Failed), false),
            (Some(CompactOutcome::Cancelled), false),
            (Some(CompactOutcome::NotEligible { before: 10 }), false),
            (None, false),
        ] {
            let (execution, _, _) = fixture().await;
            let (sender, receiver) = oneshot::channel();
            execution.state.lock().expect("state").receipt = Some((7, receiver));
            if let Some(outcome) = outcome {
                sender.send(outcome).expect("receipt");
            } else {
                drop(sender);
            }
            let (receipt, applied) = execution.terminal().await.expect("terminal");
            assert_eq!(applied, expected);
            assert_eq!(
                serde_json::from_str::<Value>(&receipt).expect("json")["checkpoint_operation"],
                7
            );
            assert!(
                execution.terminal().await.is_none(),
                "receipt consumed exactly once"
            );
        }
    }

    #[tokio::test]
    async fn checkpoint_invalid_submission_preserves_issued_source() {
        for wrong_snapshot in [false, true] {
            let (execution, mut receiver, _) = fixture().await;
            EXECUTION
                .scope(execution.clone(), async {
                    let result = HistorySearchTool.execute(json!({"mode":"discover"})).await;
                    let discovery: Value =
                        serde_json::from_str(&result.content).expect("discovery");
                    let mut input = submission(&discovery);
                    if wrong_snapshot {
                        input["snapshot"] = json!("wrong-snapshot");
                    } else {
                        input["checkpoint"]["goal"]["sources"] = json!(["durable-query-reference"]);
                    }
                    assert!(CompactContextTool.execute(input).await.is_error);
                    assert!(receiver.try_recv().is_err());
                    let detail = HistorySearchTool.execute(json!({"mode":"source_detail", "snapshot":discovery["snapshot"], "reference":discovery["sources"][0]})).await;
                    assert!(!detail.is_error, "{}", detail.content);
                    assert!(detail.content.contains("authorized source 0"));
                    let actor = async {
                        let crate::AgentActorRequest::AdmitModel { submission, response } = receiver.recv().await.expect("retry admission") else { panic!("expected admission") };
                        response.send(Ok(7)).expect("admit");
                        submission.receipt.send(CompactOutcome::Applied { before: 30, after: 10 }).expect("receipt");
                    };
                    let (retry, ()) = tokio::join!(CompactContextTool.execute(submission(&discovery)), actor);
                    assert!(!retry.is_error, "{}", retry.content);
                    assert!(execution.terminal().await.expect("terminal").1);
                })
                .await;
        }
    }

    #[tokio::test]
    async fn checkpoint_admission_reservation_blocks_rediscovery_and_recovers_after_cancel() {
        let (execution, mut receiver, _) = fixture().await;
        EXECUTION
            .scope(execution.clone(), async {
                let result = HistorySearchTool.execute(json!({"mode":"discover"})).await;
                let discovery: Value = serde_json::from_str(&result.content).expect("discovery");
                let pending = CompactContextTool.execute(submission(&discovery));
                tokio::pin!(pending);
                let request = tokio::select! {
                    biased;
                    result = &mut pending => panic!("admission must wait: {result:?}"),
                    request = receiver.recv() => request.expect("admission request"),
                };
                let rediscovered = HistorySearchTool.execute(json!({"mode":"discover"})).await;
                let rediscovered: Value =
                    serde_json::from_str(&rediscovered.content).expect("rediscovery");
                let busy = CompactContextTool.execute(submission(&rediscovered)).await;
                assert!(busy.is_error);
                assert!(busy.content.contains("busy"));
                assert!(receiver.try_recv().is_err(), "only one Actor admission");
                drop(request);
                assert!(pending.await.is_error);
                assert!(!execution.state.lock().expect("state").admitting);
            })
            .await;
        // Dropping an in-flight tool future closes the Actor response channel
        // and releases the reservation even without an admission reply.
        let result = EXECUTION
            .scope(
                execution.clone(),
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        let discovery: Value = serde_json::from_str(&result.content).expect("discovery");
        let request = {
            let pending = EXECUTION.scope(
                execution.clone(),
                CompactContextTool.execute(submission(&discovery)),
            );
            tokio::pin!(pending);
            tokio::select! {
                biased;
                _ = &mut pending => panic!("must wait"),
                request = receiver.recv() => request.expect("request"),
            }
        };
        let crate::AgentActorRequest::AdmitModel { response, .. } = request else {
            panic!("expected model admission")
        };
        assert!(response.is_closed());
        assert!(!execution.state.lock().expect("state").admitting);
    }

    #[test]
    fn checkpoint_source_projection_excludes_private_fields_and_bounds_utf8() {
        let source = Message::Assistant {
            content: format!(
                "public\n```json-tool\n{{\"secret\":\"HIDDEN_ARGUMENT\"}}\n```\n{}",
                "界".repeat(4000)
            ),
            tool_calls: vec![talos_core::message::ToolCall {
                id: "id".into(),
                name: "hidden".into(),
                input: json!({"private":"HIDDEN_NATIVE"}),
            }],
            reasoning: None,
        };
        let text = checkpoint_source_text(&source).expect("public projection");
        assert!(text.truncated);
        assert!(text.text.len() <= 8192);
        assert!(!text.text.contains("HIDDEN"));
        assert!(
            checkpoint_source_text(&Message::Context {
                content: "private".into()
            })
            .is_none()
        );
        assert!(
            checkpoint_source_text(&Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "id".into(),
                    content: "unverified".into(),
                    is_error: false,
                }
            })
            .is_none()
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn checkpoint_source_detail_reads_reopened_eligible_source() {
        let (execution, _, raw) = fixture().await;
        let root = tempfile::tempdir().expect("root");
        let owner = talos_session::DurableSession::open_or_create(root.path(), "source-detail")
            .expect("owner");
        owner
            .commit_turn("seed", &raw, &Default::default())
            .expect("commit");
        drop(owner);
        let owner = talos_session::DurableSession::open_existing(root.path(), "source-detail")
            .expect("reopen")
            .expect("session");
        let before = std::fs::read(owner.file_path()).expect("before");
        let cancel = tokio_util::sync::CancellationToken::new();
        let history = crate::checkpoint_history::History::prepare(&owner, &cancel)
            .await
            .expect("history");
        crate::checkpoint_history::HISTORY.scope(Some(history), EXECUTION.scope(execution.clone(), async {
            let result = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let discovery: Value = serde_json::from_str(&result.content).expect("discovery");
            let detail = HistorySearchTool.execute(json!({"mode":"source_detail", "snapshot":discovery["snapshot"], "reference":discovery["sources"][0]})).await;
            assert!(!detail.is_error, "{}", detail.content);
            let detail: Value = serde_json::from_str(&detail.content).expect("detail");
            assert_eq!(detail["text"], "authorized source 0");
            assert_eq!(detail["reference"], discovery["sources"][0]);
            assert_eq!(detail["checkpoint_source"], true);
            let cancelled = cancel.clone();
            *execution.before_source_delivery.lock().expect("hook") = Some(Box::new(move || cancelled.cancel()));
            let refused = HistorySearchTool.execute(json!({"mode":"source_detail", "snapshot":discovery["snapshot"], "reference":discovery["sources"][0]})).await;
            assert!(refused.is_error, "cancel after availability check must prevent disclosure");
            assert!(!refused.content.contains("authorized source"));

        })).await;
        assert_eq!(std::fs::read(owner.file_path()).expect("after"), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn checkpoint_source_detail_rejects_revoked_or_replaced_authority() {
        for change in ["permission", "generation", "prefix", "durable", "cancel"] {
            let (execution, _, mut raw) = fixture().await;
            let root = tempfile::tempdir().expect("root");
            let owner = talos_session::DurableSession::open_or_create(root.path(), "source-fence")
                .expect("owner");
            owner
                .commit_turn("seed", &raw, &Default::default())
                .expect("commit");
            let cancel = tokio_util::sync::CancellationToken::new();
            let history = crate::checkpoint_history::History::prepare(&owner, &cancel)
                .await
                .expect("history");
            crate::checkpoint_history::HISTORY.scope(Some(history), EXECUTION.scope(execution.clone(), async {
                let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
                let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
                match change {
                    "permission" => { execution.permission.as_ref().expect("permission").clear().expect("revoke"); }
                    "cancel" => cancel.cancel(),
                    "durable" => {
                        let mut bytes = std::fs::read(owner.file_path()).expect("source");
                        let offset = bytes.windows(b"authorized source".len()).position(|part| part == b"authorized source").expect("text");
                        bytes[offset] = b'X';
                        std::fs::write(owner.file_path(), bytes).expect("same-length replacement");
                    }
                    _ => {
                        if change == "prefix" { raw[0] = Message::User { content: "replacement".into() }; }
                        let source = HistorySource::bind(u64::from(change == "generation"), raw.len() as u64,
                            &raw, (0..raw.len()).map(|i| format!("raw-{i}")).collect()).expect("source");
                        execution.refresh_cooperative(Some(&source), &raw).await;
                    }
                }
                let result = HistorySearchTool.execute(json!({"mode":"source_detail", "snapshot":discovery["snapshot"], "reference":discovery["sources"][0]})).await;
                assert!(result.is_error, "{change}: {}", result.content);
                assert!(!result.content.contains("authorized source"));
            })).await;
        }
    }

    #[tokio::test]
    async fn checkpoint_source_search_reaches_beyond_cached_text_budget() {
        let (execution, _, _) = fixture().await;
        let mut raw = vec![
            Message::User {
                content: "x".repeat(8192)
            };
            280
        ];
        raw[260] = Message::User {
            content: "retain late recovery constraint".into(),
        };
        let source = HistorySource::bind_until(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
            tokio::time::Instant::now() + std::time::Duration::from_secs(10),
        )
        .await
        .expect("source");
        execution.refresh_cooperative(Some(&source), &raw).await;
        EXECUTION.scope(execution.clone(), async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            let input = json!({"mode":"source_search", "snapshot":discovery["snapshot"], "query":"late recovery"});
            execution.prepare_source_reads(&raw, &[talos_core::message::ToolCall {
                id: "source-read".into(), name: "history_search".into(), input: input.clone(),
            }]).await;
            let result = HistorySearchTool.execute(input).await;
            assert!(!result.is_error, "{}", result.content);
            let result: Value = serde_json::from_str(&result.content).expect("result");
            assert_eq!(result["results"].as_array().expect("results").len(), 1);
            assert_eq!(result["results"][0]["text"], "retain late recovery constraint");
            let reference = result["results"][0]["reference"].as_str().expect("reference");
            assert_eq!(execution.state.lock().expect("state").issued.as_ref().expect("issued").refs[reference], "raw-260");
            let detail_input = json!({"mode":"source_detail", "snapshot":discovery["snapshot"], "reference":reference});
            execution.prepare_source_reads(&raw, &[talos_core::message::ToolCall {
                id: "late-detail".into(), name: "history_search".into(), input: detail_input.clone(),
            }]).await;
            {
                let state = execution.state.lock().expect("state");
                let cached = state.issued.as_ref().expect("issued").details.values().map(|v| v.text.len()).sum::<usize>();
                let prepared = state.source_reads.values().map(|v| v.value.to_string().len()).sum::<usize>();
                assert!(cached + prepared <= SOURCE_TEXT_BUDGET);
            }
            let detail = HistorySearchTool.execute(detail_input).await;
            assert!(!detail.is_error, "{}", detail.content);
            assert_eq!(serde_json::from_str::<Value>(&detail.content).expect("detail")["text"], "retain late recovery constraint");
        }).await;
    }

    async fn prepare_source_read(execution: &Execution, raw: &[Message], input: &Value) {
        execution
            .prepare_source_reads(
                raw,
                &[talos_core::message::ToolCall {
                    id: "read".into(),
                    name: "history_search".into(),
                    input: input.clone(),
                }],
            )
            .await;
    }

    #[tokio::test]
    async fn checkpoint_source_search_interleaved_pages_remain_available() {
        let (execution, _, raw) = fixture().await;
        EXECUTION.scope(execution.clone(), async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            let mut calls = ["authorized", "source"].map(|query| talos_core::message::ToolCall {
                id: query.into(), name: "history_search".into(),
                input: json!({"mode":"source_search","snapshot":discovery["snapshot"],"query":query}),
            });
            for _ in 0..2 {
                execution.prepare_source_reads(&raw, &calls).await;
                for call in &mut calls {
                    let response = HistorySearchTool.execute(call.input.clone()).await;
                    assert!(!response.is_error, "{}", response.content);
                    let page: Value = serde_json::from_str(&response.content).expect("page");
                    assert!(page["next_cursor"].is_string());
                    call.input["cursor"] = page["next_cursor"].clone();
                }
            }
        }).await;
    }

    #[tokio::test]
    async fn checkpoint_source_search_pages_are_bound_to_query_and_snapshot() {
        let (execution, _, raw) = fixture().await;
        EXECUTION.scope(execution.clone(), async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            let mut input = json!({"mode":"source_search", "snapshot":discovery["snapshot"], "query":"authorized source"});
            let mut seen = std::collections::BTreeSet::new();
            let mut first_cursor = None;
            loop {
                prepare_source_read(&execution, &raw, &input).await;
                let result = HistorySearchTool.execute(input.clone()).await;
                assert!(!result.is_error, "{}", result.content);
                assert!(result.content.len() <= SOURCE_READ_BUDGET);
                let result: Value = serde_json::from_str(&result.content).expect("page");
                for item in result["results"].as_array().expect("results") {
                    assert!(seen.insert(item["reference"].as_str().expect("ref").to_owned()));
                }
                if result["complete"] == true { break }
                first_cursor.get_or_insert(result["next_cursor"].clone());
                input["cursor"] = result["next_cursor"].clone();
            }
            assert_eq!(seen.len(), 20, "all eligible sources, not the recent ten-user suffix");
            input["cursor"] = first_cursor.expect("cursor");
            prepare_source_read(&execution, &raw, &input).await;
            assert!(HistorySearchTool.execute(input.clone()).await.is_error, "consumed cursor");
            input.as_object_mut().expect("input").remove("cursor");
            prepare_source_read(&execution, &raw, &input).await;
            let page = HistorySearchTool.execute(input.clone()).await;
            let page: Value = serde_json::from_str(&page.content).expect("page");
            input["cursor"] = page["next_cursor"].clone();
            input["query"] = json!("different");
            prepare_source_read(&execution, &raw, &input).await;
            assert!(HistorySearchTool.execute(input.clone()).await.is_error, "query-bound cursor");
            input["query"] = json!("authorized source");
            prepare_source_read(&execution, &raw, &input).await;
            assert!(!HistorySearchTool.execute(json!({"mode":"discover"})).await.is_error);
            assert!(HistorySearchTool.execute(input).await.is_error, "rediscovery withdraws prepared reads");
        }).await;
    }

    #[tokio::test]
    async fn checkpoint_source_prepared_reads_revalidate_authority_and_source() {
        for mode in ["source_search", "source_detail"] {
            for change in ["permission", "generation", "prefix", "refresh"] {
                let (execution, _, mut raw) = fixture().await;
                EXECUTION.scope(execution.clone(), async {
                    let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
                    let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
                    let input = if mode == "source_search" {
                        json!({"mode":mode,"snapshot":discovery["snapshot"],"query":"authorized"})
                    } else {
                        json!({"mode":mode,"snapshot":discovery["snapshot"],"reference":discovery["sources"][0]})
                    };
                    prepare_source_read(&execution, &raw, &input).await;
                    assert!(!execution.state.lock().expect("state").source_reads.is_empty());
                    match change {
                        "permission" => {
                            let permission = execution.permission.clone().expect("permission");
                            *execution.before_source_delivery.lock().expect("hook") = Some(Box::new(move || {
                                permission.clear().expect("revoke at delivery");
                            }));
                        }
                        "refresh" => execution.refresh_cooperative(None, &[]).await,
                        _ => {
                            if change == "prefix" { raw[0] = Message::User { content: "changed".into() }; }
                            let source = HistorySource::bind(u64::from(change == "generation"), raw.len() as u64,
                                &raw, (0..raw.len()).map(|i| format!("raw-{i}")).collect()).expect("source");
                            execution.refresh_cooperative(Some(&source), &raw).await;
                        }
                    }
                    let result = HistorySearchTool.execute(input).await;
                    assert!(result.is_error, "{mode}/{change}: {}", result.content);
                    assert!(!result.content.contains("authorized source"));
                }).await;
            }
        }
    }

    #[tokio::test]
    async fn checkpoint_source_read_preparation_cannot_publish_across_refresh() {
        let (execution, _, raw) = fixture().await;
        EXECUTION.scope(execution.clone(), async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            let input = json!({"mode":"source_search","snapshot":discovery["snapshot"],"query":"authorized"});
            let calls = [talos_core::message::ToolCall { id: "pending-read".into(), name: "history_search".into(), input: input.clone() }];
            let pending = execution.prepare_source_reads(&raw, &calls);
            tokio::pin!(pending);
            tokio::select! {
                biased;
                () = &mut pending => panic!("projection must yield cooperatively"),
                () = tokio::task::yield_now() => {},
            }
            let source = HistorySource::bind(0, raw.len() as u64, &raw,
                (0..raw.len()).map(|i| format!("raw-{i}")).collect()).expect("source");
            execution.refresh_cooperative(Some(&source), &raw).await;
            pending.await;
            assert!(execution.state.lock().expect("state").source_reads.is_empty());
            assert!(HistorySearchTool.execute(input).await.is_error);
        }).await;
    }

    #[tokio::test]
    async fn checkpoint_source_detail_total_text_budget_is_explicit() {
        let (execution, _, _) = fixture().await;
        let raw = vec![
            Message::User {
                content: "x".repeat(8192)
            };
            280
        ];
        let source = HistorySource::bind_until(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
            tokio::time::Instant::now() + std::time::Duration::from_secs(10),
        )
        .await
        .expect("source");
        execution.refresh_cooperative(Some(&source), &raw).await;
        EXECUTION.scope(execution.clone(), async {
            let result = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            let discovery: Value = serde_json::from_str(&result.content).expect("discovery");
            let reference = {
                let state = execution.state.lock().expect("state");
                let issued = state.issued.as_ref().expect("issued");
                assert_eq!(issued.details.values().map(|v| v.text.len()).sum::<usize>(), SOURCE_TEXT_BUDGET - SOURCE_READ_BUDGET);
                issued.order[256].clone()
            };
            let result = HistorySearchTool.execute(json!({"mode":"source_detail", "snapshot":discovery["snapshot"], "reference":reference})).await;
            assert!(result.is_error);
            assert!(result.content.contains("text budget"));
        }).await;
    }

    async fn fixture() -> (
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
        execution.refresh_cooperative(Some(&source), &raw).await;
        (execution, receiver, raw)
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn history_result_rechecks_mutation_reset_and_cancel_after_query() {
        for recovery_only in [false, true] {
            for change in [
                "source",
                "execution",
                "generation",
                "permission",
                "cancel",
                "retire",
                "append",
            ] {
                let (execution, _, mut raw) = fixture().await;
                if recovery_only {
                    // A short source is valid for durable recovery but has no
                    // eligible compaction prefix.
                    raw.truncate(2);
                    let source = HistorySource::bind(
                        0,
                        raw.len() as u64,
                        &raw,
                        (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
                    )
                    .expect("short recovery-only source");
                    execution.refresh_cooperative(Some(&source), &raw).await;
                    assert!(execution.state.lock().expect("state").current.is_none());
                }
                let root = tempfile::tempdir().expect("root");
                let durable =
                    talos_session::DurableSession::open_or_create(root.path(), "delivery-race")
                        .expect("session");
                durable
                    .commit_turn("prior", &raw, &Default::default())
                    .expect("commit");
                let cancel = tokio_util::sync::CancellationToken::new();
                let history = crate::checkpoint_history::History::prepare(&durable, &cancel)
                    .await
                    .expect("history");
                crate::checkpoint_history::HISTORY.scope(Some(history), EXECUTION.scope(execution.clone(), async {
                let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
                assert!(!discovery.is_error);
                let discovery: Value = serde_json::from_str(&discovery.content).expect("snapshot");
                let baseline = HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":"authorized"})).await;
                assert!(!baseline.is_error, "healthy query must succeed before {change}");
                assert!(baseline.content.contains("authorized source 0"));
                let current_execution = execution.clone();
                let path = durable.file_path().to_path_buf();
                let cancelled = cancel.clone();
                *execution.after_history_query.lock().expect("hook") = Some(Box::new(move || Box::pin(async move {
                    match change {
                        "source" => {
                            let mut bytes = std::fs::read(&path).expect("source");
                            let offset = bytes.windows(b"authorized source".len())
                                .position(|window| window == b"authorized source").expect("original fact");
                            bytes[offset] = b'X';
                            std::fs::write(&path, bytes).expect("same-size mutation");
                        }
                        "execution" => current_execution.refresh_cooperative(None, &[]).await,
                        "generation" | "append" => {
                            if change == "append" { raw.push(Message::User { content: "later append".into() }); }
                            let generation = u64::from(change == "generation");
                            let source = HistorySource::bind(generation, raw.len() as u64, &raw,
                                (0..raw.len()).map(|i| format!("raw-{i}")).collect()).expect("next generation");
                            current_execution.refresh_cooperative(Some(&source), &raw).await;
                        }
                        "permission" => current_execution.permission.as_ref().expect("permission").clear().expect("reset"),
                        "cancel" => cancelled.cancel(),
                        "retire" => {
                            let id = uuid::Uuid::parse_str(path.file_stem().expect("stem").to_str().expect("uuid")).expect("id");
                            talos_session::retire_session_identity(&path.parent().expect("parent").join("history-bindings.sqlite"), id).expect("retire after query");
                        }
                        _ => unreachable!(),
                    }
                })));
                let result = HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":"authorized"})).await;
                assert_eq!(result.is_error, change != "append", "change={change}, recovery_only={recovery_only}: {}", result.content);
                if result.is_error { assert!(!result.content.contains("authorized source"), "no stale content"); }
            })).await;
            }
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn real_history_tool_search_detail_uses_host_prepared_durable_source() {
        let (execution, _, raw) = fixture().await;
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

    #[tokio::test]
    async fn older_refresh_cannot_restore_withdrawn_or_new_generation_authority() {
        for replace in [false, true] {
            let (execution, _, raw) = fixture().await;
            let bind = |generation| {
                HistorySource::bind(
                    generation,
                    raw.len() as u64,
                    &raw,
                    (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
                )
                .expect("source")
            };
            let old_source = bind(0);
            let old = execution.refresh_cooperative(Some(&old_source), &raw);
            tokio::pin!(old);
            tokio::select! { biased;
                _ = &mut old => panic!("older refresh must yield"),
                _ = std::future::ready(()) => {},
            }
            let new_source = bind(1);
            execution
                .refresh_cooperative(replace.then_some(&new_source), &raw)
                .await;
            old.await;
            let state = execution.state.lock().expect("state");
            assert_eq!(state.generation, replace.then_some(1));
            if replace {
                assert_eq!(
                    state
                        .current
                        .as_ref()
                        .expect("new snapshot")
                        .source
                        .generation(),
                    1
                );
            } else {
                assert!(state.current.is_none());
                assert!(state.issued.is_none());
            }
        }
    }

    #[tokio::test]
    async fn cancelled_discovery_refresh_withdraws_old_live_authority() {
        let (execution, _, raw) = fixture().await;
        let source = HistorySource::bind(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
        )
        .expect("source");
        assert!(execution.state.lock().expect("state").current.is_some());
        {
            let refresh = execution.refresh_cooperative(Some(&source), &raw);
            tokio::pin!(refresh);
            tokio::select! { biased;
                _ = &mut refresh => panic!("refresh must yield"),
                _ = std::future::ready(()) => {},
            }
        }
        assert!(execution.state.lock().expect("state").current.is_none());
        let result = EXECUTION
            .scope(
                execution,
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        assert!(
            result.is_error,
            "cancelled preparation cannot expose the old live snapshot"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn expired_discovery_withdraws_authority_and_fresh_attempt_recovers() {
        let (execution, _, raw) = fixture().await;
        let source = HistorySource::bind(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
        )
        .expect("source");
        assert!(execution.state.lock().expect("state").current.is_some());
        {
            let refresh = execution.refresh_cooperative(Some(&source), &raw);
            tokio::pin!(refresh);
            // Poll through several records so expiry discards a partial scan.
            for _ in 0..3 {
                tokio::select! { biased;
                    _ = &mut refresh => panic!("refresh must yield before publication"),
                    _ = std::future::ready(()) => {},
                }
            }
            tokio::time::advance(std::time::Duration::from_secs(2)).await;
            refresh.await;
        }
        assert!(execution.state.lock().expect("state").current.is_none());
        let unavailable = EXECUTION
            .scope(
                execution.clone(),
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        assert!(unavailable.is_error, "expired authority must not escape");
        execution.refresh_cooperative(Some(&source), &raw).await;
        let recovered = EXECUTION
            .scope(
                execution,
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        assert!(!recovered.is_error, "{}", recovered.content);
    }

    #[tokio::test]
    async fn cooperative_discovery_accepts_long_bound_history() {
        let (execution, _, _) = fixture().await;
        let raw: Vec<_> = (0..8300)
            .map(|i| Message::User {
                content: format!("fact {i}"),
            })
            .collect();
        let source = HistorySource::bind_until(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("raw-{i}")).collect(),
            tokio::time::Instant::now() + std::time::Duration::from_secs(10),
        )
        .await
        .expect("bound history");
        execution.refresh_cooperative(Some(&source), &raw).await;
        {
            let state = execution.state.lock().expect("state");
            let current = state
                .current
                .as_ref()
                .expect("long history remains compactable");
            assert_eq!(current.end, 8290);
            assert_eq!(current.order.len(), 8290);
        }
        let result = EXECUTION
            .scope(
                execution,
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        assert!(!result.is_error, "{}", result.content);
        let page: Value = serde_json::from_str(&result.content).expect("page");
        assert_eq!(page["sources"].as_array().expect("refs").len(), 20);
        assert_eq!(page["complete"], false);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn mixed_production_writers_reopen_with_discover_and_search() {
        use crate::checkpoint_history::History;
        use talos_core::message::{AgentEvent, Message};
        use talos_session::DurableSession;
        use tokio_util::sync::CancellationToken;
        let (execution, _, _) = fixture().await;
        execution.refresh_cooperative(None, &[]).await;
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "mixed-writers").expect("owner");
        session
            .session()
            .append(&Message::User {
                content: "initial question".into(),
            })
            .expect("append");
        session
            .session()
            .append_event(&AgentEvent::TurnStart)
            .expect("event");
        session
            .commit_turn(
                "first",
                &[Message::User {
                    content: "durable searchable constraint".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        session
            .session()
            .append_event(&AgentEvent::TurnStart)
            .expect("post-commit event");
        let diagnostic = talos_session::ProviderTerminalDiagnostic::from_agent_event(
            "first",
            1,
            &AgentEvent::Error {
                message: "transport failed".into(),
            },
            None,
            None,
        )
        .expect("diagnostic");
        session
            .session()
            .append_terminal_diagnostic(&diagnostic)
            .expect("diagnostic append");
        session
            .session()
            .append(&Message::User {
                content: "next question".into(),
            })
            .expect("post-event append");
        drop(session);
        let session = DurableSession::open_existing(root.path(), "mixed-writers")
            .expect("reopen")
            .expect("existing");
        let history = History::prepare_cached(
            &session,
            &CancellationToken::new(),
            &tokio::sync::Mutex::new(None),
        )
        .await
        .expect("history available");
        crate::checkpoint_history::HISTORY.scope(Some(history), EXECUTION.scope(execution, async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            assert!(!discovery.is_error, "{}", discovery.content);
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            let search = HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":"constraint"})).await;
            assert!(!search.is_error, "{}", search.content);
            let result: Value = serde_json::from_str(&search.content).expect("search");
            assert!(result["results"].as_array().expect("results").iter().any(|item| item["text"].as_str().is_some_and(|text| text.contains("durable searchable constraint"))));
        })).await;
    }

    #[tokio::test]
    async fn durable_reopened_history_search_treats_punctuation_as_text() {
        use crate::checkpoint_history::History;
        use talos_session::DurableSession;
        use tokio_util::sync::CancellationToken;
        let (execution, _, _) = fixture().await;
        execution.refresh_cooperative(None, &[]).await;
        let root = tempfile::tempdir().expect("root");
        let owner = DurableSession::open_or_create(root.path(), "literal-query").expect("owner");
        let text =
            "ORCHID-7319 preserves \"source records\"; decision: SQLite; literal OR constraint";
        owner
            .commit_turn(
                "prior",
                &[
                    Message::User {
                        content: text.into(),
                    },
                    Message::User {
                        content: "constraint distractor".into(),
                    },
                ],
                &Default::default(),
            )
            .expect("commit");
        drop(owner);
        let owner = DurableSession::open_existing(root.path(), "literal-query")
            .expect("reopen")
            .expect("owner");
        let history = History::prepare_cached(
            &owner,
            &CancellationToken::new(),
            &tokio::sync::Mutex::new(None),
        )
        .await
        .expect("history");
        crate::checkpoint_history::HISTORY.scope(Some(history), EXECUTION.scope(execution, async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            assert!(!discovery.is_error, "{}", discovery.content);
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            for query in ["ORCHID-7319", "\"source records\"", "decision: SQLite", "literal OR constraint"] {
                let search = HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":query})).await;
                assert!(!search.is_error, "{query}: {}", search.content);
                let search: Value = serde_json::from_str(&search.content).expect("search");
                let matches = search["results"].as_array().expect("results");
                assert_eq!(matches.len(), 1, "{query}: {search}");
                let detail = HistorySearchTool.execute(json!({"mode":"detail", "snapshot":discovery["snapshot"], "reference":matches[0]["reference"]})).await;
                assert!(!detail.is_error, "{}", detail.content);
                let detail: Value = serde_json::from_str(&detail.content).expect("detail");
                assert_eq!(detail["text"], text);
            }
            for (query, count) in [
                (String::new(), 2),
                ("   ".into(), 0),
                ("\"".into(), 0),
                ("x".repeat(1024), 0),
                ("\"".repeat(1024), 0),
                (format!("{}a", "中".repeat(341)), 0),
            ] {
                let search = HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":query})).await;
                assert!(!search.is_error, "{query}: {}", search.content);
                let search: Value = serde_json::from_str(&search.content).expect("search");
                assert_eq!(search["results"].as_array().expect("results").len(), count);
            }
            let oversized = HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":"x".repeat(1025)})).await;
            assert!(oversized.is_error);
            assert_eq!(oversized.content, "invalid history query bounds");
        })).await;
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn durable_discovery_searches_beyond_compaction_source_limit() {
        let (execution, _, _) = fixture().await;
        let raw: Vec<_> = (0..8300)
            .map(|i| Message::User {
                content: format!("longhistory fact {i}"),
            })
            .collect();
        execution.refresh_cooperative(None, &raw).await;
        let root = tempfile::tempdir().expect("root");
        let durable = talos_session::DurableSession::open_or_create(root.path(), "long-query")
            .expect("session");
        durable
            .commit_turn("prior", &raw, &Default::default())
            .expect("commit");
        let history = crate::checkpoint_history::History::prepare(
            &durable,
            &tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("host context");
        crate::checkpoint_history::HISTORY.scope(Some(history), EXECUTION.scope(execution.clone(), async {
            let discovery = HistorySearchTool.execute(json!({"mode":"discover"})).await;
            assert!(!discovery.is_error, "{}", discovery.content);
            let discovery: Value = serde_json::from_str(&discovery.content).expect("discovery");
            assert_eq!(discovery["compact_available"], false);
            assert_eq!(discovery["sources"], json!([]));
            let search = HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":"8299"})).await;
            assert!(!search.is_error, "{}", search.content);
            let search: Value = serde_json::from_str(&search.content).expect("search");
            assert_eq!(search["results"][0]["text"], "longhistory fact 8299");
            let detail = HistorySearchTool.execute(json!({"mode":"detail", "snapshot":discovery["snapshot"], "reference":search["results"][0]["reference"]})).await;
            assert!(!detail.is_error);
            let replacement = crate::checkpoint_history::History::prepare(&durable, &tokio_util::sync::CancellationToken::new()).await.expect("replacement scope");
            // Preparing a replacement retires the outer host scope.
            assert!(HistorySearchTool.execute(json!({"mode":"discover"})).await.is_error);
            crate::checkpoint_history::HISTORY.scope(Some(replacement), async {
                assert!(HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":"8299"})).await.is_error);
                assert!(!HistorySearchTool.execute(json!({"mode":"discover"})).await.is_error);
            }).await;
            execution.permission.as_ref().expect("permission").clear().expect("reset");
            assert!(HistorySearchTool.execute(json!({"mode":"search", "snapshot":discovery["snapshot"], "query":"8299"})).await.is_error);
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
        let (execution, mut receiver, _) = fixture().await;
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
        let (other, _, _) = fixture().await;
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
        let (execution, _, mut raw) = fixture().await;
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
        execution.refresh_cooperative(Some(&source), &raw).await;
        let first = EXECUTION
            .scope(
                execution.clone(),
                HistorySearchTool.execute(json!({"mode":"discover"})),
            )
            .await;
        let first: Value = serde_json::from_str(&first.content).expect("first page");
        let cursor = first["next_cursor"].as_str().expect("next page");
        let (other, _, _) = fixture().await;
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
        execution.refresh_cooperative(Some(&source), &raw).await;
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
            let (execution, _, mut raw) = fixture().await;
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
            execution
                .refresh_cooperative(Some(&bind(0, &raw)), &raw)
                .await;
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
                "generation" => {
                    execution
                        .refresh_cooperative(Some(&bind(1, &raw)), &raw)
                        .await
                }
                "prefix" => {
                    raw[0] = Message::User {
                        content: "changed".into(),
                    };
                    execution
                        .refresh_cooperative(Some(&bind(0, &raw)), &raw)
                        .await;
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
        let (execution, mut receiver, mut raw) = fixture().await;
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
        execution.refresh_cooperative(Some(&changed), &raw).await;
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
