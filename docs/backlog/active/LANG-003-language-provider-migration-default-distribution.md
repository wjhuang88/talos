# LANG-003: Remaining Language Migration And Default Distribution Policy

| Field | Value |
|---|---|
| Story ID | LANG-003 |
| Type | Language Provider migration policy |
| Parent | CAP-001 / #466 |
| Status | Refinement / Unclaimed |
| Selected Iteration | None |
| Source Issue | [GitHub Issue #517](https://github.com/wjhuang88/talos/issues/517) |
| Depends On | LANG-002; BUNDLE-001; DIST-001-A |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Remaining language migration and explicit default distribution decision. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Requires a separate distribution and compatibility decision before changing defaults. |

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md), [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), and [LANG-002](LANG-002-rust-language-provider-vertical-slice.md).
- [BUNDLE-001](BUNDLE-001-bundle-manifest-installation-identity.md) and [DIST-001-A](DIST-001-A-verified-manual-bundle-installation.md) before any default-distribution change.

## Goal And Scope

Migrate additional languages only after the Rust vertical slice proves the contract, and decide
which providers/assets are built in, optional or manually distributed.

## Non-Goals

No implicit network download, unreviewed Cargo feature inversion, parser deletion based only on size,
or Desktop/Browser implementation.

## Acceptance

- Each migrated language has a provider, fallback and distribution owner.
- Default builds and user-visible behavior change only through an explicit decision.
- Size measurements distinguish static parser footprint from runtime/provider assets.

## Validation And Documentation

Per-language compatibility fixtures, release/build matrix, distribution documentation and a fresh
change-control/ADR record for any default change.
