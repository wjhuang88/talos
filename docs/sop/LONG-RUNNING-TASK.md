# SOP: Long-Running Task

## Purpose

Run multi-phase or unattended Talos development safely after one consolidated confirmation, with
durable checkpoints that another Agent can resume without reconstructing the work from chat.

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
create `docs/tasks/YYYY-MM-DD-<slug>.md`. The record is a published execution baseline and must
contain:

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

Every task item must have an ID, expected output, completion gate, dependencies, and fallback.

## Consolidated Confirmation

Discover repository facts first, then ask one grouped confirmation covering every unresolved item:

- outcome, priorities, scope boundaries, acceptance, and evidence;
- authorization to edit, execute, commit, push, release, migrate, deploy, use network services,
  spend money, or perform destructive actions when applicable;
- credentials, environments, accounts, branches, worktrees, and deployment targets;
- time/cost/resource limits, retry behavior, and defaults for foreseeable ambiguity;
- conditions to defer versus conditions that must stop the run;
- checkpoint frequency, recovery record, and final delivery expectations.

Record the approved contract in the task owner before status becomes `In Progress`. One approval
must cover the full planned cycle, not only its first item. Never infer permission for push,
release, deployment, migration, spending, or destructive work from permission to edit code.

## Execution

1. Execute items in dependency order.
2. Use confirmed defaults for non-blocking ambiguity.
3. Run each item's completion gate before marking it done.
4. Record a checkpoint before entering the next implementation phase.
5. Follow `docs/sop/GIT-WORKFLOW.md`; commits preserve code state but do not replace task records.
6. Update owner documents before `docs/BOARD.md` or other derived views.
7. Put optional or unsuccessful non-blocking work in the declared residual destination.

Interrupt the user only when an unconfirmed condition prevents safe progress: missing access,
unapproved irreversible action, contradictory outcomes, material safety/security/privacy/cost risk,
or exhausted retry/fallback policy.

## Checkpoint

Append this record at every phase boundary and before handing off or stopping:

```text
Completed task items:
Current state and artifacts:
Commands/checks and actual results:
Open risks or deviations:
Next task item:
Recovery or resume instruction:
```

Do not report progress from memory. A resume instruction must identify the owning record, current
Git state/commit where applicable, and the exact next gate.

## Completion

A long-running task is complete only when:

- every required item passed its completion gate;
- every task item marked Complete names an already-existing implementation/evidence commit in its
  owner record as `Completion Commit: <SHA>`; a checkpoint or documentation status commit cannot
  cite itself as the evidence;
- required tests and runtime evidence passed;
- backlog, iteration, README, decisions, lessons, and Board owners are synchronized as applicable;
- deviations and residuals have an explicit owner;
- the final checkpoint contains recovery information even when no recovery is expected;
- the final report states actual commits/actions and anything intentionally not pushed, released,
  migrated, or deployed.

Failed validation, missing owner synchronization, unchecked required items, or absent confirmation
keeps the task `Partial` or `Blocked`.

## Task Item Template

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T1 | ... | ... | None | ... | ... | Planned |

## Related SOPs

- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/CHANGE-CONTROL.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/sop/DOC-CHECK.md`
- `docs/sop/EVOLUTION-FEEDBACK.md`
