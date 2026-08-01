# TUI-044: Transactional Batched Steering Turn

| Field | Value |
|---|---|
| Story ID | TUI-044 |
| Type | TUI / Runtime State Story |
| Priority | P1 |
| Status | Ready — preactivation architecture hardening under review; implementation not started |
| Source | [GitHub Issue #119](https://github.com/wjhuang88/talos/issues/119) |
| Selected Iteration | I169 |
| Depends On | TUI-026/I145; ADR-005; ADR-006; ADR-039; ADR-049; ADR-056; completed I170 Windows validation baseline |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / talos recovery session 2026-08-01 |
| Work Slice | TUI-044/I169 only: transactional structured steering queue transfer, lifecycle correlation, Actor arbitration, complete request budgets and durable replay parity after the merged I170 baseline. |
| Claimed At | 2026-08-01 |
| Source Issue | #119 |
| Governance Claim PR | #123 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #123 established the owner chain. I170 then completed in PR #126 at `592254d73a98166df48da0139a02df67e9cd2cd6`, satisfying the published prerequisite. A separate explicit activation is still required before implementation begins. |
| Implementation PR | Not started |
| Last Updated | 2026-08-02 — preactivation ADR-056 ownership/recovery hardening only; no product code |
| Handoff / Release Condition | Release only by explicit maintainer handoff or after a separate fresh I169 implementation PR is merged and completion evidence is recorded. |

The claim is effective on `main`, but it is not implementation authorization by itself. Activation
must create a fresh branch from the then-current `main`, re-read Issue #119 and current governance,
and keep archival PR #120 untouched.

This Story replaces only the conflicting historical identifier. Current `TUI-041` remains owned by
Issue #69 and must not be overwritten.

## Identity / Goal / Value

A Talos user who submits several steering, correction, attachment-bearing, or additional-context
items while one model turn is active needs those compatible items to become one bounded follow-up
turn without losing their individual identity, boundaries, ordering, persistence semantics or
retryability.

## Recovery Provenance

- Recovered Issue: #119, reconstructed from deleted Issue #50.
- Archival recovery PR: #120; never merge as-is.
- Historical exact head: `c984b71022a16169f26dec9f2e4a73b78a41a93d`.
- Historical branch: `recovery/pr-68-i169-20260731`; immutable and not a development branch.
- Original fresh-audit baseline: `main@c28fe6a6c70b0115e99372927a29ab4107b06b78`.
- I170 prerequisite completion baseline: `main@592254d73a98166df48da0139a02df67e9cd2cd6`.
- Preactivation architecture hardening baseline: `main@61cbb930bf9e91ddad1bc85fb79f7b13ecad317d`.
- A future activation must refresh again from the actual current `main`; none of these evidence SHAs
  freezes the future implementation Head.
- Historical `TUI-041` is obsolete for this scope because current main assigns TUI-041 to Issue #69.
  TUI-044 is the current authoritative Story ID.

## Scope

- Preserve `ConversationEngine` ownership of structured queued steering items until transfer begins,
  then retain only a non-executable escrow copy until a durable Actor receipt is reconciled.
- Preserve stable item ID, Session identity/generation, source/kind, exact text, multiline boundaries,
  attachments, FIFO sequence and item-bound limits.
- Classify ordinary user input, slash/local commands, preview requests, scheduler work and attachment
  input before queue admission.
- Use a deterministic Engine-accepted queue-sequence cutoff; never claim unobservable
  keypress/source-time ordering.
- Transfer one compatible bounded prefix through `prepare -> reserve -> send -> durable Actor
  acceptance -> receipt reconciliation -> Engine commit`.
- Store accepted-but-uncompleted Actor work in a versioned session-scoped pending journal separate
  from successful transcript history.
- Define acknowledgement as durable, idempotent `SubmissionAccepted`, not channel receive or
  `TurnStarted`.
- Reconcile lost acknowledgements through `AlreadyAccepted`/authoritative `NotAccepted`; never
  immediately roll back a successfully sent submission.
- Validate authoritative lifecycle identity using Session, Session generation, batch/receipt, Turn
  and monotonic sequence.
- Reject stale, duplicate, wrong-session, wrong-turn, wrong-generation, regressive, gap and
  uncorrelated events without queue mutation.
- Make the Session Actor the sole execution authority after journal acceptance and preserve one
  active model Turn with no ordinary Submit preemption.
- Use source-aware user/scheduler arbitration; only Success auto-advances, while Cancelled/Error
  pause unstarted pending work until explicit user resumption.
- Preserve live Actor input, successful durable persistence and resumed history as the same ordered
  user-item boundaries.
- Build one exact Provider Request Plan per initial/continuation call, budget that plan and send the
  same sealed plan, including prompts, memory, workspace context, history, tools, structured items,
  multimodal attachments, continuation overlay and output reserve.
- Preserve public `ConversationEngine::drain_steering_queue` single-item FIFO compatibility while
  adding non-breaking batch APIs.

## Non-Goals

- No Windows shell, path, fixture or process portability work owned by completed I170.
- No concurrent model Turns, persistent cross-Session steering queue, arbitrary queue
  editing/reordering UI, semantic rewriting, summarization, deduplication, global event bus,
  permission redesign, sandbox redesign or unrelated Provider protocol change.
- The session pending journal is not a general persistent-task/checkpoint runtime.
- No automatic replay of a started Cancelled/Error Turn that may have performed side effects.
- No delimiter-joined string as authoritative Actor, Provider-history or durable representation.
- No implicit queue movement across `/new`, `/resume`, `/fork`, model or Provider changes.
- No modification, rebase, merge or rewrite of recovery PR #120 or its branch.

## Decision Links And Constraints

- ADR-039 remains the authoritative ordered Session event boundary.
- ADR-049 remains authoritative for Engine-owned queue projection before transfer acknowledgement.
- ADR-056 defines the Proposed transactional transfer, durable receipt, lost-Ack reconciliation,
  Actor arbitration, lifecycle, replay and exact-request budget boundary.
- ADR-042 remains authoritative for successful Turn transcript durability; the new pending journal is
  separate from transcript state and may not create ghost messages.
- Public API changes remain additive and migration-aware under `AGENTS.md`.
- Completed I170 remains a prerequisite baseline and must not be mixed back into I169 scope.

## Preactivation Architecture Baseline

The following choices are required before TUI-044 may become Active:

1. **Ack definition** — acknowledgement occurs only after the addressed Session generation has
   atomically and idempotently recorded the complete submission in its pending journal.
2. **Unique execution authority** — after send, Engine data is frozen escrow only; after durable
   acceptance, Actor alone may execute. The Engine removes escrow only after a matching receipt.
3. **Lost Ack** — timeout enters reconciliation. Matching `AlreadyAccepted` commits escrow;
   authoritative `NotAccepted` permits rollback; ambiguity pauses without cross-Session retry.
4. **Generation** — the Session composition root assigns a monotonic generation carried by every
   structured operation, receipt and canonical Turn event.
5. **Scheduler full** — an undelivered fire remains bounded and visibly blocked under the same
   identity; it is not dropped, replaced or reported delivered.
6. **Cancel/Error** — Interrupt targets only the active Turn. Unstarted pending work is retained and
   paused; the started failed/cancelled batch is not automatically replayed.
7. **Persistence order** — successful transcript commits before pending-journal finalization. Crash
   recovery uses Turn identity to finalize without re-execution.
8. **Budget authority** — an exact sealed request plan is both validated and sent on every initial
   and continuation request.

A historical in-memory dedupe queue, generic `Duplicate` response, `try_send` drop, attachment clear,
or independently rebuilt budget estimate does not satisfy this baseline.

## Acceptance For Behavior / Technical Work

### Engine and transfer

- A/B/C accepted before the completion cutoff start one follow-up Turn and remain three independent
  FIFO user messages.
- Inputs accepted after the cutoff remain queued for a later batch.
- Empty and single-item queues preserve expected behavior.
- Prepared/in-flight items remain visible and immutable until a matching durable receipt commits
  their exact reservation.
- Pre-send full/closed/timeout/replaced-sender failures roll back without mutation.
- Post-send lost acknowledgement enters reconciliation and cannot produce duplicate execution.
- Matching `AlreadyAccepted` clears only the corresponding escrow; conflicting payloads fail closed.

### Lifecycle and Actor arbitration

- Provider tool-use terminal events do not drain the queue.
- Only matching authoritative Success prepares automatic advancement.
- Every receipt and Turn event matches Session, generation, batch/receipt or Turn, state and exact
  sequence.
- Cancelled/Error retain all unstarted queued and Actor-pending work, pause automatic advancement,
  and allow explicit user resumption without duplicates.
- Interrupt with no active Turn does not delete pending work.
- Ordinary Submit never cancels an active Turn.
- Scheduler and user work preserve one active Turn and deterministic source-aware ordering.
- Scheduler work cannot resume a paused Actor by itself.

### Persistence and replay

- Actor acceptance survives receipt loss and Actor reconstruction through the session pending
  journal.
- Unaccepted/prepared/pending work creates no successful transcript entry.
- Success commits transcript before journal deletion/finalization.
- Resume recovers pending work and committed transcript without ghost, duplicate, joined A/B/C or
  missing attachment/item boundaries.
- A crash between transcript commit and journal finalization does not re-execute the Turn.

### Classification, bounds and compatibility

- Slash/local commands and incompatible kinds never enter or silently combine with user-turn
  batches.
- Multiline text and attachment metadata remain item-bound.
- Queue, item, batch, attachment, pending-journal, Actor and complete request budgets reject or split
  only at item boundaries.
- Initial and every continuation call budget and send the same exact request plan.
- Legacy single-item drain and legacy Session operations remain source-compatible.

## Minimum Validation

- focused Engine, lifecycle, transaction, structured-input, journal, bound, persistence and
  fixed-seed stress tests from Issue #119 and ADR-056
- crash-window fixtures for journal accept, lost receipt, transcript commit and inbox finalization
- exact request-plan fingerprint tests for initial and continuation calls
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `git diff --check`
- project-governance and collaboration-claim validators
- release preflight
- exact-head Windows and Unix/macOS CI
- rebuilt real TUI acceptance
- Provider mock/request-preview evidence for initial and continuation budgets

## Activation Gate

TUI-044/I169 remains Ready, not Active.

Before product implementation:

1. merge/review the preactivation ADR-056 hardening or explicitly supersede it;
2. re-read current `main`, open Issues, PRs, branches and owner docs;
3. confirm no overlapping active implementation or newer steering owner exists;
4. receive explicit maintainer instruction to activate and implement I169;
5. create a fresh I169 implementation branch from the exact then-current `main`;
6. record activation time, Responsible Actor, Executing Agent, baseline SHA, branch and Draft PR in
   I169/TUI-044/Board before product-code changes;
7. keep ADR-056 Proposed until fresh implementation review is complete;
8. keep recovery PR #120 and `recovery/pr-68-i169-20260731` immutable.

This preactivation documentation branch is not the future implementation branch and must not be
continued for product code.

## State / Status Owners

- Story scope and acceptance: this file.
- Execution and evidence: `docs/iterations/I169-batched-steering-turn.md`.
- Architecture decision: `docs/decisions/056-transactional-steering-submission-boundary.md`.
- Remote discussion and recovered requirements: Issue #119.
- Historical implementation evidence only: Draft PR #120.
- Current operating view: `docs/BOARD.md`.

## Residual Destination

Any queue editing UX, persistent cross-Session queue, explicit retry of an already-started failed
Turn, multi-controller arbitration, Provider-specific same-role adaptation beyond safe
compatibility, general persistent task runtime or process portability expansion requires a separate
owner and decision.
