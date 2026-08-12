# ADR-058: Partial-Turn Durable Finalization Boundary

> Status: Proposed
> Date: 2026-08-11
> Owner: SESSION-008-A / I187

## Context

ADR-039 makes the session actor the authoritative turn lifecycle and persistence owner. ADR-042
adds an atomic, idempotent durable commit for successful embedded turns, but intentionally leaves
failed and interrupted turns without durable messages. The current implementation now exposes a
split boundary:

- the legacy `Session` provider-error path appends a display-safe partial prefix one message at a
  time and then appends an Error outcome marker;
- `DurableSession::commit_turn` atomically writes messages plus a Success marker;
- durable Error and Cancelled paths append only a hidden marker through the underlying `Session`;
- cancellation aborts the agent task before it can return its normalized partial messages.

As a result, a completed tool side effect visible before interruption can disappear after durable
restart, ordinary partial messages are not associated with their `turn_id`, and an embedder cannot
join the normalized transcript to a durable incomplete status without using lower-level APIs.
RUNTIME-005 cannot define bounded shutdown on top of two different finalization semantics.

## Decision

### One terminal finalization operation

`talos-session` will own one atomic, idempotent turn-finalization operation used by both the durable
embedded path and the Talos-owned session path. Its semantic input is:

```text
turn_id + admitted display-safe message prefix + Success | Error | Cancelled + persistence policy
```

The concrete pre-1.0 API may be named `finalize_turn`; existing `commit_turn` remains a
source-compatible Success wrapper during migration. `abort_turn` remains a compatibility wrapper
for an empty Error/Cancelled finalization or is deprecated with a documented replacement; it must
not silently erase already admitted facts.

Under the session write lock, finalization writes every admitted message with the same `turn_id`
and the hidden terminal outcome marker in one atomic replacement. The marker is recovery evidence,
not a visible transcript message. An empty Error/Cancelled finalization may write only the hidden
marker so journal custody is deterministic; it does not fabricate an empty completed turn.

### First terminal outcome wins

The first successfully persisted terminal outcome for a `turn_id` is authoritative.

- A retry with the same outcome and the same canonical filtered payload returns the original entry
  IDs and reports that no new write occurred.
- A retry with a different outcome or payload returns a structured conflict and changes nothing.
- A partial failure before atomic replacement leaves the prior file authoritative and is retryable.
- Recovery never infers Success, Error, or Cancelled from message entries alone.

This rule makes completion/cancellation/error races deterministic without replaying side effects.
The session actor must choose the winner before invoking finalization; storage is the final
compare-and-swap guard, not a second lifecycle scheduler.

### Only a closed, display-safe prefix is admissible

The persisted partial prefix uses the existing tool-specific persistence projection and
`PersistencePolicy`, then applies a structural closure rule:

- submitted user messages are stable facts;
- finalized assistant text is stable, but an unfinished streamed fragment is excluded;
- an assistant tool-call batch is admitted only with one completed projected result for every call
  in that batch;
- reasoning, credentials, private tool fields, raw arguments, raw provider payloads and
  non-admitted raw output remain excluded;
- a side effect with no completed projected result is not reconstructed as a fabricated result.

The turn task publishes admitted snapshots at stable boundaries to session-owned state. On
cancellation, the session actor finalizes the latest published snapshot rather than depending on a
return value from an aborted future. The actor remains the sole finalizer and event-order owner;
this does not add a global bus or a second durable writer.

### Restart projection and visible incomplete status

`DurableSession::read_messages` reconstructs model context from the same admitted entries used by
the normalized transcript, including Error/Cancelled prefixes. This prevents later turns from
contradicting completed tool effects. The hidden marker itself never enters model context.

The normalized durable API will expose turn outcome records keyed by `turn_id`, alongside the
existing transcript entries. Hosts join the outcome to `DurableTranscriptEntry::turn_id` and render
Error/Cancelled turns as incomplete/interrupted. The existing `transcript()` entry projection stays
source-compatible; no caller must parse hidden marker strings.

### Compatibility and migration

- TLOG schema version remains 1. Existing Success commits and outcome markers remain valid.
- Adding `turn_id` metadata to partial entries and persisting existing Error/Cancelled marker values
  is additive; no file rewrite is required.
- Existing marker-only Error/Cancelled turns remain valid zero-entry terminal records.
- Legacy unbound partial entries are not retroactively assigned to a turn or used to infer an
  outcome.
- Existing successful-turn behavior, entry IDs, redaction, resume, fork and `EntriesCommitted`
  ordering remain unchanged.
- Public additions remain pre-1.0. Removing compatibility methods or changing serialized public
  DTOs requires a separate migration decision.

## Required Race Semantics

| Race | Required durable result |
|---|---|
| Success finishes before cancellation wins | One Success record and the complete admitted turn |
| Cancellation wins before Success finalization | One Cancelled record and the latest closed prefix |
| Provider Error after completed tool output | One Error record containing that closed tool exchange |
| Duplicate same-outcome finalization | Original IDs, no duplicate entries or marker |
| Conflicting outcome/payload retry | Structured conflict, original record unchanged |
| No persistable fact before cancellation | Hidden Cancelled marker only; visible transcript unchanged |
| Atomic replacement failure | No terminal claim; retry may safely attempt the same finalization |

## Consequences

- SESSION-008-B can implement one durable contract instead of adding a second partial-turn format.
- RUNTIME-005 can treat active-turn finalization as one bounded stage and report its real outcome.
- Restarted model context includes only stable, policy-admitted facts and cannot silently forget a
  completed projected tool result.
- Persisting interrupted prefixes increases durable data compared with ADR-042's original
  success-only rule; the existing redaction and projection boundary therefore remains mandatory.
- Cancellation needs a session-owned stable-prefix handoff rather than immediate task abortion with
  no snapshot.

## Validation Gate

Before this ADR becomes Accepted, review must verify the current-path inventory and the proposed
public compatibility shape. SESSION-008-B must then prove:

- atomic/idempotent Success, Error and Cancelled fixtures, including conflicting retries;
- cancellation after a completed tool result and cancellation before any stable fact;
- filtering of reasoning, credentials, raw/private tool data and unfinished assistant text;
- identical transcript and model-context reconstruction from the admitted prefix;
- legacy TLOG and marker-only compatibility;
- successful-turn, resume, fork, pending-journal and workspace regression suites.

## Reversal Trigger

Revisit if a stable-prefix snapshot cannot be published without duplicating the agent's canonical
message projection, or if measured atomic full-file replacement becomes materially unsafe for
partial turns. Any replacement must preserve one actor-owned finalizer, explicit terminal markers,
first-writer conflict detection and the same privacy boundary.

## Related

- ADR-039: Runtime Event Semantic Single-Flow Boundary
- ADR-042: Embedded Durable Runtime Session Boundary
- ADR-056: Transactional Steering Submission And Turn Ownership Boundary
- SESSION-008 / Issue #45
- RUNTIME-005 / Issue #49
