# SESSION-001-C: User Forks The Active Session Safely

| Field | Value |
|---|---|
| Type | State/Product Story |
| Parent Epic | [SESSION-001](SESSION-001-interactive-session-lifecycle.md) |
| Status | Proposed |
| Depends On | SESSION-001-A complete; CMD-001 registry foundation complete |
| Unlocks | Independent branch exploration through `/fork` |

## User Goal And Value

An interactive user needs to fork the active durable session, so subsequent exploration can diverge
without modifying or appending to the source session.

## Scope

- Register `/fork` only after the typed fork transition is executable.
- Clone the durable history boundary into a distinct child identity and persistence target.
- Activate the child through SESSION-001-A and hydrate its visible history.
- Preserve source identity and bytes after fork activation.

## Exclusions

- Merge/rebase between sessions, cloud branches, deletion, rename, or arbitrary historical-point UI.

## Acceptance

- Given a durable source session, when the user runs `/fork`, then Talos activates a distinct child
  id/path containing the intended source history.
- Given the child is active, when subsequent turns complete, then only the child persistence target
  changes and the source session remains byte-for-byte unchanged.
- Given fork preparation fails, when the operation returns, then the source remains active and usable.
- [ ] Runtime tests cover identity, persistence routing, visible hydration, rollback, and resource cleanup.
- [ ] README command documentation and all status owners are synchronized after runtime evidence.

## Decision Constraints

- ADR-005/006 typed session seam and single-consumer flow apply.
- ADR-016 durable history remains authoritative; UI state is not the fork source of truth.

## Required Reads

- `docs/backlog/active/SESSION-001-interactive-session-lifecycle.md`
- `docs/backlog/active/SESSION-001-A-runtime-transition-service.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/decisions/005-tui-event-architecture.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/016-layered-memory-architecture.md`
