# CAP-001-B: Capability Registry And Resolver

| Field | Value |
|---|---|
| Story ID | CAP-001-B |
| Type | Capability infrastructure implementation |
| Parent | CAP-001 / #466 |
| Status | Active / Claimed (effective on main via PR #523 merge `4cd7b42e`) |
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
| Authorization Evidence | Maintainer authorized single-developer unattended completion of #499 then #466. No independent human reviewer is available; shared GitHub identity does not prove human separation. PR #523 requires exact-head CI, both validators and merge-time CAS; proposed claim and activation remain ineffective until merge. |
| Governance Claim PR | #523 |
| Implementation PR | Not started |
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
