# RUNTIME-002: Turn Health And Stuck Processing Recovery

| Field | Value |
|---|---|
| Story ID | RUNTIME-002 |
| Priority | P0 |
| Status | Resolved — #18 request-dispatch timeout fixed in I107 SBT111 (non-qualifying REL-002 evidence) |
| Source | [GitHub Issue #18](https://github.com/wjhuang88/talos/issues/18), [GitHub Issue #32](https://github.com/wjhuang88/talos/issues/32) |
| Depends On | `RUNTIME-001`, `TUI-027`, `PROVIDER-002` |

## Problem

Tool errors, provider failures after tool results, or event-chain drops can leave the UI stuck in a
processing state with no visible progress. Users cannot tell whether Talos is waiting for the
provider, running a tool, or already wedged.

## Acceptance

- [x] Reproduce or simulate the #18 path where a tool result/error is followed by provider failure,
      including provider HTTP request dispatch that never returns response headers.
- [x] Ensure terminal `AgentEvent::Error`, timeout, cancellation, `EndTurn`, and `MaxTokens` paths
      clear `is_processing` and emit a user-visible terminal status.
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

## 2026-07-08 Status Correction: #18 Not Fully Fixed

GitHub Issue #18 was closed incorrectly on 2026-07-08. The FS02/FS03 work fixed real stuck-state
paths in the conversation engine and bridge, but it did not fix the #18 root cause identified in
the issue comments: provider HTTP request dispatch can hang before response headers arrive.

Current code evidence:

- `crates/talos-provider/src/openai.rs` still constructs `reqwest::Client` with `Client::new()`.
- `crates/talos-provider/src/lib.rs` still constructs `reqwest::Client` with `Client::new()`.
- `ProviderTimeoutConfig` only contains `first_packet_timeout_secs` and
  `stream_idle_timeout_secs`; these protect stream parsing after a response exists, not
  `send().await` before response headers.

Required follow-up:

- Add a request-dispatch timeout for OpenAI-compatible and Anthropic providers, either by
  configuring `reqwest::Client::builder().timeout(...)` carefully or by wrapping provider
  `send().await` in `tokio::time::timeout`.
- Preserve stream-idle semantics for long streaming responses; do not accidentally impose a total
  response-body timeout that breaks valid long streams.
- Add deterministic runtime evidence for a provider call that accepts the request but never returns
  response headers, proving Talos emits a terminal provider timeout/error and clears processing.

This follow-up is selected into the 2026-07-08 Talos self-bootstrap plan before any lower-priority
feature polish.

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

## Post-Closeout Regression Fix (2026-07-07)

### Alibaba/OpenAI-Compatible ToolUse With Missing Tool IDs

- **Observed path:** an Alibaba-compatible provider streamed `tool_calls` function names
  (`tree`, `read`) and ended with `stop_reason == ToolUse`, but no complete `ToolCall` reached the
  agent because the streaming delta omitted `tool_call.id`. The TUI received
  `ToolCallStarted -> TurnEnd(ToolUse)` and correctly kept `is_processing == true` for the expected
  tool continuation, but no tool execution followed, leaving the UI stuck.
- **Provider fix:** `crates/talos-provider/src/openai.rs` now synthesizes a stable per-response id
  (`call_0`, `call_1`, ...) when an OpenAI-compatible streaming tool call has name/arguments but no
  id. This keeps assistant tool calls and following tool results pairable in the next request.
- **Agent guard:** `crates/talos-agent/src/lib.rs` now rejects `TurnEnd(ToolUse)` with zero
  collected tool calls as `AgentError::UnexpectedEvent` instead of treating it as a successful
  text-only turn.
- **Regression tests:**
  - `talos-provider::openai::tests::parse_sse_stream_synthesizes_missing_tool_call_id`
  - `talos-agent::tests::test_run_rejects_tool_use_without_tool_calls`
- **Validation:** `cargo test -p talos-provider parse_sse_stream_synthesizes_missing_tool_call_id`;
  `cargo test -p talos-agent tool_use`; `cargo fmt --all -- --check`;
  `cargo check --workspace`.

## FP1-FP2 Provider Runtime Hardening (2026-07-07)

Source: `docs/tasks/2026-07-07-provider-runtime-hardening-next-phase.md` (PRH01/PRH02).

### FP1 Provider SSE Fixtures

- **Bug found and fixed:** the `[DONE]` path in `parse_sse_stream` silently dropped accumulated
  native tool calls when a provider streamed `tool_calls` deltas but closed the stream with
  `[DONE]` and no `finish_reason` chunk. This produced `ToolCallStarted -> TurnEnd(EndTurn)` with
  no `ToolCall` — a stuck path. The `[DONE]` path now emits accumulated native tool calls (mirroring
  the stream-end fallback) and sets `stop_reason` to `ToolUse` when any are present.
- **Fixture tests added** (`cargo test -p talos-provider openai::tests::parse_sse_stream`, 8 total):
  - `parse_sse_stream_accumulates_split_id_name_args_chunks`
  - `parse_sse_stream_empty_final_delta_clean_end_turn`
  - `parse_sse_stream_done_after_tool_calls_emits_tool_use` (proves the fix)
  - `parse_sse_stream_malformed_tool_arguments_becomes_empty_object`
  - `parse_sse_stream_usage_chunk_interleaved_with_tool_calls`
  - `parse_sse_stream_multi_tool_missing_ids_synthesizes_unique_indices`
- **Acceptance met:** no fixture produces `ToolCallStarted -> TurnEnd(ToolUse)` without `ToolCall`.
- **Commit:** `bf79b39`.

### FP2 Agent ToolUse Invariants

- **Guard added:** `run_inner` now rejects duplicate tool call ids within a single provider response
  as `AgentError::UnexpectedEvent`. Text-based tool calls are unaffected (`parse_json_tool_call`
  assigns unique UUIDs).
- **Invariant tests added** (covered by full `cargo test -p talos-agent`; the `tool_use` filter
  matches `test_run_rejects_tool_use_without_tool_calls`):
  - `test_run_rejects_duplicate_tool_call_ids` (new guard)
  - `test_run_end_turn_with_tool_calls_executes_recoverably` (`EndTurn` + tool_calls is bounded-
    recoverable: tools execute, turn continues; not rejected because text-based tools legitimately
    produce this pattern)
  - `test_run_provider_error_after_tool_results_is_terminal` (tool result then provider `Error`
    produces terminal `UnexpectedEvent`, not stuck)
  - `test_run_rejects_tool_use_without_tool_calls` (existing regression guard, unchanged)
- **Valid multi-tool turn semantics unchanged:** 201 agent unit tests pass.
- **Commit:** `c26b79a`.

### Residuals (FP1-FP2)

- SSE comment lines (`: keepalive`) and `retry:` directives from proxies/gateways are handled
  implicitly by `extract_event_data` (lines without `data:` prefix are skipped) but are not
  covered by an explicit fixture. A future iteration can add fixtures if a real provider sends
  these and breaks parsing.
- `data: DONE` (without brackets, emitted by some legacy stubs) is not matched by the `[DONE]`
  check; the stream-end fallback still emits a terminal `TurnEnd`, so it is not a stuck path, but
  it is not optimally handled. Future fixture candidate.
- Rejecting `EndTurn`/`MaxTokens` + non-empty tool calls was deliberately deferred: text-based tool
  calls (`parse_text_tool_calls`) legitimately produce `EndTurn + ToolCall` events, so a blanket
  rejection would break that path. If text-based and native tool calls need different stop_reason
  semantics, a future design must distinguish them first.
- FP3-FP8 packets (processing status visibility, session evidence, connect diagnostics, large model
  UX, tool output ergonomics, trial docs) are not started; they remain in the task doc for future
  frontline assignment.

## I102 D101 SSE Fixture Matrix Extension (2026-07-07)

Source: `docs/iterations/I102-provider-runtime-reliability-gate.md` (D101).

- Six additional deterministic `parse_sse_stream_*` fixtures landed in
  `crates/talos-provider/src/openai.rs::tests` covering paths FP1-FP2 did not have an explicit
  fixture for: `finish_reason="length" → StopReason::MaxTokens`, role-only first chunk consumed
  without spurious emit, SSE `: keepalive` / `retry:` / empty `data: ` passthrough, mixed
  content + tool_calls in one delta, and multi-byte UTF-8 round-trip.
- No production parser change was needed; these are pure regression guards. The full fixture
  matrix now covers split chunks, missing ids, duplicate ids (agent-side), `[DONE]`-after-tool-call,
  usage interleaving, malformed args degradation, multi-tool missing ids, MaxTokens surfacing,
  keepalive/comment lines, empty data events, mixed-delta tool+text, and UTF-8 round-trip.
- Validation: `cargo test -p talos-provider openai::tests::parse_sse_stream` → 14 passed;
  `cargo test --workspace` → 1784 passed (was 1778 at D100 baseline); clippy/fmt/governance clean.
- Residual: D102 will extend the agent-layer invariant (rejecting malformed provider event
  sequences that the SSE fixtures cannot themselves prevent, e.g. a provider error chunk mid-stream
  that is silently consumed because `chunk.choices.is_empty()` returns true). That residual is
  already recorded under I102 `## Variance And Residuals`.

## I102 D102 Agent Turn-Loop Invariant Extension (2026-07-08)

Source: `docs/iterations/I102-provider-runtime-reliability-gate.md` (D102).

- Extended the agent turn-loop invariant set in
  `crates/talos-agent/src/lib.rs::run_inner` with a defensive guard that rejects degenerate
  `ToolCall`s (empty id or empty/whitespace name) before they enter tool execution or the next
  provider request. The OpenAI SSE parser already synthesizes ids and skips empty names, but
  other providers (Anthropic, MCP bridging, future runtimes) must not be able to silently push a
  degenerate ToolCall that would later fail tool lookup or produce ambiguous request/response
  pairing.
- Added regression guard that `StopReason::MaxTokens` without tool calls is a successful
  (truncated) agent turn, locking the boundary between agent invariant guards (degenerate tool
  paths only) and the engine-level FS04 fix that clears `is_processing` on MaxTokens.
- Four new invariant tests in `crates/talos-agent/src/tests.rs`:
  `test_run_rejects_tool_call_with_empty_name`,
  `test_run_rejects_tool_call_with_empty_id`,
  `test_run_rejects_whitespace_only_tool_call_name`,
  `test_run_max_tokens_stop_reason_without_tool_calls_is_terminal_success`.
- Validation: `cargo test -p talos-agent` → 205 passed (was 201); `cargo test --workspace` →
  1788 passed (was 1784); clippy/fmt/governance clean.
- Architecture review correction: the mid-stream provider-error-chunk path (provider sends an HTTP
  200 stream that includes a `{"error": ...}` chunk with `choices: []`) was not correctly bounded
  by the agent's `channel closed before TurnEnd` invariant. The parser skipped empty choices and
  could fall through to its EOF `TurnEnd(EndTurn)` fallback. The fix is now parser-level:
  `crates/talos-provider/src/openai.rs::parse_sse_stream` detects `data.error`, emits terminal
  `AgentEvent::Error`, and returns before any success fallback.
- Regression evidence:
  `cargo test -p talos-provider openai::tests::parse_sse_stream_error_chunk_emits_terminal_error`
  → 1 passed; `cargo test -p talos-provider
  openai::tests::extract_openai_stream_error_reads_object_error` → 1 passed.
- Post-fix validation: `cargo test -p talos-provider openai::tests::parse_sse_stream` →
  15 passed; `cargo check --workspace` → passed; `cargo test --workspace` → 1791 passed /
  0 failed / 0 ignored; `cargo clippy -p talos-provider -- -D warnings` → passed;
  `scripts/validate_project_governance.sh .` → 0 warnings; `git diff --check` → clean.

## I102 D103 Conversation-Loop Cancel Integration (2026-07-08)

Source: `docs/iterations/I102-provider-runtime-reliability-gate.md` (D103).

- Closed the FS01 surface #3 optional residual: added integration-level coverage proving
  `UserInput::Cancel` through the full conversation-loop bridge produces a terminal
  `UiOutput::Status { is_processing: false, phase: Cancelled }`.
- The engine-level `cancel_turn_clears_processing_state` test already covered the engine in
  isolation; the new test `conversation_loop_cancel_emits_terminal_cancelled_status` in
  `crates/talos-cli/src/tests.rs` drives the full bridge path
  (`UserInput::Cancel` → `run_conversation_loop` → `engine.cancel_turn()` → `UiOutput`).
- All five terminal phases now have conversation-loop integration coverage: Failed (FS02),
  TimedOut (FS02), Cancelled (D103), MaxTokens-clear (FS02), normal-EndTurn (FS03).
- Validation: `cargo test -p talos-cli --bin talos -- conversation_loop` → 9 passed (was 8);
  `cargo test --workspace` → 1789 passed (was 1788); clippy/fmt/governance clean.
- No production code change. The existing FS03 visible-signal surfaces were already verified
  in the FS03 closeout.

## I107 SBT111: Request-Dispatch Timeout Fix (2026-07-09)

### Problem Solved
The #18 root cause — `reqwest::Client::new()` had no request-level timeout, so `send().await` could hang indefinitely before response headers arrive — is now fixed.

### Implementation
- Added `dispatch_timeout_secs: u64` (default: 60) to `ProviderTimeoutConfig` (`crates/talos-config/src/types.rs`)
- Wrapped `send().await` in `tokio::time::timeout(dispatch_timeout, request_fut)` in both `OpenAIProvider::send_request` and `AnthropicProvider::send_request`
- Dispatch timeout errors classify as `ProviderError::NetworkError("request dispatch timeout: no response headers within Ns")` — retryable via `classify_retry_with_backoff`
- Stream-idle semantics preserved: timeout covers ONLY dispatch → headers, NOT streaming body (still protected by `first_packet_timeout_secs` and `stream_idle_timeout_secs`)

### Tests
- `test_dispatch_timeout_openai` — deterministic proof that a server that accepts connection but never returns headers emits a terminal dispatch timeout error
- `test_normal_request_not_dispatch_timed_out_openai` — regression guard: normal requests with fast responses are NOT timed out
- `test_dispatch_timeout_anthropic` — same for Anthropic provider
- `test_normal_request_not_dispatch_timed_out_anthropic` — same regression guard for Anthropic

### Validation
- 1795 workspace tests pass (was 1791; +4 new dispatch timeout tests)
- cargo fmt --all -- --check: clean
- cargo clippy --workspace -- -D warnings: no warnings
- scripts/validate_project_governance.sh .: 0 warnings
- scripts/talos_smoke.sh: 9/9 passed

### REL-002 Classification
Runtime: glm-5.2 via zai-coding-plan (external, NOT Talos). Per REL-002 criterion 7 ("Codex is not the primary executor for qualifying sessions"), this is NON-QUALIFYING evidence. The fix is real and useful for future Talos-primary sessions, but this session does not prove self-bootstrap capability.
