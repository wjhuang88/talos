# ARCH-017: CLI Print Mode Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I064
**Long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`
**Depends on**: ARCH-014 complete; architecture debt burn-down T0 complete

## Problem

`crates/talos-cli/src/mode_runners.rs` is still the largest production module after ARCH-014. It
mixes RPC, print, TUI, inline, interactive, MCP server, model/session lifecycle, and session command
flows. The safest next split is the print-mode runner because it has a clear entrypoint and a
bounded output loop.

## Scope

- Extract `run_print_mode` into a focused CLI mode module.
- Preserve the `crate::mode_runners::run_print_mode` call surface through re-export.
- Do not change prompt resolution, provider setup, MCP fixture behavior, runtime skill activation,
  memory prompt setup, context-file loading, session creation, output printing, or error handling.

## Acceptance Criteria

- [x] Owner story and iteration exist before code edits.
- [x] Print mode code moves out of `mode_runners.rs`.
- [x] Existing main dispatch remains source-compatible.
- [x] `cargo test -p talos-cli --quiet` passes.
- [x] Workspace gates pass.
- [x] Governance validation passes.
- [x] Remaining CLI flow split work is recorded.

## Implementation Notes

- Planned target module: `crates/talos-cli/src/mode_print.rs`.
- `mode_runners.rs` should re-export `run_print_mode` to avoid changing `main.rs` imports in this
  slice.

## Validation Evidence

- 2026-06-27: `cargo clippy -p talos-cli -- -D warnings` passed.
- 2026-06-27: `cargo test -p talos-cli --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

- `run_inline_mode` and `handle_inline_skills_command`.
- TUI mode lifecycle handler setup and bridge forwarder.
- Session command handlers: `/new`, `/resume`, `/fork`, `/delete`, `/model`.
