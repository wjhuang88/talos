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

- [ ] It identifies the user, caller, maintainer, or operator receiving the result
- [ ] It states the goal, value, scope, and explicit exclusions
- [ ] Behavior-facing work has Given/When/Then acceptance; technical work has equivalent command
      or structural evidence
- [ ] Affected crates and dependencies are identified
- [ ] No blocking assumptions remain unvalidated
- [ ] Governing ADRs/specs are linked with their implementation constraint and acceptance impact
- [ ] Minimum validation, state/status owners, residual destination, and user-facing documentation
      are identified
- [ ] Mandatory implementation context appears under `Required Reads`
- [ ] It fits within the current iteration scope (or is deferred to the backlog)

An Epic is ready when:

- [ ] Its overall outcome, boundary, major risks, and completion condition are explicit
- [ ] Its child stories are defined with stable IDs and acceptance criteria
- [ ] Dependencies between children are mapped
- [ ] At least the first child story is ready
- [ ] The parent links every child and each child links the parent

### 3. Create or Update Backlog Entry

Add a compact row to `docs/backlog/PRODUCT-BACKLOG.md` and put executable detail in
`docs/backlog/active/<ID>-<slug>.md`. The compact row must link every mandatory Story/ADR/spec under
`Required Reads`.

Use the appropriate shape:

```
Type: Product/API/State Story | Technical Story | Governance Story | Spike
Parent Epic: <ID or None>
Status: Refinement | Ready | In Progress | Review | Done | Blocked

Identity / Goal / Value:
Scope:
Exclusions:
Dependencies:
Decision links and constraints:
Uncertainty and validation path:
State/status owners:
User-facing documentation:
Required Reads:

Acceptance for behavior:
- Given <precondition>
  When <actor action>
  Then <observable result>

Acceptance for technical/governance work:
- [ ] <command or check> proves <result>
- [ ] <owner status> is synchronized
- [ ] <residual or exception> is recorded
```

### 4. Route

- If work is within current iteration scope → implement per `ITERATION-WORKFLOW.md`
- If work is new scope → follow `CHANGE-CONTROL.md`
- If work is not yet planned → stays in backlog until iteration selection

## Rules

- No implementation without a backlog entry.
- No Epic without at least one defined child story.
- Iterations select ready child Stories, not an Epic parent.
- A Story that changes observable behavior is incomplete until affected user documentation is
  updated or a documentation residual is registered.
- ADR-constrained work is not Ready until the Story links the decision and carries its constraint
  into acceptance.
- Spike results must be recorded as a decision or proposal before triggering implementation.
