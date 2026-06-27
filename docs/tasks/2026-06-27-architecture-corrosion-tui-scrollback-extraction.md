# 2026-06-27 Architecture Corrosion TUI Scrollback Extraction Task

**Status**: Complete
**Owner story**: `ARCH-015`
**Iteration**: `I062`
**Requested outcome**: Continue architecture optimization by reducing `scrollback.rs`
responsibility mixing without changing TUI behavior.

## Implementation Record

| Area | Before | After |
|---|---|---|
| Input helpers | Embedded in `scrollback.rs` | `scrollback_input.rs` |
| Status helpers | Embedded in `scrollback.rs` | `scrollback_status.rs` |
| Scrollback root | 1614 lines | 1386 lines |

## Validation Evidence

- 2026-06-27: `cargo check -p talos-tui` passed.
- 2026-06-27: `cargo clippy -p talos-tui -- -D warnings` passed.
- 2026-06-27: `cargo test -p talos-tui --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed.
- 2026-06-27: `git diff --check` passed.

## Residuals

- `scrollback.rs` remains large because Markdown/history rendering is still in the root module.
- `app.rs` and agent-side modules remain separate architecture cleanup candidates.
