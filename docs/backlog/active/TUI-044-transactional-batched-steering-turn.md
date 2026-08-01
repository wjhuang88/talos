# TUI-044: Transactional Batched Steering Turn

| Field | Value |
|---|---|
| Story ID | TUI-044 |
| Type | TUI / Runtime State Story |
| Priority | P1 |
| Status | Active — Draft PR #131 |
| Source | [GitHub Issue #119](https://github.com/wjhuang88/talos/issues/119) |
| Selected Iteration | I169 |
| Depends On | TUI-026/I145; ADR-005; ADR-006; ADR-039; ADR-042; ADR-049; Proposed ADR-056; completed I170 baseline |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / I169 implementation session 2026-08-02 |
| Work Slice | TUI-044/I169 only: structured queue admission, transactional Engine-to-Actor transfer, durable pending custody and receipts, lifecycle correlation, Actor arbitration, exact Provider request planning and replay parity. |
| Claimed At | 2026-08-01 |
| Activated At | 2026-08-02 02:32 +08:00 |
| Source Issue | #119 |
| Governance Claim PR | #123 |
| Preactivation Architecture PR | #129 |
| Implementation PR | #131 |
| Authorization Mode | Explicit single-maintainer instruction |
| Authorization Evidence | Maintainer instructed “正式激活并实施 I169” on 2026-08-02. PR #129 passed exact-head macOS/Windows, governance, collaboration, mock-smoke and remote-reconciliation gates and merged at `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`. |
| Implementation Baseline | `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae` |
| Implementation Branch | `feat/i169-tui-044-transactional-steering` |
| Last Updated | 2026-08-02 |
| Handoff / Release Condition | No release claim until PR #131 is merged, ADR-056 is reviewed, exact-head validation and rebuilt real-TUI acceptance pass, and completion evidence is recorded. |

Recovery PR #120 and `recovery/pr-68-i169-20260731` remain immutable historical evidence and are not
implementation parents. Current `TUI-041` remains owned by Issue #69.

## Identity / Goal / Value

A user who submits steering, correction, attachment-bearing or additional-context items while one
model Turn is active needs compatible items accepted before one deterministic cutoff to become one
bounded later Turn without losing item identity, exact boundaries, FIFO order, attachment binding,
persistence semantics or retryability.

## Provenance

- Recovered Issue: #119, reconstructed from deleted Issue #50.
- Archival recovery PR/head: #120 / `c984b71022a16169f26dec9f2e4a73b78a41a93d`.
- I170 prerequisite completed through PR #126.
- Proposed architecture hardening completed through PR #129.
- Windows loopback fixture repair PR #130 is independent maintenance, not I169 product scope.
- Formal implementation base/branch/PR: `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae` /
  `feat/i169-tui-044-transactional-steering` / #131.

## Scope

- Structured pre-Actor queue items preserve stable item ID, target Session identity/generation,
  source/kind, exact text, attachments, FIFO sequence and bounded metadata.
- Ordinary user input, local/slash commands, preview requests, scheduler work and attachments are
  classified before queue admission.
- The only cutoff is the Engine-accepted enqueue-sequence snapshot when matching authoritative
  Success is processed.
- One compatible bounded prefix transfers through `prepare -> reserve -> send -> durable accept ->
  receipt reconciliation -> Engine commit`.
- After send, Engine retains immutable non-executable escrow until the exact receipt is reconciled.
- Actor Ack occurs only after atomic, idempotent acceptance into a versioned session-scoped pending
  journal separate from successful transcript.
- Lost Ack reconciles through exact `AlreadyAccepted` or authoritative `NotAccepted`; ambiguous or
  conflicting state fails closed without cross-Session replay.
- Structured operations, receipts and Turn events carry Session, generation, batch/reservation/
  receipt/Turn identity and exact monotonic sequence.
- The Session Actor is sole execution authority after durable acceptance, keeps at most one active
  Turn and arbitrates user/scheduler work without ordinary Submit preemption.
- Matching Success may auto-advance. Cancel/Error pause unstarted pending work; an already-started
  terminal Turn is not automatically replayed.
- A/B/C remain distinct ordered `Message::User` or `Message::Multimodal` values in Actor input,
  Provider requests, successful transcript and resumed history.
- Success commits transcript before pending-journal finalization; recovery uses Turn identity to
  finish cleanup without re-execution.
- Every initial and continuation Provider call budgets and sends the same sealed exact request plan,
  including prompts, memory, workspace context, history, tools/schemas, structured input,
  multimodal cost, continuation overlay and output reserve.
- Public `ConversationEngine::drain_steering_queue` and legacy Session operations retain compatible
  behavior; structured variants are additive and migration-aware.

## Non-Goals

- No I170 process/path/fixture work, concurrent model Turns, global bus, queue editing/reordering,
  semantic rewrite, permission/sandbox redesign or unrelated Provider protocol change.
- No persistent cross-Session queue or implicit movement across `/new`, `/resume`, `/fork`, model or
  Provider changes.
- No general persistent-task/checkpoint runtime and no automatic retry of a started terminal Turn.
- No delimiter-joined authoritative representation.
- No modification, rebase, merge or continued development of recovery PR #120 or its branch.

## Required Architecture Contract

1. Ack only after durable idempotent pending-journal acceptance.
2. Engine escrow is non-executable after send; Actor alone executes after durable acceptance.
3. Lost Ack enters exact reconciliation, never blind rollback/resend.
4. Composition-root Session generation is carried end to end.
5. Scheduler full retains one exact bounded blocked fire; no drop, overwrite or false success.
6. Interrupt targets only the active Turn; unstarted pending survives Cancel/Error.
7. Successful transcript commit precedes pending-journal finalization.
8. One exact request plan is both budgeted and sent for every Provider call.

Historical memory-only dedupe, generic `Duplicate`, `SubmissionStarted` as Ack, `try_send` drop,
attachment clearing, delimiter batch APIs and separately rebuilt estimates are rejected.

## Acceptance

- A/B/C before cutoff produce exactly one later Turn and remain three FIFO user items; cutoff-after
  input remains for a later batch; empty/single behavior remains correct.
- Multiline, attachment, preview, slash/local, scheduler and incompatible-kind boundaries are
  preserved and classified before queueing.
- Full/closed/timeout/replaced sender before send rolls back exactly without clearing the visible
  queue. After send, lost Ack cannot cause duplicate execution.
- Every Ack/Turn event must match Session, generation, batch/receipt or Turn and exact sequence;
  stale, duplicate, regressive, gap and wrong-identity events cannot mutate state.
- Provider `TurnEnd` progress does not drain; only matching Success auto-advances.
- Ordinary Submit never preempts. Interrupt never deletes pending work. Cancel/Error pauses retained
  work and explicit user input resumes deterministic retained-user-before-scheduler ordering.
- Scheduler cannot bypass Actor arbitration, preempt/resume improperly or silently lose delivery.
- Durable acceptance survives receipt loss, Actor reconstruction and Session resume.
- Prepared/unaccepted/pending work creates no successful transcript; transcript-before-journal
  finalization prevents ghost, duplicate and crash-window re-execution.
- Item/queue/batch/attachment/journal/Actor/scheduler/initial/continuation/output-reserve bounds fail
  visibly or split only at item boundaries.
- Initial and continuation calls validate and send the same exact request plan/fingerprint.
- Legacy single-item drain and legacy Session operations remain compatible.

## Required Validation

- Focused Engine transaction, pending-journal/receipt, lifecycle, channel, scheduler, structured
  input, terminal, bounds, exact-request and persistence/replay tests.
- Crash-window and fixed-seed invariant tests proving no loss, no duplicate execution, FIFO/source
  order, one execution authority, recoverable custody, at most one active Turn and no cross-Session
  mutation.
- `cargo fmt --all -- --check`, locked workspace check/Clippy/tests and `git diff --check`.
- Project-governance and collaboration validators, release preflight, exact-head Windows and
  Unix/macOS CI.
- Rebuilt real-TUI acceptance and Provider mock/request-preview evidence.

## State Owners

- Story scope/acceptance: this file.
- Execution/evidence: `docs/iterations/I169-batched-steering-turn.md`.
- Decision: `docs/decisions/056-transactional-steering-submission-boundary.md`.
- Remote synchronization: Issue #119 and Draft PR #131.
- Historical evidence only: Draft PR #120.
- Derived view: `docs/BOARD.md`.

## Residual Destination

Queue editing, persistent cross-Session work, retry of an already-started terminal Turn,
multi-controller arbitration, Provider-specific same-role adaptation, general persistent tasks and
process portability require separate owners and decisions.
