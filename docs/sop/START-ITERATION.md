# SOP: Start Iteration

## Purpose

Define the process for beginning a new development iteration from the product backlog.

## Prerequisites

- A product backlog with ready Stories exists at `docs/backlog/PRODUCT-BACKLOG.md`.
- The implementation roadmap exists at `docs/roadmap/IMPLEMENTATION-ROADMAP.md`.
- `docs/sop/AGENT-COLLABORATION.md` has been applied to the selected Work Slice.

## Process

### 1. Inventory Existing Iterations

Before selecting new work, check `docs/iterations/` for:

- **Active** iterations — complete or explicitly pause first;
- **Review** iterations — verify or explicitly retain in Review;
- **Planned** iterations — activate, defer, block, or supersede;
- **Blocked** iterations — resolve or explicitly continue blocking.

Do not bypass unresolved iterations to start fresh work.

Record inventory and disposition in `docs/iterations/README.md` or the activation record. A Planned
item must be activated, deferred, kept blocked with its blocker, or superseded before unrelated work
is selected.

### 2. Select Stories

1. Review the implementation roadmap for the current phase.
2. Select Stories that:
   - are Ready;
   - have dependencies met;
   - fit the timebox;
   - produce a runnable, testable deliverable.
3. Prioritize dependency order, risk reduction, then user value.
4. Confirm no overlapping effective claimant or open implementation PR owns the same Work Slice.

### 3. Create Iteration Plan

Create `docs/iterations/I{NNN}-{slug}.md` from `docs/iterations/TEMPLATE.md`.

The committed plan preserves:

- selected Stories and parent relationship;
- dependencies and execution order;
- scope and non-goals;
- acceptance and planned validation;
- risks and rollback assumptions;
- user-facing documentation targets.

Do not replace a committed plan with a newer objective.

### 4. Establish The Collaboration Claim

Before activation:

1. Create or update the iteration Collaboration Claim.
2. Record one bounded Work Slice and one Responsible Actor.
3. Use the Draft-PR number backfill sequence from `AGENT-COLLABORATION.md`.
4. Run both governance validators and exact-head CI.
5. Repeat the merge-time CAS check immediately before claim merge.
6. Use an allowed independent-review, single-maintainer, direct-commit, or emergency authorization
   path.

An effective Collaboration Claim exists only when the finalized Claimed record is on the target
branch. `Claim Pending` is never stored in the iteration owner.

### 5. Activate And Begin Work

After the effective claim exists:

- refresh the target branch;
- create the implementation branch from the claim merge commit or a later target commit;
- mark selected Stories In Progress and the iteration Active;
- append the activation date, claim PR/commit, authorization, dependency inventory, and merge-time
  CAS result;
- follow `ITERATION-WORKFLOW.md`.

## Rules

- One active iteration at a time unless explicitly approved.
- Parallel non-overlapping work uses separately owned child iterations/Stories, not multiple active
  claim records in one owner.
- Scope changes require `CHANGE-CONTROL.md`.
- Record execution by appending; do not overwrite the published baseline.
- Before Complete, commit/push implementation evidence, then append
  `Completion Commit: <SHA>`. A status-only commit cannot cite itself.
- Missing completion evidence keeps the iteration Review, Partial, or Blocked.
- Select ready child Stories, not an unresolved Epic parent.
- Bounded-maintenance and emergency exceptions follow `AGENT-COLLABORATION.md` and must not be used
  to silently activate an iteration.
