# TUI-041: Batched Steering Turn

| Field | Value |
| --- | --- |
| Story ID | TUI-041 |
| Type | Product / State Story |
| Priority | P1 |
| Status | In Progress — implementation ready; I169 workspace validation blocked (2026-07-30) |
| Source | [GitHub Issue #50](https://github.com/wjhuang88/talos/issues/50) |
| Depends On | TUI-026/I145; ADR-039; ADR-049 |
| Selected Iteration | I169 |

## Identity / Goal / Value

An interactive Talos user who adds several steering messages while the current turn is processing
needs all queued inputs to reach the model in one consolidated follow-up turn, so the model can
consider the complete correction or additional context instead of executing one queued input per
turn.

## Scope

- Keep `ConversationEngine` as the single owner of the ordered steering queue.
- Drain only after authoritative `TurnEventPayload::Completed`, preserving the ADR-039 boundary.
- Atomically drain every message present at that boundary and join them in FIFO order with a
  visible blank-line separator.
- Submit exactly one follow-up session operation for the drained batch.
- Clear the bounded queue preview after the batch is removed.
- Preserve the existing public single-item drain method for downstream source compatibility; the
  Talos runtime uses the new batched method.
- Cover engine behavior and the real conversation-loop/session-submit path.

## Exclusions

- No concurrent model turns, background scheduler, persistent queue, cross-session queue, queue
  editing, reordering, or cancellation controls.
- No change to when steering is eligible to drain.
- No new event bus, side channel, dependency, `unsafe`, permission, sandbox, provider, storage, or
  public protocol variant.
- No automatic semantic rewriting, summarization, or deduplication of user input.

## Decision Links And Constraints

- ADR-039 keeps authoritative user-turn completion in the existing ordered session event flow.
- ADR-049 keeps the engine as queue owner and the TUI as a bounded read-only projection. I169
  changes only how the authoritative queue is consumed after completion, not its ownership,
  projection, or drain timing.
- `AGENTS.md` public API compatibility is a Hard constraint. The already-existing implementation
  commit removed `ConversationEngine::drain_steering_queue`; I169 must restore that method and add
  the batched method without requiring a breaking migration.

## Uncertainty And Validation Path

- GitHub Issue #50 says all queued inputs should be read together, but does not prescribe a wire
  schema. The smallest compatible representation is one user message with original entries joined
  by `\n\n`; focused tests make that boundary explicit.
- The initial implementation commit `6da9d71c` predates this governance owner. It is treated as an
  implementation candidate under review, not completion evidence.
- An initial `git fetch origin` failed with an HTTPS connection error. The PR preparation retry on
  2026-07-30 succeeded and refreshed `origin/main` to `b5fcaaf3`; the branch must be rebased onto
  that commit and revalidated before push.

## State / Status Owners

- Story scope and acceptance: this file.
- Execution and evidence: `docs/iterations/I169-batched-steering-turn.md`.
- Operating view: `docs/BOARD.md`.
- Compact backlog: `docs/backlog/PRODUCT-BACKLOG.md`.
- Earlier queue-display baseline: `docs/backlog/active/TUI-026-queued-input-display.md`.

## User-Facing Documentation

- `README.md`
- `README.zh-CN.md`

## Required Reads

- `AGENTS.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/TESTING.md`
- `docs/decisions/039-runtime-event-semantic-single-flow.md`
- `docs/decisions/049-steering-queue-projection-boundary.md`
- `docs/backlog/active/TUI-026-queued-input-display.md`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-conversation/src/engine_tests.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-cli/src/tests.rs`

## Acceptance

- Given a turn is processing and the user queues A, B, and C, when the authoritative turn
  completes, then Talos submits exactly one follow-up user turn containing `A\n\nB\n\nC` and the
  queue preview becomes empty.
- Given a provider response ends for tool use but the authoritative user turn is still active,
  when A, B, and C are queued, then Talos submits nothing and preserves all three entries.
- Given only one message is queued, when the turn completes, then its content is submitted
  unchanged.
- Given the queue is empty, when the turn completes, then no follow-up submit is created.
- Given downstream Rust code still calls the public single-item drain method, when it compiles
  against this release, then the method and FIFO single-item behavior remain available.
- Given inputs arrive after one batch was atomically drained, when the next authoritative turn
  completes, then those later inputs form a later batch and are not lost or merged retroactively.

## Minimum Validation

- `cargo test --locked -p talos-conversation drain_steering_queue`
- `cargo test --locked -p talos-cli conversation_loop_batches_all_queued_steering_into_one_submit`
- `cargo test --locked -p talos-cli conversation_loop_keeps_steering_queued_across_provider_tool_end`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `powershell -NoProfile -File scripts/validate_project_governance.ps1 .`
- Rebuilt `talos --mock --print --no-init --no-context` smoke test.

## Residual Destination

- Any request for structured multi-message provider content, queue persistence, or queue controls
  becomes a separate backlog item and decision gate; it is not silently added to I169.
- The unrelated Windows-only `talos-provider` Unix-symlink test compile failure and
  `talos-memory` benchmark line-ending failure remain repository baseline remediation work; they
  block the full workspace gate but do not expand TUI-041.
