# DEPENDENCY-001: Dependency Upgrade Governance

| Field | Value |
|---|---|
| Story ID | DEPENDENCY-001 |
| Type | Governance / Architecture Intake |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Source | [GitHub Issue #474](https://github.com/wjhuang88/talos/issues/474) |
| Selected Iteration | None |
| Depends On | Existing release, testing, and collaboration SOPs |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Requirement intake only; no dependency or Cargo changes |
| Claimed At | Not applicable |
| Source Issue | #474 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-02 |
| Handoff / Release Condition | Define and accept a bounded audit/baseline/SOP slice before implementation. |

## Scope

Capture the need for an accepted dependency baseline, cross-platform audit, and repeatable upgrade
procedure. This intake does not authorize changing manifests, lockfiles, versions, or release policy.

## Residual Destination

Audit scripts, baseline generation, and upgrade execution require separately selected iterations and
effective claims.

## 2026-09-06 Recovery Checkpoint

Remote `main@e336e438208eebca95576db9fbf245783651d0c1` still has no effective claim for #474.
Local I247 Direct commit / Active assertions and placeholder implementation are not authorization
or acceptance evidence. Keep this owner Intake / Unclaimed and unselected until a reviewed atomic
claim reaches the target branch. The full requirement, rejected evidence, technical-readiness
checks and recovery sequence are recorded in
[the recovery ledger](../../tasks/2026-09-06-dependency-governance-recovery.md).
