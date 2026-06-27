# 2026-06-27 Architecture Corrosion: Agent Configuration Decomposition

**Status**: Complete
**Parent task**: `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
**Iteration**: I071
**Backlog story**: ARCH-026

## Requested Outcome

Reduce `talos-agent/src/lib.rs` responsibility and remove duplicated prompt-builder setter logic
without changing Agent behavior.

## Artifacts To Change

- `crates/talos-agent/src/lib.rs`
- `crates/talos-agent/src/configuration.rs`
- `docs/backlog/active/ARCH-026-agent-configuration-decomposition.md`
- `docs/iterations/I071-agent-configuration-decomposition.md`
- `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/iterations/README.md`

## Success Criteria

- Agent constructor/configuration code lives in a focused module.
- Repeated setter mutation logic is centralized in one helper.
- `lib.rs` drops materially below the 914-line baseline.
- Agent targeted tests and workspace validation pass.

## Checkpoints

| Date | State | Evidence | Next |
|---|---|---|---|
| 2026-06-27 | Started | Selected Agent constructor/configuration as the first M5/M6 slice because it is behavior-preserving and includes clear duplicate setter boilerplate. | Extract module and run validation. |
| 2026-06-27 | Complete | `talos-agent/src/lib.rs` 914→655 lines; `configuration.rs` owns constructors/setters; duplicate prompt-builder mutation centralized; `cargo test -p talos-agent --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Continue M7 conversation engine. |
