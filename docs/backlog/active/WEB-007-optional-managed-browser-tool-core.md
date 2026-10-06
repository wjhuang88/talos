# WEB-007: Optional Host-Executed Managed Browser Tool Core

| Field | Value |
|---|---|
| Story ID | WEB-007 |
| Type | Architecture / API / Security Intake |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Source | GitHub Issue #452 |
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
| Source Issue | #452 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Resolve intake, relationship to WEB-005, public API/schema and security boundary before selecting an implementation iteration. |

## Intake Boundary

Issue #452 proposes an optional, unregistered-by-default native browser-tool core backed by a
host-supplied executor. Talos owns contract admission, permission integration and safe projection.
Hosts/executors retain credentials, browser lifecycle, process and site-policy authority; the
adapter never retries. This owner records intake only; it authorizes no implementation,
dependency, release, publication, or default-profile change.

The relationship with WEB-005, exact crate/feature boundary, request schema, safe projections,
executor failure isolation, and downstream consumer validation remain unresolved intake decisions.
Any implementation requires a separately selected runnable iteration and an effective
Collaboration Claim.

## Required Reads

- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/backlog/active/WEB-005-browser-session-continuity-research.md`
- `docs/backlog/active/TOOL-014-conditional-tool-backends.md`
- `docs/sop/AGENT-COLLABORATION.md`

## Status Reconciliation

This owner was created solely to reconcile open Issue #452 with the project owner matrix. It does
not supersede WEB-005 or reserve an implementation owner.

## Frame-Aware Child Intake

[WEB-007-F](WEB-007-F-frame-aware-contract.md) owns #618 / #520 Request B frame identity,
origin permission, stale-reference boundaries and shared acceptance fixtures. I295 proposes an
Active / Claimed state in #669, ineffective until merge; ADR-086 remains Proposed until governance
acceptance. Process
carrier and native-browser delivery must consume the accepted contract rather than invent their
own frame authorization. Parent #452 V1 is not silently expanded by this child.
