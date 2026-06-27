# ARCH-020: Agent Prompt Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I067
**Long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`
**Depends on**: ARCH-019 complete; architecture debt burn-down T7 boundary map

## Problem

`crates/talos-agent/src/prompt.rs` mixes embedded prompt assets, public prompt DTOs, cache marker
conversion, private section metadata, `SystemPromptBuilder` configuration and assembly behavior,
hook/cache integration, and tests in one 1232-line module. This makes prompt text, cache stability,
hidden-output boundaries, and future provider-sensitive changes harder to review independently.

## Scope

- Split prompt assets, public types, section metadata, builder implementation, and tests into
  focused child modules.
- Preserve existing `talos_agent::prompt::{SystemPromptBuilder, ToolDescription, ContextFile,
  ActivatedSkillContext, CacheType, CacheMarker, DEFAULT_IDENTITY, TOOL_CALLING_FORMAT,
  TOOL_CALLING_STRICT, MEMORY_PROMPT}` imports through re-exports.
- Keep prompt text, section order, cache marker byte ranges, hook behavior, memory section
  placement, and stable/dynamic prefix semantics unchanged.
- Do not implement MODEL-003 reasoning fields, MEM-007 active context compression, or any prompt
  wording change.

## Acceptance Criteria

- [x] Owner story and iteration exist before code edits.
- [x] `prompt.rs` becomes a small module entrypoint with focused child modules.
- [x] Existing prompt tests pass without behavior changes.
- [x] Public prompt imports remain compatible.
- [x] Workspace gates pass.
- [x] Governance validation passes.
- [x] Remaining prompt/session architecture residuals are recorded.

## Implementation Notes

- Planned target modules:
  - `crates/talos-agent/src/prompt/assets.rs`
  - `crates/talos-agent/src/prompt/types.rs`
  - `crates/talos-agent/src/prompt/sections.rs`
  - `crates/talos-agent/src/prompt/builder.rs`
  - `crates/talos-agent/src/prompt/tests.rs`
- `prompt.rs` should keep module-level documentation and public re-exports.

## Verification Evidence

- 2026-06-27: `crates/talos-agent/src/prompt.rs` reduced from 1232 to 64 lines.
- 2026-06-27: Added focused child modules under `crates/talos-agent/src/prompt/`:
  `assets.rs`, `types.rs`, `sections.rs`, `builder.rs`, and `tests.rs`.
- 2026-06-27: `cargo test -p talos-agent --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.

## Residual Architecture Candidates

- Session runtime/turn orchestration decomposition in `crates/talos-agent/src/session.rs`.
- Future prompt wording changes must use separate requirements and snapshot/cache-stability review.
- MEM-007 active context compression remains research-gated.
