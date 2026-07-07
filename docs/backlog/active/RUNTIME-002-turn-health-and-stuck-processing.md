# RUNTIME-002: Turn Health And Stuck Processing Recovery

| Field | Value |
|---|---|
| Story ID | RUNTIME-002 |
| Priority | P0 |
| Status | In Progress (FS04: runtime-level integration coverage + MaxTokens clearing fix + visible signal verification complete) |
| Source | [GitHub Issue #18](https://github.com/wjhuang88/talos/issues/18), [GitHub Issue #32](https://github.com/wjhuang88/talos/issues/32) |
| Depends On | `RUNTIME-001`, `TUI-027`, `PROVIDER-002` |

## Problem

Tool errors, provider failures after tool results, or event-chain drops can leave the UI stuck in a
processing state with no visible progress. Users cannot tell whether Talos is waiting for the
provider, running a tool, or already wedged.

## Acceptance

- Reproduce or simulate the #18 path where a tool result/error is followed by provider failure.
- Ensure every terminal error path clears `is_processing` and emits a user-visible terminal status.
- Add bounded health/status evidence for long-running turns: provider wait, tool execution, idle
  waiting, timeout, cancelled, and failed.
- If a health-check task is added, it must be an internal `tokio` task with a single owner and no
  global event bus.
- Auto-recovery actions must be conservative: notification and state cleanup first; provider retry,
  context compaction, or turn restart require explicit design and tests.

## Non-Goals

- No release gate change.
- No permission relaxation.
- No new background OS process.

## Required Reads

- `crates/talos-agent/src/lib.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-conversation/src/engine_tests.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-tui/src/app.rs`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/decisions/006-event-architecture-boundary.md`

## FS01 Audit: Terminal Event Paths And Integration Surfaces (2026-07-07)

Audit source: `crates/talos-conversation/src/engine.rs` (`handle_agent_event` lines 277-423,
`cancel_turn` lines 256-275). `StopReason` variants per `crates/talos-core/src/message.rs:126-133`:
`EndTurn`, `ToolUse`, `MaxTokens`.

### Terminal event paths

| # | Path | Clears `is_processing`? | Sets `current_phase`? | Existing engine tests |
|---|---|---|---|---|
| 1 | `TurnEnd { stop_reason: EndTurn }` | YES (engine.rs:364) | `None` (366) | `turn_end_finalizes_and_produces_status`, `turn_end_with_empty_text_still_produces_status`, `full_turn_lifecycle`, `phase_transitions_turnstart_to_thinking_to_generating_to_end` |
| 2 | `TurnEnd { stop_reason: ToolUse }` | NO — but mid-turn (followed by `ToolCall`) | `None` (366) | covered indirectly by tool-call lifecycle tests |
| 3 | `TurnEnd { stop_reason: MaxTokens }` | **NO — GAP** (engine.rs:363-365 only matches `EndTurn`) | `None` (366) | **NONE** — no engine test covers `MaxTokens` `TurnEnd` |
| 4 | `Error { message }` (non-timeout) | YES (engine.rs:380) | `Failed` (384) | `error_clears_turn_and_produces_stream_and_status`, `error_after_tool_call_clears_processing`, `error_after_tool_result_clears_processing`, `error_without_prior_turn_clears_processing`, `error_sets_visible_terminal_phase`, `error_message_becomes_tip_and_error_stream` |
| 5 | `Error { message }` (timeout) | YES (engine.rs:380) | `TimedOut` (382) | `timeout_error_sets_timed_out_phase`, `error_sets_visible_terminal_phase` |
| 6 | `cancel_turn()` (user `UserInput::Cancel`) | YES (engine.rs:258) | `Cancelled` (259) | `cancel_turn_clears_processing_state` |

### Findings

- Engine-level clearing on `Error` and `cancel_turn` is correct and has deterministic coverage
  (5 SSP140 tests in `engine_tests.rs:1856-1971` plus pre-existing error/cancel tests). The engine
  `Error` handler unconditionally clears `is_processing` and emits `UiOutput::Status` + an error
  `Tip` + an error `Stream`.
- **`TurnEnd { MaxTokens }` clearing gap (engine.rs:363-365):** the engine only clears
  `is_processing` when `stop_reason == EndTurn`. A `TurnEnd { MaxTokens }` would leave
  `is_processing == true` with `current_phase == None`, which would keep the TUI spinner running
  with no phase label. `StopReason::MaxTokens` is a real variant and `tool_execution.rs:461` maps
  it to `TurnEndReason::MaxTokens`. Whether the agent session actually forwards a `MaxTokens`
  `TurnEnd` to the conversation engine (vs. retrying, truncating, or converting to `EndTurn`) is
  not proven by any test today; FS02 must add a deterministic integration test that settles this.
- **No runtime-level integration coverage.** Every existing test drives
  `engine.handle_agent_event(...)` directly. No test drives the full pipeline
  (provider/agent session → `AgentEvent` stream → `tui_bridge::run_conversation_loop` →
  `UiOutput::Status` → TUI state) to prove a terminal error, timeout, or cancel actually reaches
  the UI with `is_processing == false`.
- **No TUI-bridge cancel test.** `tui_bridge.rs:143-149` handles `UserInput::Cancel` by calling
  `engine.cancel_turn()` and forwarding outputs, but no test drives a `Cancel` through the bridge.
- **No TUI spinner-reset test tied to terminal status.** TUI tests in `crates/talos-tui/src/tests.rs`
  use hand-crafted `is_processing: false` snapshots; none drive a terminal `Status` through the
  conversation loop and assert the spinner stops.

### FS02 integration surfaces to cover

1. Deterministic test proving a terminal provider/tool error clears `is_processing` end-to-end at
   the runtime level (not only at the engine-unit level). Narrowest target: drive the conversation
   loop with a scripted `AgentEvent::Error` after `TurnStart` + `ToolCall` + `ToolResult` and
   assert the emitted `UiOutput::Status` has `is_processing == false`.
2. Deterministic test for `TurnEnd { MaxTokens }` reachability and clearing behavior. Either:
   (a) prove the agent never forwards `MaxTokens` as a terminal `TurnEnd` to the engine (document
       and test that boundary), or
   (b) prove the engine must clear `is_processing` on `MaxTokens` and add the engine fix + test.
3. Optional: deterministic test for `UserInput::Cancel` through `tui_bridge` emitting a terminal
   `Status { is_processing: false, phase: Cancelled }`.

### FS03 visible-signal surfaces

- The engine already emits `UiOutput::Tip { kind: Error }`, `UiOutput::Stream { source: Error }`,
  and `UiOutput::Status { is_processing: false, phase: Failed/TimedOut }` on every `Error` path.
- FS03 should verify (and only if needed supplement) that the TUI surfaces these as a visible
  terminal status without changing provider semantics or adding background watchdog threads.

## FS02-FS03 Execution Evidence (2026-07-07)

### FS02: runtime-level integration coverage + MaxTokens clearing fix

- **Fix:** `crates/talos-conversation/src/engine.rs` `TurnEnd` handler now clears `is_processing`
  for any stop reason except `ToolUse` (previously only `EndTurn`). This closes the `MaxTokens`
  stuck-processing gap where a provider returning `MaxTokens` would leave `is_processing == true`
  with `phase == None`, keeping the TUI spinner running with no phase label.
- **Engine tests added** (`engine_tests.rs`):
  - `turn_end_max_tokens_clears_processing` — proves `MaxTokens` now clears `is_processing`.
  - `turn_end_tool_use_keeps_processing_for_continuation` — regression guard proving `ToolUse`
    still keeps `is_processing` true for the following tool-call continuation.
- **Conversation-loop integration tests added** (`crates/talos-cli/src/tests.rs`):
  - `conversation_loop_clears_processing_on_provider_error_after_tool_result` — drives
    `TurnStart → ToolCall → ToolResult → Error` through `run_conversation_loop` and asserts the
    final `UiOutput::Status` has `is_processing == false`, `phase == Failed`.
  - `conversation_loop_clears_processing_on_timeout_error` — asserts `phase == TimedOut`.
  - `conversation_loop_clears_processing_on_max_tokens_turn_end` — asserts the MaxTokens fix
    reaches the UI through the full bridge path.

### FS03: visible signal verification + success-path regression guard

- **Finding:** the TUI already surfaces terminal phases visibly: `preview_text_for_state`
  (`app.rs:1071-1079`) renders `"⏱ timed out"`, `"✗ failed"`, `"cancelled"`; `scrollback_status.rs:280-282`
  maps phases to status-bar text; `scrollback.rs:243` renders `TipKind::Error` with error coloring.
  No TUI refactor was needed.
- **Integration tests added** (`crates/talos-cli/src/tests.rs`):
  - `conversation_loop_emits_visible_error_signals_on_terminal_failure` — proves the conversation
    loop forwards `UiOutput::Tip { kind: Error }`, `UiOutput::Stream { source: Error }`, and a
    terminal `UiOutput::Status { is_processing: false }` on a provider error.
  - `conversation_loop_normal_end_turn_success_path_unchanged` — regression guard proving the
    normal `EndTurn` success path still clears `is_processing` and resets `phase` to `None`.

### Residuals

- The optional `UserInput::Cancel` through `tui_bridge` integration test (FS01 surface #3) was not
  added; the existing engine-level `cancel_turn_clears_processing_state` test plus the bridge's
  straightforward forwarding of `engine.cancel_turn()` outputs provide sufficient coverage for the
  current frontline package. A future iteration can add this if cancel-path regressions appear.
- No health-check watchdog task was added; per RUNTIME-002 acceptance and the frontline plan,
  deterministic state transitions are preferred over background polling loops.

