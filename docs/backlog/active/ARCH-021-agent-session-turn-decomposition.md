# ARCH-021: Agent Session Turn Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I068
**Long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`
**Depends on**: ARCH-020 complete; architecture debt burn-down T9 boundary map

## Problem

`crates/talos-agent/src/session.rs` mixes the `AppServerSession` actor loop, turn forwarding
plumbing, cancellation/error forwarding, history commit record, and a large async test suite in one
1150-line module. This increases review risk around session lifecycle changes because test volume
and turn forwarding details obscure the actor state machine.

## Scope

- Extract turn forwarding data/logic into a focused child module.
- Move the session test suite into a child test module.
- Preserve `AppServerSession` public API, SQ/EQ behavior, cancellation behavior, history commit
  behavior, pre-turn deterministic compaction, and skill-context gating.
- Do not change persistence, session topology, permission behavior, cancellation semantics, or
  memory prompt injection.

## Acceptance Criteria

- [x] Owner story and iteration exist before code edits.
- [x] `session.rs` is reduced materially and keeps the actor loop as the visible owner.
- [x] Existing session tests pass without behavior changes.
- [x] Workspace gates pass.
- [x] Governance validation passes.
- [x] Remaining session architecture residuals are recorded.

## Implementation Notes

- Planned target modules:
  - `crates/talos-agent/src/session/turn.rs`
  - `crates/talos-agent/src/session/tests.rs`
- Keep `TurnRecord`, `TurnForwarding`, and `run_turn_with_forwarding` crate-private to the
  `session` module.

## Verification Evidence

- 2026-06-27: `crates/talos-agent/src/session.rs` reduced from 1150 to 193 lines.
- 2026-06-27: Added `crates/talos-agent/src/session/turn.rs` for turn forwarding and
  `crates/talos-agent/src/session/tests.rs` for session tests.
- 2026-06-27: `cargo test -p talos-agent --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.

## Residual Architecture Candidates

- Future split of `AppServerSession::run` operation handlers if change pressure grows.
- Session persistence/topology changes require separate owner stories and validation.
