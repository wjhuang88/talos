# DIST-001-A: Verified Manual Bundle Installation

| Field | Value |
|---|---|
| Story ID | DIST-001-A |
| Type | Distribution implementation |
| Parent | CAP-001 / #466; DIST-001 |
| Status | Planned / Claimed (pending governance merge) |
| Selected Iteration | I260 |
| Source Issue | [GitHub Issue #509](https://github.com/wjhuang88/talos/issues/509) |
| Depends On | BUNDLE-001; CAP-001-C |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | Explicit local/manual Bundle installation with verification and rollback. |
| Claimed At | 2026-09-11 |
| Authorization Evidence | Proposed claim is ineffective until PR #540 merges to main; implementation is not authorized. |
| Governance Claim PR | #540 |
| Implementation PR | Not started |
| Authorization Mode | Independent review |
| Last Updated | 2026-09-11 |
| Handoff / Release Condition | Requires PR #540 merge, then a fresh implementation branch and independent security review. |

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md), [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), and [BUNDLE-001](BUNDLE-001-bundle-manifest-installation-identity.md).
- [DIST-001 policy](DIST-001-optional-runtime-asset-distribution.md); installation must remain distinct from activation and permission grant.

## Goal And Scope

Install a user-selected Bundle from a local or manually supplied artifact after integrity and
compatibility checks. Installation remains separate from Plugin activation and permission grant.

## Non-Goals

No online discovery/download, marketplace, silent executable install, automatic activation or
permission bypass.

## Acceptance

- Bundle identity, integrity, compatibility, destination and rollback are deterministic.
- Invalid, unknown or partial artifacts fail closed without corrupting installed state.
- Installation does not load, activate or register executable Plugins automatically.

## Validation And Documentation

Offline artifact fixtures, corruption/rollback tests, platform path checks, security review and
user-facing installation documentation.
