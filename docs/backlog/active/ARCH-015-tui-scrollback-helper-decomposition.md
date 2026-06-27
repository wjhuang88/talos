# ARCH-015: TUI Scrollback Helper Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I062
**Source**: Continued architecture optimization after ARCH-012 through ARCH-014
**Depends on**: ARCH-014 complete

## Problem

`crates/talos-tui/src/scrollback.rs` remained one of the largest production modules after the
memory, config, and CLI helper decompositions. It mixed viewport components, input text rendering,
credential masking, cursor calculations, status bar formatting, history rendering, and Markdown
segment rendering in one file.

## Scope

- Extract low-risk input and status formatting helpers from `scrollback.rs`.
- Preserve the existing `crate::scrollback::*` call surface for app code and tests.
- Do not change viewport layout, scrollback insertion, status content, credential masking behavior,
  Markdown rendering, or terminal control behavior.

## Acceptance Criteria

- [x] Input rendering/cursor/credential helpers move out of `scrollback.rs`.
- [x] Status bar formatting helpers move out of `scrollback.rs`.
- [x] Existing `crate::scrollback::*` imports remain compatible.
- [x] TUI targeted check, clippy, and tests pass.
- [x] Workspace gates pass.
- [x] Governance validation passes.
- [x] Remaining TUI decomposition work is recorded.

## Implementation Notes

- Added `crates/talos-tui/src/scrollback_input.rs` for input line counting, cursor position,
  credential display masking, and input text construction.
- Added `crates/talos-tui/src/scrollback_status.rs` for status line construction, model/provider
  truncation, and usage cost formatting.
- `scrollback.rs` re-exports the moved helpers so existing app/test call sites do not churn.

## Verification Evidence

- 2026-06-27: `cargo check -p talos-tui` passed.
- 2026-06-27: `cargo clippy -p talos-tui -- -D warnings` passed.
- 2026-06-27: `cargo test -p talos-tui --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

- `crates/talos-tui/src/scrollback.rs` is still 1386 lines. The next TUI split should move
  Markdown/history rendering into a dedicated module after this helper split is stable.
- `crates/talos-tui/src/app.rs` still mixes event handling, frame assembly, and terminal cursor
  management; it should get a separate owner story.
- `crates/talos-agent/src/compaction.rs`, `prompt.rs`, and `session.rs` remain agent-side
  production candidates.
