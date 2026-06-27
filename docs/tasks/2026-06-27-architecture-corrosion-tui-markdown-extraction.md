# 2026-06-27 Architecture Corrosion TUI Markdown Extraction Task

**Status**: Complete
**Owner story**: `ARCH-016`
**Iteration**: `I063`
**Requested outcome**: Continue architecture optimization by reducing `scrollback.rs` Markdown and
history segment rendering responsibilities without changing TUI behavior.

## Implementation Record

| Area | Before | After |
|---|---|---|
| Markdown/history segment helpers | Embedded in `scrollback.rs` | `scrollback_markdown.rs` |
| Scrollback root | 1386 lines | 756 lines |

## Validation Evidence

- 2026-06-27: `cargo clippy -p talos-tui -- -D warnings` passed.
- 2026-06-27: `cargo test -p talos-tui --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed.
- 2026-06-27: `git diff --check` passed.

## Residuals

- `scrollback.rs` now mostly owns viewport components and history-message assembly.
- `app.rs` and agent-side modules remain separate architecture cleanup candidates.
