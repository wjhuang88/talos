# ADR-096: Whole-Request Checkpoint And Manual Operation Boundary

## Status

Accepted I312 / handoff T4 contract: #817 merged at
`02fee3cc9537e6bf688d167036389b4af3011aeb`. The claim is effective;
this decision does not establish implementation completion or close #390 or T5.

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

The legacy public `Compactor::{compact, apply_collapse, apply_autocompact,
manual_compact}` signatures have only messages and a provider reference. They
cannot establish complete request budgets, tool disclosure authority or raw-source
lineage. Retire their independent summarizer: retain the signatures with explicit
deprecation, but fail closed before preprocessing or provider invocation.
`compact`, `apply_collapse` and `apply_autocompact` always return
`CompactionError::CompactionFailed` with migration guidance; `manual_compact`
returns the exact original messages and `CompactionStatus::Failed`, including for
empty or below-threshold inputs. Do not forge authority from untrusted history.

This preserves call-site signatures, **not behavioral compatibility**. Ship only
in the same next pre-1.0 minor release. Callers migrate live work to
`RuntimeHandle::compact(operation_id)` or `SessionOp::Compact` and consume the
typed terminal receipt. The Agent's single checkpoint pipeline supplies automatic
and manual semantics; no legacy provider fallback remains. Pure deterministic
utilities remain explicit caller-owned transforms, not session admission, privacy
authorization or permission to replace raw history. Shared bounded summary stream
decoding stays available to the checkpoint pipeline.

Add a typed compact operation to SessionOp carrying generation and operation ID;
use typed transient outcomes under non-exhaustive SessionEvent. Existing operation
wire tags retain behavior. Adding a SessionOp variant breaks exhaustive Rust
matches: every in-repository bridge/adapter and both independent Runtime SDK
fixtures must be updated to handle the new variant explicitly. Do not silently
ignore the operation in an adapter that advertises manual support.

Automatic checkpoint feedback uses additive `SessionEvent::AutomaticCompactStatus`
with typed bounded state/failure categories and numeric estimates, emitted directly
to Session consumers rather than durable Agent transcript events. It may precede
ordinary turn admission; consumers must not treat it as a turn terminal or a manual
operation receipt. Initial and continuation attempts share this observer and the
Runtime interrupt/shutdown commit gate. UI adapters discard old generations.

Manual `CompactOutcome::FailedWithEstimate` carries the already measured whole
request estimate and a bounded `CompactFailureReason` (preparation, provider,
timeout or commit rejection). Consumers must handle this additional variant in
the same next-minor migration. Failures whose estimate was not obtained or could
not be propagated retain `Failed` and display an unavailable estimate, never a
fabricated zero. Outer preparation timeouts preserve an already measured estimate;
task join or response loss may still lose it. Neither outcome carries
provider strings or checkpoint content; prior context remains unchanged.

Manual `CancelledWithEstimate` preserves a measured request count across idle or
tool-boundary cancellation. Per-operation measurement is reset on admission and
terminal receipt replay retains the original value. Cancellation before measurement
uses `Cancelled`; consumers handle the new variant in the next-minor migration.

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

## I313 bounded recovery source opening — 2026-11-01 implementation amendment

Recovery reads must not block opening a special file before cancellation checks.
The Session bounded-page reader uses the repository's existing platform strategy:
Unix `O_NONBLOCK | O_NOFOLLOW` followed by opened-handle regular-file validation;
Windows local absolute drive paths only, `FILE_FLAG_OPEN_REPARSE_POINT`, and
opened-handle regular-file/non-reparse validation. Device/pipe/network namespaces
are unsupported; parent-directory identity and source lineage remain host-owned
checks, not guarantees of this raw reader. Filesystem/kernel operations are still
synchronous, with cooperative cancellation between bounded reads, not hard-real-
time I/O deadlines. No model may choose the filesystem path.

This adds Session's direct target-specific use of already-locked `libc` and
`windows-sys` constants, matching the existing tools recovery implementation.
No package versions change, no new native implementation or Talos-authored unsafe
is introduced. It refines the original no-new-dependency assumption solely for
safe OS opening; independent platform/privacy review and native CI are required.

## I313 recovery provenance storage increment — 2026-11-01

The existing atomic terminal marker JSON may carry an optional `history_recovery`
array. Its entries bind format version, original entry/turn/call/tool identity and
SHA-256 digests of the final stored representation and eligible model projection.
The public terminal outcome struct and SessionEntry fields are unchanged; legacy
readers ignore the additive JSON field. No second transcript or cleartext result
copy is created by this increment. Conversation exports omit internal terminal
markers while preserving original conversation entries.

The additive `finalize_turn_with_recovery` API accepts runtime-owner projections,
then binds only exact matches to the final PersistencePolicy-filtered Tool text.
Omitted or redacted mismatches receive no recovery receipt. Storage cannot infer
runtime provenance from text/tool names in an existing transcript. Retry requires
the same filtered entries and receipt set; disagreement fails without rewriting.
An unsupported receipt version or source digest mismatch fails closed. These are
runtime provenance records, not signatures protecting against arbitrary host log
editing, and do not authorize session/branch membership.

This first storage increment deliberately does not recover a model projection
that differs from the stored tool text. Such cases require an explicitly permitted
projection representation and the execution-owner envelope before T5 acceptance;
they are not silently treated as delivered. Actor admission, snapshot lineage,
index maintenance and model-facing query remain separate unfinished integration.

### I313 explicit tool recovery view

A tool whose model and persistence views differ may explicitly implement the
additive, default-refusing AgentTool recovery_projection hook. Execution-owner
provenance remains mandatory. The current path accepts this view only after
private-token filtering and equality with the tool's permitted persistence view;
Session persistence denial and sensitive-line redaction still prevent receipts.
Read opts into its existing numbered, snapshot/anchor-free persistence view.
CLI/Runtime permission wrappers forward the hook without granting execution or
persistence permission. Callback panic refuses recovery. Durable receipt binding
permits the existing line-ending normalization only and hashes final stored text.

This enables recovery of sanitized Read results without retaining transient edit
capabilities. It does not implement a separate permitted recovery representation
whose text differs from the persisted tool entry; that acceptance remains open.
