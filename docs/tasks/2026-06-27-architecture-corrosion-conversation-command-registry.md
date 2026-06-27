# 2026-06-27 Architecture Corrosion: Conversation Command Registry Decomposition

**Status**: Complete
**Parent task**: `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
**Iteration**: I072
**Backlog story**: ARCH-027

## Requested Outcome

Reduce `talos-conversation/src/engine.rs` responsibility by moving slash-command registry metadata
and completion logic into a focused module without changing command behavior.

## Artifacts To Change

- `crates/talos-conversation/src/engine.rs`
- `crates/talos-conversation/src/command_registry.rs`
- `crates/talos-conversation/src/lib.rs`
- `docs/backlog/active/ARCH-027-conversation-command-registry-decomposition.md`
- `docs/iterations/I072-conversation-command-registry-decomposition.md`
- `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/iterations/README.md`

## Success Criteria

- Command registry metadata and completion logic live in a focused module.
- Public command registry exports remain stable.
- `engine.rs` drops materially below the 960-line baseline.
- Conversation targeted tests and workspace validation pass.

## Checkpoints

| Date | State | Evidence | Next |
|---|---|---|---|
| 2026-06-27 | Started | Selected command registry extraction because it is the lowest-risk M7 slice and already has help/completion/menu test coverage. | Extract module and run validation. |
| 2026-06-27 | Complete | `talos-conversation/src/engine.rs` 960→739 lines; `command_registry.rs` owns command metadata/completion; `cargo test -p talos-conversation --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Continue M8 provider adapter. |
