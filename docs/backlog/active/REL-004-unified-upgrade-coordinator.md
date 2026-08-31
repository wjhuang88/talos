# REL-004: Unified Talos Upgrade Coordinator

| Field | Value |
|---|---|
| Story ID | REL-004 |
| Type | Release / CLI Architecture Intake |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Source | GitHub Issue #453 |
| Selected Iteration | None |
| Implementation PR | Not started |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #453 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Resolve upgrade architecture and release-contract relationship before selecting a runnable implementation iteration. |

## Intake Boundary

Issue #453 proposes a unified `talos upgrade` coordinator using a detached helper transaction across
supported operating systems and installation methods. This owner records intake only. It does not
authorize CLI changes, release/tag/publication changes, helper processes, package-manager commands,
or permission/persistence behavior.

REL-003/v0.8.0 publication remains closed and is not reopened by this intake. Any implementation
must reuse the existing release contract, preserve package-manager ownership, and receive a
separately selected runnable iteration with an effective Collaboration Claim.

## Required Reads

- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/RELEASE.md`
- `docs/sop/RELEASE-WORKFLOW.md`
- `docs/backlog/active/REL-003-v080-github-and-crates-publication.md`

## Status Reconciliation

This owner was created solely to reconcile open Issue #453 with the project owner matrix. It does
not supersede REL-003 or reserve implementation authority.
