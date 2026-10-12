# Iteration I301: Missing Pending-Sidecar Startup Recovery

> Document status: Planned
> Published plan date: 2026-10-12
> Planned objective: eliminate database initialization during recovery of an absent pending sidecar.
> MVP deliverable: new/legacy Runtime admits commands without a recovery-only SQLite side effect.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Missing pending-sidecar recovery read fast path, storage/Actor regressions and startup documentation only; no existing-store durability or custody change. |
| Claimed At | 2026-10-12 |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer repeatedly authorized continuing the intermittent-timeout investigation and repair through GitHub CI. No separate maintainer reviewer is available; independent Agent storage/API review, exact-head CI, both validators and merge-time CAS remain required. |
| Implementation PR | Not started |
| Last Updated | 2026-10-12 |
| Handoff / Release Condition | No implementation before this atomic claim and activation merges into main; release only after evidence-bearing closeout or explicit handoff. |

## Published Baseline

Selected Story: [RUNTIME-010](../backlog/active/RUNTIME-010-missing-pending-sidecar-startup.md),
Ready; no parent. Preserve all scope, exclusions, Given/When/Then acceptance, validation and
documentation targets in that owner. Accepted ADR-008/056/058 and this claim merge are dependencies.
The final Active/Claimed proposal becomes effective only when merged into main.

### Selection Inventory And Dispositions

Selection baseline: main `f036111f6e969feea47ad784f3cb3effd7e20a33` (includes #708).
All iteration document headers were inventoried on 2026-10-12:

| Owner | Current state | Disposition |
|---|---|---|
| I293 | Active / Claimed | Retain other-session Search router scope; no transfer or overlap. |
| I294 | Active / Claimed | Retain other-session Search provider evidence; no transfer or overlap. |
| I300 | Active / Claimed (#704 merged) | Retain T7 Git permission/diff repair; no overlap in behavior or implementation files. |
| I277 | Review | Retain deferred human/device acceptance. |
| I249 | Planned | Retain dependency-pilot deferral; no dependency version change here. |
| I164 | Paused | Retain superseded layout target; no reactivation. |
| DEPENDENCY-003-A | Review / Claimed | Retain separately owned native dependency evidence and platform gates. |

No Blocked iteration header was found. I290/I291/#590 and I296-I299 remain terminal.
Existing explicit non-overlap authorization plus the maintainer's continued direction governs this
independent Runtime repair; this record does not activate another owner's work.
Only open PR found at selection was #705, whose bounded diagnostic exception excludes production
storage changes. RUNTIME-008/009 and SESSION-011 are unclaimed future semantic/API work; this repair
preserves their contracts and does not claim them. Recheck inventory/overlap before merge.

### Risks And Rollback

Filesystem absence must preserve real I/O errors; concurrent creation cannot turn a read snapshot
into custody authority. Existing journal schema validation and WAL/FULL remain unchanged. Revert only
this repair if regression evidence shows loss, hidden error or altered Actor recovery; retain all
historical failing CI evidence. Windows runner timing is variable and cannot prove universal latency.

## Actual Activation And Execution

2026-10-12: governance-only draft prepared; no production/test/dependency edit or implementation
branch. Official Rust 1.99.0 toolchain is available locally. Final PR number, exact-head checks,
independent review and merge-time CAS must be recorded before claim becomes effective.

## Verification Evidence / Completion Evidence

Pending governance checks and implementation. No Completion Commit exists; not Complete.

## Variance And Residuals

#705 retains PowerShell latency, old-failure attribution and existing-store startup cost. This
iteration does not claim the entire intermittent-timeout investigation is closed.
