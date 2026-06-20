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

### Phase 1: Governance Context Injection

Build a governance context layer that reads the project's governance state
from `docs/` and injects a structured summary into the system prompt:

- Current iteration, active stories, blocker status
- Backlog priority view
- Recent board state
- Pending ADRs and decisions
- Validation status (governance harness results)

The injection is **always-on but bounded** — a compact summary, not the full
governance document dump. The `AGENTS.md` at project root remains the
authoritative behavior rules; this layer adds project *state* awareness.

### Phase 2: Governance Gate Enforcement

Add built-in checks that the agent evaluates at key workflow points:

| Gate | Trigger | Check |
|---|---|---|
| Story selection | Before starting new work | Inventory active/review/planned/blocked iterations per SOP |
| Completion claim | Before marking work done | Evidence recorded, status owners synced |
| Change control | When scope changes mid-iteration | Record variance per CHANGE-CONTROL.md |
| Evolution feedback | After defect/regression | Capture lesson per EVOLUTION-FEEDBACK.md |

These are not hard blocks — they're **guidance injections** that prompt the
agent (or the user) to follow governance process. Hard enforcement is a
future configuration option.

### Phase 3: Project Management Web UI (WEB-001 extension)

Expose governance data through the WEB-001 web dashboard:

- **Iteration Board**: Kanban-style view of current iteration (stories in columns: Planned / In Progress / Review / Complete)
- **Product Backlog**: Filterable table view of backlog items with priority, status, dependencies
- **ADR Index**: Decision records with status and date
- **Validation Status**: Governance harness check results

The web UI reads from the same `docs/` sources the governance context layer
uses — single source of truth, no duplication.

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
| WEB-001 | Phase 3 provides the project management web UI |
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
