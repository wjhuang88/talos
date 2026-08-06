# 056: Transactional Steering Submission And Turn Ownership Boundary

## Status

**Accepted (2026-08-06).**

ADR-056 is accepted for TUI-044 / I169 based on merged PR #131, exact-head automated acceptance,
independent review remediation and rebuilt real-terminal validation.

Acceptance evidence:

- implementation PR: #131;
- exact accepted Head: `90165cace4625c0f27616b3e1b9871bcb6a10186`;
- final exact-head CI: run `31010166558` / CI #1233, attempt 1, all jobs successful;
- completion / merge commit: `685d3b4f4088a172551f8c844a89f5dee9469430`;
- completed Story and Iteration: TUI-044 / I169;
- completed source Issue: #119;
- non-blocking diagnostic residual: #136.

This decision supersedes ADR-049 for queue consumption, transactional ownership transfer, Actor
arbitration, terminal handling, pending-work recovery, structured persistence and exact request-plan
semantics. ADR-049 remains active for the Engine-owned, read-only queue projection before durable
Actor acknowledgement.

## Context

Issue #119 requires compatible steering input accepted while one model Turn is active to become one
bounded later Turn while each input remains a distinct ordered user item.

A destructive single-item drain or delimiter join cannot safely implement this behavior because:

- input and lifecycle events arrive through independent channels;
- deletion before bounded Actor acceptance can lose user data;
- delimiters cannot preserve arbitrary multiline, kind, identity or attachment boundaries;
- uncorrelated completion can drain input into the wrong Session, generation or Turn;
- ordinary Submit must not implicitly cancel an active Turn;
- user and Scheduler work require one arbitration authority;
- Cancel/Error, restart, replay, Session mutation and full Provider request budgeting require explicit
  ownership and recovery semantics;
- an in-memory acknowledgement or recent-ID cache cannot prove custody after lost Ack, Actor
  replacement or process restart.

TUI-044 is the current Story owner. Recovery PR #120 and branch
`recovery/pr-68-i169-20260731` remain immutable archival evidence, not implementation authority.

## Decision

### 1. Structured authoritative queue

Before Actor acceptance, `ConversationEngine` owns a structured FIFO queue. Every item retains:

- stable item identity;
- target Session identity and accepted Session generation;
- monotonic enqueue sequence;
- source and kind classification;
- exact text;
- attachment identity and bounded metadata.

Local slash/session/model/provider commands and preview-only operations are classified before model
work admission. Incompatible kinds are never silently combined through text delimiters.

### 2. Observable deterministic cutoff

The batch cutoff is the Engine-accepted enqueue-sequence snapshot processed with matching canonical
Success. Input still waiting in another receiver/channel belongs to a later batch. Talos does not use
wall-clock time, channel-send time, biased selection, `try_recv()` draining or inferred keypress order
to claim an unobservable cross-channel ordering guarantee.

### 3. Prepare, reserve and escrow

`prepare` freezes one compatible bounded FIFO prefix without deleting it. The reservation keeps the
items visible and immutable and blocks conflicting Session/model/provider mutation.

Before send, Engine remains sole owner. Full/closed/timeout/replaced-sender failure rolls back the
reservation exactly.

After successful channel send, Engine retains immutable non-executable escrow. The escrow exists for
reconciliation and projection only; it cannot be re-batched or executed. Only the addressed Session
Actor may acquire execution authority.

### 4. Durable Session-scoped pending journal

The Actor acknowledges only after atomically and idempotently storing the complete structured
submission in a versioned Session-scoped pending journal separate from successful transcript.

Logical states include:

```text
AcceptedPending
Running(turn_id)
PausedPending
TerminalCancelled(turn_id?)
TerminalError(turn_id)
Committed(turn_id)
```

The journal is bounded, schema-versioned, payload-conflict detecting, restart-safe and protected by
the same local storage/path rules as Session data. Legacy Sessions without the sidecar have an empty
pending inbox.

### 5. Durable receipt and lost-Ack reconciliation

The ownership acknowledgement is a canonical receipt emitted only after journal commit. It carries
exact Session, generation, batch, reservation and receipt identity and distinguishes:

```text
AcceptedPending
AlreadyAccepted { state, turn_id? }
Rejected { bounded_reason }
```

A matching `AcceptedPending` or payload-identical `AlreadyAccepted` lets Engine commit only the exact
reserved prefix. A missing receipt enters bounded reconciliation, never blind rollback/resend.

Reconciliation outcomes are:

- matching `AlreadyAccepted`: commit Engine escrow;
- authoritative `NotAccepted` from the same generation: release for retry;
- unavailable or ambiguous generation: freeze and surface recovery state;
- fingerprint conflict: fail closed.

### 6. Durable generation and correlated lifecycle

`SessionGeneration` is assigned and persisted by the Session composition root. Live replacement
advances it atomically only after admission is fenced and the old Scheduler/Actor are acknowledged as
retired. Process reconstruction rehydrates the durable generation unchanged.

Structured operations, receipts and canonical Turn events carry exact Session, generation and
relevant batch/receipt/Turn identities. Turn events also carry an exact monotonic sequence.

Wrong Session, wrong generation, no active Turn, wrong Turn, duplicate/regressive sequence, sequence
gap and invalid-state events cannot mutate active state or drain input.

### 7. One Actor execution authority

After durable acceptance, the Session Actor is the sole execution arbiter for user, Scheduler and
compatibility work and maintains at most one active model Turn.

- Ordinary Submit never cancels or preempts active work.
- Interrupt targets only the exact active Session generation and Turn.
- Within a source domain, accepted work remains FIFO.
- Success may advance eligible pending work.
- Cancel/Error pauses unstarted pending work.
- An explicit user submission may authorize resume without moving ahead of older retained user work.
- Retained user work is ahead of Scheduler work on explicit resume.
- Scheduler work never resumes a paused Actor by itself.

### 8. Scheduler delivery

Scheduler work uses the same structured envelope, durable generation and Actor receipt boundary.
Until journal acceptance, the Scheduler owns the exact fire/submission identity.

On full queue, timeout, closed sender or transient rejection, the fire remains one bounded blocked
identity. It is not dropped or reported delivered, does not busy-loop, and a recurring task does not
replace it with a later tick until delivery or explicit cancellation.

### 9. Terminal and pre-start failure policy

A started Cancelled/Error Turn is terminal and is not automatically replayed because side effects may
already have occurred. Unstarted pending work remains durable and paused.

A deterministic pre-Provider failure can be resolved by an explicit generation-bound terminalization
operation without Provider execution so an impossible FIFO head cannot pin later work forever.

### 10. Session mutation and activation barrier

`/new`, resume, fork, model/provider/variant change and other Session mutation are rejected while the
current Session has active, prepared, awaiting-receipt, accepted, running or paused custody.

Same-Session model/provider/variant replacement follows one durable barrier:

1. prepare the target runtime identity;
2. fence old-generation admission;
3. retire and join the old Scheduler and Actor;
4. commit one exact activation record/marker at the quiescent Session-log tail;
5. rebuild from a fresh canonical history read;
6. publish the new generation route.

Failure before publication leaves the Session fenced/stopped with explicit recovery; it cannot expose
a partially switched route.

### 11. Provider and transcript representation

One structured batch produces one model/tool/usage Turn, but its original items adapt to ordered User
or Multimodal messages. Text joins are allowed only for bounded secondary projections such as memory
lookup, never as authoritative Actor input, Provider history or transcript.

Successful finalization order is:

1. idempotently commit model-visible successful Turn entries to canonical transcript;
2. mark the pending-journal identity Committed;
3. emit correlated terminal lifecycle.

Restart scans Running custody. Transcript proof finalizes it without Provider re-execution; ambiguous
execution state fails closed rather than replaying.

### 12. Exact request-plan boundary

Every initial and continuation Provider call constructs one sealed exact request plan containing all
model-visible request components, including dynamic prompts, memory/context, history, tools/schemas,
structured input, multimodal cost, continuation overlays and output reserve.

Budget validation and dispatch consume the same plan/fingerprint. A separately reconstructed estimate
is not authoritative.

### 13. Bounds, compatibility and cleanup

Hard bounds cover queue items/bytes, batch items/bytes, attachments, journal custody, Actor pending
work, Scheduler blocked delivery and complete initial/continuation Provider context. Overflow rejects
visibly or splits only at item boundaries; no user item is silently truncated, discarded or duplicated.

Public single-item drain and legacy Session operations remain available through additive migration.

Session artifact ownership uses a transcript-last commit boundary. WAL/SHM/SQLite sidecars are
removed before transcript deletion; partial failure reports no false success and remains retryable.
Bounded orphan reconciliation validates root, UUID, suffix, symlink, regular-file and live-owner
safety before cleanup.

## Accepted Invariants

- Exactly one component has execution authority at every stage.
- Before durable receipt, Engine queue/escrow is the recoverable owner.
- After durable receipt, only the addressed Actor generation may execute.
- Lost Ack never causes blind duplicate execution.
- Ordinary Submit never implicitly cancels active work.
- A/B/C remain distinct ordered messages through request, transcript and resume.
- Pending custody and successful transcript are separate durable concepts.
- Successful transcript commit precedes journal finalization.
- Session generation replacement cannot expose old commands to a new Actor.
- Budget validation and Provider dispatch use the same exact request plan.
- Failure paths preserve recoverability and never report false success.

## Validation Evidence

Acceptance is bound to:

- exact implementation Head `90165cace4625c0f27616b3e1b9871bcb6a10186`;
- exact-head CI run `31010166558` / CI #1233, all four jobs successful on attempt 1;
- independent architecture/code review and remediation of earlier blocking findings;
- rebuilt macOS release binary SHA-256
  `2fe9f07679bd3f513165e849c59335ef11f47662852283c8f22051e954b2683d`;
- real-terminal A/B/C queue, continuation, restart, restoration, fork isolation, deletion,
  retryability and maintenance-command walkthrough;
- merge commit `685d3b4f4088a172551f8c844a89f5dee9469430`;
- maintainer classification of Issue #136 as a separately owned non-blocking diagnostic residual.

## Consequences

### Positive

- Steering input is neither lost nor duplicated under backpressure, lost Ack or restart.
- Original user boundaries and attachments survive Provider adaptation and durable replay.
- Session generation and Actor replacement have one explicit durable authority boundary.
- Scheduler work shares the same lifecycle instead of bypassing Session ownership.
- Context admission is based on the actual request sent.
- Fork/delete/recovery behavior aligns with one transcript-last artifact lifecycle.

### Costs

- The implementation requires durable pending custody, opaque identities, reconciliation and more
  explicit lifecycle states.
- Session mutation can fail closed while custody remains unresolved.
- Protocol consumers must tolerate additive structured operations/events.
- Operators must retain sidecars and terminal tombstones/ledger summaries according to bounds rather
  than treating transcript as the only Session state.

## Residuals And Reversal Triggers

Issue #136 owns the missing executable recovery-command wording on direct `/delete` cleanup failure.
The underlying retryability and no-false-success behavior are accepted; the wording gap does not
change this decision.

Separate ADRs or amendments are required for:

- arbitrary queue editing/reordering;
- persistent cross-Session steering;
- automatic retry of an already-started terminal Turn;
- multi-controller arbitration;
- general persistent task/checkpoint runtime;
- changes to transcript-first success finalization or durable receipt ownership.

Revisit or supersede ADR-056 if production evidence shows duplicate execution, lost accepted input,
cross-generation mutation, transcript/journal divergence, request-plan drift or unbounded custody that
cannot be corrected while preserving this boundary.

## Historical Record

The complete Proposed-era plan and remediation chronology remain in git history at
`main@685d3b4f4088a172551f8c844a89f5dee9469430`, PR #131 and Issue #119. This Accepted record
preserves the final decision and evidence without retaining obsolete review-handoff language as
current policy.
