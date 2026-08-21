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

## Work Modes

Select one mode in the task record before activation. The mode controls scheduling only; it never
weakens implementation, security, merge, or evidence requirements.

| Mode | Scheduling rule | Closure rule |
|---|---|---|
| Standard | Every required acceptance gate is completed before the next dependent item starts. | Close only after all acceptance rows pass or have explicitly owned residuals. |
| Deferred Human Validation | At activation, create one validation Issue and a later evidence-only cleanup iteration. Eligible natural-person or device-dependent rows move to that Issue, so non-overlapping implementation items may continue. | Source owners remain `Review`; close the long task only after the cleanup iteration resolves every required row or records a separately owned corrective item. |

`Deferred Human Validation` is therefore a work mode, not a pass, waiver, or implementation status.
The task record must name the tracker Issue and cleanup iteration even when no row is currently
known; rows are appended when their exact implementation heads become available.

## Startup Contract

Before execution, create one task record in the owning iteration. If no iteration owns the work,
create `docs/tasks/YYYY-MM-DD-<slug>.md`.

The record is a published execution baseline and must contain:

```text
Outcome:
In scope:
Out of scope:
Work mode: `Standard` or `Deferred Human Validation`
Deferred validation tracker: `<GitHub Issue>` and `<cleanup iteration>`, or `None`
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

## Atomic Claim And Activation Gate

In one governance-only transition before implementation:

1. Apply `docs/sop/AGENT-COLLABORATION.md`.
2. Backfill the actual Draft claim PR number.
3. Finalize the proposed record as Claimed and In Progress while stating that both are ineffective
   until target-branch merge.
4. Run exact-head CI, `validate_project_governance.sh`, and `validate_collaboration_claims.sh`.
5. Repeat merge-time CAS checks.
6. Merge using an allowed Authorization Mode.
7. Start implementation from the claim merge commit or later target commit.

Do not create a second activation PR unless a recorded dependency can become true only after the
claim merge.

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

## Deferred Human Validation Mode

A maintainer may select this mode in the startup contract when natural-person review or
device-dependent manual acceptance will be unavailable during an otherwise runnable long task.
It is a first-class work mode: activation creates the tracker and cleanup iteration before the
first implementation item runs. The mode changes scheduling, not evidence truth or merge safety.

At activation:

1. Create one GitHub Issue that inventories every deferred validation row. Link it from the task
   record and name the planned evidence-only cleanup iteration. This creation is an activation
   prerequisite, but an open row does not block the next non-overlapping implementation item.
2. For each row, record the source owner, exact implementation head when known, required human or
   device evidence, and the condition that adds the row when the implementation head does not yet
   exist.
3. Add a final ordered cleanup-validation item before long-task closure. The task cannot be
   Complete while a required tracker row remains open; implementation children may continue while
   that row is open when their own dependencies and gates hold.

Per child implementation:

- exact-head CI, locked checks, independent Agent technical review with identity limits disclosed,
  applicable governance validation and merge-time CAS remain merge gates;
- independent security review required by `AGENTS.md` for sandbox, `talos-permission`,
  process-hardening or permission-policy changes cannot be deferred;
- only explicitly listed natural-person review and device/manual acceptance rows may move to the
  tracker;
- the source Story/iteration remains Review and records the scheduling variance; it is not marked
  Complete or described as human-accepted;
- after the implementation merge and tracker update, the next non-overlapping child may start when
  the task record explicitly dispositions the Review item and all of that child's own activation
  gates hold.

During the cleanup phase, execute every row against both its recorded source head and the final
integrated runtime head where behavior can interact. A failed row keeps the source owner Review and
creates a separately governed corrective item. Close the tracker and source owners only after
owner-first evidence synchronization.

## Execution

1. Execute items in dependency order.
2. Establish each child through one atomic claim+activation governance merge when its dependencies
   are already satisfied; do not schedule a separate activation PR by default.
3. Use confirmed defaults for non-blocking ambiguity.
4. Converge each implementation phase locally and push only its stable stage candidate.
5. Run each Completion Gate before marking the item done.
6. Record a checkpoint before the next implementation phase.
7. Follow `GIT-WORKFLOW.md`; commits preserve code state but do not replace task records.
8. Update owner documents before Board or other derived views.
9. Put optional or unsuccessful non-blocking work in the declared residual destination.
10. Keep Collaboration Claim fields current when implementation PR, claimant, authorization, or
   handoff changes.

Remote CI/review is a phase boundary, not the inner development loop. If a stable candidate receives
blocking findings, batch the related corrections locally, rerun the full local checkpoint and update
the same PR once. Do not create a PR per finding or per status-field correction.

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
