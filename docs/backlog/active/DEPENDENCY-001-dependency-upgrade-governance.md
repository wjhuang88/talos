# DEPENDENCY-001: Dependency Upgrade Governance

> Document status: Complete / Closed

| Field | Value |
|---|---|
| Story ID | DEPENDENCY-001 |
| Type | Technical Governance Story |
| Priority | P1 |
| Status | Complete |
| Source | [GitHub Issue #474](https://github.com/wjhuang88/talos/issues/474) |
| Selected Iteration | I248 |
| Depends On | Existing release, testing, and collaboration SOPs |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session |
| Work Slice | Full #474 baseline, cross-platform audit, SOP and live handoff; no dependency upgrades |
| Claimed At | 2026-09-06 |
| Source Issue | #474 |
| Governance Claim PR | #496 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | #496 merged as `7c3f6421a073aad62e6a4aaf8f6e1c7df9a674bd`; claim effective. Actual merge is not a substitute for missing historical CAS/review evidence or new implementation gates. |
| Implementation PR | #497; acceptance follow-up #498 |
| Last Updated | 2026-09-07 |
| Handoff / Release Condition | #498 native Windows acceptance passed; full upgrade remains separately owned by DEPENDENCY-002 / I250 |
| Completion Commit | `dbd847ec5092d8097d985582c3289692745cf681` |

## Scope

Deliver the accepted dependency baseline, Bash/PowerShell audit and generic upgrade procedure
specified in #474 through I248. The proposed scope includes deterministic and live evidence,
candidate handoff and documentation. It excludes dependency upgrades, manifests, lockfiles,
versions, public APIs, new helper runtimes and release execution. Claim and activation became
effective when #496 merged into main as `7c3f6421`.

## Residual Destination

Audit scripts and baseline generation belong to I248's proposed claim. Concrete dependency
upgrades and workspace dependency centralization remain separately governed follow-ups; neither is
required or authorized merely to close this mechanism Story.

## Completion Evidence

- Completion Commit: `dbd847ec5092d8097d985582c3289692745cf681`
- Implementation PR #497 merged as `7b4e21ce6514cc4ce7e79e0a2b2491ffe497be27` after exact-head CI
  `34043207621` and independent review `5563452959`.

## 2026-09-06 Recovery Checkpoint

The pre-merge recovery note above is historical. After #496 merged as `7c3f6421`, I248 is the
effective claimed iteration. The full requirement, rejected evidence, technical-readiness checks
and recovery sequence are recorded in
[the recovery ledger](../../tasks/2026-09-06-dependency-governance-recovery.md).

## I248 Selection Proposal

I248 is the effective iteration for the full #474 outcome. Implementation starts from
`main@7c3f6421` and remains within its Work Slice.

The requested first real upgrade is tracked separately by planned I249. It must consume a fresh
I248 live report and may select exactly one candidate; it does not widen I248 or authorize a batch.
See [I248](../../iterations/I248-dependency-governance-closed-loop.md).

## Required Reads And Acceptance Owner

Current scope clarification (2026-09-06): the user's full-upgrade request is a long-task outcome
beyond the mechanism, not satisfied by I249's one-package pilot. Preserve I249's published baseline;
the fresh full audit must feed a full-upgrade owner and appropriately isolated slices before Cargo
changes. I248 is Complete/Closed after #498 native Windows acceptance. The full-upgrade outcome
remains owned by I250.

Read #474 in full, I248, the recovery ledger, root and member Cargo manifests, Cargo.lock,
rust-toolchain.toml and REQUIREMENT-INTAKE, START-ITERATION, ITERATION-WORKFLOW,
AGENT-COLLABORATION, TESTING, CHANGE-CONTROL, DOC-CHECK and RELEASE SOPs before implementation.
I248's preserved Published Baseline and appended readiness matrix carry the executable acceptance
contract; #474's complete checklist remains mandatory. Completion of a static example is not
completion of the Story. A maintainer running the actual audit frontends is the receiving user.

## 2026-09-07 Final Acceptance

Completion Commit: `dbd847ec5092d8097d985582c3289692745cf681`, `5d2e444308d395171e821d103b35d4ce28756b88`.
#498 merge `a7d168ee` contains the missing platform validation: CI `34074942674` passed the
native Windows auditor and Unix parity suite; independent review `5564114577` approved the
exact head. I248's appended final acceptance checkpoint is the detailed evidence owner.
