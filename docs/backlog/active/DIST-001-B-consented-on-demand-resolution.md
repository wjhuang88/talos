# DIST-001-B: Consented On-Demand Capability Resolution

| Field | Value |
|---|---|
| Story ID | DIST-001-B |
| Type | Distribution and resolution implementation |
| Parent | CAP-001 / #466; DIST-001 |
| Status | Complete / Closed |
| Selected Iteration | I275 |
| Source Issue | [GitHub Issue #515](https://github.com/wjhuang88/talos/issues/515) |
| Depends On | DIST-001-A; CAP-001-B/C; BUNDLE-001 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | Explicitly consented on-demand Bundle resolution for a missing optional capability. |
| Claimed At | 2026-09-15 |
| Authorization Evidence | Claim PR #566 merged as 2499b9b8; implementation authorized within this Work Slice. |
| Governance Claim PR | #566 |
| Implementation PR | #567 (merged) |
| Completion Commit | 8ec6c271 |
| Authorization Mode | Independent review |
| Last Updated | 2026-09-15 |
| Handoff / Release Condition | Handoff to distribution maintenance; no release or publication authorization implied. |

## Activation Checkpoint — 2026-09-15

Claim PR #566 exact head `d5f58df8`, base `606f4d8f`, CI run `34975543686` merged to `main`
as `2499b9b8`. Implementation starts from `main@2499b9b8`; the Published Baseline and
exclusions remain unchanged.

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

## Completion Checkpoint — 2026-09-15

Completion Commit: `8ec6c271`

Implementation PR #567 merged as `8ec6c271` following exact-head CI `34984742102` and independent
security review for `d9e9c4e0`. Guarded cancellation/timeout, identity validation, rollback and
staging cleanup are covered; installed Bundles remain inactive until existing activation gates.
