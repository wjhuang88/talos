# 056: Transactional Steering Submission And Turn Ownership Boundary

## Status

Proposed for TUI-044/I169 on 2026-08-01. This decision recovers the reviewed architecture constraints preserved by Draft PR #120, but it is not accepted merely because the historical implementation or tests existed.

If accepted, this decision supersedes ADR-049 only for queue consumption, ownership transfer, turn arbitration, terminal handling, structured persistence and request-budget semantics. ADR-049's engine-owned read-only queue projection before acknowledgement remains in force.

## Context

Issue #119 recovers the product requirement formerly tracked by deleted Issue #50: compatible steering inputs accepted while one model turn is active should be considered together in one later turn.

The original narrow change replaced one-item FIFO removal with `mem::take().join("\n\n")`. Historical review proved this is a core asynchronous data-flow change:

- input and lifecycle events arrive through independent channels;
- deleting before bounded Session Queue acceptance loses user data on failure;
- a delimiter cannot preserve arbitrary multiline, kind, identity or attachment boundaries;
- uncorrelated completion can drain input into the wrong Session or Turn;
- ordinary Submit must not implicitly cancel an active turn;
- scheduler work must share one actor arbitration boundary;
- Cancel/Error, persistence, replay, session mutation and complete request budgets need explicit semantics.

Current main also assigns historical Story ID TUI-041 to Issue #69. TUI-044 is therefore the current Story owner for this recovered decision.

## Hard Constraints

| Constraint | Source |
|---|---|
| Only matching canonical Session lifecycle may complete a user turn | ADR-039 |
| Accepted input cannot be lost under bounded queue backpressure | ADR-005; user-data integrity |
| Ordinary Submit cannot implicitly cancel an active turn | explicit interruption boundary |
| Exactly one authoritative queue owner exists at each lifecycle stage | ADR-049; recovery review |
| Public API/protocol changes are additive or require migration | `AGENTS.md` |
| One explicit owned flow; no new global event bus | ADR-006 |
| Queue cutoff cannot claim unobservable wall-clock/source-time ordering | cross-channel correctness |
| I170 Windows process portability remains a separate PR | recovery scope split |

## Decision

### 1. Deterministic cutoff

An item belongs to the current batch only after the bridge has classified it, bound its attachments and metadata, inserted it into the authoritative engine queue, and made it part of the queue snapshot observed when matching completion is processed.

Input still waiting in another channel at that linearization point belongs to a later batch. Talos does not use timestamps, `try_recv()` draining, biased selection, or inferred keypress order to claim a stronger cutoff.

### 2. Structured items and Engine ownership

Before acknowledgement, `ConversationEngine` owns a FIFO queue of structured items. Every item retains:

- opaque stable item ID;
- monotonic enqueue sequence;
- source and kind;
- exact text and multiline boundaries;
- item-bound attachments and bounded metadata.

The historical or public single-item `drain_steering_queue` remains available. New batch APIs freeze and commit prefixes without requiring destructive migration.

### 3. Transactional transfer

Transfer is equivalent to:

```text
prepare -> reserve -> send -> acknowledge -> commit
```

- `prepare` freezes one compatible bounded prefix without deleting it;
- reservation/send are bounded and tied to the current sender/session generation;
- Actor acknowledgement identifies the submission and confirms ownership acceptance;
- only the matching acknowledgement commits/removes the prepared prefix;
- full, closed, timeout, sender/session replacement, shutdown, lost acknowledgement or pre-start rejection releases the preparation and preserves the original queue.

The Engine remains the queue projection source while transfer is prepared. The bridge coordinates; it does not become a second owner.

### 4. Actor ownership and arbitration

After acknowledgement, the Session Actor is the sole owner of the structured submission and all accepted pending work. It serializes user, scheduler and compatibility submissions and maintains at most one active model turn.

A normal Submit never cancels the active turn. Only the explicit interrupt operation may request cancellation.

Ordering policy:

1. explicit user input that resumes a paused Actor receives the defined resume priority;
2. otherwise already accepted Actor FIFO remains stable;
3. scheduler work never bypasses or preempts an active user turn.

### 5. Lifecycle identity and sequencing

The bridge tracks explicit Idle/Submitting/Running/Cancelling/PausedAfterFailure-equivalent states.

Only events matching the active:

- Session identity;
- Turn identity;
- sender/runtime generation;
- expected monotonic sequence;

may advance state or prepare a batch.

Stale, duplicate, wrong-session, wrong-turn, wrong-generation, regressive, gap and uncorrelated fallback events are rejected without queue mutation. Provider-level tool-use termination is not authoritative user-turn completion.

Diagnostics are bounded and contain identity/status metadata only, never user text or attachments.

### 6. Terminal policy

| Outcome | Engine queue | Actor pending | Automatic advancement |
|---|---|---|---|
| Success | prepare next compatible bounded prefix | preserve FIFO | Yes |
| Cancelled | retain and expose paused | retain paused | No |
| Error | retain and expose paused | retain paused | No |

A later explicit user submission requests resume through the same Actor. There is no hidden retry and item/submission identities prevent duplicate execution.

### 7. Input kinds and session mutation

- local slash/session/model/provider commands are classified before queueing and do not enter user-turn batches;
- PreviewRequest and incompatible kinds do not combine with UserTurn items;
- attachments remain bound to the item accepted with them;
- `/new`, `/resume`, `/fork`, model/provider change or attachment mutation cannot silently move retained or prepared work to another Session;
- mutation is rejected or requires a separately designed explicit discard/move action while active, prepared or retained work exists.

### 8. Provider and durable representation

One structured batch is one model/tool/usage turn, but its original items are adapted as ordered `Message::User` or `Message::Multimodal` entries.

A text join may be used only as a bounded secondary projection such as memory lookup. It is never the authoritative Actor input or durable record.

Successful live input, persistence and resumed Session history preserve identical item boundaries. Failed/unstarted transfers create no successful transcript entries, ghosts or duplicates.

### 9. Bounds and complete context budgeting

Hard limits cover at least:

- item text and attachment metadata;
- queue item count and aggregate bytes/tokens;
- batch item count and aggregate text/attachments;
- Actor pending items and aggregate size;
- initial complete Provider Request;
- continuation complete Provider Request;
- output reserve.

Request accounting includes system prompt, dynamic prompt, memory, workspace context, history, tool definitions, structured items, multimodal attachments and continuation overlay.

Overflow rejects or splits only at item boundaries. No silent truncation, loss or delimiter-only collapse is permitted.

### 10. Public compatibility

Legacy `SessionOp` and `ConversationEngine::drain_steering_queue` entrypoints remain supported. Structured variants are additive. Legacy operations normalize into single compatibility submissions at the Actor boundary.

Downstream exhaustive matches may require a documented pre-1.0 migration note, but existing single-item callers are not forced into a destructive migration.

## Rejected Alternatives

- delimiter joining of arbitrary user text;
- deleting then reconstructing the queue on send failure;
- draining currently ready input from another channel to approximate source-time ordering;
- biased `select!` as an ordering proof;
- ordinary Submit preemption;
- TUI-owned authoritative queue state;
- persistence before Actor start acknowledgement;
- initial-only context accounting that omits continuation overlays or tool definitions.

## Validation Gate

Implementation must deterministically cover the full Issue #119 matrix, including:

- Engine single/batch/empty/cutoff/Ack/rollback/dedupe;
- matching and rejected lifecycle identities/sequences;
- full/closed/timeout/replaced sender/session/Ack loss/start rejection/shutdown;
- A/B/C, multiline, attachments, slash, preview, scheduler and incompatible kinds;
- queue/batch/attachment/Actor/initial/continuation/output-reserve bounds;
- success persistence, Cancel/Error retention, resume parity, no ghost/duplicate/delimiter-only replay;
- fixed-seed interleavings;
- locked workspace format/check/Clippy/tests;
- governance, collaboration claim and release preflight;
- exact-head Windows and Unix/macOS CI;
- rebuilt real-TUI and provider mock/request-preview evidence.

No historical CI or recovery-branch test result satisfies a current-main gate.

## Relationship To I170

I170 owns Windows PowerShell, timeout, environment, path, long-list and portability fixture behavior. I169 may consume the merged I170 baseline but may not include I170 implementation files or restore its historical commits.

## Rollback

Before release, revert the current-main structured steering implementation and retain the known one-item ADR-049 behavior. After a release containing additive durable protocol variants, retain deserialization/compatibility support and disable automatic batching through product policy rather than rewriting persisted history.

## Reversal Triggers

Revisit if provider adapters cannot safely encode consecutive same-role user items, remote Session implementations need a different acknowledgement seam, Actor latency materially regresses, or measured request budgeting requires model-specific policies. Any replacement must preserve user-data ownership, retry and replay invariants.
