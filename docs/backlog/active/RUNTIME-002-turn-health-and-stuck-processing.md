# RUNTIME-002: Turn Health And Stuck Processing Recovery

| Field | Value |
|---|---|
| Story ID | RUNTIME-002 |
| Priority | P0 |
| Status | In Progress (SSP140: engine-level is_processing verification complete) |
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
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-tui/src/app.rs`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/decisions/006-event-architecture-boundary.md`

