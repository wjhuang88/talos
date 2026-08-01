# Iteration I169: Transactional Batched Steering Turn

> Document status: Active — formally activated 2026-08-02
> Published plan date: 2026-08-01
> Preactivation hardening date: 2026-08-02
> Activation baseline: `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`
> Implementation branch: `feat/i169-tui-044-transactional-steering`
> Planned objective: implement Issue #119 using TUI-044 and Proposed ADR-056 while preserving structured item boundaries, transactional ownership, recoverable pending custody, exact request planning and durable replay.
> Baseline rule: preserve this published objective, dependencies, exclusions, acceptance and validation; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos TUI proves A/B/C accepted during one active Turn start one later model Turn as three ordered user items, with durable receipt-based transfer, recovery-safe custody and replay parity.

The effective Collaboration Claim is owned by
`docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`. The maintainer explicitly
instructed “正式激活并实施 I169” on 2026-08-02. Preactivation architecture PR #129 passed exact-head
macOS/Windows, governance, collaboration, mock-smoke and remote-reconciliation gates and merged at
`a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`. This iteration was then activated on a fresh branch
from that exact commit. Recovery PR #120 and its branch remain immutable.

## Published Baseline

### Selected Story

| Story | Parent | State At Activation | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-044` | None | Active | TUI-026/I145; ADR-005/006/039/042/049; Proposed ADR-056; completed I170 | One transactional, bounded, structured follow-up Turn after matching authoritative Success. |

### Recovery And Activation Provenance

- Recovered Issue: #119.
- Archival implementation PR: #120.
- Historical exact head: `c984b71022a16169f26dec9f2e4a73b78a41a93d`.
- Historical branch: `recovery/pr-68-i169-20260731`; immutable and never an implementation parent.
- I170 prerequisite completed in PR #126.
- Preactivation architecture hardening completed in PR #129.
- Independent Windows test-fixture repair completed in PR #130 and is not I169 scope.
- Formal activation authorization: maintainer instruction on 2026-08-02 at 02:32 +08:00.
- Responsible Actor: `@wjhuang88`.
- Executing Agent: `GPT-5.6 Thinking / I169 implementation session 2026-08-02`.
- Exact implementation base: `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`.
- Fresh implementation branch: `feat/i169-tui-044-transactional-steering`.
- Draft implementation PR: pending creation from the activation commit and must be backfilled before
  product-code commits.
- ADR-056 status: Proposed during implementation and independent review.

## Scope

1. Structured Engine queue with stable item, batch, reservation and transfer-attempt identity;
   Session identity/generation; source/kind; exact text; attachments; FIFO sequence; bounded metadata.
2. Deterministic Engine-accepted enqueue-sequence cutoff when matching authoritative Success is
   processed; no wall-clock, `try_recv`, biased-select or inferred-keypress ordering claim.
3. Transactional `prepare -> reserve -> send -> durable accept -> reconcile -> commit` transfer,
   retaining non-executable Engine escrow after send until the exact receipt is reconciled.
4. Versioned session-scoped pending journal separate from successful transcript, with idempotent
   acceptance, `AlreadyAccepted`, authoritative `NotAccepted`, conflict detection and restart recovery.
5. Monotonic Session generation carried by structured operations, receipts and structured canonical
   Turn events; exact Session/generation/batch/receipt/Turn/sequence validation.
6. Actor-owned source-aware arbitration with at most one active Turn, no ordinary Submit preemption,
   scheduler delivery retention and deterministic explicit-user resume.
7. Success auto-advance; Cancel/Error pause unstarted pending work; no automatic replay of an
   already-started terminal Turn that may have produced side effects.
8. Structured A/B/C Provider and transcript representation as distinct ordered User/Multimodal
   messages, with attachment and multiline boundaries preserved.
9. Successful transcript commit before pending-journal finalization, with Turn-ID crash recovery and
   no ghost or duplicate replay.
10. One exact sealed Provider Request Plan per initial and continuation call, used by both complete
    context-budget validation and the actual Provider call.
11. Additive protocol/API migration and preservation of public single-item
    `ConversationEngine::drain_steering_queue` FIFO behavior.
12. Current-main tests, governance, exact-head platform CI, rebuilt real-TUI and Provider
    mock/request-preview evidence.

## Non-Goals

- No I170 Windows shell/process/path/fixture work.
- No concurrent model Turns, global event bus, queue editing/reordering UI or semantic rewrite.
- No persistent cross-Session steering queue or implicit movement across Session/model/Provider changes.
- No general persistent-task/checkpoint runtime.
- No automatic retry of a started Cancelled/Error Turn.
- No permission, sandbox or unrelated Provider-protocol redesign.
- No delimiter-joined string as authoritative queue, Actor, Provider or durable representation.
- No direct merge, rebase, modification or continued development of PR #120 or its branch.

## Acceptance

### Structured behavior

- A/B/C accepted before cutoff become exactly one later Turn and remain three distinct FIFO user
  messages in Actor input, Provider adaptation, successful transcript and resume.
- Cutoff-after items remain for a later batch; empty/single-item behavior remains correct.
- Multiline, attachments, preview, slash/local and mixed kinds are classified before queueing;
  incompatible kinds are never silently combined.

### Ownership and failure recovery

- Before send, Engine is sole owner; full/closed/reserve-timeout/replaced-sender failures roll back
  exactly without clearing the visible queue.
- After send, Engine escrow is immutable and non-executable; only durable Actor acceptance grants
  execution authority. At least one recoverable copy exists until terminal finalization.
- Ack is emitted only after pending-journal commit and must match Session, generation, batch,
  reservation and receipt before Engine removes the exact prefix.
- Lost Ack enters reconciliation; matching `AlreadyAccepted` commits escrow, authoritative
  `NotAccepted` permits retry, ambiguity pauses, and conflicting payload fails closed.
- Actor acceptance survives lost receipt, Actor reconstruction and Session resume.

### Lifecycle and arbitration

- Provider `TurnEnd` progress never drains queued steering.
- Only matching authoritative Success auto-advances.
- Wrong Session/generation/batch/Turn, duplicate/regressive/gap/no-active events cannot mutate state.
- Ordinary Submit never preempts the active Turn; Interrupt targets only the matching active Turn and
  never removes pending work when idle.
- Cancel/Error retain Engine queued and unstarted Actor pending work, pause automatic advancement and
  allow deterministic explicit-user resume without duplicate execution.
- Scheduler work passes through the same Actor, cannot preempt or resume a paused Actor by itself and
  is retained visibly on delivery backpressure rather than dropped or falsely marked delivered.

### Persistence, request and compatibility

- Prepared, unaccepted and pending work creates no successful transcript entries.
- Success atomically/idempotently commits transcript before journal finalization; a crash between
  those steps finalizes by Turn identity without re-execution.
- Resume has no joined A/B/C string, ghost, duplicate or missing item/attachment boundary.
- Queue/item/batch/attachment/journal/Actor/scheduler/initial/continuation/output-reserve limits fail
  visibly or split only at item boundaries.
- Every initial and continuation Provider call validates and sends the same exact request plan.
- Legacy single-item drain and legacy Session operations remain compatible.

## Implementation Slices

1. **Identity and custody types** — typed structured input/receipt states plus a versioned pending
   journal and focused unit tests.
2. **Engine reservation** — structured queue, deterministic cutoff, immutable preparation/escrow,
   exact commit/rollback and legacy drain compatibility.
3. **Protocol and Actor receipt** — additive Session operations/events, generation, durable accept,
   reconcile, no ordinary preemption and recovery of unstarted pending work.
4. **Actor arbitration and terminal policy** — one active Turn, user/scheduler ordering,
   Cancel/Error pause and scheduler delivery retention.
5. **Bridge state machine** — attachment-bound admission, lifecycle validation, bounded send/Ack,
   reconciliation, UI commit and Session mutation gates.
6. **Exact request plan** — distinct structured messages, complete initial/continuation budgets and
   Provider request-preview fingerprint evidence.
7. **Transcript/replay** — transcript-before-journal finalization, crash fixtures, resume parity and
   migration compatibility.
8. **Acceptance closeout** — fixed-seed stress, full locked gates, exact-head CI, real TUI, docs,
   independent architecture review and ADR disposition.

The pending-journal/receipt slice is the highest-risk boundary and must be independently reviewable.
No implementation slice may weaken the Proposed contract simply to match historical PR #120.

## Planned Validation

### Engine

- single FIFO drain, empty/single/multi item, cutoff-after input, compatibility split, reservation
  visibility, exact Ack-only clear, rollback and identity-conflict rejection.

### Pending Journal / Receipt

- new accept, duplicate identical accept, conflicting payload, lost receipt, `AlreadyAccepted`,
  authoritative `NotAccepted`, reopen, Actor reconstruction, unknown schema and all bounds.
- crash windows before journal commit, after journal commit/before receipt, after transcript
  commit/before journal finalization.

### Lifecycle / Channel / Actor

- matching Success/Cancelled/Error, Provider progress non-terminal, wrong/stale/duplicate/regressive/
  gap/no-active events, full/closed/timeout, sender/session replacement, Ack loss and shutdown.
- no ordinary preemption; Interrupt active-only; retained-user-before-scheduler explicit resume;
  scheduler blocked-delivery retention and cancellation.

### Structured Input / Request / Persistence

- A/B/C distinct, multiline, attachments, slash, preview, scheduler and incompatible kinds.
- every bound from Issue #119 and ADR-056.
- exact initial/continuation request-plan fingerprint parity.
- Success persistence, Cancel/Error retention, no active-batch auto-replay, resume parity and no
  ghost/duplicate/delimiter-only replay.

### Fixed-Seed Invariants

Interleave enqueue, cutoff, completion, cancel, reserve failure, send, lost receipt, reconcile,
scheduler, Actor replacement and Session replacement. Assert after every step:

- no loss;
- no duplicate execution;
- FIFO/source ordering;
- exactly one execution authority;
- at least one recoverable copy for non-terminal work;
- at most one active Turn;
- no cross-Session or cross-generation mutation.

### Full Gates

- `cargo fmt --all -- --check`.
- `cargo check --workspace --locked`.
- `cargo clippy --workspace --locked -- -D warnings`.
- `cargo test --workspace --locked`.
- `git diff --check`.
- project-governance and collaboration-claim validators.
- release preflight.
- exact-head Windows and Unix/macOS CI.
- rebuilt real-TUI walkthrough.
- no-Provider mock smoke and Provider Request Preview / Mock Request evidence.

## Documentation Targets

- TUI-044 and this iteration owner.
- ADR-056; ADR-039/042/049 only where the accepted boundary is actually extended.
- Board, Product Backlog, iteration index and governance manifest.
- README/user documentation only after behavior exists.
- Issue #119 synchronization and the separate implementation PR.

## Risks And Rollback

- Dual execution authority: Engine escrow is non-executable; only durable receipt grants Actor
  execution authority.
- Lost Ack duplicate: reconcile exact identity and payload; never blindly resend.
- Actor memory loss: acknowledge only after session pending journal commit.
- Stale events: validate Session/generation and exact batch/receipt/Turn/sequence.
- Scheduler drop: retain one exact blocked fire with visible state.
- Terminal side effects: never auto-replay an already-started Cancelled/Error Turn.
- Transcript/journal divergence: transcript first, journal finalization second, Turn-ID recovery.
- Request drift: budget and send one sealed request plan.
- Public enum migration: retain legacy variants and document pre-1.0 exhaustive-match changes.
- Rollback before release reverts structured behavior to ADR-049 while preserving any released
  protocol/journal readers and idempotent cleanup; historical recovery objects are never rewritten.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-01 | Recovery audit | Current main retained the unsafe single-item/destructive steering path; historical PR #120 was archival evidence only. |
| 2026-08-01 | Governance correction | TUI-044 replaced the conflicting historical TUI-041 identifier for Issue #119. |
| 2026-08-01 | Prerequisite | I170 completed through PR #126. |
| 2026-08-02 | Architecture hardening | PR #129 defined durable receipt, pending journal, lost-Ack reconciliation, generation, scheduler, terminal, persistence-order and exact-request-plan contracts. |
| 2026-08-02 | Baseline repair | PR #130 independently stabilized a Windows loopback test fixture exposed by PR #129 validation. |
| 2026-08-02 | Formal activation | Maintainer explicitly instructed activation and implementation. Re-read current facts, found no overlapping implementation authority and created `feat/i169-tui-044-transactional-steering` from exact `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`. |

## Verification Evidence

- Collaboration claim PR #123 is merged.
- Architecture hardening PR #129 exact Head `d0e60d65038cd890e411a44c65783d6dc34a74c7`
  passed CI run `30713776456` before squash merge `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`.
- Independent Windows fixture PR #130 exact Head `fe87c4265bafd1be67e20e635c176eefe08ac6cc`
  passed CI run `30713367293` before merge `57d99596b3882162d0d5b06ace42fb5faed95b3e`.
- Activation branch is created from exact architecture merge commit.
- Product implementation and behavior evidence are pending the separate Draft implementation PR.

## Completion Evidence

- Completion Commit: pending.

## Variance And Residuals

- Historical delimiter-only and in-memory ownership implementations are obsolete evidence, not code
  authority.
- The implementation must remain independent of I170 and must never continue PR #120, PR #129 or
  their branches.
- Broader interrupted-turn persistence, graceful-shutdown expansion, general persistent tasks,
  queue editing and cross-Session movement remain separately owned.

## Retrospective

- Pending implementation and independent review.
