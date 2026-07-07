# Provider Runtime Hardening Next Phase

## Status

Planned.

## Trigger

The 2026-07-07 Alibaba-compatible provider incident showed that Talos can still enter a stuck
processing state when an OpenAI-compatible streaming provider emits partial tool-use metadata that
does not exactly match OpenAI's canonical shape. The immediate fix synthesizes missing streaming
tool-call ids and rejects `ToolUse` turns with no collected tool calls. The next phase hardens the
full provider/runtime boundary so similar compatibility issues fail visibly and are covered by
fixtures before users hit them in the TUI.

## Scope

- Provider stream parsing compatibility.
- Agent turn-loop invariants around `ToolUse`, tool calls, and tool results.
- TUI/runtime status visibility when waiting for provider response vs waiting for tool execution.
- Session-log evidence fidelity for tool-use incidents.

## Non-Goals

- No provider-specific secret handling changes.
- No permission policy redesign.
- No new model catalog storage layer.
- No release, tag, or publish action.

## Task List

| ID | Theme | Task | Acceptance | Verification |
|---|---|---|---|---|
| PRH00 | Baseline | Record the Alibaba missing-id tool-use fix as the baseline. | RUNTIME-002 references the incident, fix, and tests. | Existing commit + `cargo test -p talos-provider`; `cargo test -p talos-agent`. |
| PRH01 | Provider fixtures | Add OpenAI-compatible SSE fixture tests for missing id, split id/name/args chunks, empty final delta, `[DONE]` after `tool_calls`, and provider-specific usage-only chunks. | Each fixture either emits a complete `ToolCall` + `TurnEnd(ToolUse)` or emits a terminal `Error`; no fixture can produce `ToolCallStarted -> TurnEnd(ToolUse)` without `ToolCall`. | `cargo test -p talos-provider openai::tests::parse_sse_stream`. |
| PRH02 | Agent invariants | Add invariant tests for malformed provider event sequences: `ToolUse` with zero calls, tool calls without `ToolUse`, duplicate ids, and tool results rejected by provider on next turn. | Malformed sequences become explicit `AgentError::UnexpectedEvent` or bounded recoverable errors; valid multi-tool turns remain unchanged. | `cargo test -p talos-agent tool_use`; targeted new invariant tests. |
| PRH03 | Runtime/TUI status | Split processing visibility into at least model-waiting and tool-waiting phases using existing status plumbing. | While a turn is active, the UI can distinguish waiting for provider stream, waiting for local tool execution, and terminal failed/timed-out states. No background watchdog is added unless a deterministic transition cannot cover the case. | `cargo test -p talos-cli conversation_loop`; `cargo test -p talos-tui processing`. |
| PRH04 | Session evidence | Improve session/event persistence around tool-use incidents so JSONL or future binary logs preserve enough evidence to diagnose missing `ToolCall`/`ToolResult` sequences. | A reproduced malformed provider stream leaves a clear persisted terminal error or diagnostic entry; no silent processing-only tail. | `cargo test -p talos-session tool`; targeted CLI/session test if needed. |
| PRH05 | Provider metadata | Audit provider protocol metadata for Alibaba-compatible entries and confirm the selected adapter and endpoint shape are visible in diagnostics without requiring users to infer protocol behavior. | `/model` or config diagnostics can show provider protocol/adapter for configured providers; standard providers still do not ask for URL during connect. | `cargo test -p talos-config provider`; CLI diagnostic test if added. |
| PRH06 | Closeout | Run focused package tests plus workspace check and update owner docs. | RUNTIME-002 or follow-up owner docs list residuals, validation commands, and activation guidance for any deferred work. | `cargo fmt --all -- --check`; `cargo check --workspace`; governance validation. |

## Activation Guidance

- Start with PRH01 and PRH02. They are the highest-value regression net and should be completed
  before broad TUI status work.
- Treat PRH03 as user-visible polish only after provider/agent invariants are deterministic.
- If PRH05 reveals incorrect models.dev metadata mapping, split that into a separate provider
  catalog task instead of broadening this runtime hardening phase.

## Required Reads

- `crates/talos-provider/src/openai.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-conversation/src/engine.rs`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
