# ARCH-016: TUI Scrollback Markdown Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I063
**Source**: Continued architecture optimization after ARCH-015
**Depends on**: ARCH-015 complete

## Problem

After ARCH-015 extracted input and status helpers, `crates/talos-tui/src/scrollback.rs` still mixed
viewport components and history-message assembly with Markdown, code block, Mermaid, horizontal
rule, fill-segment, and table rendering. This kept visual formatting logic embedded in the
scrollback root.

## Scope

- Extract Markdown/history segment rendering helpers from `scrollback.rs`.
- Preserve the existing `crate::scrollback::*` call surface for app code and tests.
- Do not change Markdown styling, code block formatting, Mermaid fallback behavior, table
  rendering, horizontal rule rendering, or scrollback insertion behavior.

## Acceptance Criteria

- [x] Markdown/code/table rendering helpers move out of `scrollback.rs`.
- [x] Existing `crate::scrollback::*` imports remain compatible.
- [x] TUI targeted clippy and tests pass.
- [x] Workspace fmt/check/clippy pass.
- [x] Workspace tests pass.
- [x] Governance validation passes.
- [x] Remaining TUI decomposition work is recorded.

## Implementation Notes

- Added `crates/talos-tui/src/scrollback_markdown.rs`.
- Moved Markdown segment rendering, code block rendering, Mermaid rendering, table block rendering,
  horizontal-rule helpers, width/fill helpers, and inline delimiter parsing out of `scrollback.rs`.
- `scrollback.rs` re-exports the moved helpers to avoid app/test call-site churn.

## Verification Evidence

- 2026-06-27: `cargo clippy -p talos-tui -- -D warnings` passed.
- 2026-06-27: `cargo test -p talos-tui --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

- `crates/talos-tui/src/scrollback.rs` is now 756 lines and mostly owns viewport components plus
  history-message assembly. A later slice can decide whether history replay deserves its own module.
- `crates/talos-tui/src/app.rs` still mixes event handling, frame assembly, and terminal cursor
  management; it should get a separate owner story.
- `crates/talos-agent/src/compaction.rs`, `prompt.rs`, and `session.rs` remain agent-side
  production candidates.
