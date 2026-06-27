# 2026-06-28 Architecture Corrosion: Exploration Types Decomposition

**Status**: Complete
**Parent task**: `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
**Iteration**: I074
**Backlog story**: ARCH-029

## Requested Outcome

Reduce `talos-exploration/src/lib.rs` responsibility by moving public domain entity definitions
into a focused module without changing store, schema, FTS, citation, or ingestion behavior.

## Artifacts To Change

- `crates/talos-exploration/src/lib.rs`
- `crates/talos-exploration/src/types.rs`
- `docs/backlog/active/ARCH-029-exploration-types-decomposition.md`
- `docs/iterations/I074-exploration-types-decomposition.md`
- `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/iterations/README.md`

## Success Criteria

- Exploration domain types live in a focused module.
- Crate-root public type imports remain stable.
- `lib.rs` drops materially below the 1070-line baseline.
- Exploration targeted tests and workspace validation pass.

## Checkpoints

| Date | State | Evidence | Next |
|---|---|---|---|
| 2026-06-28 | Started | Selected exploration type extraction because it is behavior-preserving and avoids SQLite/FTS/schema changes. | Extract module and run validation. |
| 2026-06-28 | Complete | `talos-exploration/src/lib.rs` 1070→958 lines; `types.rs` owns domain entities; `cargo test -p talos-exploration --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Continue M10 secondary audit. |
