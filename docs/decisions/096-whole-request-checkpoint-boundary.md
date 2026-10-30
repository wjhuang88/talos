# ADR-096: Whole-Request Checkpoint And Manual Operation Boundary

## Status

Accepted as the proposed I312 / handoff T4 contract in #817. Ineffective until
finalized atomic activation merges. This is no authorization to implement and does not close #390 or T5.

## Context and constraints

Current history-only compaction mutates Actor history before the sealed request
budget exists. The summarizer accepts text without checking a successful terminal
and can substitute an empty-summary placeholder. MEM-005 requires whole-request
policy and manual controls; #390 refines semantic retention into checkpoints.

Hard: raw TLOG/legacy JSONL remains authoritative; permissions/privacy, complete
exchange pairing, stable prompt prefix and exact request admission remain intact.
Hard: public Rust enum expansion needs migration, independent review and release
compatibility evidence. Soft: initial thresholds/bounds in the linked design may
be tuned using measured fixtures without weakening accepted semantics.

## Decision proposed

Use the existing request-plan estimate/admission authority for all ordinary and
summarizer requests. Centralize Normal/Advisory/Critical/Emergency policy; perform
checkpoint work only at complete-exchange boundaries. Raw history is never
replaced by checkpoint text. Keep versioned derived projection and lineage in
memory; reconstruct from authoritative history on resume/fork rather than adding
a new durable store. Model changes invalidate stale projection candidates.

One checkpoint schema, renderer and bounded sequential folding pipeline serves
automatic/manual T4 and later model-directed T5. Validate every intermediate result
and exact final request, then apply atomically. Errors/cancellation keep the old
projection. Content-free events report results without revealing hidden payloads.
An explicit Agent checkpoint projection strips private arguments/reasoning and
rejects unverifiable recovered tool payloads; existing persistence sanitization
alone is insufficient. No claim of general credential discovery is made.

See [the proposed design](../proposals/t4-whole-request-compaction-design.md) for
schema fields, byte/token/reference limits, deadlines, receipts, cutoffs and tests.

## Public API and migration

Add a typed compact operation to SessionOp carrying generation and operation ID;
use typed transient outcomes under non-exhaustive SessionEvent. Existing operation
wire tags retain behavior. Adding a SessionOp variant breaks exhaustive Rust
matches: every in-repository bridge/adapter and both independent Runtime SDK
fixtures must be updated to handle the new variant explicitly. Do not silently
ignore the operation in an adapter that advertises manual support.

Ship only in the next pre-1.0 minor release, never as an existing-minor patch.
Consumers must add the typed compact handling or explicit unsupported response to
exhaustive matches, rebuild against the new minor, and preserve target-generation
checks. Update Runtime SDK migration documentation before merge. No release/tag
is part of this iteration. A later non-exhaustive API migration is separate scope.

## Alternatives and consequences

History-only trimming does not budget prompt/schema/reservations and cannot meet
semantic acceptance. Replacing durable history destroys recovery provenance.
Another summarizer or transcript database introduces competing authority. An
unbounded single summary call cannot handle text-heavy restart safely.

Derived state may be recomputed differently after restart; raw facts and valid
projection, not identical generated prose, are guaranteed. Generated checkpoints
remain fallible interpretations. Recent verbatim history and source references
mitigate loss; T5 separately delivers bounded model-driven historical recovery.

## Verification and rollback

Prove whole-request dispatch captures, structured validation, privacy, 50-turn
heavy-tool and text-heavy restart behavior, cancellation/atomicity, deduplication,
manual runtime reachability and raw export/replay preservation. Independent API
and privacy review, locked full preflight including external SDKs, exact-head
platform CI and CAS remain required. Rollback discards derived projection state
without deleting raw records or changing permission policy.
