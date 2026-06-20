# GOV-003: Built-in Project Governance Logic

**Status**: Planned
**Priority**: P2
**Source**: User request 2026-06-20
**Depends on**: WEB-001 (for management UI surface); I035 (shared config for governance policy)

## Problem

The `agent-project-governance` skill (wjhuang88, v2.0.0) provides structured project
management methodology — backlog/story/iteration processes, delivery gates, DoD,
ADR/SOP structure, release/testing gates, and Agent-friendly engineering governance.
It currently works as a `SKILL.md` file injected into agent context.

This has limitations:
- Requires the skill to be present in the workspace
- Skill content competes with other context for prompt tokens
- Governance state (backlog, iterations, board) is disconnected from Talos's
  runtime awareness
- No structured view of project health is available outside the TUI scrollback

Moving this logic into Talos's core would make project governance a
**first-class runtime capability** — the agent is always aware of project
state, can enforce governance gates, and project management data can be
rendered in the WEB-001 web dashboard.

## Scope

The goal is to implement the full `agent-project-governance` methodology as a
built-in Talos subsystem. When active, the agent can discover, initialize,
maintain, and validate a project's governance structure — guiding the user
through precise task management without relying on an external SKILL.md file.

### Capability 0: Entry Point — `/agile` Command

A slash command registered in CMD-001's `CommandRegistry` as the user-facing
entry point for project governance:

```
/agile              → Show current governance state + available actions
/agile init         → Initialize governance for this project (guided)
/agile status       → Show iteration/backlog/board/validation summary
/agile validate     → Run validation harness and report results
```

Behavior by project state:
- **uninitialized**: `/agile` shows the gap and offers `/agile init`
- **adopting**: `/agile` shows migration progress and next slice
- **conformant**: `/agile` shows iteration status, backlog top-N, validation
- **degraded**: `/agile` shows what's missing and offers repair

The command delegates to GOV-003's governance engine; it does not mutate
conversation display state directly. Follows CMD-001 typed-owner pattern.

When entering a workspace, Talos inspects the project and classifies its
governance state:

| State | Meaning | Agent behavior |
|---|---|---|
| `uninitialized` | No governance entrypoint exists | Explain the gap, offer to initialize the smallest slice |
| `discovered` | Custom process assets exist, no standard mapping | Summarize reusable assets, propose adoption |
| `adopting` | Manifest exists, capabilities incomplete | Continue migration slice |
| `conformant` | Standard structure matches manifest | Use existing router; audit only what the task affects |
| `degraded` | Declared files/gates missing or stale | Propose baseline repair |

The agent checks `.agent-governance/manifest.yaml`, `AGENTS.md`, and `docs/`
structure to determine state. Classification happens once on workspace entry,
refreshed when governance docs change.

### Capability 2: Constraint Classification

The agent classifies every requirement and decision into:

| Type | Meaning | Used for |
|---|---|---|
| **Hard** | Immutable fact, platform limit, irreversible operation | Deriving mandatory gates |
| **Soft** | Policy or convention that can change | Recording in ADRs when a choice affects one |
| **Assumption** | Unvalidated belief | Flagging for validation; creating Spikes when blocking |

Every gate traces to a specific Hard constraint. The agent challenges
Soft constraints that are treated as Hard, and flags Assumptions before
they become implementation decisions.

### Capability 3: Standard Structure Initialization & Adoption

The agent can build a project's governance structure from scratch or migrate
existing assets:

1. Establish control entrypoints: `AGENTS.md`, `.agent-governance/manifest.yaml`,
   `EVOLUTION.md`, `docs/sop/EVOLUTION-FEEDBACK.md`
2. Extract active content from non-standard documents into standard owners
3. Create standard directories: `docs/backlog/`, `docs/iterations/`,
   `docs/decisions/`, `docs/roadmap/`, `docs/proposals/`, `docs/reference/`,
   `docs/sop/`, `docs/archive/`
4. Record preserved/superseded/archived sources in the manifest migration mapping
5. Add daily execution gates: testing, Git, requirement intake, iteration flow,
   change control
6. Add planning layers: backlog, iterations, decisions, roadmap
7. Derive `docs/BOARD.md` as an operating view (never source of truth)
8. Create project-local validation harness for mechanical governance rules

### Capability 4: Backlog & Story Management

The agent manages the product backlog as a first-class data structure:

- **Compact entrypoint** (`PRODUCT-BACKLOG.md`) + **item files** (`docs/backlog/active/`)
  with `Required Reads` links
- **Story decomposition**: Epics → executable Stories with parent/child identity,
  dependencies, and readiness gates
- **Story formats**: behavior-facing (Given/When/Then), technical, governance, Spike
- **Acceptance criteria**: testable, verifiable, with explicit evidence requirements
- **Backlog compaction**: preserve decision usefulness through active item files
  and archive indexes

### Capability 5: Iteration Management

The agent plans and executes iterations following the SOP:

- **Before selection**: inventory all active/review/planned/blocked iterations
- **Baseline integrity**: published iteration plans are preserved; changed
  targets use a new iteration ID
- **Deliverables**: every iteration produces a runnable, testable result
- **Execution records**: appended to the plan, not replacing it
- **Verification**: runtime evidence, not only passing unit tests
- **Retrospective**: outcome, documentation sync, lessons learned

### Capability 6: Closure Protocol

The agent follows a five-stage closure contract for every governance change:

1. **Establish**: confirm current state, preserved assets, scope, closure items
2. **Implement**: create or update the smallest complete artifact slice
3. **Verify**: run structural checks (harness), inspect semantic consistency
4. **Synchronize**: update manifest/capability state, backlog/iteration status,
   lessons, dependency/blocker records
5. **Deliver**: report changed artifacts, checks, residual gaps, and status

Status is strict: `complete` only when implemented + synced + verified + gaps
registered. Partial and blocked are explicit, not hidden behind recommendations.

### Capability 7: Governance Context Injection

A compact, bounded summary of governance state is injected into the system
prompt before each turn:

- Current iteration, active stories, blocker status
- Backlog priority view (top-N ready items)
- Recent board state (Now/Next/Blocked)
- Validation status (harness results)

This keeps the agent aware of project context without dumping full governance
documents into the prompt.

### Capability 8: Project Management Web UI (WEB-001)

Expose governance data through the WEB-001 web dashboard:

- **Iteration Board**: Kanban view (Planned / In Progress / Review / Complete)
- **Product Backlog**: Filterable table with priority, status, dependencies
- **ADR Index**: Decision records with status and dates
- **Validation Status**: Harness check results
- **Project Classification**: Current governance state and migration progress

### Capability 9: Validation Harness

The agent can run and interpret the project-local governance validation:

- `scripts/validate_project_governance.sh` on Unix
- Checks: required files, capability evidence, AGENTS.md sections, local links,
  completion claims without evidence
- Result surfaces in TUI status and WEB-001 validation page
- Failures are actionable: each one links to the owning document

### Capability 10: Change Control & Evolution

- **Change control**: when scope changes mid-iteration, record variance per
  `docs/sop/CHANGE-CONTROL.md`
- **Evolution feedback**: after defects, regressions, or planning drift,
  capture lessons per `docs/sop/EVOLUTION-FEEDBACK.md`
- **Decision records**: significant technical choices recorded as ADRs with
  Constraint Decomposition, Decision, and Reversal Trigger

## Governance State Model

The governance context layer needs a structured view of project state:

```rust
struct GovernanceState {
    iteration: Option<IterationStatus>,   // Current active iteration
    backlog: Vec<StorySummary>,            // Top-N backlog items
    board: BoardSnapshot,                  // Now/Next/Blocked/Later
    decisions: Vec<DecisionSummary>,       // Recent ADRs
    validation: ValidationStatus,          // Harness results
}
```

Sources:
- `docs/iterations/` — active iteration
- `docs/backlog/PRODUCT-BACKLOG.md` — backlog items
- `docs/BOARD.md` — operating view
- `docs/decisions/` — ADRs
- `scripts/validate_project_governance.sh` — harness output

## Non-Goals

- Do not replace `AGENTS.md` — this adds project *state* context, not behavior rules.
- Do not auto-commit or auto-modify governance documents without user intent.
- Do not require a governance manifest to function — degrade gracefully for
  projects without formal governance setup.
- Do not implement the full `agent-project-governance` skill as built-in code —
  focus on the context injection and gate enforcement layers. The skill's
  methodology remains the authority on *how* governance is structured.

## Acceptance Criteria

- [ ] Governance context is injected into system prompt as a compact, bounded
      summary when the project has governance documents.
- [ ] Context gracefully degrades to empty when governance docs are absent.
- [ ] Governance state model reads from standard `docs/` paths.
- [ ] Iteration, backlog, board, and decision state are parseable and
      refreshable at turn boundaries.
- [ ] WEB-001 project management pages render governance data (Phase 3).
- [ ] Tests cover: empty project, partial governance, full governance,
      state refresh on document change.

## Relationship To Other Requirements

| Requirement | Relationship |
|---|---|
| CMD-001 | `/agile` command registered as BuiltinCommand in CommandRegistry |
| I035 | Shared config may include governance policy flags |
| SKILL-002 | Governance context injection follows the same activation model |
| AGENT-001 | Governance docs follow standard `docs/` conventions |

## Required Reads

- `https://github.com/wjhuang88/agent-project-governance` (methodology authority)
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/CHANGE-CONTROL.md`
- `docs/sop/EVOLUTION-FEEDBACK.md`
