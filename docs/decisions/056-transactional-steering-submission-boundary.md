# 056: Transactional Steering Submission And Turn Ownership Boundary

## Status

Proposed for TUI-044/I169. Preactivation architecture hardening was recorded on 2026-08-02 from
`main@61cbb930bf9e91ddad1bc85fb79f7b13ecad317d`. I169 is Active in PR #131 and has reached a
fresh independent-review handoff after remediation under maintainer implementation authorization.
This lifecycle update does not accept ADR-056, complete I169/TUI-044, close Issue #119 or authorize
merge; Ready-for-review is only a handoff state and is not approval.

The I170 prerequisite completed through PR #126. This decision recovers the reviewed constraints
preserved by archival Draft PR #120, but neither the historical implementation nor this Proposed
record constitutes acceptance. Acceptance requires a fresh current-main implementation, complete
failure/recovery evidence, exact-head CI, real-TUI evidence and independent architecture review.

If accepted, this decision supersedes ADR-049 only for queue consumption, ownership transfer, turn
arbitration, terminal handling, pending-work recovery, structured persistence and request-budget
semantics. ADR-049's engine-owned read-only queue projection before acknowledgement remains in
force.

The active PR #131 remediation also treats same-Session model/provider replacement as a durable
activation barrier: the old generation is fenced and retired, the model-switch marker is committed
exactly once at the quiescent log tail, and only then may the replacement Actor and its command/event
routes become reachable. A marker persistence failure is a hard publication failure, not a warning;
restart/retry must preserve one marker and the same model-visible ordering. This language records the
implementation under review while ADR-056 remains Proposed.

## Context

Issue #119 requires compatible steering inputs accepted while one model turn is active to be
considered together in one later turn while remaining distinct ordered user items.

The original narrow change replaced one-item FIFO removal with `mem::take().join("\n\n")`. Historical
review proved this is a core asynchronous data-flow change:

- input and lifecycle events arrive through independent channels;
- deleting before bounded Session Queue acceptance loses user data on failure;
- a delimiter cannot preserve arbitrary multiline, kind, identity or attachment boundaries;
- uncorrelated completion can drain input into the wrong Session or Turn;
- ordinary Submit must not implicitly cancel an active turn;
- scheduler work must share one actor arbitration boundary;
- Cancel/Error, persistence, replay, session mutation and complete request budgets need explicit
  semantics;
- an in-memory acknowledgement or recent-ID cache cannot prove recovery after lost acknowledgement,
  actor replacement or process restart.

Current main assigns historical Story ID TUI-041 to Issue #69. TUI-044 is the current Story owner for
this recovered decision.

## Hard Constraints

| Constraint | Source |
|---|---|
| Only matching canonical Session lifecycle may complete a user turn | ADR-039 |
| Accepted input cannot be lost under bounded queue backpressure | ADR-005; user-data integrity |
| Ordinary Submit cannot implicitly cancel an active turn | explicit interruption boundary |
| Exactly one component has execution authority at each lifecycle stage | ADR-049; recovery review |
| Actor acknowledgement must survive lost delivery and actor reconstruction | Issue #119 recovery requirement |
| Pending work and successful transcript are different durable concepts | ADR-042; no-ghost invariant |
| Public API/protocol changes are additive or require migration | `AGENTS.md` |
| One explicit owned flow; no new global event bus | ADR-006 |
| Queue cutoff cannot claim unobservable wall-clock/source-time ordering | cross-channel correctness |
| Completed I170 Windows process portability remains a separate baseline | recovery scope split |

## Terminology And Identity

The implementation may use different Rust names, but it must preserve these logical identities:

- `SessionId`: stable logical Session identity.
- `SessionGeneration`: durable runtime epoch for one logical Session. Live replacement advances it atomically; process reconstruction of the same authority rehydrates it unchanged.
- `QueueItemId`: stable identity of one accepted input item.
- `BatchId`: stable identity of one compatible queue-prefix snapshot.
- `ReservationId`: identity of the exact Engine prefix frozen for transfer.
- `TransferAttemptId`: one bounded send/reconciliation attempt; retries do not create new item or
  batch identities.
- `ReceiptId`: durable Actor acceptance receipt.
- `TurnId`: canonical model-turn identity assigned only when accepted pending work starts.

IDs are opaque and content-independent. Text hashes are not identities. Reusing a `BatchId` with a
different Session, generation, item list, text or attachment metadata is a conflict and fails
closed.

## Authoritative Data Model

### Queued item

One queued item retains at least:

```text
QueueItem {
  item_id
  session_id
  accepted_generation
  enqueue_sequence
  source
  kind
  exact_text
  attachments[] { attachment_id, digest, mime, byte_count, path/reference metadata }
  bounded size metadata
}
```

`source` is item-level (`User`, `Scheduler`, or compatibility/external), not inferred from a text
prefix. `kind` is fixed before admission (`UserTurn`, `PreviewRequest`, or a future explicitly
approved kind). Local slash/session/model/provider commands are resolved before this model and do
not become model-work items.

### Prepared batch

```text
PreparedBatch {
  batch_id
  reservation_id
  session_id
  session_generation
  cutoff_sequence
  ordered_items[]
  transfer_attempt_id
  aggregate bounds
}
```

A prepared batch is an immutable compatible FIFO prefix. New Engine inputs remain behind that
reservation and cannot be retroactively added.

### Durable Actor receipt

```text
SubmissionReceipt {
  receipt_id
  batch_id
  reservation_id
  session_id
  session_generation
  payload_fingerprint
  state
  turn_id?
}
```

The content-free protocol response may expose identity and state, but not user text or attachment
paths. The underlying session-scoped pending journal necessarily stores recoverable item data under
the same local protection boundary as Session data.

## Decision

### 1. Deterministic cutoff

An item belongs to the current batch only after the bridge has classified it, bound its attachments
and metadata, inserted it into the authoritative Engine queue, assigned its monotonic enqueue
sequence and made it part of the queue snapshot present when matching authoritative completion is
processed.

The authoritative cutoff is the Engine's accepted enqueue-sequence snapshot at that linearization
point. Input still waiting in another channel belongs to a later batch. Talos does not use wall-clock
timestamps, `try_recv()` draining, biased selection, channel-send time or inferred keypress order to
claim a stronger cross-channel happens-before relationship.

### 2. Engine queue and reservation

Before transfer, `ConversationEngine` owns the structured FIFO queue and is the only source of its
UI projection.

`prepare` freezes a compatible bounded prefix without deleting or rewriting it. While prepared:

- items remain visible in the Engine projection;
- the reservation prevents another batch from selecting them;
- their text, kind, source, identity and attachments are immutable;
- session/model/provider mutation is blocked;
- the legacy public `drain_steering_queue` remains source-compatible but is not used as the
  authoritative runtime batch path.

If SQ capacity cannot be reserved, the channel is closed, the reserve timeout expires, or sender
identity changes before a send occurs, the reservation rolls back and the Engine remains the sole
owner.

### 3. Unique execution authority during send and acknowledgement

A successful channel send does not by itself prove Actor ownership. After send, the Engine changes
the reservation to `AwaitingReceipt`:

- the Engine retains an escrow copy for recovery and projection;
- that escrow is not eligible for execution, re-batching or movement to another Session;
- only the addressed Actor may acquire execution authority;
- the bridge must reconcile uncertainty instead of blindly rolling back and resending.

This distinction preserves one execution authority while retaining a recoverable copy. The Engine
must never treat `AwaitingReceipt` as ordinary queued work.

### 4. Session-scoped pending journal

The Session Actor must not acknowledge ownership until it has atomically and idempotently recorded
the complete structured submission in a session-scoped pending journal owned by `talos-session` or
an equivalent Session storage boundary.

The pending journal is not transcript history. It records uncompleted work and receipt state. It is
stored separately from model-visible Session entries and must never appear in transcript/replay as
an already-started user message.

Minimum journal states are equivalent to:

```text
AcceptedPending
Running(turn_id)
PausedPending
TerminalCancelled(turn_id?)
TerminalError(turn_id)
Committed(turn_id)
```

`AcceptedPending`, `Running` and `PausedPending` retain the structured items. Terminal records retain
only the data required by the recovery/idempotency policy; successful transcript data remains in
the canonical Session log.

Journal requirements:

- atomic create/update using the repository's reviewed local-storage patterns;
- stable schema version;
- absence of the sidecar/record means an empty inbox for legacy Sessions;
- exact `SessionId + SessionGeneration + BatchId + ReservationId` keying;
- payload fingerprint conflict detection;
- bounded total records, items, bytes and attachment metadata;
- idempotent reopen and reconciliation after restart;
- no credentials, provider raw responses, reasoning or raw tool output;
- the same local file permissions and path-safety expectations as Session storage.

For a resumable Session, durable journal success is mandatory before acknowledgement. A runtime
that cannot provide that Session storage capability must reject transactional batched admission
visibly rather than claim restart-safe ownership.

### 5. Actor acknowledgement semantics

The sole ownership acknowledgement is a canonical `SubmissionAccepted`-equivalent receipt emitted
after the pending journal transaction succeeds.

It means:

> The addressed Session generation has durable, idempotent execution authority for this exact
> reservation and can reconstruct it without the Engine copy.

It does not mean the model turn has started. `TurnStarted` is a later, separately correlated event.

The receipt contains at least `SessionId`, `SessionGeneration`, `BatchId`, `ReservationId`,
`ReceiptId` and disposition:

```text
AcceptedPending
AlreadyAccepted { receipt_id, state, turn_id? }
Rejected { bounded_reason }
```

`AlreadyAccepted` is required for lost-ack recovery. A generic `Duplicate` rejection is insufficient
because it does not tell the Engine whether it can safely commit its escrow copy.

After a matching `AcceptedPending` or payload-identical `AlreadyAccepted`, the Engine commits only
the exact reserved item IDs and updates the UI projection. A mismatched receipt does not mutate the
queue.

### 6. Lost acknowledgement and reconciliation

If a send succeeded but no valid receipt arrives within the bounded acknowledgement window, the
bridge enters `Reconciling`, not `RolledBack`.

Reconciliation uses the same Session identity and generation and either:

- queries the receipt by `BatchId + ReservationId`; or
- resubmits the identical envelope through an explicitly idempotent operation.

Outcomes:

- `AlreadyAccepted` with matching fingerprint: commit Engine escrow;
- explicit `NotAccepted` from the still-authoritative generation: release reservation for retry;
- generation/session unavailable or ambiguous: keep escrow frozen, pause automatic progress and
  require recovery of the addressed Session; never send it to a new Session;
- conflicting fingerprint: fail closed and surface bounded diagnostics.

A sender replacement cannot reinterpret an old in-flight submission as belonging to the new sender.
Session mutation remains blocked until the old reservation is reconciled or explicitly discarded by
a separately approved UX.

### 7. Session generation

`SessionGeneration` is assigned by the Session composition root, not inferred only from channel
object identity. It starts at a defined value for a newly created Session and is stored durably with
the Session pending-journal metadata. A live replacement of the Actor/runtime for the same logical
Session atomically advances the durable generation. A process reconstruction after memory loss
rehydrates the stored generation unchanged so already accepted envelopes remain addressable; it does
not invent generation zero or silently rewrite those envelopes.

Every structured Session operation, receipt and canonical Turn event carries:

- `SessionId`;
- `SessionGeneration`;
- the relevant batch/receipt/turn identity;
- a per-turn monotonic sequence for Turn events.

A local sender epoch may be used to detect a changed channel before send, but it cannot replace the
authoritative Session generation.

The bridge validates in this order:

1. current Session identity;
2. current Session generation;
3. state permits the event;
4. matching batch/receipt or active Turn identity;
5. exact expected sequence;
6. payload valid for the current state.

Wrong-session, wrong-generation, no-active-turn, wrong-turn, duplicate/regressive sequence,
sequence gap and uncorrelated events do not mutate state or drain input. A sequence gap fails closed;
the bridge does not skip forward.

### 8. Actor arbitration and one active turn

After acceptance, the Session Actor is the sole execution arbiter for user, scheduler and
compatibility work. It maintains at most one active model turn.

A normal Submit never cancels or preempts the active turn. Only an explicit Interrupt targeting the
matching Session generation and active Turn may request cancellation.

Pending ordering is deterministic:

1. within each source domain, acceptance order is FIFO;
2. work already running is never preempted by a later submission;
3. after Success, accepted pending work advances according to FIFO and compatibility boundaries;
4. after Cancel/Error, automatic advancement pauses;
5. retained user-origin work is ahead of scheduler work when an explicit user submission resumes a
   paused Actor;
6. the explicit resuming user item joins the user FIFO and authorizes resume; it does not reorder
   older retained user items;
7. scheduler work never itself resumes a paused Actor.

### 9. Scheduler delivery and SQ-full semantics

Scheduler work uses the same structured submission and Actor receipt boundary. Source identity is
not encoded only in visible text. Every scheduler owner is spawned from one authoritative
`{sender, SessionId, SessionGeneration}` target after that generation has been assigned; a raw sender
or hard-coded generation zero is not a valid delivery route.

A generated scheduler fire receives a stable fire/submission identity. Until the Actor journal
accepts it, the scheduler remains responsible for that exact fire.

On SQ full, reserve timeout, closed sender or transient Actor rejection:

- the fire is not dropped and is not reported as delivered;
- it remains in a bounded `DeliveryBlocked`/pending state under the same identity;
- retry is bounded and does not busy-loop;
- the task's status exposes a content-free delivery failure/blocked condition;
- cancellation can remove the undelivered fire explicitly.

For a recurring task, an outstanding blocked fire is not replaced by a later tick. The task remains
blocked until that fire is delivered or cancelled; changing recurrence coalescing semantics requires
a separate decision. This prevents silent loss while keeping I169 out of persistent-task scope.

### 10. Interrupt, Cancelled and Error semantics

Interrupt targets only the active Turn. If no Turn is active, it does not pop, reject or reorder
pending work.

On matching `Cancelled` or `Error`:

- the active started batch reaches a terminal state and is never automatically replayed, because it
  may have performed side effects;
- Engine-owned queued work remains queued;
- Actor-owned submissions that have not started remain durable and become `PausedPending`;
- scheduler and user pending work retain identities and FIFO/source ordering;
- automatic advancement stops;
- an explicit user submission may authorize resume under the arbitration policy above when the
  retained request is admissible;
- a deterministic pre-Provider failure such as `ContextBudgetExceeded` requires an explicit,
  generation-bound pre-start resolution. The user may terminalize that exact paused identity as
  `TerminalCancelled` without Provider execution, after which later accepted work may advance. New
  input alone must not repeatedly retry an impossible FIFO head forever.

The current failed/cancelled Turn is not silently retried. Any explicit retry UX for that already
started batch is outside I169 unless separately approved. Partial-turn persistence continues to obey
its existing owner and must not be expanded opportunistically.

### 11. Session mutation and shutdown

`/new`, `/resume`, `/fork`, model/provider change, skill/context mutation and attachment mutation are
rejected while the current Session has:

- an active Turn;
- prepared or awaiting-receipt Engine work;
- Actor `AcceptedPending`, `Running` or `PausedPending` work.

No retained item moves implicitly across Session identity or generation.

Shutdown stops admission first. It then:

- safely rejects reservations that were never sent;
- reconciles sent-but-unacknowledged reservations where possible;
- preserves acknowledged pending journal records;
- interrupts/finalizes the active Turn according to the existing bounded shutdown owner;
- emits display-safe outcomes without replaying side effects.

I169 must remain compatible with the broader graceful-shutdown work owned elsewhere; it does not
claim that owner complete.

### 12. Provider and durable transcript representation

One structured batch is one model/tool/usage Turn, but original items adapt to ordered
`Message::User` or `Message::Multimodal` entries. A text join may be used only as a bounded secondary
projection such as memory retrieval input. It is never authoritative Actor input, Provider history
or durable transcript.

Successful Turn finalization order is:

1. atomically/idempotently commit the successful Turn's model-visible messages to the canonical
   Session transcript using the Turn identity;
2. mark the pending journal record `Committed` or remove it idempotently;
3. emit the successful terminal evidence.

The transcript commit precedes inbox deletion. If a crash occurs between steps 1 and 2, recovery
finds the committed Turn identity and finalizes the journal without re-executing it. Reversing that
order is forbidden because it can lose accepted work.

Unaccepted, prepared or merely acknowledged pending work creates no transcript entry. Cancel/Error
terminal handling follows the existing partial-turn persistence policy and never writes unstarted
pending submissions as successful user messages.

### 13. Exact Provider request planning and budgets

Initial and continuation requests use an exact `ProviderRequestPlan`-equivalent value:

```text
ProviderRequestPlan {
  ordered messages
  system/dynamic/memory/workspace sections
  tool definitions and schemas
  provider options
  structured multimodal projection
  continuation overlay
  output reserve
}
```

The same immutable plan instance, or a semantically identical sealed representation with a verified
fingerprint, is both budget-validated and sent to the Provider. Estimation and sending must not
independently rebuild the request.

Every initial request and every continuation constructs a new exact plan from the current canonical
Turn state and validates:

- system and dynamic prompts;
- memory/todo/scheduler additions;
- workspace context;
- conversation history;
- structured user items;
- tool definitions and JSON schemas;
- assistant/tool continuation messages and overlay;
- multimodal attachment cost through Provider-specific accounting or a documented conservative
  upper bound;
- framing/safety margin where required;
- output reserve.

Overflow policy:

- before Turn start, history may be compacted under existing policy and the prepared batch may split
  only at item boundaries;
- one item that still cannot fit is visibly rejected without truncation;
- after Turn start, an over-budget continuation terminates the current Turn as Error and pauses
  pending work; it never truncates tool results, user text, attachments or output reserve silently.

Request Preview/Mock evidence must show the budgeted plan is the sent plan for both initial and
continuation paths.

### 14. Bounds

Hard limits cover at least:

- per-item text bytes;
- per-item attachment count and metadata bytes;
- Engine queue items and aggregate bytes;
- prepared batch items and aggregate text/attachments;
- pending-journal records, items and aggregate bytes;
- Actor pending work;
- Scheduler blocked delivery;
- receipt/reconciliation attempts and diagnostic rate;
- initial and continuation request budgets;
- output reserve.

All arithmetic is checked/saturating as appropriate. Overflow rejects visibly or splits only at
item boundaries. No silent truncation, loss, delimiter collapse or unbounded retry is permitted.

### 15. Public and durable compatibility

The existing public `ConversationEngine::drain_steering_queue` retains single-item FIFO behavior.
New Engine batch APIs are additive.

Legacy `SessionOp::Submit`, `SubmitMultimodal` and `PreviewRequest` remain accepted and normalize at
the Actor boundary as one-item compatibility submissions. New structured operations/events are a
pre-1.0 public-enum migration risk; release notes require downstream exhaustive matches to add the
new variants or a forward-compatible wildcard.

The pending journal is an additive versioned Session sidecar/record domain, not a rewrite of the
successful transcript format. Legacy Sessions without it read as an empty inbox. Once a release can
write the new journal or protocol variants, rollback keeps their readers and idempotent cleanup
support even when product batching is disabled.

Unknown future journal versions fail closed without mutating or executing pending work.

## Ownership State Model

```text
EngineQueued
  -> EnginePrepared
  -> AwaitingSqCapacity
  -> AwaitingReceipt (Engine escrow; non-executable)
  -> ActorAcceptedPending (durable receipt; Actor execution authority)
  -> EngineCommitted (escrow removed after matching receipt)
  -> ActorRunning(turn_id)
  -> ActorCommitted | TerminalCancelled | TerminalError

EnginePrepared/AwaitingSqCapacity
  -> rollback to EngineQueued on pre-send failure

AwaitingReceipt
  -> Reconciling on timeout/lost acknowledgement
  -> ActorAcceptedPending on matching Accepted/AlreadyAccepted
  -> EngineQueued only after authoritative NotAccepted
  -> PausedAmbiguous when old Session generation cannot be reconciled

ActorAcceptedPending
  -> PausedPending after another Turn Cancel/Error or deterministic pre-start failure
  -> ActorRunning only when arbitration permits

PausedPending
  -> TerminalCancelled on an explicit exact-generation pre-start cancel
  -> ActorRunning only through the documented resume policy
```

At no point may both Engine and Actor have execution authority. The Engine escrow after send is a
recovery copy only. At no point may neither component have a recoverable copy.

## Content-Free Diagnostics

Diagnostics may include bounded reason codes, counts and shortened opaque IDs. They do not include
user text, attachment paths, provider payloads, tool arguments, credentials or reasoning.

Repeated stale/gap/reconciliation diagnostics are rate- or count-bounded so malicious or buggy
producers cannot grow UI/log memory without limit.

## Rejected Alternatives

- delimiter joining of arbitrary user text;
- deleting then reconstructing the queue on send failure;
- treating channel receive or `TurnStarted` as the ownership acknowledgement;
- memory-only recent-ID dedupe as lost-ack recovery;
- rolling back immediately after an acknowledgement timeout;
- draining currently ready input from another channel to approximate source-time ordering;
- biased `select!` as an ordering proof;
- ordinary Submit preemption;
- Interrupt deleting a pending submission;
- Scheduler `try_send` drop/coalesce on full;
- clearing attachments when a sender changes;
- TUI-owned authoritative queue state;
- writing pending inbox records as successful transcript messages;
- deleting inbox state before successful transcript commit;
- separately rebuilding estimated and actual Provider requests;
- initial-only context accounting that omits continuation overlays or tool definitions.

## Implementation Slices

A fresh implementation should remain one I169 PR but use independently reviewable commits:

1. typed identities, structured items, protocol compatibility and generation envelope;
2. Engine queue/reservation/cutoff without Actor execution;
3. session pending journal, idempotent receipts and reconciliation;
4. Actor arbitration, scheduler delivery and terminal pause semantics;
5. bridge state machine, receipt commit and mutation gates;
6. exact Provider request plan and initial/continuation budgeting;
7. transcript/journal recovery parity and migration fixtures;
8. stress, real-TUI, mock/request-preview, docs and governance evidence.

The pending-journal/receipt slice is the highest-risk review boundary and must be independently
reviewed before later UI simplification can hide its states.

## Validation Gate

Implementation must deterministically cover the full Issue #119 matrix, including:

- Engine single/batch/empty/cutoff/Ack/rollback/dedupe;
- reservation escrow and prohibition on double execution authority;
- new acceptance, lost Ack, `AlreadyAccepted`, conflict and authoritative `NotAccepted`;
- journal create/reopen/idempotency/crash windows/unknown-version failure;
- matching and rejected Session/generation/Turn identities and sequences;
- full/closed/timeout/replaced sender/session/start rejection/shutdown;
- A/B/C, multiline, attachments, slash, preview, scheduler and incompatible kinds;
- Scheduler blocked-delivery recovery without drop;
- Cancel/Error retention, active-batch no-auto-replay and explicit user resume ordering;
- item/queue/batch/attachment/journal/Actor/initial/continuation/output-reserve bounds;
- success transcript-before-inbox-finalization crash recovery;
- no ghost/duplicate/delimiter-only replay;
- exact-plan budget/send fingerprints for initial and continuation calls;
- fixed-seed interleavings checking no loss, no duplicate, FIFO, unique execution authority, at most
  one active Turn and no cross-Session mutation;
- locked workspace format/check/Clippy/tests;
- governance, collaboration claim and release preflight;
- exact-head Windows and Unix/macOS CI;
- rebuilt real-TUI and Provider mock/request-preview evidence.

No historical CI or recovery-branch result satisfies a current-main gate.

## Relationship To Adjacent Owners

- I170 is Complete and remains outside I169.
- Partial/interrupted active-Turn persistence remains owned by its existing Session work; I169 only
  prevents unstarted pending work from becoming ghost transcript.
- Broader graceful shutdown/finalization remains owned by its existing Runtime issue; I169 supplies
  compatible pending-work reconciliation semantics.
- Persistent task runtime is not introduced. The pending journal is Session-scoped delivery state,
  not a general autonomous task/checkpoint engine.

## Activation Gate

Before product implementation:

- merge/review the preactivation architecture hardening or explicitly supersede it;
- re-read current `main`, Issue #119, open PRs/branches and owner documents;
- confirm no newer or overlapping steering authority exists;
- receive explicit maintainer instruction to activate and implement I169;
- create a fresh implementation branch from the exact then-current `main`;
- record activation time, Responsible Actor, Executing Agent, baseline SHA, branch and Draft PR in
  I169/TUI-044/Board before product-code mutation;
- keep ADR-056 Proposed through implementation review;
- keep recovery PR #120 and its branch immutable.

## Rollback

Before release, revert the current-main structured steering implementation and retain the known
one-item ADR-049 behavior. After a release containing additive journal/protocol variants, retain
read/deserialization and idempotent cleanup support and disable automatic batching through product
policy rather than rewriting persisted history.

## Reversal Triggers

Revisit if Provider adapters cannot safely encode consecutive same-role user items, remote Session
implementations need a different durable receipt seam, measured pending-journal cost is unacceptable,
Actor latency materially regresses, or exact request planning requires Provider-specific policy.
Any replacement must preserve unique execution authority, recoverable custody, retry and replay
invariants.

## Independent-review remediation handoff (2026-08-04)

Lifecycle remains unchanged while PR #131 awaits a new independent review: **TUI-044 / I169 are Active, ADR-056 is Proposed, and Issue #119 is Open**.

The latest implementation evidence tightens same-Session generation replacement into one acknowledged ownership handoff:

- SQLite admission and generation advance share one immediate transaction. Generation G cannot advance while any `accepted_pending`, `running`, or `paused_pending` custody remains.
- After the durable G → G+1 fence, fresh generation-G submissions are rejected as `WrongGeneration` without creating journal custody; historical same-ID reconciliation remains observable.
- The old generation-bound Bridge route is revoked, the old Scheduler is cancelled and joined, and reliable Actor `Shutdown` is queued and joined before the G+1 Actor and Scheduler are spawned and published.
- Race and reconstruction evidence covers concurrent admission versus fencing, full Actor queues, old-Scheduler cancellation, Actor receiver closure, durable generation 1+ reopen, stale-command rejection, journal state, receipt generation, and Provider call counts.

This evidence is a review handoff only. It does not mark the Story, Iteration, ADR, Issue, or PR as Complete, Accepted, Approved, or merge-ready; exact-head CI and independent approval remain mandatory gates.

## 2026-08-04 exact activation identity remediation

The current review cycle requires same-Session model activation durability to carry the complete
ADR-048 identity: provider, model, and normalized variant. PR #131 now records a machine-readable
activation object containing the durable target generation, deterministic activation ID, exact
previous identity, and exact target identity. `None`, empty, and `default` variants normalize to the
same baseline identity.

Visible marker text is not the idempotency key. Only an exact activation object may be reused after
an interrupted commit/publication cut point; a new intentional switch, including a variant-only
switch on the same provider/model, creates a distinct activation. Session startup restores the
variant from this Session-owned record before Provider construction, so a later global config write
failure cannot silently restore different request semantics.

This is implementation evidence under review. TUI-044 and I169 remain Active, ADR-056 remains
Proposed, and Issue #119 remains Open.
