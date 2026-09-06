# DEPENDENCY-001: Dependency Upgrade Governance

| Field | Value |
|---|---|
| Story ID | DEPENDENCY-001 |
| Type | Governance / Architecture Intake |
| Priority | P1 |
| Status | In Progress / Claimed |
| Source | [GitHub Issue #474](https://github.com/wjhuang88/talos/issues/474) |
| Selected Iteration | I248 |
| Depends On | Existing release, testing, and collaboration SOPs |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session |
| Work Slice | Full #474 baseline, cross-platform audit, SOP and live handoff; no dependency upgrades |
| Claimed At | 2026-09-06 |
| Source Issue | #474 |
| Governance Claim PR | #496 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Draft governance PR #496; claim and activation proposed, ineffective until merge; independent review pending |
| Implementation PR | Not started |
| Last Updated | 2026-09-02 |
| Handoff / Release Condition | Merge #496, then local convergence before one stable implementation candidate. |

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

## I248 Selection Proposal

I248 is the replacement candidate for the full #474 outcome. It remains `Planned / Unclaimed` until
the non-terminal inventory is recorded and an atomic claim+activation record reaches target `main`.
See [I248](../../iterations/I248-dependency-governance-closed-loop.md).
