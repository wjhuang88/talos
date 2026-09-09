# DIST-001-B: Consented On-Demand Capability Resolution

| Field | Value |
|---|---|
| Story ID | DIST-001-B |
| Type | Distribution and resolution implementation |
| Parent | CAP-001 / #466; DIST-001 |
| Status | Refinement / Unclaimed |
| Selected Iteration | None |
| Source Issue | [GitHub Issue #515](https://github.com/wjhuang88/talos/issues/515) |
| Depends On | DIST-001-A; CAP-001-B/C; BUNDLE-001 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Explicitly consented on-demand Bundle resolution for a missing optional capability. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Requires manual installation evidence, consent UX/security decision and network policy owner. |

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md), [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), [DIST-001-A](DIST-001-A-verified-manual-bundle-installation.md), and [BUNDLE-001](BUNDLE-001-bundle-manifest-installation-identity.md).
- [DIST-001 policy](DIST-001-optional-runtime-asset-distribution.md); no startup network dependency or silent executable download is authorized.

## Goal And Scope

Resolve a missing optional capability only after explicit user consent, bounded network policy,
verification and separate install/activation steps.

## Non-Goals

No startup network dependency, silent executable/model download, marketplace, broad auto-approval,
or bypass of permission/sandbox policy.

## Acceptance

- User intent, request identity, consent, download, verification and rollback are auditable.
- Cancellation, timeout, network failure, stale metadata and denial fail closed.
- Installed content remains inactive until the existing activation/permission boundaries allow it.

## Validation And Documentation

Mock/offline resolution fixtures, network-failure and cancellation tests, security review, consent
documentation and exact changed-file/network authority inventory.
