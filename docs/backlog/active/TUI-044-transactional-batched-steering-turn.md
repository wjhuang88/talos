# TUI-044: Transactional Batched Steering Turn

| Field | Value |
|---|---|
| Story ID | TUI-044 |
| Type | TUI / Runtime State Story |
| Priority | P1 |
| Status | Active — formally activated 2026-08-02 from `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae` |
| Source | [GitHub Issue #119](https://github.com/wjhuang88/talos/issues/119) |
| Selected Iteration | I169 |
| Depends On | TUI-026/I145; ADR-005; ADR-006; ADR-039; ADR-042; ADR-049; Proposed ADR-056; completed I170 baseline |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed — Active |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / I169 implementation session 2026-08-02 |
| Work Slice | TUI-044/I169 only: structured queue admission, transactional Engine-to-Actor transfer, durable pending custody and receipts, lifecycle correlation, Actor arbitration, complete request planning and replay parity. |
| Claimed At | 2026-08-01 |
| Activated At | 2026-08-02 02:32 +08:00 |
| Source Issue | #119 |
| Governance Claim PR | #123 |
| Preactivation Architecture PR | #129 |
| Authorization Mode | Explicit single-maintainer instruction |
| Authorization Evidence | Maintainer instructed “正式激活并实施 I169” on 2026-08-02. PR #129 then passed exact-head macOS/Windows, governance, collaboration, mock-smoke and remote reconciliation gates and merged at `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`. |
| Implementation Baseline | `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae` |
| Implementation Branch | `feat/i169-tui-044-transactional-steering` |
| Implementation PR | Pending creation from the activation commit; must be backfilled before product-code commits |
| Last Updated | 2026-08-02 |
| Handoff / Release Condition | No release claim until the separate implementation PR is merged, ADR-056 is reviewed, exact-head validation and rebuilt real-TUI acceptance pass, and completion evidence is recorded. |

The effective claim is limited to this Story and Iteration. Recovery PR #120 and
`recovery/pr-68-i169-20260731` remain immutable historical evidence and are not implementation
parents. Current `TUI-041` remains owned by Issue #69.

## Identity / Goal / Value

A user who submits steering, correction, attachment-bearing or additional-context items while one
model Turn is active needs compatible items accepted before one deterministic cutoff to become one
bounded later Turn without losing item identity, exact boundaries, FIFO order, attachment binding,
persistence semantics or retryability.

## Recovery And Activation Provenance

- Recovered Issue: #119, reconstructed from deleted Issue #50.
- Archival recovery PR: #120; never merge, rebase or continue as implementation.
- Historical exact head: `c984b71022a16169f26dec9f2e4a73b78a41a93d`.
- Historical branch: `recovery/pr-68-i169-20260731`; immutable.
- I170 prerequisite completed through PR #126.
- Proposed architecture hardening completed through PR #129.
- A Windows loopback test-fixture defect discovered by PR #129 was repaired independently in PR #130;
  it is not part of I169 product scope.
- Formal implementation baseline: `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`.
- Fresh implementation branch: `feat/i169-tui-044-transactional-steering`.

## Scope

- Replace the authoritative pre-Actor `Vec<String>` steering queue with structured items carrying
  stable item ID, target Session identity/generation, source/kind, exact text, attachments, FIFO
  sequence and bounded metadata.
- Classify ordinary user input, local/slash commands, preview requests, scheduler work and
  attachment-bearing input before queue admission.
- Use the Engine-accepted enqueue-sequence snapshot when the matching authoritative Success is
  processed as the only batch cutoff.
- Transfer one compatible bounded prefix through `prepare -> reserve -> send -> durable accept ->
  receipt reconciliation -> Engine commit`.
- Retain an immutable, non-executable Engine escrow copy after send until the exact durable receipt
  is reconciled.
- Persist Actor-accepted but uncompleted work in a versioned, session-scoped pending journal separate
  from successful transcript history.
- Define acknowledgement as durable, idempotent pending-journal acceptance, never channel receive or
  `TurnStarted`.
- Reconcile lost acknowledgements through exact `AlreadyAccepted` or authoritative `NotAccepted`;
  never blindly resend into another Session or generation.
- Carry and validate Session, Session generation, batch/reservation/receipt, Turn and exact monotonic
  sequence identities across the canonical flow.
- Make the Session Actor the sole execution authority after durable acceptance, maintain at most one
  active model Turn and remove ordinary Submit preemption.
- Route user and scheduler work through one source-aware Actor arbitration boundary.
- Allow only matching Success to auto-advance; Cancel/Error retain and pause unstarted pending work,
  while an already-started terminal Turn is not automatically replayed.
- Preserve A/B/C as distinct ordered `Message::User` or `Message::Multimodal` values in live Actor
  input, Provider request adaptation, successful transcript and resumed history.
- Build one exact sealed Provider Request Plan for each initial and continuation call; budget and send
  that same plan, including prompts, memory, workspace context, history, tools/schemas, structured
  input, multimodal cost, continuation overlay and output reserve.
- Preserve source compatibility for public `ConversationEngine::drain_steering_queue` and legacy
  Session operations while adding migration-aware structured variants.

## Non-Goals

- No Windows shell/process/path/fixture work owned by completed I170.
- No concurrent model Turns, global bus, arbitrary queue editing/reordering UI, semantic rewriting,
  summarization or deduplication.
- No persistent cross-Session steering queue and no implicit movement across `/new`, `/resume`,
  `/fork`, model or Provider changes.
- No general persistent-task/checkpoint runtime; the pending journal is Session delivery custody.
- No automatic retry of a started Cancelled/Error Turn that may have produced side effects.
- No permission, sandbox or unrelated Provider-protocol redesign.
- No delimiter-joined string as authoritative queue, Actor input, Provider history or durable record.
- No modification, rebase, merge or rewrite of recovery PR #120 or its branch.

## Decision Links And Constraints

- ADR-005 defines bounded SQ/backpressure and single-consumer EQ.
- ADR-006 forbids a new global pub/sub side channel.
- ADR-039 defines authoritative ordered Session lifecycle and Provider `TurnEnd` as progress only.
- ADR-042 owns successful atomic transcript persistence and no-ghost semantics.
- ADR-049 remains authoritative for Engine-owned queue projection before receipt reconciliation.
- ADR-056 remains Proposed during implementation and defines the transactional ownership, pending
  journal, receipt, terminal, request-plan and rollback contracts.
- Public API changes remain additive and migration-aware under `AGENTS.md`.

## Required Architecture Contract

1. **Ack** — emitted only after the addressed Session generation atomically and idempotently records
   the complete immutable submission in its pending journal.
2. **Unique execution authority** — Engine escrow is never executable after send; Actor alone may
   execute after durable acceptance. Engine removes escrow only on a matching receipt.
3. **Lost Ack** — timeout enters reconciliation. Matching `AlreadyAccepted` commits escrow;
   authoritative `NotAccepted` permits retry; ambiguity pauses without cross-Session replay.
4. **Generation** — the Session composition root assigns a monotonic generation carried by every
   structured operation, receipt and canonical structured Turn event.
5. **Scheduler full** — one undelivered fire remains bounded and visibly blocked under the same
   identity; it is not dropped, replaced or reported delivered.
6. **Cancel/Error** — Interrupt targets only the active Turn. Unstarted pending work is retained and
   paused; the started terminal batch is not automatically replayed.
7. **Persistence order** — successful transcript commit precedes pending-journal finalization;
   recovery uses Turn identity to finish cleanup without re-execution.
8. **Budget authority** — the exact sealed request plan is both validated and sent on every initial
   and continuation call.

Historical memory-only dedupe, generic `Duplicate`, `SubmissionStarted` as ownership Ack,
`try_send` drop/coalescing, attachment clearing, delimiter batch APIs and separately rebuilt budget
estimates do not satisfy this contract.

## Acceptance

### Structured input and cutoff

- A/B/C accepted before the cutoff produce exactly one later model Turn and remain three FIFO user
  items through Actor, Provider, persistence and resume.
- Inputs accepted after cutoff remain for a later batch.
- Empty and single-item queues preserve expected behavior.
- Multiline text, attachment metadata and per-item identity remain bound.
- Slash/local commands and incompatible kinds never enter or silently combine with user batches.

### Ownership and transaction

- Before send, Engine is sole owner; full/closed/reserve-timeout/replaced-sender failures roll back
  exactly without clearing UI projection.
- After send, Engine holds non-executable escrow and the addressed Actor may durably acquire execution
  authority; at every state there is one execution authority and at least one recoverable copy.
- Actor Ack occurs only after idempotent journal acceptance.
- Queue removal requires the exact Session, generation, batch, reservation and receipt.
- Lost Ack reconciliation cannot duplicate execution; conflicts fail closed and content-free.

### Lifecycle and arbitration

- Provider tool-use end never drains the queue.
- Only matching authoritative Success auto-advances.
- Wrong Session/generation/batch/Turn, duplicate/regressive/gap/no-active events cannot mutate state.
- Ordinary Submit never preempts an active Turn.
- Interrupt never removes pending work when no Turn is active.
- Cancel/Error pause retained work; explicit user input resumes deterministic retained-user-before-
  scheduler arbitration without duplication.
- Scheduler cannot bypass Actor arbitration, preempt user work, resume a paused Actor by itself or
  silently lose an undelivered fire.

### Persistence, replay and bounds

- Durable acceptance survives receipt loss, Actor reconstruction and Session resume.
- Prepared, unaccepted or pending work creates no successful transcript entry.
- Success commits transcript before journal finalization; a crash between them does not re-execute.
- Resume has no delimiter-only replay, ghost, duplicate or missing item/attachment boundary.
- Item, queue, batch, attachment, pending-journal, Actor, scheduler delivery, initial request,
  continuation request and output-reserve bounds reject visibly or split only at item boundaries.
- Initial and continuation calls validate and send the same exact request plan/fingerprint.
- Legacy single-item drain and legacy Session operations retain compatible behavior.

## Required Validation

- Engine FIFO/single/empty/batch/cutoff/reservation/escrow/Ack/rollback/conflict tests.
- Pending-journal accept/reopen/idempotency/conflict/reconcile/unknown-schema/bounds/crash-window tests.
- Matching and rejected Session/generation/batch/receipt/Turn/sequence lifecycle tests.
- Full/closed/timeout/sender replacement/session replacement/lost Ack/start rejection/shutdown tests.
- A/B/C, multiline, attachments, slash, preview, scheduler and incompatible-kind tests.
- Cancel/Error retention, no active-batch auto-replay and explicit resume-order tests.
- Exact initial/continuation request-plan fingerprint and complete-budget tests.
- Fixed-seed interleavings proving no loss, no duplicate execution, FIFO/source ordering, unique
  execution authority, recoverable custody, at most one active Turn and no cross-Session mutation.
- `cargo fmt --all -- --check`.
- `cargo check --workspace --locked`.
- `cargo clippy --workspace --locked -- -D warnings`.
- `cargo test --workspace --locked`.
- `git diff --check`.
- project-governance and collaboration-claim validators.
- release preflight, exact-head Windows and Unix/macOS CI.
- rebuilt real-TUI acceptance and Provider mock/request-preview evidence.

## Activation Record

Formal activation is authorized and recorded on the fresh implementation branch before product-code
mutation. The next governance commit must backfill the separate Draft implementation PR number in
this file, I169 and the Board. ADR-056 remains Proposed throughout implementation review.

## State / Status Owners

- Story scope and acceptance: this file.
- Execution and evidence: `docs/iterations/I169-batched-steering-turn.md`.
- Architecture decision: `docs/decisions/056-transactional-steering-submission-boundary.md`.
- Remote requirement and synchronization: Issue #119.
- Historical evidence only: Draft PR #120.
- Derived operating view: `docs/BOARD.md`.

## Residual Destination

Queue editing UX, persistent cross-Session work, explicit retry of a started terminal Turn,
multi-controller arbitration, Provider-specific same-role adaptation beyond safe compatibility,
general persistent tasks and process portability require separate owners and decisions.
