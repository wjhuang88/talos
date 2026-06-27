# Iteration I063: TUI Scrollback Markdown Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue architecture optimization by extracting Markdown and history segment
>   rendering from `talos-tui/src/scrollback.rs` without changing TUI behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Markdown/code/table rendering helpers live outside `scrollback.rs`, existing
>   `crate::scrollback::*` call sites keep working, and TUI targeted tests pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-016` | Architecture residual cleanup | Promoted from remaining TUI scrollback residual | ARCH-015 complete | Extract `scrollback_markdown.rs`. |

## Scope

- Move Markdown, code block, Mermaid, table, horizontal-rule, and fill-segment helpers out of
  `scrollback.rs`.
- Keep viewport component behavior and app/test call paths unchanged.
- Run TUI targeted gates and workspace gates.

## Acceptance

- [x] `scrollback_markdown.rs` exists and owns Markdown/history segment rendering helpers.
- [x] Existing `crate::scrollback::*` call sites remain valid.
- [x] `cargo clippy -p talos-tui -- -D warnings` passes.
- [x] `cargo test -p talos-tui --quiet` passes.
- [x] Workspace fmt/check/clippy pass.
- [x] Workspace tests pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | Added `scrollback_markdown.rs` and moved Markdown, code block, Mermaid, table, horizontal-rule, width/fill, and inline delimiter helpers out of `scrollback.rs`. |
| 2026-06-27 | `scrollback.rs` reduced from 1386 to 756 lines while preserving `crate::scrollback::*` call paths. |
| 2026-06-27 | TUI targeted clippy/tests and workspace fmt/check/clippy passed. |
| 2026-06-27 | Full gates passed: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check`. |

## Validation Plan

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residual Work

The remaining scrollback root owns viewport components and history-message assembly. `app.rs` and
agent-side compaction/prompt/session modules remain separate architecture cleanup candidates.

## Closure State

I063 is complete. This slice intentionally stops after moving Markdown/history segment rendering;
further TUI cleanup should use a new owner story for either scrollback history replay or `app.rs`
event/frame assembly.
