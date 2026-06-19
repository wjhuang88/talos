# SESSION-001-A: Session Runtime Transition Service

| Field | Value |
|---|---|
| Type | Technical Story |
| Parent Epic | [SESSION-001](SESSION-001-interactive-session-lifecycle.md) |
| Status | Ready |
| Depends On | MEM-002 and MEM-004 complete; ADR-005; ADR-006 |
| Unlocks | SESSION-001-B; SESSION-001-C |

## Objective And Value

The maintainer needs one prepare/commit/rollback service for replacing the active session runtime,
so interactive lifecycle commands cannot split Agent context, persistence, conversation state, and
visible history across different sessions.

## Scope

- Define a typed transition request and prepared replacement runtime.
- Keep the old runtime active until preparation/hydration succeeds.
- Commit the replacement atomically and shut down old session-owned resources.
- Return structured failure without partially mutating the active runtime.
- Reuse existing startup create/resume composition where practical.

## Exclusions

- Command registration, picker UI, new/resume/fork product flows.
- Session deletion, rename, remote control, or global pub/sub.

## Decision Constraints

- [ADR-005](../../decisions/005-tui-event-architecture.md): transitions cross the typed session seam; UI does not rebuild Agent state directly.
- [ADR-006](../../decisions/006-event-architecture-boundary.md): no global event bus; state replacement remains single-owner and auditable.

## Acceptance

- [ ] Tests prove failed preparation leaves the old runtime active and writable.
- [ ] Tests prove successful commit updates Agent history, persistence target, conversation/status,
      visible history source, and session-owned resource handles as one transition.
- [ ] Active-turn behavior is explicit and cancellation/refusal cannot race transition commit.
- [ ] `cargo check --workspace`, clippy, and workspace tests pass.
- [ ] Parent Epic, Product Backlog, iteration, and Board owners are synchronized.

## Uncertainty

- Exact ownership of the prepared Agent/session bundle must be confirmed from the current
  AppServerSession composition before implementation; if it requires a public API break, stop for
  an ADR and migration plan.

## Documentation

No README behavior is exposed by this enabling Story. Runtime architecture/reference docs must be
updated if ownership boundaries change.

## Required Reads

- `docs/backlog/active/SESSION-001-interactive-session-lifecycle.md`
- `docs/backlog/active/MEM-002-conversation-context-continuity.md`
- `docs/backlog/active/MEM-004-workspace-session-topology.md`
- `docs/decisions/005-tui-event-architecture.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `crates/talos-session/src/lib.rs`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-cli/src/tui_bridge.rs`
