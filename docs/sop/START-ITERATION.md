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

## Scope
{What this iteration delivers}

## Selected Stories
- [ ] #E{epic}-S{story}: {Title}
- [ ] #E{epic}-S{story}: {Title}

## Acceptance Criteria
{What must be true when this iteration is complete}

## Risks
{What could go wrong and mitigation strategies}
```

### 4. Begin Work

- Mark selected stories as "In Progress" in the backlog.
- Follow `ITERATION-WORKFLOW.md` for daily execution.

## Rules

- One active iteration at a time unless explicitly approved.
- Iteration scope changes require `CHANGE-CONTROL.md`.
- Record results by appending to the iteration file, not overwriting the plan.
