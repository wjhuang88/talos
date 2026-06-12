# TUI-004: TUI state model — unified messages, tips, and event-bus hook

## Outcome

The `talos-tui` crate adopts a structured state model where every message,
tip, and lifecycle transition carries explicit attributes. This replaces the
current flat `TuiState` field-pile (`chat_lines: Vec<ChatLine>`,
`status_message: Option<(String, Instant)>`, `is_processing: bool`, …) with
typed, attribute-bearing data structures that:

1. Make each message's role, status, and provenance inspectable at runtime.
2. Give tips a classification, text, and per-kind TTL (auto-expiring from the
   status bar, never written to scrollback).
3. Keep steering/followup queues independent but linked to message status.
4. Expose state changes through a `TuiStateEvent` channel so a future global
   event bus can subscribe without touching TUI internals.

## Motivation

The current `TuiState` has these problems:

- `ChatLine` has no lifecycle: there is no way to tell whether an assistant
  message is "streaming" or "completed", or whether a user message is
  "queued" or "accepted".
- `status_message: Option<(String, Instant)>` is a temporary patch — tips
  have different kinds and TTLs, but the model treats them all the same.
- All state changes are implicit (direct struct field mutations); there is no
  notification mechanism for a future global event bus to observe.
- `append_system` writes transient hints as permanent `ChatLine::Text` entries
  into scrollback, polluting the transcript with noise like "Press Ctrl+C
  again to exit."

This story makes the state machine explicit, auditable, and bus-ready.

## Status

Review. Depends on TUI-002 sub-slice A (I022, landed) for the inline-by-default
viewport model. Independent of TUI-002 sub-slices B-E (bottom_pane, etc.).
I023 has landed the event-driven `talos-conversation` boundary, Codex-style
single-row history insertion, multiline user blocks, one-row streaming preview,
Markdown block classification, and conservative styled Markdown rendering for
assistant/tool/system/error streams. Remaining closure work is tracked in the
I023 one-week handoff plan.

## Priority

P1. The current `status_message` hack and `ChatLine` without status are
already causing bugs (duplicate content, tip persistence, rendering glitches).
Fixing the data model removes a class of bugs by construction.

## Design

### Data types

```rust
/// Who produced this message.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MessageRole {
    User,
    Assistant,
    System,        // /help, /plugins, /status — persistent, shown in scrollback
    ToolCall,
}

/// Where this message is in its lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MessageStatus {
    Pending,       // steering/followup queue, not yet sent to agent
    Accepted,      // sent to agent, waiting for first delta
    Streaming,     // deltas arriving
    Completed,     // turn finished
    Failed,        // error
}

/// A single message in the transcript.
#[derive(Debug, Clone)]
pub(crate) struct ChatMessage {
    role: MessageRole,
    status: MessageStatus,
    content: String,
    tool_info: Option<ToolInfo>,   // name, args, provenance, result
    created_at: Instant,           // internal ordering only; not displayed
}

/// What kind of transient tip this is.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TipKind {
    CtrlCHint,      // "Press Ctrl+C again …" — TTL 2s
    SteeringHint,   // "Message queued (steering) …" — TTL 2s
    ApprovalResult, // "Tool call approved/denied" — TTL 3s
    LagWarning,     // "Warning: dropped N event(s)" — TTL 3s
}

/// A transient tip shown in the status bar.
#[derive(Debug, Clone)]
pub(crate) struct Tip {
    kind: TipKind,
    text: String,
    created_at: Instant,
    ttl: Duration,
}

/// TUI-internal state change events.
/// Future global event bus will subscribe to a translated version;
/// this is the TUI-layer interface only.
#[derive(Debug, Clone)]
pub(crate) enum TuiStateEvent {
    MessageAdded { index: usize, role: MessageRole },
    MessageStatusChanged { index: usize, old: MessageStatus, new: MessageStatus },
    TipAdded { kind: TipKind },
    TipExpired { kind: TipKind },
    ProcessingChanged { is_processing: bool },
}
```

### TuiState restructure

```rust
pub(crate) struct TuiState {
    // ── Transcript ──
    messages: Vec<ChatMessage>,
    // ── Streaming ──
    streaming_text: String,
    streaming_scrolled: usize,
    // ── Tips ──
    tips: Vec<Tip>,
    // ── Input ──
    input_buffer: String,
    cursor_pos: usize,
    // ── Queues (independent, linked to message status) ──
    steering_queue: Vec<String>,
    followup_queue: Vec<String>,
    // ── Control ──
    ctrl_c_state: CtrlCState,
    is_processing: bool,
    should_exit: bool,
    // ── Approval ──
    approval_state: ApprovalState,
    pending_approval_response: Option<tokio::sync::oneshot::Sender<ApprovalChoice>>,
    // ── Stats ──
    usage: Usage,
    model_name: String,
    branch_id: Option<String>,
    plugin_observations: Vec<PluginObservation>,
    // ── Event-bus hook ──
    event_tx: Option<mpsc::UnboundedSender<TuiStateEvent>>,
}
```

### Key behaviors

| Behavior | Current (hack) | New (model) |
|---|---|---|
| Ctrl+C hint | `status_message = Some(("…", now))` + manual TTL check | `Tip { kind: CtrlCHint, ttl: 2s }` + `expire_tips()` in main loop |
| Steering hint | `append_system("Message queued …")` → permanent ChatLine | `Tip { kind: SteeringHint, ttl: 2s }` → auto-expiring |
| User message | `ChatLine::Text("> 你好")` | `ChatMessage { role: User, status: Accepted }` |
| Queued message | `steering_queue.push(input)` with no ChatLine | `ChatMessage { role: User, status: Pending }` + `steering_queue.push(input)` |
| Assistant streaming | `current_turn_text` accumulation | `ChatMessage { role: Assistant, status: Streaming }` with `streaming_text` field |
| Assistant completed | `finalize_turn` → clone to ChatLine | `finalize_turn` → `status: Completed`, push content to scrollback |
| Tool call | `ChatLine::ToolCall { … }` with no lifecycle | `ChatMessage { role: ToolCall, status: Accepted }` → result updates status |
| State change notification | None (implicit mutation) | `event_tx.send(TuiStateEvent::…)` |

### Steering/followup queue relationship

Queues stay independent (`steering_queue: Vec<String>`,
`followup_queue: Vec<String>`), but when a message is queued, a
`ChatMessage { role: User, status: Pending }` is added to `messages`
with the queue index recorded. When the queued message is drained and
sent to the agent, its status transitions to `Accepted`. This makes
the transcript complete: every user input appears in `messages` with
its lifecycle visible, even when it was queued during a turn.

### Scrollback integration

`messages` replaces `chat_lines` as the source for scrollback flush.
`chat_line_to_text_lines` becomes `message_to_text_lines`, mapping
`MessageRole` to the same plain-text format. The `last_pushed_history`
index now counts into `messages` instead of `chat_lines`.

### Tip expiry

`expire_tips()` is called in the main loop every frame. Each `Tip` is
checked against `created_at + ttl`. Expired tips are removed and a
`TuiStateEvent::TipExpired` is emitted. In the future, tip creation
will also schedule a delayed event on the global bus for precise
expiry timing.

### Event-bus hook

`TuiState` holds an `Option<mpsc::UnboundedSender<TuiStateEvent>>`.
When `None`, state changes are silent (current behavior). When set,
every status transition, message addition, and tip lifecycle event is
emitted. The global event bus (future work) will translate
`TuiStateEvent` into its own type system. No translation is defined
yet — the channel is the interface contract.

## Acceptance Criteria

### Given/When/Then

- **Given** a TUI session, **when** user sends a message, **then** a
  `ChatMessage { role: User, status: Accepted }` appears in `messages`
  and its plain-text form is flushed to scrollback.

- **Given** a processing turn, **when** user presses Enter, **then** a
  `ChatMessage { role: User, status: Pending }` appears in `messages`,
  a `Tip { kind: SteeringHint }` appears in `tips`, and the message is
  added to `steering_queue`.

- **Given** a streaming turn, **when** TextDelta arrives, **then** the
  `streaming_text` grows and complete lines are flushed to scrollback;
  the last `Assistant` message in `messages` has `status: Streaming`.

- **Given** a streaming turn, **when** TurnEnd arrives, **then** the
  `Assistant` message status transitions to `Completed`, remaining
  streaming text is flushed, and `status_message` is removed from `tips`.

- **Given** a tip with TTL=2s, **when** 2 seconds pass, **then**
  `expire_tips()` removes it and emits `TipExpired`.

- **Given** an `event_tx` channel, **when** any message status changes,
  **then** a `TuiStateEvent::MessageStatusChanged` is sent.

- **Given** no `event_tx` (default), **when** any state changes,
  **then** behavior is identical to current (silent mutations).

### Structural

- `ChatLine` enum is removed; all references use `ChatMessage`.
- `status_message` field is removed; all references use `Tip`.
- `chat_lines` field is removed; all references use `messages`.
- All focused TUI/conversation tests pass after migration.
- `cargo check --workspace` and `cargo test --workspace` exit 0.

## Dependencies

- TUI-002 sub-slice A (I022) — landed. The inline-by-default viewport
  model is prerequisite: scrollback flush uses `insert_history`, viewport
  is fixed-height, and `current_turn_text` streaming is already
  incrementally flushed.

## Risks

| Risk | Mitigation |
|---|---|
| `ChatMessage` migration breaks `/copy`, `/export`, transcript serializers | Reuse `append_line_plain` / `append_line_markdown` logic; map `MessageRole` to the same text format |
| `Tip` expiry timing differs from current `status_message` TTL | TTL values match current behavior (2s for Ctrl+C, 3s for approval); no user-visible change |
| `event_tx` channel adds latency to state mutations | Channel is optional; when `None`, no overhead. Send is non-blocking (`UnboundedSender`) |
| Steering queue + Pending message duplication | Queue holds raw text; `ChatMessage` holds formatted display content. Both reference the same input but serve different purposes (dispatch vs. display) |
| Future Markdown block rendering hides streaming content while waiting for a block boundary | Keep preview one row, hold only the active structured block, expose classifier status in the preview animation, and fall back to visible plain rows for malformed, oversized, or unterminated blocks. See `docs/proposals/tui-stream-markdown-rendering.md` |

## Required Reads

- `crates/talos-tui/src/state.rs` — current `TuiState`, `ChatLine`, `CtrlCState`, `ApprovalState`
- `crates/talos-tui/src/app.rs` — `flush_scrollback`, `finalize_scrollback`, `build_status_text`, `handle_agent_event`, `handle_input_event`
- `docs/backlog/active/TUI-002-codex-overhaul.md` — inline-by-default architecture (prerequisite)
- `docs/iterations/I022-tui-inline-default.md` — I022 iteration record (prerequisite, landed)
- `docs/proposals/tui-stream-markdown-rendering.md` — future single-line and block Markdown recognition/rendering design
- `docs/proposals/reasoning-thinking-field.md` — future `ReasoningDelta` event will need `TuiStateEvent` hook
