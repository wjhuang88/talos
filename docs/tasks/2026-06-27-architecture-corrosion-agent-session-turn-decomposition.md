# 2026-06-27 Agent Session Turn Decomposition Task

**Status**: Complete
**Owner story**: `docs/backlog/active/ARCH-021-agent-session-turn-decomposition.md`
**Iteration**: `docs/iterations/I068-agent-session-turn-decomposition.md`
**Parent long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`

## Goal

Split session turn forwarding and tests out of `crates/talos-agent/src/session.rs` without
changing the session actor state machine, cancellation, history commit, pre-turn compaction, skill
context, or public API.

## Scope

- Create `session/turn.rs` for turn record/forwarding/run logic.
- Create `session/tests.rs` for the existing async session tests.
- Run targeted and workspace validation.

## Out of Scope

- Persistence or topology changes.
- Permission behavior changes.
- Cancellation semantics changes.
- Memory prompt injection changes.
- New dependencies, network validation, commit, push, tag, or release.

## Plan

| Step | Action | Status |
|---|---|---|
| 1 | Map current session responsibilities and risks. | Complete |
| 2 | Create ARCH-021/I068/task owner records. | Complete |
| 3 | Mechanically split turn forwarding and tests. | Complete |
| 4 | Run targeted agent tests and workspace gates. | Complete |
| 5 | Synchronize owner docs, Board, backlog, iterations README, and long-task checkpoint. | Complete |

## Boundary Map

- Public API: `AppServerSession::new` and `AppServerSession::run`.
- Actor-loop responsibilities retained in `session.rs`: SQ receive loop, active turn handle,
  cancellation token ownership, deterministic pre-turn compaction, skill context gating, history
  commits, and shutdown.
- Extracted turn responsibilities: forwarding `AgentEvent` to `SessionEvent`, cancellation race,
  agent task result mapping, panic/error reporting, and `TurnRecord` handoff.

## Validation Evidence

- 2026-06-27: `crates/talos-agent/src/session.rs` reduced from 1150 to 193 lines.
- 2026-06-27: `cargo test -p talos-agent --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.

## Residual Work

- Final architecture audit in the parent long task after this slice closes.
