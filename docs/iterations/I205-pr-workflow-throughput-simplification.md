# Iteration I205: PR Workflow Throughput Simplification

> Document status: Active
> Published plan date: 2026-08-17
> Planned objective: measure recent Talos delivery overhead and select a smaller PR/review workflow
> that preserves evidence-bearing safety gates while eliminating mechanically preventable churn.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a reproducible governance audit and decision packet identifies the smallest
> separately implementable process change, with scenario tests, migration and rollback.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-18 |
| Work Slice | Execute only GOV-007/I205's evidence-only audit and decision packet: measure recent claim/implementation/closeout/review churn, classify causes, map retained gates to demonstrated failures or Hard constraints, define the ordinary/protected/release/maintenance scenario matrix, migration and rollback, and identify the smallest separately claimable implementation slice. No SOP, validator, CI workflow, branch-protection, product/runtime, release-policy, security-gate or child-activation change. |
| Claimed At | 2026-08-18 |
| Source Issue | None |
| Governance Claim PR | #284 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer requested an evidence-based PR-flow simplification and unattended continuation of the long task. Claim head `5af455930a84871042b53b7bb1de12651edcc6a7` passed CI `32046397520`, both governance validators and merge-time CAS with no blocking feedback; PR #284 merged as `fd1eaad9076bed1b110e17bade3ff0dc48040fdf`. Any executable rule change requires its own bounded claim and review. |
| Implementation PR | Pending |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | After this claim merges, activate I205 separately from the claim merge or later main and execute only the evidence/decision Spike. Establish a new bounded claim before changing SOPs, validators, workflows or branch protection. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| GOV-007 | Ready | Current collaboration, Git, DOC-CHECK and CI routing contracts | Evidence-based workflow simplification decision with a runnable follow-up slice |

### Scope And Non-Goals

The complete scope, preserved hard gates and exclusions are owned by
`docs/backlog/active/GOV-007-pr-workflow-throughput-simplification.md`. I205 is an
infrastructure-only governance Spike and claims no product behavior.

### Planned Validation

- Reproducible GitHub/API audit of recent claim, implementation, closeout and correction PRs.
- Scenario matrix covering ordinary, protected-security, release and bounded-maintenance work.
- Both governance validators and `git diff --check`.
- Exact-head CI and review required by the eventual effective claim.

### Documentation To Update

- GOV-007 and this iteration owner.
- A decision/reference report containing measured evidence and the selected target flow.
- Only the affected collaboration/Git/DOC-CHECK/CI SOPs after a later accepted implementation slice.

### Risks And Rollback

- Risk: optimizing PR count can hide owner drift or weaken independent review.
- Rollback: retain the current workflow; do not change executable governance until a separately
  reviewed implementation proves the replacement checks.

## Non-Terminal Coordination Record

- I164 remains Paused.
- I188, I189, I195 and I196 remain Planned/Claimed and unactivated.
- I197-I201 remain Planned/Unclaimed through the #227 coordination proposal.
- I205 is scheduled before the long task resumes implementation so accepted simplifications can
  reduce later ceremony; planning does not activate it.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-17 | Selection | Maintainer requested PR-flow simplification after observing that commit and review round trips exceeded the underlying code/document changes. I205 is Planned/Unclaimed and makes no rule change. |
| 2026-08-18 | Claim proposal | Governance PR #284 proposes the bounded evidence-only claim from exact `main@a9cfef02`; the claim is ineffective until merge and does not activate I205. |
| 2026-08-18 | Activation | Claim PR #284 merged as `fd1eaad9` after exact-head CI `32046397520`, both validators and merge-time CAS. I205 activates as the sole Active iteration for the evidence/decision Spike; no executable process-rule or product change is authorized. |
| 2026-08-18 | Implementation PR preparation | Audit branch `docs/i205-workflow-audit` starts exactly at activation merge `4635ef2b`; no audit artifact or executable governance change exists before the Draft PR number is assigned. |

## Claim Preparation Checkpoint - 2026-08-18

Exact target baseline: `main@a9cfef02e31027d19e297482eda0d77fffd6ce3c`.

| State | Iterations | Disposition |
|---|---|---|
| Active | None | No active work blocks the audit-only claim. |
| Review | None | I188 closed through PR #283 before this selection resumed. |
| Planned / Claimed | I189, I195, I196 | Preserve each existing owner/claim and keep unactivated; Dashboard #233 remains independently owned. |
| Planned / Unclaimed | I197-I201, I205-I208, I210 | Claim only I205's audit slice. Preserve every product/runtime child and its published dependency order. |
| Blocked | None at iteration level | Backlog/Epic blockers remain authoritative and are not bypassed. |
| Paused | I164 | Preserve its superseded state; do not resume. |

Open PRs #120/#121 remain archival Drafts and #233 remains Dashboard-owned. No open PR or effective
claim overlaps the I205 evidence-only workflow audit. The Draft claim is ineffective until its
finalized `Claimed` record merges to `main`; no audit implementation or executable process-rule
change is authorized by this checkpoint.

## Verification Evidence

- Pending effective claim and execution.

## Completion Evidence

- No completion evidence. A status-only commit cannot certify this iteration.

## Variance And Residuals

- Any accepted SOP, validator or workflow implementation uses a separately bounded Work Slice.

## Retrospective

- Pending.
