# Iteration I169: Transactional Batched Steering Turn

> Document status: Active — PR #131 review handoff
> Published plan date: 2026-08-01
> Preactivation hardening date: 2026-08-02
> Activation date: 2026-08-02
> Activation baseline: `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`
> Implementation branch: `feat/i169-tui-044-transactional-steering`
> Implementation PR: #131
> Planned objective: implement Issue #119 using TUI-044 and Proposed ADR-056 while preserving structured item boundaries, transactional ownership, recoverable pending custody, exact request planning and durable replay.
> Baseline rule: preserve this published objective, dependencies, exclusions, acceptance and validation; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos TUI proves A/B/C accepted during one active Turn start one later model Turn as three ordered user items, with durable receipt-based transfer, recovery-safe custody and replay parity.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / I169 implementation session 2026-08-02 |
| Work Slice | I169 execution only: structured identities and Engine escrow, durable session pending journal and receipt reconciliation, one-Turn Actor arbitration, bridge lifecycle correlation, exact Provider request planning, transcript/replay parity and required acceptance evidence. |
| Claimed At | 2026-08-02 |
| Source Issue | #119 |
| Governance Claim PR | #123 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The repository owner explicitly instructed “正式激活并实施 I169”; preactivation architecture PR #129 passed exact-head platform/governance gates and merged before this fresh branch and Draft implementation PR were created. |
| Implementation PR | #131 |
| Last Updated | 2026-08-04 |
| Handoff / Release Condition | Remain Active until the exact PR #131 Head passes the complete Issue #119/ADR-056 matrix and independent review moves the work to Review; no release or completion claim before merge and recorded Completion Commit. |

The effective Story claim is also recorded in
`docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`. Recovery PR #120 and its branch
remain immutable.

The maintainer explicitly instructed “正式激活并实施 I169” on 2026-08-02. Preactivation architecture
PR #129 passed exact-head macOS/Windows, governance, collaboration, mock-smoke and remote-
reconciliation gates and merged at `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`. I169 was activated
on a fresh branch from that exact commit and is implemented in PR #131 at review handoff.

## Published Baseline

### Selected Story

| Story | Parent | State At Activation | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-044` | None | Active | TUI-026/I145; ADR-005/006/039/042/049; Proposed ADR-056; completed I170 | One transactional, bounded, structured follow-up Turn after matching authoritative Success. |

### Provenance And Authorization

- Recovered Issue: #119.
- Archival PR/head/branch: #120 / `c984b71022a16169f26dec9f2e4a73b78a41a93d` /
  `recovery/pr-68-i169-20260731`; immutable and never implementation parents.
- I170 prerequisite: PR #126.
- Preactivation architecture hardening: PR #129.
- Independent Windows fixture repair: PR #130; not I169 product scope.
- Maintainer authorization: explicit instruction on 2026-08-02 at 02:32 +08:00.
- Responsible Actor: `@wjhuang88`.
- Executing Agent: `GPT-5.6 Thinking / I169 implementation session 2026-08-02`.
- Exact base/branch/PR: `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae` /
  `feat/i169-tui-044-transactional-steering` / #131.
- ADR-056 remains Proposed during implementation and independent review.

## Scope

1. Structured Engine queue with stable item, batch, reservation and attempt identity; Session
   identity/generation; source/kind; exact text; attachments; FIFO sequence; bounded metadata.
2. Deterministic Engine-accepted enqueue-sequence cutoff when matching authoritative Success is
   processed; no wall-clock, `try_recv`, biased-select or inferred-keypress ordering claim.
3. Transactional `prepare/reserve/send/durable-accept/reconcile/commit` transfer with immutable,
   non-executable Engine escrow after send until receipt reconciliation.
4. Versioned session-scoped pending journal separate from successful transcript, with idempotent
   acceptance, `AlreadyAccepted`, authoritative `NotAccepted`, conflict detection and restart
   recovery.
5. Durable Session generation on structured operations, receipts and structured Turn events; live
   replacement advances it atomically, process reconstruction rehydrates it unchanged, and exact
   Session/generation/batch/receipt/Turn/sequence validation applies end to end.
6. Actor-owned user/scheduler arbitration with at most one active Turn, no ordinary Submit
   preemption, retained delivery under backpressure and deterministic explicit-user resume.
7. Success auto-advance; Cancel/Error pause unstarted pending work; deterministic pre-start failure
   has an explicit exact-generation cancel/terminalize action; no automatic replay of an already-
   started terminal Turn.
8. A/B/C as distinct ordered User/Multimodal messages in Actor input, Provider requests, successful
   transcript and resumed history.
9. Successful transcript commit before pending-journal finalization, with Turn-ID crash recovery and
   no ghost or duplicate replay.
10. One sealed exact Provider Request Plan for every initial and continuation call, used by both
    complete budget validation and actual Provider dispatch.
11. Additive protocol/API migration preserving public single-item
    `ConversationEngine::drain_steering_queue` and legacy Session operations.
12. Focused and full tests, governance, exact-head platform CI, rebuilt real-TUI and Provider
    mock/request-preview evidence.

## Non-Goals

- No I170 process/path/fixture work, concurrent model Turns, global bus, queue editing/reordering,
  semantic rewrite, permission/sandbox redesign or unrelated Provider protocol change.
- No persistent cross-Session steering queue or implicit movement across Session/model/Provider
  changes.
- No general persistent-task/checkpoint runtime and no automatic retry of a started terminal Turn.
- No delimiter-joined authoritative representation.
- No merge, rebase, modification or continued development of PR #120 or its branch.

## Acceptance

### Structured Input And Cutoff

- A/B/C before cutoff become exactly one later Turn and remain three distinct FIFO user messages in
  Actor input, Provider adaptation, successful transcript and resume.
- Cutoff-after input remains for a later batch; empty and single-item behavior remains correct.
- Multiline, attachments, preview, slash/local, scheduler and incompatible kinds are classified
  before queueing and preserve per-item boundaries.

### Ownership And Recovery

- Before send, Engine is sole owner and full/closed/reserve-timeout/replaced-sender failures roll
  back exactly without clearing queue projection.
- After send, Engine escrow is non-executable; only journal-backed Actor acceptance grants execution
  authority. One recoverable copy exists until terminal finalization.
- Ack occurs only after journal commit and must match exact Session, generation, batch, reservation
  and receipt before Engine removes the exact prefix.
- Lost Ack reconciles through `AlreadyAccepted` or authoritative `NotAccepted`; ambiguity pauses and
  conflicting payload fails closed.
- Actor acceptance survives lost receipt, Actor reconstruction and Session resume.

### Lifecycle And Arbitration

- Provider progress/`TurnEnd` never drains. Only matching canonical Success auto-advances.
- Wrong Session/generation/batch/Turn, duplicate, regressive, gap and no-active events cannot mutate
  state.
- Ordinary Submit never preempts. Interrupt targets only the matching active Turn and does not remove
  pending work when idle.
- Cancel/Error pause Engine queued and unstarted Actor pending work. Admissible retained work resumes
  in retained-user-before-scheduler order; deterministic pre-start failure can be terminalized by an
  exact-generation command without Provider execution so it cannot pin the FIFO forever.
- Scheduler cannot bypass Actor arbitration, preempt user work, resume a paused Actor by itself or
  silently lose an undelivered fire.

### Persistence, Request And Compatibility

- Prepared, unaccepted and pending work creates no successful transcript messages.
- Success commits transcript before journal finalization; crash recovery finalizes by Turn identity
  without re-execution.
- Resume has no delimiter-only replay, ghost, duplicate or missing item/attachment boundary.
- Item/queue/batch/attachment/journal/Actor/scheduler/initial/continuation/output-reserve limits fail
  visibly or split only at item boundaries.
- Every initial and continuation call validates and sends the same exact request plan/fingerprint.
- Legacy single-item drain and legacy Session operations remain compatible.

## Implementation Slices

1. **Identity and custody types** — structured input/receipt states plus versioned pending journal.
2. **Engine reservation** — structured queue, cutoff, preparation/escrow, exact commit/rollback and
   legacy drain compatibility.
3. **Protocol and Actor receipt** — additive operations/events, generation, durable accept,
   reconciliation, no ordinary preemption and pending recovery.
4. **Actor arbitration and terminal policy** — one active Turn, user/scheduler ordering, Cancel/Error
   pause and retained scheduler delivery.
5. **Bridge state machine** — attachment-bound admission, lifecycle validation, bounded send/Ack,
   reconciliation, UI commit and Session mutation gates.
6. **Exact request plan** — structured Provider messages, complete initial/continuation budgets and
   preview fingerprint evidence.
7. **Transcript/replay** — transcript-before-journal finalization, crash fixtures and resume parity.
8. **Acceptance closeout** — fixed-seed stress, full locked gates, exact-head CI, real TUI, docs,
   independent review and ADR disposition.

The pending-journal/receipt slice is the highest-risk review boundary. No slice may weaken the
Proposed contract merely to match historical PR #120.

## Planned Validation

- Engine single/empty/multi/cutoff/reservation/escrow/Ack/rollback/conflict tests.
- Pending journal new/duplicate/conflict/reconcile/reopen/reconstruction/unknown-schema/bound tests.
- Crash windows before journal commit, after journal commit/before receipt and after transcript
  commit/before journal finalization.
- Matching and rejected Session/generation/batch/receipt/Turn/sequence lifecycle tests.
- Full/closed/timeout/sender replacement/session replacement/lost Ack/rejection/shutdown tests.
- No ordinary preemption; active-only Interrupt; Cancel/Error retention; explicit user resume and
  scheduler blocked-delivery tests.
- A/B/C, multiline, attachments, slash, preview, scheduler and incompatible-kind tests.
- Exact initial/continuation request-plan fingerprint and complete-budget tests.
- Fixed-seed interleavings proving no loss, no duplicate execution, FIFO/source order, one execution
  authority, recoverable custody, at most one active Turn and no cross-Session mutation.
- `cargo fmt --all -- --check`, locked workspace check/Clippy/tests and `git diff --check`.
- Governance/collaboration validators, release preflight, exact-head Windows and Unix/macOS CI.
- Rebuilt real-TUI walkthrough, no-Provider smoke and Provider Request Preview / Mock Request
  evidence.

## Documentation Targets

- TUI-044, this iteration and ADR-056.
- ADR-039/042/049 only where the accepted implementation actually extends their boundary.
- Board, Product Backlog, iteration index and governance manifest.
- README/user documentation only after behavior exists.
- Issue #119 and PR #131 review-handoff synchronization.

## Risks And Rollback

- Dual authority: Engine escrow is non-executable; only durable receipt grants Actor execution.
- Lost Ack duplicate: exact reconciliation; never blind resend.
- Actor memory loss: acknowledge only after session journal commit.
- Stale lifecycle: exact Session/generation/batch/receipt/Turn/sequence checks.
- Scheduler loss: retain one exact blocked fire with visible state.
- Terminal side effects: never auto-replay a started Cancelled/Error Turn.
- Transcript/journal divergence: transcript first, journal finalization second, Turn-ID recovery.
- Request drift: validate and send one sealed request plan.
- Public enum migration: retain legacy variants and document pre-1.0 exhaustive-match changes.
- Rollback before release returns behavior to ADR-049 while retaining any released journal/protocol
  readers and idempotent cleanup; recovery objects are never rewritten.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-01 | Recovery audit | Current main retained unsafe single-item/destructive steering; PR #120 remained archival evidence. |
| 2026-08-01 | Governance correction | TUI-044 replaced the conflicting historical TUI-041 identifier. |
| 2026-08-01 | Prerequisite | I170 completed through PR #126. |
| 2026-08-02 | Architecture hardening | PR #129 defined durable receipt, pending journal, lost-Ack, generation, scheduler, terminal, persistence-order and exact-request contracts. |
| 2026-08-02 | Baseline repair | PR #130 independently stabilized the Windows loopback test fixture found by PR #129 validation. |
| 2026-08-02 | Formal activation | Maintainer explicitly authorized implementation; no overlap was found; fresh branch created from exact `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`; Draft PR #131 opened. |
| 2026-08-02 | Slice 1 | Added structured submission identities/bounds and a versioned session-scoped pending journal with durable receipts, idempotent reopen, conflict rejection, pause/recovery and terminal tombstones. |
| 2026-08-02 | Review baseline port | Copied the preserved Review code for Engine/Actor/Bridge/Agent files into the fresh PR branch as a separately identifiable starting snapshot; recovery refs remain unchanged and the snapshot is not completion evidence. |
| 2026-08-04 | Review handoff | Remediation code and regression evidence passed CI #967 at `8cd00fe311fcb349488f3fea69e03f406fc2631e`; governance lifecycle is synchronized for a fresh exact-head CI and independent review. ADR-056 remains Proposed and no merge/completion is authorized. |
| 2026-08-03 | Independent-review remediation | PR #131 returned to Draft. Remediation defines explicit pre-start cancellation, ordinary-Session terminal-outcome recovery, generation-bound scheduler delivery, durable runtime-generation reconstruction, UUID Engine identities and fixed-seed interleaving evidence. Exact-head CI and repeat independent acceptance remain pending. |

## Verification Evidence

- Claim PR #123 is merged.
- Architecture PR #129 exact Head `d0e60d65038cd890e411a44c65783d6dc34a74c7` passed CI
  `30713776456` before merge `a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`.
- Fixture PR #130 exact Head `fe87c4265bafd1be67e20e635c176eefe08ac6cc` passed CI
  `30713367293` before merge `57d99596b3882162d0d5b06ace42fb5faed95b3e`.
- Activation branch and PR #131 remain bound to the exact architecture merge baseline.
- Independent-review remediation and regression evidence are implemented. The implementation tree at
  `8cd00fe311fcb349488f3fea69e03f406fc2631e` passed CI #967; this lifecycle synchronization requires
  a new exact-head CI before fresh independent review.

## Completion Evidence

- Completion Commit: pending.

## Variance And Residuals

- Historical delimiter-only and in-memory ownership implementations are obsolete evidence, not code
  authority.
- I169 remains independent of I170 and does not continue PR #120 or PR #129 branches.
- Interrupted active-Turn persistence, broader shutdown, general persistent tasks, queue editing and
  cross-Session movement remain separately owned.

## Retrospective

- Pending implementation and independent review.

## Independent-review remediation handoff (2026-08-04)

Lifecycle remains unchanged while PR #131 awaits a new independent review: **TUI-044 / I169 are Active, ADR-056 is Proposed, and Issue #119 is Open**.

The latest implementation evidence tightens same-Session generation replacement into one acknowledged ownership handoff:

- SQLite admission and generation advance share one immediate transaction. Generation G cannot advance while any `accepted_pending`, `running`, or `paused_pending` custody remains.
- After the durable G → G+1 fence, fresh generation-G submissions are rejected as `WrongGeneration` without creating journal custody; historical same-ID reconciliation remains observable.
- The old generation-bound Bridge route is revoked, the old Scheduler is cancelled and joined, and reliable Actor `Shutdown` is queued and joined before the G+1 Actor and Scheduler are spawned and published.
- Race and reconstruction evidence covers concurrent admission versus fencing, full Actor queues, old-Scheduler cancellation, Actor receiver closure, durable generation 1+ reopen, stale-command rejection, journal state, receipt generation, and Provider call counts.

This evidence is a review handoff only. It does not mark the Story, Iteration, ADR, Issue, or PR as Complete, Accepted, Approved, or merge-ready; exact-head CI and independent approval remain mandatory gates.

## Final-history handoff remediation (2026-08-04)

- Model/provider replacement performs fallible Provider, MCP, tool, skill and context preparation before the irreversible generation fence.
- It then advances durable generation, revokes old routes, joins the old Scheduler and Actor, reads canonical final transcript history, appends the switch marker, and constructs/publishes the replacement Actor.
- Focused race evidence queues a final old-generation transcript commit during retirement and proves replacement history observes it before the switch marker.
- Provider-discovery connection bounding remains test-only; production timeout behavior is outside I169 and unchanged.
