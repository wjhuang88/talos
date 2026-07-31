# 056: Transactional Steering Submission And Turn Ownership Boundary

## Status

Proposed / Review for TUI-041/I169. The maintainer's reviews on PRs #64 and #68 define required
constraints, but do not constitute acceptance of this decision. If accepted, this decision
supersedes ADR-049 only where ADR-049 deferred queue-control and drain-cardinality semantics; its
single-owner UI projection rule remains in force.

## Context

GitHub Issue #50 asks Talos to consider every steering input queued during one model turn together
instead of launching one follow-up turn per item. The initial implementation changed
`Vec<String>::remove(0)` into `mem::take().join("\n\n")`. PR #64 and #68 review showed that this
changes a core asynchronous data flow rather than a collection helper:

- `user_rx` and canonical session events are independent channels;
- `SessionOp::Submit` currently preempts an active actor turn even without `Interrupt`;
- the TUI bridge discards `session_id`, `turn_id`, and `sequence`;
- draining before bounded-SQ acceptance can lose every queued item;
- joining strings destroys item, input-kind, and attachment boundaries;
- scheduler submissions share the same SQ without source-aware arbitration;
- Cancel, Error, session switch, persistence, replay, and context limits need explicit semantics.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Only matching canonical session `Completed` may close a user turn | Hard | ADR-039 | No |
| Bounded SQ backpressure must not lose accepted user input | Hard | ADR-005; user-data integrity | No |
| New submissions must not implicitly cancel an active turn | Hard | explicit interruption boundary; PR #64 review invariant I2 | No |
| Queue state has one authoritative owner at each lifecycle stage | Hard | ADR-049; PR #64 review | No |
| Public protocol/API changes require a decision and migration | Hard | `AGENTS.md` | No |
| Use one explicit single-consumer flow; no global pub/sub | Hard | ADR-006 | No |
| “All messages typed while thinking” means source-time ordering | Assumption | informal Issue #50 wording | Yes; rejected below |
| A batch should be unbounded so it always contains the whole queue | Assumption | initial implementation | Yes; rejected below |
| Existing `SessionOp` variants remain supported | Soft compatibility constraint | current public API | Yes, but retained |

## Decision

### 1. Cutoff: bridge-accepted, not wall-clock source time

An item belongs to the current steering batch only after the bridge has classified it, attached any
per-item content, inserted it into the authoritative engine queue, and emitted its queue snapshot
before the bridge accepts the matching canonical `Completed` event.

An Enter press still waiting in `user_rx` at that linearization point belongs to a later batch.
Snapshot appearance is the observable acceptance boundary. Talos does not use `try_recv()`, a
biased `select!`, or wall-clock timestamps to claim a stronger cross-channel happens-before order.

### 2. Structured user steering and reversible ownership transfer

`ConversationEngine` owns structured steering items before actor acceptance. Every item keeps:

- an opaque item ID and monotonic enqueue sequence;
- source (`User`, `Scheduler`, or compatibility/external);
- kind (`UserTurn` or `PreviewRequest`);
- original text;
- item-bound image attachments.

The bridge performs `prepare -> bounded reserve/send -> actor acknowledgement -> commit`:

1. `prepare` freezes a prefix without deleting it;
2. bounded SQ capacity is reserved with a timeout against the currently selected sender;
3. the actor acknowledges the submission ID when it becomes its pending owner;
4. only the matching acknowledgement commits/removes the prepared engine prefix;
5. closed/full/timeout/sender-change rejection releases preparation and retains original FIFO.

The engine queue remains the UI projection source while a transfer is prepared. The bridge is a
coordinator, not another queue owner.

### 3. Session Actor owns accepted submission arbitration

After acknowledgement, `AppServerSession` is the single owner of pending submissions from user,
scheduler, and compatibility callers. It serializes them and never cancels the active turn merely
because another Submit arrives. Only `SessionOp::Interrupt` may request cancellation.

Within one source/priority domain, actor acceptance order is FIFO. Across sources:

1. an explicit user submission that resumes a paused actor has priority;
2. otherwise already-accepted actor FIFO is preserved;
3. scheduler submissions never implicitly preempt a user turn.

The actor emits bounded, content-free lifecycle evidence with submission ID/source/item count and
the correlated session/turn identity. Full user content is not logged.

### 4. Explicit bridge turn state and identity validation

The TUI conversation state distinguishes at least:

```text
Idle
Submitting(submission_id)
Running(session_id, turn_id, next_sequence, submission_id?)
Cancelling(session_id, turn_id, next_sequence)
PausedAfterFailure
```

Only matching `session_id + turn_id` and the expected monotonic sequence may advance or complete a
running/cancelling turn. Stale, duplicate, wrong-session, regression, and gap events are ignored
for state mutation and produce bounded diagnostics. Provider-level `AgentEvent::TurnEnd` never
drains steering.

### 5. Terminal policy

| Terminal outcome | User steering | Actor pending submissions | Automatic next turn |
| --- | --- | --- | --- |
| Success | prepare the next compatible bounded batch | preserve FIFO | Yes |
| Cancelled | retain and expose as paused | retain, paused | No |
| Error | retain and expose as paused | retain, paused | No |

A later explicit user message while paused joins retained user steering and requests resumption.
No automatic retry occurs, and rollback/retry IDs prevent duplicate execution.

### 6. Input-kind and persistence semantics

- Local slash/session/model/provider commands never enter a steering batch.
- `PreviewRequest` is classified before enqueue and never combines with `UserTurn`.
- User-turn items may batch only with adjacent compatible user-turn items.
- Attachments bind to the item accepted with the corresponding text; session switch cannot move
  them to another item or session.
- One batch is one tool/usage/permission turn, but original items are supplied to the agent as
  ordered `Message::User` / `Message::Multimodal` entries. Provider input and durable successful
  transcript therefore preserve recoverable A/B/C boundaries instead of persisting a joined
  delimiter string.
- Live UI renders the original accepted items in the same order as durable replay.
- Memory may use a bounded text projection for retrieval, but that projection is not authoritative.

### 7. Bounds and context policy

The first implementation uses hard limits shared by queue and tests:

- 64 KiB UTF-8 per text item;
- 128 queued user items and 1 MiB queued text in one conversation engine;
- 32 items and 256 KiB text in one batch;
- existing image count/byte/pixel limits continue to apply per item and batch;
- the actor validates the compacted history plus pending input against its model context budget
  before starting.

Oversized new items are rejected visibly. A queue exceeding one batch is split at an item boundary;
remaining items stay queued. Nothing is silently truncated. If context remains over budget after
compaction, the submission is rejected back to retained/paused state with a bounded diagnostic.

### 8. Session mutation and sender changes

`/new`, `/resume`, `/fork`, model/provider switching, and attachment mutation are rejected while a
turn, prepared transfer, or retained steering queue exists. This prevents an old completion or SQ
sender from draining input into a new session. A future explicit move/discard UX requires a new
story; it is not inferred here.

### 9. Public protocol migration

Talos adds structured submission operations/events while retaining legacy `Submit`,
`SubmitMultimodal`, and `PreviewRequest` variants. Legacy operations are normalized at the actor
boundary as single compatibility submissions. The additive enum surface is a pre-1.0 minor-release
change for downstream exhaustive matches; callers must include a wildcard arm and migrate to the
structured operation when they need source, identity, batching, or acknowledgement.

No global bus, new runtime dependency, permission bypass, or `unsafe` is introduced.

## Rejected Alternatives

- **Drain ready `user_rx` entries before completion.** Ready-state observation is not source-time
  ordering and risks consuming/reordering heterogeneous input.
- **Biased `tokio::select!`.** It can starve completion and still cannot prove physical-time order.
- **Join with another delimiter.** No delimiter recovers arbitrary multiline item boundaries.
- **Delete then push the joined string back on failure.** It loses item IDs, kinds, attachments,
  and exact rollback state.
- **Continue implicit actor preemption.** It violates single-active-turn and explicit-interrupt
  invariants, especially with scheduler and stale completion races.
- **Move all projection state into the TUI.** It violates ADR-049's single-owner rule.

## Validation And Evidence Gate

Implementation cannot move to Review until deterministic tests cover:

- bridge-accepted cutoff with both channels ready;
- matching/stale/duplicate/wrong-session/sequence-gap completion;
- Cancel/Error pause and explicit resume;
- closed/full/timeout/sender-change prepare rollback;
- scheduler ordering and no implicit preemption;
- PreviewRequest/slash/multiline/attachment classification;
- item, queue, batch, Unicode, and context bounds;
- live versus durable replay equivalence and no ghost/duplicate user entries;
- seeded stress interleavings with the ten PR-review invariants;
- locked workspace format/check/Clippy/test, Windows fixtures, governance validation, CI success,
  and a real rebuilt-TUI transcript.

## Rollback

Before release, revert the structured submission/state commits and restore the ADR-049 one-item
drain baseline. After a release containing the protocol additions, retain deserialization of the
new variants and disable batching through product policy rather than rewriting durable history.

## Reversal Triggers

Revisit this decision if actor-owned pending arbitration cannot preserve existing runtime/SDK
latency, multiple remote session implementations require a different seam, provider protocols
cannot accept consecutive same-role user messages without a safe adapter projection, or measured
context behavior requires a model-specific budget policy.
