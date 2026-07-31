# SOP: Requirement Intake

## Purpose

Define how new features and changes enter the project. Every governed implementation must be a
concrete, verifiable Story before implementation begins.

Bounded single-PR maintenance and emergency work follow the exceptions in
`docs/sop/AGENT-COLLABORATION.md`.

## Process

### 1. Receive Request

When a feature or change is requested:

1. Clarify the intent and problem.
2. Identify affected crates and layers.
3. Classify the scope:
   - **Story** — single, testable unit of work;
   - **Epic** — large outcome requiring child Stories;
   - **Spike** — research to validate an assumption.
4. Check whether an existing Issue, Story, iteration, task, ADR, or PR already owns the scope.

### 2. Check Readiness

A Story is ready to implement when:

- [ ] It identifies the user, caller, maintainer, or operator receiving the result.
- [ ] It states goal, value, scope, and explicit exclusions.
- [ ] Behavior work has Given/When/Then acceptance; technical work has equivalent command or
      structural evidence.
- [ ] Affected crates and dependencies are identified.
- [ ] No blocking assumptions remain unvalidated.
- [ ] Governing ADRs/specs are linked with implementation constraints and acceptance impact.
- [ ] Minimum validation, state/status owners, residual destination, and user-facing documentation
      are identified.
- [ ] Mandatory implementation context appears under `Required Reads`.
- [ ] It fits current iteration scope or is explicitly deferred.
- [ ] Before activation, an effective Collaboration Claim identifies Responsible Actor, Work Slice,
      claim PR/commit, authorization, and date.

An Epic is ready when:

- [ ] Overall outcome, boundary, major risks, and completion condition are explicit.
- [ ] Child Stories have stable IDs and acceptance criteria.
- [ ] Dependencies between children are mapped.
- [ ] At least the first child Story is ready.
- [ ] Parent and children link each other.

### 3. Create Or Update Backlog Entry

Add a compact row to `docs/backlog/PRODUCT-BACKLOG.md` and executable detail in
`docs/backlog/active/<ID>-<slug>.md`. The compact row links mandatory Story/ADR/spec material under
`Required Reads`.

Use this owner shape:

```markdown
# <ID>: <Title>

**Status**: Refinement | Ready | In Progress | Review | Done | Blocked
**Type**: Product/API/State Story | Technical Story | Governance Story | Spike
**Parent Epic**: <ID or None>

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed | Claimed | Handoff Pending | Released | Closed |
| Responsible Actor | `@github-user` or Not assigned |
| Executing Agent | `<agent/model or session label>` or Not assigned |
| Work Slice | `<bounded, non-overlapping scope>` or Not assigned |
| Claimed At | `YYYY-MM-DD` or Not applicable |
| Source Issue | `#123` or None |
| Governance Claim PR | `#456`, `Direct commit <SHA>`, or Not applicable |
| Authorization Mode | Independent review | Single-maintainer merge | Direct commit | Emergency override | Not applicable |
| Authorization Evidence | `<review/check/reason/incident>` or Not applicable |
| Implementation PR | `#789`, Not started, or None |
| Last Updated | `YYYY-MM-DD` |
| Handoff / Release Condition | `<condition>` or None |

## Identity / Goal / Value

...

## Scope

...

## Exclusions

...

## Dependencies

...

## Decision Links And Constraints

...

## Uncertainty And Validation Path

...

## State/Status Owners

...

## User-Facing Documentation

...

## Required Reads

...

## Acceptance For Behavior

- Given <precondition>
  When <actor action>
  Then <observable result>

## Acceptance For Technical/Governance Work

- [ ] <command or check> proves <result>.
- [ ] <owner status> is synchronized.
- [ ] <residual or exception> is recorded.
```

`Claim Pending` is never persisted. A Draft claim PR may temporarily leave the owner Unclaimed with
`Governance Claim PR: Pending`; after the PR number is assigned, finalize the proposed record as
Claimed with the actual `#NN` before review.

Parallel non-overlapping work uses separate child owner documents, not multiple active claim records
inside one Story.

### 4. Route

- If work is within current iteration scope, establish/verify the effective claim, then follow
  `ITERATION-WORKFLOW.md`.
- If work is new scope, follow `CHANGE-CONTROL.md` before claiming implementation.
- If work is not planned, leave it in backlog as Unclaimed.
- If it meets bounded-maintenance or emergency criteria, follow `AGENT-COLLABORATION.md`.

## Rules

- No governed implementation without an owner entry and effective claim.
- No Epic without at least one defined child Story.
- Iterations select ready child Stories, not Epic parents.
- Observable behavior is incomplete until user documentation is updated or a residual is owned.
- ADR-constrained work is not Ready until decision constraints appear in acceptance.
- Spike results must become a decision or proposal before implementation.
- Owner state is updated before backlog index, Board, and Issue.
