# SOP: Agent Collaboration And Task Claiming

## Purpose

Define how human-operated and autonomous agents claim, execute, hand off, and close Talos work
without duplicating effort or allowing GitHub Issues, owner documents, the Board, and
implementation state to drift apart.

Use this SOP when an agent:

- claims a GitHub Issue and converts it into a governed task item;
- claims an existing backlog Story, iteration, or task item;
- continues work handed off by another agent;
- uses branches and pull requests for implementation;
- blocks, releases, completes, cancels, or transfers claimed work.

This SOP governs collaboration ownership. It does not replace requirement intake, iteration
planning, implementation, change control, testing, or Git workflow SOPs.

## Non-Negotiable Rules

1. **Claim before implementation.** No agent may begin committed implementation work for a new
   task until the task has a merged or directly committed governance claim.
2. **Owner documents define task truth.** GitHub Issues are intake and discussion surfaces.
   `docs/BOARD.md` is a derived operating view. Scope, delivery status, ownership, blockers,
   acceptance, and completion evidence live in the relevant owner document.
3. **A pending PR is not an effective claim.** Under PR-based development, a claim becomes
   effective only after the governance claim PR is merged into the target branch.
4. **One effective claimant per task.** Multiple agents may investigate or review a task, but only
   one claimant owns implementation unless the owner document defines separate, non-overlapping
   work slices.
5. **Claim state and delivery state are different.** A task can be claimed while still `Ready`,
   `Planned`, `Blocked`, or `Review`. Do not overload delivery status to express collaboration
   ownership.
6. **Implementation and claim establishment use separate PRs.** The governance claim PR must merge
   before the implementation branch is created or implementation changes are committed or pushed.
7. **Completion requires existing implementation evidence.** A task may be marked `Complete` only
   after the implementation commit exists on the target branch and is recorded in the owner
   document as `Completion Commit: <SHA>`.
8. **Status changes flow outward from the owner document.** Update the owner document first, then
   inventories and the Board, then synchronize the originating GitHub Issue.

## Sources Of Truth

| Concern | Authoritative Location | Notes |
|---|---|---|
| Requirement discussion and external reports | GitHub Issue | Not authoritative for repository delivery status |
| Story scope and acceptance | Backlog Story owner document | Usually under `docs/backlog/active/` |
| Iteration objective and execution | Iteration owner document | Under `docs/iterations/` |
| Long-running or program execution | Task owner document | Under `docs/tasks/` |
| Current operating view | `docs/BOARD.md` | Derived from owner documents |
| Story inventory | `docs/backlog/PRODUCT-BACKLOG.md` | Must agree with Story owner documents |
| Iteration inventory | `docs/iterations/README.md` | Must agree with iteration owner documents |
| Code review and merge state | GitHub Pull Request | Does not replace owner-document status |
| Completion evidence | Existing implementation commit SHA | Must already exist before `Complete` is recorded |

When sources disagree, stop implementation and repair the owner-document chain before continuing.

## Required Preflight

Before claiming any work:

1. Refresh the target branch and inspect the current repository state.
2. Read the complete GitHub Issue, including comments, assignees, linked PRs, and closure state.
3. Search the repository for the Issue number, proposed task ID, matching scope, acceptance
   language, and existing backlog, iteration, task, proposal, or ADR owners.
4. Inspect `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, and every
   relevant Active, Review, Planned, and Blocked iteration.
5. Check open pull requests and branches for overlapping implementation.
6. Confirm dependencies, activation gates, and required decisions are satisfied.
7. Determine whether the work already has an owner, needs a new backlog Story, requires refinement,
   or duplicates or changes existing work.

Do not create a second task item merely because the existing item uses different wording.

## Collaboration Claim Record

Every claimed task must contain a collaboration record in its owner document:

```markdown
## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | `@github-user` |
| Executing Agent | `<agent/model or session label>` |
| Claimed At | `YYYY-MM-DD` |
| Source Issue | `#123` or `None` |
| Governance Claim PR | `#456` or `Direct commit <SHA>` |
| Implementation PR | `Not started` |
| Last Updated | `YYYY-MM-DD` |
| Handoff / Release Condition | `<condition or None>` |
```

Allowed claim states:

- `Unclaimed`: available for selection.
- `Claim Pending`: represented by an open governance PR but not yet effective.
- `Claimed`: effective ownership is present on the target branch.
- `Handoff Pending`: current claimant is transferring responsibility.
- `Released`: claimant stopped and the task is available again.
- `Closed`: the associated delivery work is Complete or Cancelled.

A claim does not expire automatically. A maintainer must explicitly release or transfer a stale
claim after reviewing repository and GitHub activity.

## Path A: Claim A GitHub Issue And Form A Task Item

### 1. Validate The Issue

Determine whether the Issue is:

- a defect or requirement requiring a new backlog Story;
- already represented by an existing owner document;
- a duplicate;
- insufficiently specified and requiring requirement intake;
- part of an existing iteration or program;
- blocked on an ADR, security review, dependency, or other activation gate.

### 2. Create Or Select The Owner Document

When no owner exists:

1. Assign a unique Story or task ID using repository conventions.
2. Create the owner document in the appropriate governance directory.
3. Record the source Issue, problem, scope, non-goals, dependencies, acceptance criteria,
   validation, affected user-facing documentation, and collaboration claim record.
4. Add the item to the relevant inventory.
5. Add it to the Board only when it belongs in the current operating view.

Do not create an iteration merely to express ownership. Create an iteration only when the work has
a committed runnable objective and satisfies `START-ITERATION.md`.

### 3. Synchronize The Issue

Post a claim comment containing:

```markdown
This Issue is being converted into governed task `<TASK-ID>`.

Owner document: `<path>`
Claimant: `@user` via `<agent>`
Claim mode: `<PR | direct>`
Governance claim: `<PR link, commit, or pending>`
Implementation has not started.

The claim becomes effective only after the governance update is merged or committed to the target
branch.
```

Issue assignment and labels may support visibility, but neither replaces the owner-document claim.

## Path B: Claim An Existing Task Item

An existing task may be claimed only when:

- its owner document exists and is internally consistent;
- its delivery state permits work;
- dependencies and activation gates are satisfied or explicitly remain Blocked;
- no effective claimant owns the same implementation scope;
- no open implementation PR already covers the scope;
- selecting it does not bypass unresolved iteration governance.

The claim update must:

1. update the collaboration claim record;
2. update delivery status only when its actual lifecycle changes;
3. activate or create an iteration only when required by iteration governance;
4. update the relevant inventory;
5. update the Board after the owner document;
6. synchronize the originating Issue when one exists.

## PR-Based Claim And Development Flow

PR-based work uses three distinct phases.

### Phase 1: Governance Claim PR

Create a governance-only branch from the current target branch. The claim PR may contain:

- creation or correction of the owner document;
- the collaboration claim record;
- Story or iteration activation records;
- backlog and iteration index updates;
- Board synchronization;
- documentation links required to make the task discoverable;
- governance validation fixes directly required by the claim.

It must not contain production implementation, implementation tests, unrelated refactoring,
dependency changes for the future implementation, generated implementation artifacts, or
speculative API or schema changes.

Suggested title:

```text
docs(governance): claim <TASK-ID> for <objective> [model:<model-name>]
```

The PR description must identify the source Issue, owner document, claimant, reserved scope,
dependencies, activation gate, intended implementation PR, and confirmation that no implementation
work is included.

Run:

```bash
scripts/validate_project_governance.sh .
```

The claim is not effective while this PR is open.

#### Mandatory Merge Gate

The governance claim PR must be approved and merged before:

- creating the implementation branch;
- committing or pushing implementation code;
- opening a draft implementation PR;
- changing production manifests or dependencies for the task.

Read-only investigation, repository inspection, and uncommitted disposable experiments are allowed,
but they do not establish ownership and must not be presented as task implementation.

The implementation branch must start from the governance claim merge commit or a later target-branch
commit containing that claim.

### Phase 2: Implementation PR

After the governance claim is merged:

1. refresh the target branch;
2. verify the claim is still effective;
3. create the implementation branch from the updated target branch;
4. implement only the claimed scope;
5. append execution and validation evidence to the owner document;
6. update the claim record with the implementation PR number;
7. move delivery state to `Review` when implementation is submitted.

The implementation PR must reference the source Issue, task ID, owner document, governance claim PR,
acceptance criteria, validation, residual work, and explicit non-goals.

The implementation PR normally must not mark the task `Complete`, because its implementation commit
does not yet exist on the target branch.

### Phase 3: Governance Closure PR

After the implementation PR is merged:

1. obtain the existing implementation or merge commit SHA;
2. update the owner document first;
3. record `Completion Commit: <SHA>`;
4. record accurate acceptance and validation evidence;
5. set the truthful delivery and claim states;
6. update backlog and iteration inventories;
7. update the Board;
8. comment on the originating Issue with the status and implementation commit;
9. close the Issue only when repository Issue Sync rules permit closure.

A status-only commit cannot cite itself as completion evidence.

## Direct-Commit Flow

When repository policy explicitly permits direct commits, preserve the same ordering:

1. commit and push the governance claim;
2. begin implementation only after the claim exists on the target branch;
3. commit and push implementation;
4. commit and push governance closure citing the existing implementation SHA.

Combining these steps into one commit is not permitted.

## Execution Updates

Update governance when scope or acceptance changes, a dependency changes state, ownership transfers,
work pauses or releases, an implementation PR changes state, validation exposes residual work, or a
new ADR or security review becomes necessary.

Do not edit the Board first. Do not use Issue comments as the only record of a blocker or handoff.

## Handoff

Before handing off claimed work:

1. stop implementation changes;
2. record current state, completed work, unpublished work, validation, remaining acceptance items,
   resume instructions, and branch or PR references in the owner document;
3. set claim state to `Handoff Pending`;
4. update derived views;
5. synchronize the Issue;
6. have the successor establish responsibility through a merged governance update.

The previous claimant remains responsible until the transfer is recorded on the target branch.

## Release Or Abandonment

When an agent stops work without a successor:

- set claim state to `Released`;
- return delivery status to its truthful state, usually `Ready`, `Planned`, `Blocked`, or `Partial`;
- record why work stopped;
- preserve implementation and validation evidence;
- correct derived views;
- update the Issue without closing it unless the task is Cancelled.

Never leave abandoned work `In Progress` solely because work once began.

## Conflict Resolution

The target branch is authoritative:

- the first compatible claim merged into the target branch wins;
- an open claim PR does not reserve the task;
- a later claimant must refresh and repeat preflight before merge;
- overlapping claim PRs must be closed, narrowed, or converted into explicit non-overlapping slices;
- never silently overwrite another claimant in the owner document;
- maintainer direction overrides claimant order and must be recorded in the owner document.

## Completion Checklist

Before declaring collaboration and status closure complete:

- [ ] Owner document contains the final truthful delivery status.
- [ ] Completion references one or more existing implementation commit SHAs.
- [ ] Collaboration claim is Closed, Released, or transferred.
- [ ] Acceptance and validation evidence are recorded.
- [ ] Backlog and iteration inventories match the owner document.
- [ ] Board matches the owner document.
- [ ] Source Issue contains the final status and commit reference.
- [ ] Issue closure agrees with repository Issue Sync rules.
- [ ] No stale implementation or claim PR remains open.
- [ ] Governance validation passes.

## Related SOPs

- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/CHANGE-CONTROL.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/sop/DOC-CHECK.md`
- `docs/sop/EVOLUTION-FEEDBACK.md`
