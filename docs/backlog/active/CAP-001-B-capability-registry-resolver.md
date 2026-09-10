# CAP-001-B: Capability Registry And Resolver

| Field | Value |
|---|---|
| Story ID | CAP-001-B |
| Type | Capability infrastructure implementation |
| Parent | CAP-001 / #466 |
| Status | Review / Claimed (#524 merged; acceptance-hardening follow-up pending) |
| Selected Iteration | I254 |
| Source Issue | [GitHub Issue #512](https://github.com/wjhuang88/talos/issues/512) |
| Depends On | ADR-072 Accepted; CAP-001-A / I252 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Capability identity lookup, registry ownership and bounded resolver contract only. |
| Claimed At | 2026-09-10 |
| Authorization Evidence | Maintainer authorized single-developer unattended completion of #499 then #466. Claim #523 is effective on main at 4cd7b42e958e73235597a984b6805219ea1f2256. Shared GitHub identity does not prove human separation; each stable implementation candidate requires fresh exact-head CI and independent Agent-role API review. |
| Governance Claim PR | #523 |
| Implementation PR | #524 (merged); acceptance-hardening follow-up pending stable push |
| Authorization Mode | Single-maintainer merge |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | Implementation starts only after PR #523 reaches main; preserve CAP-001-C, Bundle, permission and product boundaries. Exact-head implementation API/security review remains required where applicable. |

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md) and [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md).
- CAP-001-A/I252 descriptor evidence; do not infer Plugin carrier or Bundle installation authority from this registry scope.

## Goal And Scope

Provide one deterministic, UI-neutral registry/resolver boundary for the Capability and Provider
descriptors delivered by I252. The resolver may select an already available Provider and report
unavailable or incompatible capability states; it must not silently install executable content.

## Non-Goals

No Plugin loader, Bundle installer, network discovery, permission-policy rewrite, language/browser
implementation, persisted manifest migration, Desktop/Dashboard work, release or publication.

## Acceptance

- Capability IDs, Provider compatibility and provenance use I252 contracts.
- Resolution is deterministic, offline at startup, cancellation/deadline bounded and fail-closed.
- Registration, lookup and resolver errors are typed and UI-neutral.
- Missing optional Providers produce a safe unavailable result without process failure.
- Tool/schema disclosure remains owned by TOOL-012/TOOL-014; registration does not expose tools.
- No existing public API or persisted field is silently renamed.

## Validation And Documentation

Focused registry/resolver tests, offline fixtures, locked affected-workspace checks, governance
validators and architecture/API documentation. A later implementation owner must record changed
files and an independent permission/security/API review where applicable.

## Residuals

Plugin/Carrier adapters remain CAP-001-C. Bundle installation remains BUNDLE-001 and DIST-001-A/B.

## Selection Preparation (2026-09-10)

[I254](../../iterations/I254-capability-registry-resolver.md) records the runnable API deliverable,
acceptance, validation and non-terminal inventory from `main@540ad257`. This is local preparation
only: the Story remains Ready / Unclaimed until the finalized atomic claim reaches main. Existing
Issue #512 remains the sole requirement tracker; local design/test/review corrections do not create
additional Issues or iterations. No Rust, Cargo, storage or runtime behavior is changed here.

## Finalized Claim Proposal (2026-09-10)

PR #523 established Active / Claimed for this owner and I254 atomically, merging to `main` as
`4cd7b42e`. Scope and acceptance are unchanged. Registration does not attest trust, grant execution
permission, install code or disclose a tool. Implementation starts from this merge or later.

## Post-Merge Acceptance Checkpoint (2026-09-10)

PR #524 merged at `d7a836dee37a003a8f36476c124ba63cbc5decf2`; I254 records its exact-head
CI/review and the subsequent locally converged acceptance hardening. This supersedes the
pre-claim/current-candidate descriptions above without changing scope. The resolver remains
library-only, with cooperative cancellation/deadline checks and panic containment for promptly
returning host callbacks, not preemption. Registration neither grants permission nor exposes tools.
Full local release preflight passed; new exact-head remote evidence is still required for the
follow-up. I254 and this owner remain Review / Claimed; #512 stays open and CAP-001-C remains
unclaimed. Completion Commit: pending final acceptance and follow-up merge.
