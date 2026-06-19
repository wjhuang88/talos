# SESSION-001-B: User Creates Or Resumes An Interactive Session

| Field | Value |
|---|---|
| Type | State/Product Story |
| Parent Epic | [SESSION-001](SESSION-001-interactive-session-lifecycle.md) |
| Status | Proposed |
| Depends On | SESSION-001-A complete; CMD-001 registry foundation complete |
| Unlocks | Interactive session continuity through `/new` and `/resume` |

## User Goal And Value

An interactive user needs to create or resume a workspace session without restarting Talos, so
they can change conversation context without carrying hidden state from the previous session.

## Scope

- Register `/new` and `/resume` as BuiltinCommands after their typed lifecycle operations exist.
- List workspace-scoped resume candidates deterministically.
- Hydrate durable history and visible history from the selected target.
- Preserve the old session when preparation fails.

## Exclusions

- Fork, deletion, rename, cross-workspace resume, and model switching.
- Session picker presentation beyond the minimum runnable command path.

## Acceptance

- Given an idle interactive session, when the user runs `/new`, then the next turn uses a fresh
  Agent context and persistence target while process-level configuration remains available.
- Given resumable sessions in two workspaces, when the user invokes `/resume`, then only active
  workspace candidates are selectable in deterministic order.
- Given target hydration fails, when resume is attempted, then the original session remains active
  and the user receives a visible error.
- Given a model/tool turn is active, when a lifecycle command is invoked, then the documented
  refusal or confirmed-cancellation policy is applied without racing state replacement.
- [ ] README command documentation and all status owners are synchronized after runtime evidence.

## Decision Constraints

- ADR-005/006 typed session seam and single-consumer flow apply.
- MEM-004 workspace identity is authoritative for candidate filtering.

## Required Reads

- `docs/backlog/active/SESSION-001-interactive-session-lifecycle.md`
- `docs/backlog/active/SESSION-001-A-runtime-transition-service.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/backlog/active/MEM-004-workspace-session-topology.md`
- `docs/decisions/005-tui-event-architecture.md`
- `docs/decisions/006-event-architecture-boundary.md`
