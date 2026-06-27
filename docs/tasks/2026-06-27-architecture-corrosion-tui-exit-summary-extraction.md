# 2026-06-27 Architecture Corrosion: TUI Exit Summary Extraction

**Status**: Complete
**Parent task**: `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
**Iteration**: I070
**Backlog story**: ARCH-025

## Requested Outcome

Reduce TUI app-root corrosion by extracting exit-summary formatting while preserving the existing
visible output.

## Artifacts To Change

- `crates/talos-tui/src/app_summary.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/lib.rs`
- `docs/backlog/active/ARCH-025-tui-exit-summary-decomposition.md`
- `docs/iterations/I070-tui-exit-summary-decomposition.md`
- `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
- `docs/iterations/README.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`

## Success Criteria

- Exit-summary formatting is owned by a focused module.
- `app.rs` drops materially below the 1118-line baseline.
- TUI targeted tests and workspace validation pass.
- Owner docs and derived Board state agree.

## Checkpoints

| Date | State | Evidence | Next |
|---|---|---|---|
| 2026-06-27 | Started | Selected exit-summary formatting because it does not touch frame/cursor/input behavior. | Extract helper and run validation. |
| 2026-06-27 | Complete | `app.rs` 1118→1005 lines; `app_summary.rs` owns exit-summary formatting; `cargo test -p talos-tui --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Return to the two-month plan at M5. |
