# SOP: Start Iteration

## Purpose

Define the process for beginning a new development iteration from the product backlog.

## Prerequisites

- A product backlog with ready stories exists at `docs/backlog/PRODUCT-BACKLOG.md`
- The implementation roadmap exists at `docs/roadmap/IMPLEMENTATION-ROADMAP.md`

## Process

### 1. Inventory Existing Iterations

Before selecting new work, check `docs/iterations/` for:

- **Active** iterations — must be completed or explicitly paused first
- **Review** iterations — must pass verification before starting new work
- **Planned** iterations — activate, defer, or continue blocking
- **Blocked** iterations — resolve or explicitly continue blocking

Do not bypass unresolved iterations to start fresh work.

Record the inventory and disposition in `docs/iterations/README.md` or the activation record. A
`Planned` item must be activated, explicitly deferred, kept blocked with its blocker, or marked
superseded before unrelated backlog work is selected.

### 2. Select Stories

1. Review the implementation roadmap for the current phase.
2. Select stories from the backlog that:
   - Are in "Ready" status
   - Have all dependencies met
   - Fit within the iteration timebox (typically 2-3 weeks)
3. Prioritize by: dependency order, risk reduction, user value.

### 3. Create Iteration Plan

Create `docs/iterations/I{NNN}-{slug}.md` with:

```markdown
# Iteration I{NNN}: {Title}

> Document status: Planned
> Published plan date: YYYY-MM-DD
> Planned objective: ...
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the runnable/testable result produced by this iteration.

## Published Baseline
- Selected ready Stories and parent relationship
- Dependencies and execution order
- Scope and explicit non-goals
- Acceptance and planned validation
- Risks and rollback assumptions
- User-facing documentation to update

## Actual Activation And Execution
| Date | Type | Record |
|---|---|---|
```

Use `docs/iterations/TEMPLATE.md`; do not replace a committed plan with a newer objective.

### 4. Begin Work

- Mark selected stories as "In Progress" in the backlog.
- Follow `ITERATION-WORKFLOW.md` for daily execution.

## Rules

- One active iteration at a time unless explicitly approved.
- Iteration scope changes require `CHANGE-CONTROL.md`.
- Record results by appending to the iteration file, not overwriting the plan.
- Select ready child Stories, not a multi-stage parent Epic with unresolved children.
- The selected set must produce a runnable, testable deliverable. If it cannot, refine the slice
  or record an explicit infrastructure-only exception.
