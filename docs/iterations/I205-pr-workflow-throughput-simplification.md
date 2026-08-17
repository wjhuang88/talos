# Iteration I205: PR Workflow Throughput Simplification

> Document status: Planned
> Published plan date: 2026-08-17
> Planned objective: measure recent Talos delivery overhead and select a smaller PR/review workflow
> that preserves evidence-bearing safety gates while eliminating mechanically preventable churn.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a reproducible governance audit and decision packet identifies the smallest
> separately implementable process change, with scenario tests, migration and rollback.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Establish an effective claim from current main before audit implementation or process-rule edits. |

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

## Verification Evidence

- Pending effective claim and execution.

## Completion Evidence

- No completion evidence. A status-only commit cannot certify this iteration.

## Variance And Residuals

- Any accepted SOP, validator or workflow implementation uses a separately bounded Work Slice.

## Retrospective

- Pending.
