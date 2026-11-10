# ADR-102: Operation-oriented approval presentation

Status: Proposed; local candidate, independent review and merge pending
Date: 2026-11-10
Owner: [TUI-APPROVAL-001](../tasks/2026-11-10-approval-scope-display.md)

## Decision

Separate a trusted in-process operation presentation from the compiler's exact
permission evidence. Native Git Checkout displays operation, repository, target,
current inspection/execution stage and effects; the complete facet text remains
available under details. The first Read Path is the root by the native Checkout
producer's contract. Both provenance and known stage capabilities must match;
foreign tools and legacy producers retain the exact-scope surface.

This presentation restricts decisions to ApproveOnce/Deny. The numeric 2 alias is
labelled single-use and maps to ApproveOnce, never a hidden reusable grant. There
is no need to page every evidence row before this single-use decision. Existing
non-operation session approval still requires the exact scope to be displayed.
Separate inspection and execution admissions are retained; approving inspection
is explicitly not final checkout authority. No combined capability, repository
wildcard grant, deadline extension, or skipped stale-source validation is added.
Source-set minimization for branch creation remains separate security-sensitive
work; this change does not claim to eliminate the conservative source capture.

## API and migration

Add `ApprovalPresentation { summary, once_only }` and an optional `presentation`
field to `UiOutput::ToolApprovalRequest`. This is a Rust in-process UI value, not
a serialized configuration/protocol or durable wire format. It follows other
nonserialized UiOutput payloads; no dependency or storage migration is required.
The new field is source-breaking for exhaustive event construction/destructuring.
Workspace constructors migrate atomically: default to None, adapters with trusted
operation metadata supply Some, consumers bind the field or use `..`. External
pre-1.0 consumers must make the same change before building against the next
compatible product release; publish this migration with that release, not as an
unannounced semver-compatible API claim. Legacy None preserves prior behavior.

## Validation and limits

Test thousands of facets without dropping or broadening exact scope, native versus
foreign provenance, inspection versus execution, single-use numeric choices from
summary, details navigation and legacy reusable approval. Exact-head independent
review/CI and real-terminal acceptance remain required before delivery.
