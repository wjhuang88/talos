# TUI-041: Batched Steering Turn

| Field | Value |
| --- | --- |
| Story ID | TUI-041 |
| Type | Product / State Story |
| Priority | P1 |
| Status | In Progress — architecture rework required by PR #64/#68 review (2026-07-30) |
| Source | [GitHub Issue #50](https://github.com/wjhuang88/talos/issues/50) |
| Depends On | TUI-026/I145; ADR-005; ADR-006; ADR-039; ADR-049; ADR-056 |
| Selected Iteration | I169 |

## Identity / Goal / Value

An interactive Talos user who adds several steering messages while the current turn is processing
needs all queued inputs to reach the model in one consolidated follow-up turn, so the model can
consider the complete correction or additional context instead of executing one queued input per
turn.

## Scope

- Keep `ConversationEngine` as the single owner of the ordered steering queue.
- Drain only after authoritative `TurnEventPayload::Completed`, preserving the ADR-039 boundary.
- Use ADR-056's bridge-accepted cutoff and keep each accepted item's identity, kind, text, and
  attachments recoverable.
- Transfer one bounded compatible batch through prepare/reserve/send/acknowledge/commit without
  loss or duplicate execution.
- Make the Session Actor the sole arbiter after SQ acknowledgement; a normal Submit never
  implicitly cancels an active turn.
- Validate matching session/turn/sequence identity before completion can prepare a batch.
- Preserve queued items and pause after Cancel/Error; only Success auto-advances.
- Keep live UI, actor input, durable replay, scheduler ordering, and context bounds consistent.
- Preserve the existing public single-item drain method for downstream source compatibility; the
  Talos runtime uses the new batched method.
- Cover engine behavior and the real conversation-loop/session-submit path.

## Exclusions

- No concurrent model turns, persistent cross-session queue, queue editing, or arbitrary
  reordering controls.
- No source-time cutoff claim; an Enter press still waiting in `user_rx` belongs to the next batch.
- No new event bus, side channel, dependency, `unsafe`, permission, sandbox, provider, storage, or
  global protocol transport.
- No automatic semantic rewriting, summarization, delimiter-only authority, or deduplication of
  user input.
- No implicit move of retained input across `/new`, `/resume`, `/fork`, model, or provider changes.

## Decision Links And Constraints

- ADR-039 keeps authoritative user-turn completion in the existing ordered session event flow.
- ADR-049 keeps the engine as queue owner and the TUI as a bounded read-only projection. I169
  retains that rule before actor acknowledgement. ADR-056 owns the later transactional handoff,
  actor arbitration, terminal, persistence, scheduler, and bounded-batch semantics.
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
- Confirmed review evidence: PR #64 reviews `4819565218` and `4819746766`, PR #64 comment
  `5132132605`, and the migrated PR #68 gate comment `5132551226` require G0-G13 before review.
- The earlier inference that the change was a bounded drain-cardinality correction was false.
  ADR-056 records the corrected core-data-flow scope and the selected validation path.

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
  completes, then Talos submits one bounded follow-up user turn whose structured actor input and
  durable replay preserve A/B/C as three FIFO user items.
- Given B/C are still waiting in `user_rx` when matching completion is accepted, when A is already
  visible in the authoritative queue snapshot, then only A belongs to the current batch and B/C
  deterministically form a later batch.
- Given a provider response ends for tool use but the authoritative user turn is still active,
  when A, B, and C are queued, then Talos submits nothing and preserves all three entries.
- Given only one message is queued, when the turn completes, then its content is submitted
  unchanged.
- Given the queue is empty, when the turn completes, then no follow-up submit is created.
- Given downstream Rust code still calls the public single-item drain method, when it compiles
  against this release, then the method and FIFO single-item behavior remain available.
- Given inputs arrive after one batch was atomically drained, when the next authoritative turn
  completes, then those later inputs form a later batch and are not lost or merged retroactively.
- Given a stale, duplicate, wrong-session, wrong-turn, regression, or sequence-gap completion,
  when it arrives, then it cannot mutate active state or drain the queue and emits only bounded
  content-free diagnostics.
- Given bounded SQ is full, closed, times out, or switches sender during preparation, when batch
  transfer fails, then the original items, identities, attachments, FIFO, and snapshot remain.
- Given Cancelled or Error completes the matching turn, then pending user/scheduler work is
  retained and paused until an explicit user submission resumes it.
- Given scheduler and user work coexist, then the actor maintains one active turn, no normal
  Submit preempts it, and source-aware ordering follows ADR-056.
- Given PreviewRequest, multiline text, slash input, and attachments are interleaved, then each is
  classified before enqueue and incompatible items are never delimiter-joined.
- Given queue/batch/context limits are reached, then Talos rejects or splits only at item
  boundaries, never silently truncates or loses input.
- Given a batch completes and the session is resumed, then live user entries and durable replay
  have the same original item boundaries with no ghost or duplicate entries.

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
- Windows validation repaired the unrelated `talos-provider` Unix-only symlink fixture,
  `talos-memory` CRLF artifact comparison, and two hard-coded `/tmp` permission fixtures. Full
  workspace validation now reaches a pre-existing `talos-tools` portability blocker: 27 tests
  assume a host `sh`, Unix output/path separators, or Unix permission strings. Changing the tool's
  Windows process contract requires a separately reviewed security/runtime scope; it is not
  concealed by skipping those tests in TUI-041.
- All G0-G13 findings from PR #64/#68 are now in scope. The initial four-file join implementation
  is not acceptable completion evidence and must be replaced, not incrementally justified.
