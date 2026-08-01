# Iteration I169: Transactional Batched Steering Turn

> Document status: Planned — preactivation architecture hardening under review; implementation not started
> Published plan date: 2026-08-01
> Preactivation hardening date: 2026-08-02
> Planned objective: implement Issue #119 from current main using TUI-044 and ADR-056 while preserving structured item boundaries, transactional ownership, recoverable pending custody and durable replay.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos TUI proves A/B/C accepted during one active turn start one later model turn as three ordered user items, with durable receipt-based ownership transfer, rollback/reconciliation safety and replay parity.

The effective Collaboration Claim is owned by
`docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`. The I170 prerequisite completed
in PR #126. I169 remains inactive until the hardened Proposed architecture is reviewed and an
explicit activation creates a new implementation branch from the then-current `main`.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-044` | None | Ready | TUI-026/I145; ADR-005/006/039/049/056; completed I170 | One transactional, bounded, structured follow-up Turn after matching authoritative Success. |

### Recovery Provenance

- Recovered Issue: #119.
- Archival implementation PR: #120.
- Historical exact head: `c984b71022a16169f26dec9f2e4a73b78a41a93d`.
- Historical branch: `recovery/pr-68-i169-20260731`; immutable and never used for continued
  development.
- Recovery implementation is design/test evidence only. It is not current-main CI evidence and is
  not assumed compatible with the current tool contribution, registry, permission or Session
  architecture.
- Original audit baseline: `main@c28fe6a6c70b0115e99372927a29ab4107b06b78`.
- Satisfied prerequisite baseline: I170 completion at
  `main@592254d73a98166df48da0139a02df67e9cd2cd6`.
- Preactivation hardening baseline:
  `main@61cbb930bf9e91ddad1bc85fb79f7b13ecad317d`.
- Activation must re-read and branch from the actual current `main`; evidence SHAs are not a frozen
  future implementation base.

### Scope

- Structured Engine queue with stable item identity, Session identity/generation, kind/source,
  exact text, attachments and FIFO sequence.
- Deterministic Engine-accepted queue-sequence cutoff when matching authoritative completion is
  processed.
- Transactional `prepare/reserve/send/durable-accept/reconcile/commit` ownership transfer.
- Engine escrow after send that is recoverable and visible but cannot execute or re-batch.
- Session-scoped versioned pending journal, separate from successful transcript, written before
  Actor acknowledgement.
- Idempotent `SubmissionAccepted`/`AlreadyAccepted` receipt and authoritative `NotAccepted`
  reconciliation for lost Ack.
- Session/generation/batch/receipt/Turn/sequence lifecycle validation with bounded content-free
  rejection diagnostics.
- Actor-owned pending arbitration, one active Turn, no normal Submit preemption and source-aware
  scheduler ordering.
- Success auto-advance; Cancel/Error pause unstarted pending work; explicit user resume; no automatic
  replay of an already-started failed/cancelled Turn.
- Scheduler delivery retention on full/timeout/closed sender; no silent drop or false delivery.
- Exact initial and continuation Provider Request Plan budgeting with output reserve, using the same
  sealed plan for validation and send.
- Successful transcript-before-inbox-finalization ordering and crash recovery without ghost or
  duplicate execution.
- Live/persisted/resumed user-item boundary equivalence.
- Additive public protocol/API changes and preserved
  `ConversationEngine::drain_steering_queue` compatibility.
- Current-main governance, collaboration claim, release preflight, CI and real-TUI evidence.

### Non-Goals

- No completed I170 Windows shell/process/path/fixture work in the I169 implementation scope.
- No delimiter-only authoritative batch, concurrent model Turns, persistent cross-Session steering
  queue, arbitrary queue editing, semantic rewriting/deduplication, global bus, permission/sandbox
  redesign or unrelated Provider changes.
- No general persistent task/checkpoint runtime; the pending journal is Session delivery custody.
- No automatic retry of a started Turn after side effects may have occurred.
- No historical governance overwrite and no direct merge/rebase/modification of PR #120 or its
  recovery branch.

### Acceptance

#### Structured input and cutoff

- A/B/C remain three FIFO user items in Actor input, Provider request adaptation, persistence and
  resume history while producing only one follow-up model Turn.
- Input accepted after the Engine cutoff forms the next batch.
- Empty and single-item queues preserve expected behavior.
- Multiline, attachment, preview, slash and mixed-kind classification preserves boundaries and
  rejects incompatible batching.

#### Ownership and transaction

- Before send, Engine is the sole owner and pre-send failures roll back exactly.
- After send, Engine holds only non-executable escrow while the addressed Actor may durably acquire
  execution authority.
- Actor acknowledgement occurs only after idempotent pending-journal acceptance.
- Engine queue removal requires a matching receipt for the exact Session, generation, batch and
  reservation.
- Lost Ack reconciles through `AlreadyAccepted` or authoritative `NotAccepted`; it never blindly
  retries into a new Session or generation.
- At every state there is one execution authority and at least one recoverable copy.

#### Lifecycle and arbitration

- Stale/duplicate/wrong-session/wrong-turn/wrong-generation/regressive/gap/uncorrelated events do
  not mutate queue or Turn state.
- Provider tool-use end is non-terminal for queue advancement.
- Ordinary Submit never preempts an active Turn.
- Interrupt targets only the matching active Turn and never removes pending work.
- Cancel/Error pauses unstarted Actor pending and Engine queued work; an explicit user Submit
  resumes deterministic user-before-scheduler arbitration.
- Scheduler cannot bypass Actor arbitration, resume a paused Actor or silently lose an undelivered
  fire.

#### Persistence and replay

- Pending acceptance survives lost receipt, Actor reconstruction and Session resume.
- Unaccepted, prepared or pending work creates no successful transcript message.
- Successful transcript is atomically/idempotently committed before pending journal finalization.
- A crash after transcript commit but before inbox cleanup finalizes by Turn identity without
  re-execution.
- Resume contains no delimiter-only replay, ghost entry, duplicate item or missing attachment/item
  boundary.

#### Bounds, request and compatibility

- Item, queue, batch, attachment, journal, Actor pending, scheduler delivery, initial request,
  continuation request and output-reserve limits are visible and item-boundary safe.
- Initial and continuation calls validate and send the same exact request plan/fingerprint.
- Existing public single-item drain API and legacy Session operations remain available with their
  compatible behavior.

### Planned Validation

#### Engine

- single FIFO drain, multi-item batch, empty queue, cutoff-after input, reservation visibility,
  Ack-only exact clear, rollback and duplicate identity protection

#### Receipt / Journal

- new accept, duplicate identical accept, conflicting payload, lost receipt, `AlreadyAccepted`,
  authoritative `NotAccepted`, journal reopen, unknown schema, bounded journal and Actor restart
- crash windows before journal commit, after journal commit/before receipt, after transcript
  commit/before inbox finalization

#### Lifecycle

- matching Success, Cancelled, Error, Provider tool-use non-terminal, stale, duplicate, wrong
  Session/Turn/generation, regression, gap, no-active-turn and shutdown

#### Channel / Transaction

- full, closed, reserve timeout, sender replacement before send, sender/session replacement after
  send, Ack loss, Actor rejection and retry/reconciliation without duplication

#### Structured Input / Scheduler

- A/B/C distinct, multiline, attachments, slash, preview, scheduler, mixed incompatible kinds and
  item identity
- scheduler blocked-delivery retention, explicit cancellation and no false delivered state
- paused retained user work, resuming user input and scheduler ordering

#### Request / Bounds / Persistence

- every Issue #119 and ADR-056 bound
- exact initial and continuation plan fingerprint parity
- success persistence, Cancel/Error retention, active-batch no-auto-replay, resume parity and no
  ghost/duplicate/delimiter-only replay

#### Fixed-seed stress

Interleave enqueue, completion, cancel, reserve failure, send, receipt loss, reconcile, scheduler,
Actor replacement, Session replacement and cutoff boundaries. Check after each step:

- no loss;
- no duplicate execution;
- FIFO/source ordering;
- unique execution authority;
- at least one recoverable copy for non-terminal work;
- at most one active Turn;
- no cross-Session or cross-generation mutation.

#### Full Gates

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `git diff --check`
- governance and collaboration validators
- release preflight
- Windows and Unix/macOS exact-head CI
- rebuilt real TUI walkthrough
- `--mock` or equivalent no-Provider smoke
- Provider Request Preview / Mock Request checks for initial and continuation paths

### Documentation To Update During Implementation

- TUI-044 owner
- ADR-056
- TUI-026/ADR-049 only where the accepted boundary is extended
- iteration index, Product Backlog, Board and governance manifest
- README/user documentation only after behavior exists
- Issue #119 synchronization comments and implementation PR links

### Risks And Rollback

- Risk: both Engine and Actor believe they may execute one item. Engine escrow is explicitly
  non-executable; only a durable Actor receipt grants execution authority.
- Risk: a lost Ack is treated as a failed send and replays the batch. Require receipt reconciliation
  and `AlreadyAccepted`.
- Risk: Actor memory loss drops accepted pending work. Ack only after the session pending journal
  commits.
- Risk: stale lifecycle events advance a new Session. Bind every operation/event to Session,
  generation and exact batch/receipt/Turn identity.
- Risk: scheduler full handling drops work. Retain one exact blocked fire with visible status.
- Risk: Cancel deletes pending or auto-replays a side-effecting active Turn. Interrupt only the active
  Turn; retain unstarted pending; never auto-replay the terminal active batch.
- Risk: transcript and pending inbox diverge. Commit transcript first, then finalize journal, with
  Turn-ID crash recovery.
- Risk: context accounting omits tools, overlays or attachments. Budget and send one exact sealed
  request plan on every Provider call.
- Risk: public protocol additions break exhaustive downstream matches. Keep legacy variants and
  document the pre-1.0 additive migration.
- Rollback: revert the current-main structured implementation while retaining released
  journal/protocol readers and idempotent cleanup; never rewrite historical recovery objects.

## Preactivation Architecture Decision Record

The 2026-08-02 hardening resolves the eight implementation blockers identified by the current-main
audit:

| Blocker | Selected Proposed Contract |
|---|---|
| Ack meaning | Durable idempotent pending-journal acceptance, before Turn start |
| Actor pending recovery | Versioned session-scoped pending journal separate from transcript |
| Lost Ack | Receipt reconciliation with `AlreadyAccepted` / authoritative `NotAccepted` |
| Generation | Monotonic Session generation assigned by the composition root and carried everywhere |
| Scheduler full | Retained bounded blocked delivery; no drop/false success |
| Cancel pending | Interrupt active Turn only; retain and pause unstarted pending |
| Persistence ordering | Successful transcript commit before pending-journal finalization |
| Context budget | One exact Provider Request Plan is both validated and sent for every call |

These are Proposed design constraints, not implementation evidence and not ADR acceptance.

## Implementation Slices After Activation

A future fresh implementation PR should use independently reviewable commits:

1. **Identity and protocol** — typed IDs, generation envelope, structured items and legacy
   compatibility.
2. **Engine reservation** — cutoff, prepare, escrow, commit/rollback without Actor execution.
3. **Pending journal and receipt** — durable idempotent accept, `AlreadyAccepted`, conflict and
   reconciliation. This is the highest-risk slice.
4. **Actor arbitration** — one active Turn, scheduler delivery, Cancel/Error pause and explicit
   resume.
5. **Bridge state machine** — Idle/Prepared/AwaitingReceipt/Reconciling/Running/Cancelling/Paused,
   lifecycle validation and mutation gates.
6. **Exact request plan** — structured Provider messages, initial/continuation budget and preview.
7. **Transcript/replay** — transcript-before-inbox finalization, crash fixtures and migration.
8. **Acceptance closeout** — stress, full gates, exact-head CI, real TUI, docs and governance.

No product-code slice may be added to the preactivation documentation branch.

## Activation Gate

I170 no longer blocks activation, but I169 remains Planned. Before implementation begins:

1. merge/review the preactivation ADR-056/TUI-044/I169 hardening or explicitly supersede it;
2. re-read current `main`, Issue #119, open PRs, branches and owner docs;
3. confirm no newer or overlapping steering implementation authority exists;
4. receive explicit maintainer instruction to **formally activate and implement I169**;
5. create a new implementation branch from the exact then-current `main` — not this documentation
   branch and not the recovery branch;
6. create a separate Draft implementation PR and record its number;
7. update I169/TUI-044/Board from Planned/Ready to Active before product-code mutation;
8. record activation time, Responsible Actor, Executing Agent, baseline SHA, implementation branch
   and Draft PR;
9. keep recovery PR #120 and `recovery/pr-68-i169-20260731` immutable;
10. preserve ADR-056 as Proposed until the fresh implementation and independent review establish
    acceptance evidence.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-01 | Recovery audit | Current main retained only single-item `Vec<String>` steering drain. Historical structured implementation remained missing and required redesign on current Session and composition boundaries. |
| 2026-08-01 | Governance correction | Historical TUI-041 conflicts with current Issue #69 ownership. TUI-044 was established as the recovered Issue #119 Story. |
| 2026-08-01 | Prerequisite satisfied | I170 completed through merged PR #126 at `592254d73a98166df48da0139a02df67e9cd2cd6`. The Windows/current-main prerequisite is cleared; no I169 code branch or implementation PR was created by the I170 closeout. |
| 2026-08-02 | Preactivation audit | Re-read `main@61cbb930bf9e91ddad1bc85fb79f7b13ecad317d`, Issue #119, PR #120, owner documents and current code. No overlapping implementation was found. Eight ownership/recovery blockers were identified. |
| 2026-08-02 | Architecture hardening | ADR-056/TUI-044/I169 were expanded with durable receipt, pending-journal, lost-Ack reconciliation, generation, scheduler-full, Cancel/Error, persistence-order and exact-request-plan contracts. State remains Planned/Ready; no product code was modified. |

## Verification Evidence

- Governance claim PR #123 is merged and the TUI-044 owner chain is effective on `main`.
- I170 prerequisite evidence: PR #126, exact implementation Head
  `8cfe8edb2dbda581244f583fb809591391a54298`, CI run `30705366763`, walkthrough artifact
  `8820174164`, merge commit `592254d73a98166df48da0139a02df67e9cd2cd6`.
- Preactivation hardening branch starts exactly at
  `61cbb930bf9e91ddad1bc85fb79f7b13ecad317d` and changes governance/design documents only.
- I169 implementation and behavior evidence remain pending explicit activation.

## Completion Evidence

- Completion Commit: pending.

## Variance And Residuals

- The historical delimiter-only MVP is explicitly obsolete; TUI-044/ADR-056 structured invariants
  are authoritative for this recovery.
- The historical PR #120 in-memory pending/dedupe model is evidence only and does not satisfy the
  hardened durable receipt/reconciliation contract.
- I169 remains independent from completed I170 and must start from a fresh current-main branch,
  never from the recovery branch, PR #120 or the preactivation documentation branch.
- Satisfying the prerequisite and hardening the Proposed design do not authorize implementation
  automatically.

## Retrospective

- Pending implementation.
