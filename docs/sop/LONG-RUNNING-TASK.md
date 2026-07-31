# SOP: Long-Running Task

## Purpose

Run multi-phase or unattended Talos development safely after one consolidated confirmation, with
durable checkpoints that another Agent can resume without reconstructing work from chat.

## Trigger

Use this SOP when any condition applies:

- the user asks for unattended, overnight, autonomous, background, or long-running work;
- work has at least three ordered implementation/validation phases;
- a build, migration, evaluation, or batch operation may exceed 30 minutes;
- multiple repositories, worktrees, releases, migrations, or external systems must be coordinated;
- interruption could lose expensive progress or leave state difficult to reconstruct.

Do not use this SOP for a short isolated change.

## Startup Contract

Before execution, create one task record in the owning iteration. If no iteration owns the work,
create `docs/tasks/YYYY-MM-DD-<slug>.md`.

The record is a published execution baseline and must contain:

```text
Outcome:
In scope:
Out of scope:
Ordered task items:
Dependencies and prerequisites:
Artifacts and state owners to update:
Validation and acceptance evidence:
Branch, worktree and checkpoint plan:
Allowed permissions and external actions:
Destructive or irreversible operations:
Time, cost and resource limits:
Failure, retry and fallback policy:
Default decisions for foreseeable ambiguity:
Residual-work destination:
```

It must also contain one Collaboration Claim:

```markdown
## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed | Claimed | Handoff Pending | Released | Closed |
| Responsible Actor | `@github-user` or Not assigned |
| Executing Agent | `<agent/model or session label>` or Not assigned |
| Work Slice | `<bounded multi-phase scope>` or Not assigned |
| Claimed At | `YYYY-MM-DD` or Not applicable |
| Source Issue | `#123` or None |
| Governance Claim PR | `#456`, `Direct commit <SHA>`, or Not applicable |
| Authorization Mode | Independent review | Single-maintainer merge | Direct commit | Emergency override | Not applicable |
| Authorization Evidence | `<review/check/reason/incident>` or Not applicable |
| Implementation PR | `#789`, Not started, or None |
| Last Updated | `YYYY-MM-DD` |
| Handoff / Release Condition | `<condition>` or None |
```

`Claim Pending` is an open-PR derived condition and is never persisted. Parallel independent work
uses separately owned child task records rather than multiple active claims in one task.

Every task item has an ID, expected output, completion gate, dependencies, and fallback.

## Claim And Activation Gate

Before status becomes In Progress:

1. Apply `docs/sop/AGENT-COLLABORATION.md`.
2. Backfill the actual Draft claim PR number.
3. Finalize the proposed record as Claimed.
4. Run exact-head CI, `validate_project_governance.sh`, and `validate_collaboration_claims.sh`.
5. Repeat merge-time CAS checks.
6. Merge using an allowed Authorization Mode.
7. Start implementation from the claim merge commit or later target commit.

Existing pre-adoption long tasks are grandfathered as defined by `AGENT-COLLABORATION.md`.

## Consolidated Confirmation

Discover repository facts first, then ask one grouped confirmation covering unresolved items:

- outcome, priorities, scope boundaries, acceptance, and evidence;
- authorization to edit, execute, commit, push, release, migrate, deploy, use network services,
  spend money, or perform destructive actions;
- credentials, environments, accounts, branches, worktrees, and deployment targets;
- time/cost/resource limits, retry behavior, and defaults;
- defer versus stop conditions;
- checkpoint frequency, recovery record, and final delivery expectations.

Record the approved contract and authorization evidence before In Progress. One approval covers the
planned cycle, not only its first item. Never infer permission for push, release, deployment,
migration, spending, or destructive work from permission to edit code.

## Execution

1. Execute items in dependency order.
2. Use confirmed defaults for non-blocking ambiguity.
3. Run each Completion Gate before marking the item done.
4. Record a checkpoint before the next implementation phase.
5. Follow `GIT-WORKFLOW.md`; commits preserve code state but do not replace task records.
6. Update owner documents before Board or other derived views.
7. Put optional or unsuccessful non-blocking work in the declared residual destination.
8. Keep Collaboration Claim fields current when implementation PR, claimant, authorization, or
   handoff changes.

Interrupt the user only when an unconfirmed condition prevents safe progress: missing access,
unapproved irreversible action, contradictory outcomes, material safety/security/privacy/cost risk,
or exhausted retry/fallback policy.

## Checkpoint

Append at every phase boundary and before handoff/stopping:

```text
Completed task items:
Current state and artifacts:
Commands/checks and actual results:
Open risks or deviations:
Next task item:
Recovery or resume instruction:
```

Do not report progress from memory. Resume instructions identify owner record, Git state/commit, and
exact next gate.

## Handoff

Before transfer:

- set Claim State to Handoff Pending;
- record current state, commits, branches/PRs, validation, remaining acceptance, and exact resume
  instructions;
- keep the current claimant responsible until a successor claim reaches the target branch.

## Completion

A long-running task is complete only when:

- every required item passed its Completion Gate;
- each Complete item names an already-existing implementation/evidence commit as
  `Completion Commit: <SHA>`;
- required tests and runtime evidence passed;
- backlog, iteration, README, decisions, lessons, and Board owners are synchronized;
- deviations and residuals have explicit owners;
- the final checkpoint contains recovery information;
- the Collaboration Claim is Closed and agrees with delivery state;
- the final report states actual commits/actions and anything intentionally not pushed, released,
  migrated, or deployed.

Failed validation, missing synchronization, unchecked required items, absent confirmation, or an
incomplete claim keeps the task Partial or Blocked.

## Task Item Template

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T1 | ... | ... | None | ... | ... | Planned |

## Related SOPs

- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/CHANGE-CONTROL.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/sop/DOC-CHECK.md`
- `docs/sop/EVOLUTION-FEEDBACK.md`
