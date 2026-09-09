# CAP-001-B: Capability Registry And Resolver

| Field | Value |
|---|---|
| Story ID | CAP-001-B |
| Type | Capability infrastructure implementation |
| Parent | CAP-001 / #466 |
| Status | Ready / Unclaimed |
| Selected Iteration | None |
| Source Issue | [GitHub Issue #512](https://github.com/wjhuang88/talos/issues/512) |
| Depends On | ADR-072 Accepted; CAP-001-A / I252 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Capability identity lookup, registry ownership and bounded resolver contract only. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | A fresh runnable iteration and effective claim are required before code or Cargo changes. |

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
