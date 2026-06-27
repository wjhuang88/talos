# ARCH-014: CLI Mode Runtime Helper Extraction

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I061
**Source**: Continued architecture optimization after ARCH-012 and ARCH-013
**Depends on**: ARCH-013 complete

## Problem

After memory and config decomposition, `crates/talos-cli/src/mode_runners.rs` remained the largest
production module. It mixes top-level mode runners with reusable runtime helpers for session model
metadata, memory prompt provider setup, model context files, and MCP fixture configuration.

## Scope

- Extract helper responsibilities that do not need to live inside the mode-runner orchestration file.
- Preserve existing `crate::mode_runners::*` test-facing helpers where needed or update tests to the
  new owner module.
- Do not change mode behavior, command routing, session lifecycle, provider setup, or permissions.

## Acceptance Criteria

- [x] Runtime helper code moves out of `mode_runners.rs`.
- [x] `mode_runners.rs` line count is reduced and remains behavior-preserving.
- [x] CLI targeted tests pass.
- [x] Workspace gates pass.
- [x] Governance validation passes.
- [x] Remaining runner decomposition work is recorded.

## Implementation Notes

- Added `crates/talos-cli/src/mode_runtime.rs`.
- Moved:
  - session model metadata creation/restoration;
  - memory prompt provider setup;
  - model metadata context file generation;
  - agent context file loading;
  - MCP fixture config override.
- `mode_runners.rs` now imports these helpers and keeps the mode orchestration code.

## Verification Evidence

- 2026-06-27: `cargo test -p talos-cli` passed, including CLI unit tests and e2e tests.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

- `crates/talos-cli/src/mode_runners.rs` is still 1912 lines. The next CLI decomposition should split
  print/inline/TUI/session command flows into separate modules with their own owner story.
- `crates/talos-tui/src/scrollback.rs`, `crates/talos-tui/src/app.rs`, and
  `crates/talos-agent/src/compaction.rs` remain production candidates.
