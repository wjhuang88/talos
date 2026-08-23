use std::collections::VecDeque;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};
use std::time::Duration;

use axum::response::sse::Event;
use futures_core::Stream;
use schemars::JsonSchema;
use serde::Serialize;
use talos_core::message::{AgentEvent, StopReason, Usage};
use talos_core::provider::ProviderProgress;
use talos_core::session::{SessionEvent, TurnCompletionStatus, TurnEventPayload};
use talos_core::tool::ToolProvenance;
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::{Instant, Interval};
use uuid::Uuid;

use crate::redact_text;

pub(crate) const ACTIVITY_EVENT_LIMIT: usize = 256;
pub(crate) const ACTIVITY_BYTE_LIMIT: usize = 512 * 1024;
pub(crate) const LOG_LINE_LIMIT: usize = 512;
pub(crate) const LOG_BYTE_LIMIT: usize = 1024 * 1024;
pub(crate) const ENTRY_BYTE_LIMIT: usize = 16 * 1024;
pub(crate) const SSE_CLIENT_LIMIT: usize = 8;
pub(crate) const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
pub(crate) const RETRY_INTERVAL: Duration = Duration::from_secs(2);

const ERROR_TEXT_LIMIT: usize = 1024;
const SUMMARY_FIELD_LIMIT: usize = 256;
const SUMMARY_FIELD_COUNT: usize = 8;

/// A safe, serialized activity item exposed by the Dashboard SSE endpoint.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActivityPayload {
    /// The current Session association changed.
    Session { session_id: String },
    /// The configured model/provider identity changed.
    Model { provider: String, model: String },
    /// A Turn entered a new lifecycle state.
    Turn {
        session_id: String,
        turn_id: String,
        state: String,
    },
    /// Provider dispatch or bounded retry progress changed.
    Provider {
        session_id: String,
        phase: String,
        attempt: u32,
        max_attempts: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        delay_ms: Option<u64>,
    },
    /// Safe tool lifecycle metadata without raw arguments or results.
    Tool {
        session_id: String,
        state: String,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provenance: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        summary: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        failed: Option<bool>,
    },
    /// Authoritative token usage from a provider Turn end.
    Usage {
        session_id: String,
        input_tokens: u32,
        output_tokens: u32,
        cache_read_tokens: u32,
        cache_write_tokens: u32,
        reasoning_tokens: u32,
    },
    /// A redacted runtime error.
    Error { session_id: String, message: String },
    /// One redacted line observed after the existing log writer accepted it.
    Log { line: String },
}

#[derive(Debug, Clone)]
struct StoredEntry {
    id: u64,
    event: &'static str,
    data: String,
    bytes: usize,
}

#[derive(Debug, Default)]
struct BoundedRing {
    entries: VecDeque<StoredEntry>,
    bytes: usize,
    count_limit: usize,
    byte_limit: usize,
}

impl BoundedRing {
    fn new(count_limit: usize, byte_limit: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            bytes: 0,
            count_limit,
            byte_limit,
        }
    }

    fn push(&mut self, entry: StoredEntry) {
        if entry.bytes > self.byte_limit {
            return;
        }
        self.bytes += entry.bytes;
        self.entries.push_back(entry);
        while self.entries.len() > self.count_limit || self.bytes > self.byte_limit {
            if let Some(removed) = self.entries.pop_front() {
                self.bytes = self.bytes.saturating_sub(removed.bytes);
            }
        }
    }
}

#[derive(Debug)]
struct FeedState {
    next_id: u64,
    activity: BoundedRing,
    logs: BoundedRing,
    log_partial: Vec<u8>,
    discarding_long_line: bool,
    tool_names: VecDeque<(String, String, String)>,
    turn_states: VecDeque<(String, String, String)>,
}

impl Default for FeedState {
    fn default() -> Self {
        Self {
            next_id: 1,
            activity: BoundedRing::new(ACTIVITY_EVENT_LIMIT, ACTIVITY_BYTE_LIMIT),
            logs: BoundedRing::new(LOG_LINE_LIMIT, LOG_BYTE_LIMIT),
            log_partial: Vec::new(),
            discarding_long_line: false,
            tool_names: VecDeque::new(),
            turn_states: VecDeque::new(),
        }
    }
}

#[derive(Debug)]
struct FeedInner {
    stream_id: String,
    state: Mutex<FeedState>,
    notify: Arc<Notify>,
    clients: Arc<Semaphore>,
}

/// Dashboard-scoped, bounded presentation feed for safe activity and logs.
#[derive(Debug, Clone)]
pub struct DashboardActivityFeed {
    inner: Arc<FeedInner>,
}

impl Default for DashboardActivityFeed {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardActivityFeed {
    /// Create an empty process-local feed with a new stream identity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(FeedInner {
                stream_id: Uuid::new_v4().simple().to_string(),
                state: Mutex::new(FeedState::default()),
                notify: Arc::new(Notify::new()),
                clients: Arc::new(Semaphore::new(SSE_CLIENT_LIMIT)),
            }),
        }
    }

    /// Publish configured provider/model identity using presentation-only facts.
    pub fn project_model(&self, provider: &str, model: &str) {
        self.push_activity(ActivityPayload::Model {
            provider: bounded_redacted(provider, SUMMARY_FIELD_LIMIT),
            model: bounded_redacted(model, SUMMARY_FIELD_LIMIT),
        });
    }

    /// Publish the Session currently associated with the observed event queue.
    pub fn project_session(&self, session_id: &str) {
        self.push_activity(ActivityPayload::Session {
            session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
        });
    }

    /// Project one Session event through a strict safe allowlist.
    ///
    /// `owning_session_id` comes from the CLI bridge forwarder that owns and
    /// drains the event queue. It is used only for legacy event variants that
    /// do not carry their Session association themselves.
    pub fn project_session_event(&self, owning_session_id: &str, event: &SessionEvent) {
        if let Some((session_id, agent_event)) = nested_agent_event(owning_session_id, event) {
            match agent_event {
                AgentEvent::ToolCall { call, .. } => {
                    self.remember_tool(session_id, &call.id, &call.name);
                }
                AgentEvent::ToolResult { result } => {
                    if let Some(name) = self.take_tool(session_id, &result.tool_use_id) {
                        self.push_activity(ActivityPayload::Tool {
                            session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
                            state: "completed".to_string(),
                            name,
                            provenance: None,
                            summary: Vec::new(),
                            failed: Some(result.is_error),
                        });
                    }
                    return;
                }
                _ => {}
            }
        }
        let payloads = project_session_event(owning_session_id, event);
        for payload in payloads {
            self.push_activity(payload);
        }
    }

    /// Observe bytes only after the existing tracing writer accepted them.
    ///
    /// Bytes are framed into lines, redacted again at the Dashboard boundary,
    /// and bounded before entering the in-memory log presentation ring.
    pub fn observe_written_log_bytes(&self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        let mut state = self.lock_state();
        for &byte in bytes {
            if byte == b'\n' {
                if state.discarding_long_line {
                    state.discarding_long_line = false;
                    state.log_partial.clear();
                    continue;
                }
                let line = std::mem::take(&mut state.log_partial);
                push_log_line(&mut state, &line);
                continue;
            }
            if state.discarding_long_line {
                continue;
            }
            if state.log_partial.len() >= ENTRY_BYTE_LIMIT {
                let mut line = std::mem::take(&mut state.log_partial);
                line.extend_from_slice(b" [truncated]");
                push_log_line(&mut state, &line);
                state.discarding_long_line = true;
            } else {
                state.log_partial.push(byte);
            }
        }
        drop(state);
        self.inner.notify.notify_waiters();
    }

    pub(crate) fn try_stream(&self, last_event_id: Option<&str>) -> Option<ActivityStream> {
        let permit = self.inner.clients.clone().try_acquire_owned().ok()?;
        Some(ActivityStream::new(self.clone(), last_event_id, permit))
    }

    fn push_activity(&self, payload: ActivityPayload) {
        let Ok(data) = serde_json::to_string(&payload) else {
            return;
        };
        if data.len() > ENTRY_BYTE_LIMIT {
            return;
        }
        let mut state = self.lock_state();
        if is_duplicate_turn_state(&mut state, &payload) {
            return;
        }
        let id = take_next_id(&mut state);
        state.activity.push(StoredEntry {
            id,
            event: "activity",
            bytes: data.len(),
            data,
        });
        drop(state);
        self.inner.notify.notify_waiters();
    }

    fn remember_tool(&self, session_id: &str, call_id: &str, name: &str) {
        let mut state = self.lock_state();
        state
            .tool_names
            .retain(|(session, call, _)| session != session_id || call != call_id);
        state.tool_names.push_back((
            session_id.to_string(),
            call_id.to_string(),
            bounded_redacted(name, SUMMARY_FIELD_LIMIT),
        ));
        while state.tool_names.len() > ACTIVITY_EVENT_LIMIT {
            state.tool_names.pop_front();
        }
    }

    fn take_tool(&self, session_id: &str, call_id: &str) -> Option<String> {
        let mut state = self.lock_state();
        let index = state
            .tool_names
            .iter()
            .position(|(session, call, _)| session == session_id && call == call_id)?;
        state.tool_names.remove(index).map(|(_, _, name)| name)
    }

    fn lock_state(&self) -> MutexGuard<'_, FeedState> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn batch_after(&self, cursor: &Cursor) -> Batch {
        let state = self.lock_state();
        let mut entries = state
            .activity
            .entries
            .iter()
            .chain(state.logs.entries.iter())
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.id);

        let latest_id = state.next_id.saturating_sub(1);
        let oldest_id = entries.first().map_or(state.next_id, |entry| entry.id);
        let reset_reason = match cursor {
            Cursor::Initial => None,
            Cursor::Valid { stream_id, id } if stream_id != &self.inner.stream_id => {
                Some("stream_changed")
            }
            Cursor::Valid { id, .. }
                if *id > latest_id || (*id != 0 && id.saturating_add(1) < oldest_id) =>
            {
                Some("cursor_outside_window")
            }
            Cursor::Invalid => Some("invalid_cursor"),
            Cursor::Valid { .. } => None,
        };

        let after = if reset_reason.is_some() {
            0
        } else {
            match cursor {
                Cursor::Valid { id, .. } => *id,
                Cursor::Initial | Cursor::Invalid => 0,
            }
        };
        entries.retain(|entry| entry.id > after);
        Batch {
            reset_reason,
            entries,
        }
    }

    #[cfg(test)]
    fn retained(&self) -> Vec<StoredEntry> {
        self.batch_after(&Cursor::Initial).entries
    }
}

fn is_duplicate_turn_state(state: &mut FeedState, payload: &ActivityPayload) -> bool {
    let ActivityPayload::Turn {
        session_id,
        turn_id,
        state: turn_state,
    } = payload
    else {
        return false;
    };
    if let Some(index) = state
        .turn_states
        .iter()
        .position(|(session, turn, _)| session == session_id && turn == turn_id)
    {
        if state.turn_states[index].2 == *turn_state {
            return true;
        }
        state.turn_states.remove(index);
    }
    state
        .turn_states
        .push_back((session_id.clone(), turn_id.clone(), turn_state.clone()));
    while state.turn_states.len() > ACTIVITY_EVENT_LIMIT {
        state.turn_states.pop_front();
    }
    false
}

fn take_next_id(state: &mut FeedState) -> u64 {
    let id = state.next_id;
    state.next_id = state.next_id.saturating_add(1);
    id
}

fn push_log_line(state: &mut FeedState, bytes: &[u8]) {
    let line = String::from_utf8_lossy(bytes)
        .trim_end_matches('\r')
        .to_string();
    if line.is_empty() {
        return;
    }
    let payload = ActivityPayload::Log {
        line: bounded_redacted(&line, ENTRY_BYTE_LIMIT.saturating_sub(256)),
    };
    let Ok(data) = serde_json::to_string(&payload) else {
        return;
    };
    if data.len() > ENTRY_BYTE_LIMIT {
        return;
    }
    let id = take_next_id(state);
    state.logs.push(StoredEntry {
        id,
        event: "log",
        bytes: data.len(),
        data,
    });
}

fn project_session_event(owning_session_id: &str, event: &SessionEvent) -> Vec<ActivityPayload> {
    match event {
        SessionEvent::SubmissionStarted {
            session_id,
            turn_id,
            ..
        }
        | SessionEvent::StructuredSubmissionStarted {
            session_id,
            turn_id,
            ..
        } => vec![turn_payload(session_id, turn_id, "started")],
        SessionEvent::StructuredTurnEvent {
            session_id,
            turn_id,
            payload,
            ..
        }
        | SessionEvent::TurnEvent {
            session_id,
            turn_id,
            payload,
            ..
        } => project_turn_payload(session_id, turn_id, payload),
        SessionEvent::AgentEvent { event } => project_agent_event(owning_session_id, "", event),
        SessionEvent::TurnStarted { turn_id } => {
            vec![turn_payload(owning_session_id, turn_id, "started")]
        }
        SessionEvent::TurnCompleted { turn_id, status } => {
            project_completion(owning_session_id, turn_id, status)
        }
        SessionEvent::Error { message } => vec![error_payload(owning_session_id, message)],
        SessionEvent::SubmissionRejected { session_id, .. }
        | SessionEvent::SubmissionPaused { session_id, .. } => vec![ActivityPayload::Error {
            session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
            message: "Submission did not start".to_string(),
        }],
        SessionEvent::SubmissionQueued { .. }
        | SessionEvent::SubmissionResolved { .. }
        | SessionEvent::SubmissionReceipt { .. }
        | SessionEvent::EntriesCommitted { .. }
        | SessionEvent::ApprovalRequired { .. } => Vec::new(),
        _ => Vec::new(),
    }
}

fn nested_agent_event<'a>(
    owning_session_id: &'a str,
    event: &'a SessionEvent,
) -> Option<(&'a str, &'a AgentEvent)> {
    match event {
        SessionEvent::StructuredTurnEvent {
            session_id,
            payload: TurnEventPayload::Progress { event },
            ..
        }
        | SessionEvent::TurnEvent {
            session_id,
            payload: TurnEventPayload::Progress { event },
            ..
        } => Some((session_id, event)),
        SessionEvent::AgentEvent { event } => Some((owning_session_id, event)),
        _ => None,
    }
}

fn project_turn_payload(
    session_id: &str,
    turn_id: &str,
    payload: &TurnEventPayload,
) -> Vec<ActivityPayload> {
    match payload {
        TurnEventPayload::Started => vec![turn_payload(session_id, turn_id, "started")],
        TurnEventPayload::Progress { event } => project_agent_event(session_id, turn_id, event),
        TurnEventPayload::Completed { status } => project_completion(session_id, turn_id, status),
        _ => Vec::new(),
    }
}

fn project_agent_event(
    session_id: &str,
    turn_id: &str,
    event: &AgentEvent,
) -> Vec<ActivityPayload> {
    match event {
        AgentEvent::TurnStart => vec![turn_payload(session_id, turn_id, "started")],
        AgentEvent::ProviderProgress { progress } => {
            provider_payload(session_id, progress).into_iter().collect()
        }
        AgentEvent::ToolCallStarted { name } => vec![ActivityPayload::Tool {
            session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
            state: "starting".to_string(),
            name: bounded_redacted(name, SUMMARY_FIELD_LIMIT),
            provenance: None,
            summary: Vec::new(),
            failed: None,
        }],
        AgentEvent::ToolCall {
            call,
            provenance,
            summary_fields,
        } => vec![ActivityPayload::Tool {
            session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
            state: "running".to_string(),
            name: bounded_redacted(&call.name, SUMMARY_FIELD_LIMIT),
            provenance: Some(project_provenance(provenance)),
            summary: summary_fields
                .iter()
                .take(SUMMARY_FIELD_COUNT)
                .map(|field| bounded_redacted(field, SUMMARY_FIELD_LIMIT))
                .collect(),
            failed: None,
        }],
        AgentEvent::ToolResult { .. } => Vec::new(),
        AgentEvent::TurnEnd { stop_reason, usage } => {
            let state = match stop_reason {
                StopReason::EndTurn => "completed",
                StopReason::ToolUse => "awaiting_tool",
                StopReason::MaxTokens => "token_limit",
            };
            vec![
                turn_payload(session_id, turn_id, state),
                usage_payload(session_id, usage),
            ]
        }
        AgentEvent::Error { message } => vec![error_payload(session_id, message)],
        AgentEvent::TextDelta { .. }
        | AgentEvent::ThinkingDelta { .. }
        | AgentEvent::ReasoningComplete { .. } => Vec::new(),
        _ => Vec::new(),
    }
}

fn project_completion(
    session_id: &str,
    turn_id: &str,
    status: &TurnCompletionStatus,
) -> Vec<ActivityPayload> {
    match status {
        TurnCompletionStatus::Success { .. } => {
            vec![turn_payload(session_id, turn_id, "completed")]
        }
        TurnCompletionStatus::Cancelled => {
            vec![turn_payload(session_id, turn_id, "cancelled")]
        }
        TurnCompletionStatus::Error { message } => vec![
            turn_payload(session_id, turn_id, "failed"),
            error_payload(session_id, message),
        ],
    }
}

fn turn_payload(session_id: &str, turn_id: &str, state: &str) -> ActivityPayload {
    ActivityPayload::Turn {
        session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
        turn_id: bounded_redacted(turn_id, SUMMARY_FIELD_LIMIT),
        state: state.to_string(),
    }
}

fn provider_payload(session_id: &str, progress: &ProviderProgress) -> Option<ActivityPayload> {
    let (phase, attempt, max_attempts, delay_ms) = match progress {
        ProviderProgress::InitialDispatch {
            attempt,
            max_attempts,
        } => ("dispatch", *attempt, *max_attempts, None),
        ProviderProgress::RetryDispatch {
            attempt,
            max_attempts,
        } => ("retry_dispatch", *attempt, *max_attempts, None),
        ProviderProgress::ScheduledBackoff {
            attempt,
            max_attempts,
            delay_ms,
        } => ("retry_backoff", *attempt, *max_attempts, Some(*delay_ms)),
        ProviderProgress::FirstPacketWait {
            attempt,
            max_attempts,
        } => ("first_packet", *attempt, *max_attempts, None),
        _ => return None,
    };
    Some(ActivityPayload::Provider {
        session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
        phase: phase.to_string(),
        attempt,
        max_attempts,
        delay_ms,
    })
}

fn usage_payload(session_id: &str, usage: &Usage) -> ActivityPayload {
    ActivityPayload::Usage {
        session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        cache_read_tokens: usage.cache_read_tokens,
        cache_write_tokens: usage.cache_write_tokens,
        reasoning_tokens: usage.reasoning_tokens,
    }
}

fn error_payload(session_id: &str, message: &str) -> ActivityPayload {
    ActivityPayload::Error {
        session_id: bounded_redacted(session_id, SUMMARY_FIELD_LIMIT),
        message: bounded_redacted(message, ERROR_TEXT_LIMIT),
    }
}

fn project_provenance(provenance: &ToolProvenance) -> String {
    match provenance {
        ToolProvenance::Native => "native".to_string(),
        ToolProvenance::McpRemote { server } => {
            format!("mcp:{}", bounded_redacted(server, SUMMARY_FIELD_LIMIT))
        }
        ToolProvenance::Plugin { name, carrier, .. } => format!(
            "plugin:{}:{}",
            bounded_redacted(name, SUMMARY_FIELD_LIMIT),
            bounded_redacted(carrier, SUMMARY_FIELD_LIMIT)
        ),
    }
}

fn bounded_redacted(value: &str, max_bytes: usize) -> String {
    let redacted = redact_text(value);
    if redacted.len() <= max_bytes {
        return redacted;
    }
    let mut end = max_bytes.saturating_sub(" [truncated]".len());
    while !redacted.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    format!("{} [truncated]", &redacted[..end])
}

#[derive(Debug)]
enum Cursor {
    Initial,
    Valid { stream_id: String, id: u64 },
    Invalid,
}

impl Cursor {
    fn parse(value: Option<&str>) -> Self {
        let Some(value) = value else {
            return Self::Initial;
        };
        let Some((stream_id, id)) = value.rsplit_once(':') else {
            return Self::Invalid;
        };
        match id.parse::<u64>() {
            Ok(id) if !stream_id.is_empty() => Self::Valid {
                stream_id: stream_id.to_string(),
                id,
            },
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug)]
struct Batch {
    reset_reason: Option<&'static str>,
    entries: Vec<StoredEntry>,
}

/// One bounded SSE client stream. The semaphore permit is released on drop.
pub(crate) struct ActivityStream {
    feed: DashboardActivityFeed,
    cursor: Cursor,
    pending: VecDeque<Event>,
    notified: Pin<Box<tokio::sync::futures::OwnedNotified>>,
    heartbeat: Interval,
    _permit: OwnedSemaphorePermit,
}

impl ActivityStream {
    fn new(
        feed: DashboardActivityFeed,
        last_event_id: Option<&str>,
        permit: OwnedSemaphorePermit,
    ) -> Self {
        let notified = Box::pin(feed.inner.notify.clone().notified_owned());
        Self {
            feed,
            cursor: Cursor::parse(last_event_id),
            pending: VecDeque::new(),
            notified,
            heartbeat: tokio::time::interval_at(
                Instant::now() + HEARTBEAT_INTERVAL,
                HEARTBEAT_INTERVAL,
            ),
            _permit: permit,
        }
    }

    fn refill(&mut self) {
        let batch = self.feed.batch_after(&self.cursor);
        if let Some(reason) = batch.reset_reason {
            let data = serde_json::json!({
                "stream_id": self.feed.inner.stream_id,
                "reason": reason,
            });
            self.pending.push_back(
                Event::default()
                    .event("reset")
                    .id(format!("{}:0", self.feed.inner.stream_id))
                    .retry(RETRY_INTERVAL)
                    .data(data.to_string()),
            );
            self.cursor = Cursor::Valid {
                stream_id: self.feed.inner.stream_id.clone(),
                id: 0,
            };
            return;
        }
        if let Some(entry) = batch.entries.into_iter().next() {
            self.cursor = Cursor::Valid {
                stream_id: self.feed.inner.stream_id.clone(),
                id: entry.id,
            };
            self.pending.push_back(
                Event::default()
                    .event(entry.event)
                    .id(format!("{}:{}", self.feed.inner.stream_id, entry.id))
                    .retry(RETRY_INTERVAL)
                    .data(entry.data),
            );
        }
    }
}

impl Stream for ActivityStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(event) = self.pending.pop_front() {
            return Poll::Ready(Some(Ok(event)));
        }
        self.refill();
        if let Some(event) = self.pending.pop_front() {
            return Poll::Ready(Some(Ok(event)));
        }

        if self.notified.as_mut().poll(cx).is_ready() {
            self.notified = Box::pin(self.feed.inner.notify.clone().notified_owned());
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        if self.heartbeat.poll_tick(cx).is_ready() {
            return Poll::Ready(Some(Ok(Event::default().comment("heartbeat"))));
        }
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{MessageToolResult, ReasoningBlock, ToolCall};
    use talos_core::session::SubmissionRejectionReason;

    fn data(feed: &DashboardActivityFeed) -> String {
        feed.retained()
            .into_iter()
            .map(|entry| entry.data)
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn projection_drops_forbidden_agent_payloads() {
        let feed = DashboardActivityFeed::new();
        let forbidden = [
            AgentEvent::TextDelta {
                delta: "prompt-secret".to_string(),
            },
            AgentEvent::ThinkingDelta {
                delta: "reasoning-secret".to_string(),
            },
            AgentEvent::ReasoningComplete {
                blocks: vec![
                    ReasoningBlock::Thinking {
                        text: "thinking-block-secret".to_string(),
                        signature: Some("reasoning-signature-secret".to_string()),
                    },
                    ReasoningBlock::Redacted {
                        data: "encrypted-reasoning-secret".to_string(),
                    },
                    ReasoningBlock::Plain {
                        text: "plain-reasoning-secret".to_string(),
                    },
                ],
            },
            AgentEvent::ToolResult {
                result: MessageToolResult {
                    tool_use_id: "call-secret".to_string(),
                    content: "tool-result-secret".to_string(),
                    is_error: false,
                },
            },
        ];
        for event in forbidden {
            feed.project_session_event("session-safe", &SessionEvent::AgentEvent { event });
        }
        let retained = data(&feed);
        assert!(!retained.contains("prompt-secret"));
        assert!(!retained.contains("reasoning-secret"));
        assert!(!retained.contains("thinking-block-secret"));
        assert!(!retained.contains("reasoning-signature-secret"));
        assert!(!retained.contains("encrypted-reasoning-secret"));
        assert!(!retained.contains("plain-reasoning-secret"));
        assert!(!retained.contains("tool-result-secret"));
        assert!(!retained.contains("call-secret"));
        assert!(retained.is_empty());
    }

    #[test]
    fn projection_drops_approval_payloads_and_rejection_reasons() {
        let feed = DashboardActivityFeed::new();
        feed.project_session_event(
            "session-safe",
            &SessionEvent::ApprovalRequired {
                tool_name: "write_file".to_string(),
                arguments: r#"{"path":"/private/secret","token":"approval-secret"}"#.to_string(),
                call_id: "approval-call-secret".to_string(),
            },
        );
        feed.project_session_event(
            "session-safe",
            &SessionEvent::SubmissionRejected {
                session_id: "session-safe".to_string(),
                submission_id: "submission-secret".to_string(),
                sender_generation: 1,
                reason: SubmissionRejectionReason::WrongGeneration,
            },
        );

        let retained = data(&feed);
        for forbidden in [
            "write_file",
            "/private/secret",
            "approval-secret",
            "approval-call-secret",
            "submission-secret",
        ] {
            assert!(
                !retained.contains(forbidden),
                "leaked {forbidden}: {retained}"
            );
        }
        assert!(retained.contains("Submission did not start"), "{retained}");
    }

    #[test]
    fn projection_deduplicates_compatibility_wrappers_for_one_turn_state() {
        let feed = DashboardActivityFeed::new();
        for payload in [
            TurnEventPayload::Started,
            TurnEventPayload::Progress {
                event: AgentEvent::TurnStart,
            },
            TurnEventPayload::Progress {
                event: AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage {
                        input_tokens: 1,
                        output_tokens: 2,
                        cache_read_tokens: 0,
                        cache_write_tokens: 0,
                        reasoning_tokens: 0,
                    },
                },
            },
            TurnEventPayload::Completed {
                status: TurnCompletionStatus::Success {
                    final_text: String::new(),
                    new_messages: Vec::new(),
                },
            },
        ] {
            feed.project_session_event(
                "session-1",
                &SessionEvent::TurnEvent {
                    session_id: "session-1".to_string(),
                    turn_id: "turn-1".to_string(),
                    sequence: 1,
                    payload,
                },
            );
        }

        let retained = data(&feed);
        assert_eq!(retained.matches(r#""state":"started""#).count(), 1);
        assert_eq!(retained.matches(r#""state":"completed""#).count(), 1);
        assert_eq!(retained.matches(r#""kind":"usage""#).count(), 1);
    }

    #[test]
    fn tool_projection_admits_only_safe_metadata() {
        let feed = DashboardActivityFeed::new();
        feed.project_session_event(
            "session-1",
            &SessionEvent::AgentEvent {
                event: AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "secret-call-id".to_string(),
                        name: "read_file".to_string(),
                        input: serde_json::json!({"path": "/secret", "api_key": "sk-live"}),
                    },
                    provenance: ToolProvenance::Native,
                    summary_fields: vec!["path=src/main.rs".to_string()],
                },
            },
        );
        feed.project_session_event(
            "session-1",
            &SessionEvent::AgentEvent {
                event: AgentEvent::ToolResult {
                    result: MessageToolResult {
                        tool_use_id: "secret-call-id".to_string(),
                        content: "private tool output".to_string(),
                        is_error: false,
                    },
                },
            },
        );
        let retained = data(&feed);
        assert!(retained.contains("read_file"));
        assert!(retained.contains("native"));
        assert!(retained.contains("src/main.rs"));
        assert!(!retained.contains("secret-call-id"));
        assert!(!retained.contains("/secret"));
        assert!(!retained.contains("sk-live"));
        assert!(!retained.contains("private tool output"));
        assert!(retained.contains("completed"));
    }

    #[test]
    fn errors_and_logs_are_redacted_and_entry_bounded() {
        let feed = DashboardActivityFeed::new();
        feed.project_session_event(
            "session-1",
            &SessionEvent::Error {
                message:
                    "Authorization: Bearer abc token=xyz Cookie: sid=secret <script>x</script>"
                        .to_string(),
            },
        );
        feed.observe_written_log_bytes(b"api_key=sk-live password=hunter2\npartial token=abc");
        feed.observe_written_log_bytes(b"123\n");
        feed.observe_written_log_bytes(&vec![b'x'; ENTRY_BYTE_LIMIT + 200]);
        feed.observe_written_log_bytes(b"\n");
        let retained = feed.retained();
        let joined = retained
            .iter()
            .map(|entry| entry.data.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        for secret in ["abc", "xyz", "sid=secret", "sk-live", "hunter2"] {
            assert!(!joined.contains(secret), "leaked {secret}: {joined}");
        }
        assert!(joined.contains("***"));
        assert!(joined.contains("partial token=***"));
        assert!(joined.contains("truncated"));
        assert!(retained.iter().all(|entry| entry.bytes <= ENTRY_BYTE_LIMIT));
    }

    #[test]
    fn rings_enforce_count_bytes_and_monotonic_ids() {
        let feed = DashboardActivityFeed::new();
        for index in 0..(ACTIVITY_EVENT_LIMIT + 20) {
            feed.project_model("mock", &format!("model-{index}"));
        }
        for index in 0..(LOG_LINE_LIMIT + 20) {
            feed.observe_written_log_bytes(format!("line-{index}\n").as_bytes());
        }
        let state = feed.lock_state();
        assert_eq!(state.activity.entries.len(), ACTIVITY_EVENT_LIMIT);
        assert_eq!(state.logs.entries.len(), LOG_LINE_LIMIT);
        assert!(state.activity.bytes <= ACTIVITY_BYTE_LIMIT);
        assert!(state.logs.bytes <= LOG_BYTE_LIMIT);
        let mut ids = state
            .activity
            .entries
            .iter()
            .chain(state.logs.entries.iter())
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn replay_distinguishes_window_stream_and_invalid_cursors() {
        let feed = DashboardActivityFeed::new();
        feed.project_model("mock", "one");
        feed.project_model("mock", "two");
        let stream = feed.inner.stream_id.clone();

        let in_window = feed.batch_after(&Cursor::parse(Some(&format!("{stream}:1"))));
        assert!(in_window.reset_reason.is_none());
        assert_eq!(in_window.entries.len(), 1);

        let old_stream = feed.batch_after(&Cursor::parse(Some("old:1")));
        assert_eq!(old_stream.reset_reason, Some("stream_changed"));
        assert_eq!(old_stream.entries.len(), 2);

        let invalid = feed.batch_after(&Cursor::parse(Some("invalid")));
        assert_eq!(invalid.reset_reason, Some("invalid_cursor"));

        for index in 0..(ACTIVITY_EVENT_LIMIT + 4) {
            feed.project_model("mock", &format!("evict-{index}"));
        }
        let overrun = feed.batch_after(&Cursor::parse(Some(&format!("{stream}:1"))));
        assert_eq!(overrun.reset_reason, Some("cursor_outside_window"));
    }

    #[tokio::test]
    async fn client_limit_and_drop_cleanup_are_deterministic() {
        let feed = DashboardActivityFeed::new();
        let streams = (0..SSE_CLIENT_LIMIT)
            .map(|_| feed.try_stream(None).expect("permit should be available"))
            .collect::<Vec<_>>();
        assert!(feed.try_stream(None).is_none());
        drop(streams);
        assert!(feed.try_stream(None).is_some());
    }
}
