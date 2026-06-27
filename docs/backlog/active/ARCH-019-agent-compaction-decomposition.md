# ARCH-019: Agent Compaction Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I066
**Long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`
**Depends on**: ARCH-018 complete; architecture debt burn-down T5 boundary map

## Problem

`crates/talos-agent/src/compaction.rs` mixes public types, policy defaults, layer execution,
status reporting, helper functions, and a large test suite in one 1447-line module. This makes
future MEM-003/MEM-007 work harder to review because behavior changes and structural changes are
not isolated.

## Scope

- Split compaction into focused modules for constants, policy, public result/status types, the
  `Compactor` engine, and tests.
- Preserve existing `talos_agent::compaction::{Compactor, CompactionPolicy, CompactionError,
  CompactionResult, CompactionStatus}` imports through re-exports.
- Keep deterministic layers, LLM-deferred layers, circuit breaker semantics, trigger thresholds,
  hidden-output status behavior, and prompt text unchanged.
- Do not implement MEM-003 LLM compaction proof work or MEM-007 active context compression.

## Acceptance Criteria

- [x] Owner story and iteration exist before code edits.
- [x] `compaction.rs` becomes a small module entrypoint with focused child modules.
- [x] Existing compaction tests pass without behavior changes.
- [x] Public compaction imports remain compatible.
- [x] Workspace gates pass.
- [x] Governance validation passes.
- [x] Remaining compaction/prompt/session architecture residuals are recorded.

## Implementation Notes

- Planned target modules:
  - `crates/talos-agent/src/compaction/constants.rs`
  - `crates/talos-agent/src/compaction/policy.rs`
  - `crates/talos-agent/src/compaction/types.rs`
  - `crates/talos-agent/src/compaction/engine.rs`
  - `crates/talos-agent/src/compaction/tests.rs`
- `compaction.rs` should keep the module-level documentation and public re-exports.

## Verification Evidence

- 2026-06-27: `crates/talos-agent/src/compaction.rs` reduced from 1447 to 41 lines.
- 2026-06-27: Added focused child modules under `crates/talos-agent/src/compaction/`:
  `constants.rs`, `policy.rs`, `types.rs`, `engine.rs`, and `tests.rs`.
- 2026-06-27: `cargo test -p talos-agent --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

- MEM-003 LLM compaction proof and provider-through-session architecture.
- MEM-007 pre-entry active context compression.
- Prompt builder/cache-boundary decomposition in `crates/talos-agent/src/prompt.rs`.
- Session runtime/turn orchestration decomposition in `crates/talos-agent/src/session.rs`.
