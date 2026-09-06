# DEPENDENCY-001: Dependency Upgrade Governance

| Field | Value |
|---|---|
| Story ID | DEPENDENCY-001 |
| Type | Technical Governance Story |
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
| Last Updated | 2026-09-06 |
| Handoff / Release Condition | Merge #496, then local convergence before one stable implementation candidate. |

## Scope

Deliver the accepted dependency baseline, Bash/PowerShell audit and generic upgrade procedure
specified in #474 through I248. The proposed scope includes deterministic and live evidence,
candidate handoff and documentation. It excludes dependency upgrades, manifests, lockfiles,
versions, public APIs, new helper runtimes and release execution. Claim and activation remain
ineffective until #496 merges into main.

## Residual Destination

Audit scripts and baseline generation belong to I248's proposed claim. Concrete dependency
upgrades and workspace dependency centralization remain separately governed follow-ups; neither is
required or authorized merely to close this mechanism Story.

## 2026-09-06 Recovery Checkpoint

Remote `main@e336e438208eebca95576db9fbf245783651d0c1` still has no effective claim for #474.
Local I247 Direct commit / Active assertions and placeholder implementation are not authorization
or acceptance evidence. Keep this owner Intake / Unclaimed and unselected until a reviewed atomic
claim reaches the target branch. The full requirement, rejected evidence, technical-readiness
checks and recovery sequence are recorded in
[the recovery ledger](../../tasks/2026-09-06-dependency-governance-recovery.md).

## I248 Selection Proposal

I248 is the replacement candidate for the full #474 outcome. #496 proposes Active / Claimed;
the target-branch owner remains Intake / Unclaimed until that atomic record merges into `main`.
See [I248](../../iterations/I248-dependency-governance-closed-loop.md).

## Required Reads And Acceptance Owner

Read #474 in full, I248, the recovery ledger, root and member Cargo manifests, Cargo.lock,
rust-toolchain.toml and REQUIREMENT-INTAKE, START-ITERATION, ITERATION-WORKFLOW,
AGENT-COLLABORATION, TESTING, CHANGE-CONTROL, DOC-CHECK and RELEASE SOPs before implementation.
I248's preserved Published Baseline and appended readiness matrix carry the executable acceptance
contract; #474's complete checklist remains mandatory. Completion of a static example is not
completion of the Story. A maintainer running the actual audit frontends is the receiving user.
