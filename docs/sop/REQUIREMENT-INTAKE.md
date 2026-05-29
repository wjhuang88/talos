# SOP: Requirement Intake

## Purpose

Define how new features and changes enter the project. Every piece of work must be a concrete,
verifiable story before implementation begins.

## Process

### 1. Receive Request

When a feature or change is requested:

1. Clarify the intent. What problem does this solve?
2. Identify affected crates and layers.
3. Classify the scope:
   - **Story** — Single, testable unit of work (most common)
   - **Epic** — Large outcome requiring multiple stories with dependencies
   - **Spike** — Research task to validate an assumption before committing

### 2. Check Readiness

A story is ready to implement when:

- [ ] It has a clear, verifiable acceptance criterion
- [ ] Affected crates and dependencies are identified
- [ ] No blocking assumptions remain unvalidated
- [ ] It fits within the current iteration scope (or is deferred to the backlog)

An Epic is ready when:

- [ ] Its child stories are defined with IDs and acceptance criteria
- [ ] Dependencies between children are mapped
- [ ] At least the first child story is ready

### 3. Create or Update Backlog Entry

Add the story to `docs/backlog/PRODUCT-BACKLOG.md` with:

```
### #E{epic}-S{story}: {Title}

**Description**: What this does and why.
**Acceptance Criteria**:
- [ ] {measurable criterion 1}
- [ ] {measurable criterion 2}
**Depends On**: #E{epic}-S{story} (or "None")
**Estimate**: S/M/L/XL
**Status**: Ready | In Progress | Done | Blocked
```

### 4. Route

- If work is within current iteration scope → implement per `ITERATION-WORKFLOW.md`
- If work is new scope → follow `CHANGE-CONTROL.md`
- If work is not yet planned → stays in backlog until iteration selection

## Rules

- No implementation without a backlog entry.
- No Epic without at least one defined child story.
- Spike results must be recorded as a decision or proposal before triggering implementation.
