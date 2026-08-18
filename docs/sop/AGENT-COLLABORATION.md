# SOP: Agent Collaboration And Task Claiming

## Purpose

Define how human-operated and autonomous agents claim, execute, hand off, and close Talos work
without duplicating effort or allowing GitHub Issues, owner documents, the Board, and
implementation state to drift apart.

This SOP governs collaboration ownership. It does not replace requirement intake, iteration
planning, implementation, change control, testing, release, or Git workflow SOPs.

## Adoption And Migration

This document was introduced by bootstrap PR `#83`.

- **Effective commit**: `51e0683efc1044f1016d58fa2377965213031f0c` (the squash-merge commit on `main`).
- **Effective timestamp (UTC)**: `2026-07-31T06:49:24Z`.
- **Effective timestamp (Asia/Singapore)**: `2026-07-31 14:49:24 +08:00`.
- PR `#83` itself is an adoption change and is not required to satisfy a workflow that did not yet
  exist on its target branch.
- Tasks, iterations, and implementation PRs that existed before the effective timestamp are
  grandfathered. They may finish under their published governance baseline.
- A grandfathered owner must add a Collaboration Claim before a new implementation branch or new
  implementation PR is started after the effective boundary, or when the owner next changes
  delivery state, claimant, scope, or handoff state.
- Existing implementation PRs opened before the effective timestamp do not need a retroactive claim
  PR merely to finish their already-published scope.
- A mismatch blocks only the affected owner-document chain. Unrelated Board or documentation drift
  does not stop independent work elsewhere in the repository.

The commit and timestamps above are the canonical offline audit boundary. GitHub PR `#83` remains
supporting provenance, but auditors do not need the PR page to determine whether work is pre- or
post-adoption.

## Applicability

A governed task and effective claim are required before new committed implementation work when any
of the following applies:

- product behavior, runtime behavior, API, security, storage, dependency, release, or architecture
  changes;
- a GitHub Issue, backlog Story, iteration, task, release owner, ADR, or security finding already
  owns the scope;
- work spans multiple implementation commits or PRs;
- work changes owner-document delivery state or acceptance;
- work is delegated to an autonomous or separately operated agent.

A separate claim PR is not required when an effective owner-document claim already exists on the
target branch.

### Bounded Single-PR Maintenance

A single PR may proceed without creating a governed task only when all of these are true:

- the change is a typo, broken link, wording-only documentation correction, formatting cleanup,
  reviewer-requested correction within the same PR, or mechanically bounded CI/fixture maintenance;
- it does not change product behavior, public API, security policy, dependency resolution, release
  authorization, persistent data, or an existing owner-document status;
- it does not expand the scope of another claimed task;
- it fits in one reviewable PR and leaves no residual implementation work.

The PR description must state why the bounded-maintenance exception applies. If any condition stops
being true, pause and establish a governed claim.

### Release And Reviewer Follow-Ups

- Release execution uses its existing release task owner and authorization record. A separate claim
  PR is unnecessary when that owner and claim already exist on the target branch.
- A reviewer-only follow-up on the current PR inherits that PR's owner and claim when it does not
  widen scope. New product scope requires a new or amended claim.

### Emergency Override

A maintainer may bypass the normal claim-first order for an active production incident, exploited or
credible security issue, repository outage, release-pipeline outage, or other time-critical safety
repair.

The minimum emergency record is:

- authorizing maintainer;
- incident or security reference, or a concise reason when disclosure must remain private;
- exact emergency scope;
- branch or commit used;
- validation performed or explicitly deferred;
- rollback or containment action.

Create or reconcile the owner document, Collaboration Claim, Board/Issue state, and residual work no
later than two business days after containment. `Authorization Mode` must be `Emergency override`.
Emergency authority does not waive security review when disclosure and time permit it.

## Non-Negotiable Rules

1. **Target-branch truth establishes ownership.** Proposed branch content and open PRs have no
   ownership effect until their claim record exists on the target branch.
2. **Claim before normal implementation.** New committed implementation work starts only after an
   effective claim, except for the bounded-maintenance and emergency paths above.
3. **Owner documents define task truth.** GitHub Issues are intake/discussion surfaces and
   `docs/BOARD.md` is a derived operating view.
4. **One owner scope, one effective claimant.** Parallel work requires separately identified child
   owner documents with non-overlapping Work Slice values. Do not place multiple active claim
   records in one owner document.
5. **Claim state and delivery state are distinct.** Collaboration state must not be inferred from
   `Ready`, `Active`, `Review`, `Blocked`, or `Complete`.
6. **Completion requires existing evidence.** `Complete` requires an already-existing
   implementation/evidence commit recorded as `Completion Commit: <SHA>`.
7. **Owner first, derived views second.** Update the owner document, then inventories and Board,
   then the originating Issue.

## Sources Of Truth

| Concern | Authoritative Location | Notes |
|---|---|---|
| Requirement discussion and external reports | GitHub Issue | Not authoritative for repository delivery state |
| Story scope and acceptance | Backlog Story owner document | Usually under `docs/backlog/active/` |
| Iteration objective and execution | Iteration owner document | Under `docs/iterations/` |
| Long-running or program execution | Task owner document | Under `docs/tasks/` |
| Release execution | Release task owner | Usually under `docs/tasks/` |
| Collaboration ownership | Collaboration Claim in owner document on target branch | Open PR state is derived only |
| Current operating view | `docs/BOARD.md` | Derived from owner documents |
| Code review and merge state | GitHub Pull Request | Does not replace owner-document state |
| Completion evidence | Existing implementation commit SHA | Must predate the closure status change |

When sources disagree, repair the affected owner-document chain before continuing that scope.

## Persistent Claim Model

`Claim Pending` is not a persistent owner-document state. It is a derived GitHub condition meaning
that an open claim PR proposes ownership but target-branch ownership has not changed.

Allowed persistent states are:

- `Unclaimed`: available for selection.
- `Claimed`: target branch records an effective claimant and work slice.
- `Handoff Pending`: current claimant remains responsible while a successor is being established.
- `Released`: previous claimant stopped and the scope is available again.
- `Closed`: delivery is Complete or Cancelled and the claim is terminal.

A claim PR directly proposes `Claimed`, but the proposed value has no effect until merge.

## Collaboration Claim Record

Every newly claimed owner uses one record:

```markdown
## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | `@github-user` |
| Executing Agent | `<agent/model or session label>` |
| Work Slice | `<bounded, non-overlapping scope>` |
| Claimed At | `YYYY-MM-DD` |
| Source Issue | `#123` or `None` |
| Governance Claim PR | `#456` or `Direct commit <SHA>` |
| Authorization Mode | `Independent review`, `Single-maintainer merge`, `Direct commit`, or `Emergency override` |
| Authorization Evidence | `<review, exact-head checks, maintainer reason, or incident record>` |
| Implementation PR | `Not started`, `#789`, or `None` |
| Last Updated | `YYYY-MM-DD` |
| Handoff / Release Condition | `<condition or None>` |
```

Rules:

- One owner document contains at most one effective Collaboration Claim.
- Parallel non-overlapping slices require child owner documents with stable IDs.
- `Claimed` and `Handoff Pending` require complete actor, scope, date, claim reference,
  authorization, and evidence fields.
- `Closed` must agree with a Complete or Cancelled delivery state. Complete owners must also contain
  valid completion commit evidence.
- A claim does not expire automatically. A maintainer explicitly transfers or releases stale work.

## Required Preflight

Before proposing a claim:

1. Refresh the target branch.
2. Read the complete Issue and all linked PRs/comments when an Issue exists.
3. Search for the Issue number, proposed task ID, matching scope, owner documents, ADRs, and
   acceptance language.
4. Inventory relevant Active, Review, Planned, and Blocked iterations.
5. Check open PRs and branches for overlapping work.
6. Confirm dependencies, activation gates, and required decisions.
7. Decide whether the request maps to an existing owner, needs requirement intake, is duplicate,
   or is eligible for a bounded-maintenance exception.

Do not create a second owner merely because wording differs.

## Claim A GitHub Issue And Form A Task

1. Validate whether the Issue is new work, duplicate, refinement, continuation, or blocked work.
2. Select an existing owner or create the appropriate Story/task owner.
3. Record source Issue, scope/non-goals, dependencies, acceptance, validation, documentation, and
   the Collaboration Claim.
4. Update the relevant inventory and Board only after the owner.
5. Comment on the Issue with owner path, claimant, proposed claim PR, and the statement that the
   claim is ineffective until target-branch merge.

Issue assignment and labels improve visibility but do not establish repository ownership.

## Claim An Existing Task

An existing owner may be claimed only when:

- its delivery state permits work;
- dependencies and activation gates are satisfied or explicitly Blocked;
- no effective claimant owns the same Work Slice;
- no overlapping implementation PR exists;
- selection does not bypass unresolved iteration governance.

Update the owner first, then inventory/Board, then Issue.

## PR-Based Claim Flow

### 1. Open A Draft Claim PR

Create a governance-only branch from the current target branch. Before a PR number exists, the draft
owner record may remain `Unclaimed` with `Governance Claim PR: Pending`.

Open the Draft PR to obtain its number. The draft is not reviewable or mergeable in this state.

### 2. Finalize The Proposed Claim

Update the same branch so the exact-head owner record contains:

- `Claim State: Claimed`;
- the actual `Governance Claim PR: #NN`;
- complete Work Slice and authorization fields;
- no production implementation, implementation tests, speculative dependencies, or generated
  implementation artifacts.

Run:

```bash
scripts/validate_project_governance.sh .
scripts/validate_collaboration_claims.sh .
```

Only this finalized exact head may enter claim review.

### 3. Authorization Paths

Use one of these paths:

1. **Independent review**: an independent maintainer/reviewer approves. This remains mandatory for
   security-sensitive, sandbox, permission, process-hardening, or otherwise explicitly protected
   scopes.
2. **Single-maintainer merge**: when no independent reviewer is available, the maintainer may merge
   after exact-head CI and both governance validators pass, the PR records why independent review
   is unavailable, and no unresolved blocking review feedback remains.
3. **Direct commit**: permitted only when repository policy explicitly allows it and a maintainer
   records exact validation and reason. It is not a convenience substitute for normal review.
4. **Emergency override**: follows the emergency section and requires post-containment
   reconciliation.

The PR author does not need to approve their own PR under the single-maintainer path.

### Deferred Human Validation Does Not Change Merge Authorization

`LONG-RUNNING-TASK.md` may schedule an owner-defined natural-person review or device/manual
acceptance row into a linked cleanup-validation Issue. That mode does not replace the authorization
path above and does not permit a false Complete state:

- exact-head CI, applicable independent Agent technical review, both governance validators and
  merge-time CAS still apply before merge;
- independent security review for sandbox, `talos-permission`, process-hardening or
  permission-policy scope remains non-deferrable;
- the source owner stays Review until its deferred row passes;
- a later child may proceed only when the long-task owner records the prior Review disposition,
  non-overlap and the later child's own effective claim.

### 4. Mandatory Merge-Time CAS Preflight

Immediately before merge, re-check all of the following against the exact head:

- the claim branch contains the latest target branch or is otherwise reported mergeable without
  hidden conflicts;
- the target owner document has not gained another effective claimant;
- no new overlapping implementation or claim PR exists;
- the proposed Responsible Actor and Work Slice still match current owner truth;
- dependencies and activation gates remain satisfied;
- `Governance Claim PR` matches the actual PR number;
- exact-head CI and both governance validators passed;
- required authorization evidence is present;
- no unresolved blocking review feedback remains.

If any check changes, do not merge. Refresh the branch, update the owner, and rerun exact-head
validation. This is the collaboration compare-and-swap gate.

### 5. Merge Establishes Ownership

The claim becomes effective only when the finalized `Claimed` record exists on the target branch.
The implementation branch starts from that claim merge commit or a later target-branch commit.

Do not create the implementation branch, commit/push implementation, change production dependencies,
or open a draft implementation PR before this point.

Read-only investigation and disposable uncommitted experiments do not establish ownership and must
not be represented as implementation progress.

## Implementation PR

After claim merge:

1. refresh the target branch and verify the claim remains effective;
2. create the implementation branch from the claim merge or later target commit;
3. implement only the Work Slice;
4. append execution and validation evidence to the owner;
5. record the implementation PR number;
6. move delivery state to `Review` when submitted.

The implementation PR references the Issue, task ID, owner, claim PR, acceptance, validation,
residuals, and non-goals. It normally does not mark the owner Complete because its implementation
commit is not yet on the target branch.

## Governance Closure

After implementation merge:

1. obtain the existing implementation or merge SHA;
2. update the owner first and record `Completion Commit: <SHA>`;
3. record acceptance and validation evidence;
4. set truthful delivery and claim states;
5. update inventories and Board;
6. synchronize and close the Issue only when Issue Sync rules permit it.

A status-only commit cannot cite itself.

## Direct-Commit Sequence

When Direct commit is authorized, preserve the same order:

1. commit/push the claim record;
2. begin implementation only after the claim is on the target branch;
3. commit/push implementation;
4. commit/push closure citing the existing implementation SHA.

Do not combine these into one commit.

## Handoff, Release, And Abandonment

For handoff:

1. stop implementation changes;
2. record current state, commits, validation, remaining acceptance, branch/PR, and exact resume gate;
3. set `Handoff Pending`;
4. keep the current claimant responsible until a successor claim reaches the target branch.

For release without a successor, set `Released`, restore truthful delivery state, preserve evidence,
record why work stopped, and synchronize derived views and Issue. Do not leave abandoned work
`In Progress` merely because implementation once began.

## Conflict Resolution

- Target-branch owner truth wins.
- Open PRs do not reserve work.
- First compatible merged claim wins.
- Later overlapping claims must close, narrow, or create separate child owners.
- Never silently replace another claimant.
- Maintainer overrides must be recorded with authorization evidence.

## Validation And Completion Checklist

Before claim merge or collaboration closure:

- [ ] Persistent Claim State is allowed; `Claim Pending` is not stored.
- [ ] Responsible Actor, Work Slice, dates, PR/commit reference, and authorization are complete.
- [ ] Merge-time CAS preflight was repeated against exact head.
- [ ] `scripts/validate_project_governance.sh .` passes.
- [ ] `scripts/validate_collaboration_claims.sh .` passes.
- [ ] Owner, inventories, Board, and Issue agree.
- [ ] Complete owners cite existing implementation evidence.
- [ ] No stale claim or implementation PR remains open.

## Related SOPs

- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/CHANGE-CONTROL.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/sop/LONG-RUNNING-TASK.md`
- `docs/sop/DOC-CHECK.md`
- `docs/sop/EVOLUTION-FEEDBACK.md`
