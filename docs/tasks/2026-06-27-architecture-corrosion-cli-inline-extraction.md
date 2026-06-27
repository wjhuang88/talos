# 2026-06-27 Architecture Corrosion: CLI Inline Mode Extraction

**Status**: Complete
**Parent task**: `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
**Iteration**: I069
**Backlog story**: ARCH-024

## Requested Outcome

Reduce CLI architecture corrosion by extracting inline mode from the residual CLI runner module
without changing runtime behavior.

## Artifacts To Change

- `crates/talos-cli/src/mode_inline.rs`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-cli/src/main.rs`
- `docs/backlog/active/ARCH-024-cli-inline-mode-decomposition.md`
- `docs/iterations/I069-cli-inline-mode-decomposition.md`
- `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
- `docs/iterations/README.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`

## Success Criteria

- Inline-mode code is owned by a focused module.
- `mode_runners.rs` drops materially below the 1778-line baseline.
- CLI targeted tests and workspace validation pass.
- Owner docs and derived Board state agree.

## Checkpoints

| Date | State | Evidence | Next |
|---|---|---|---|
| 2026-06-27 | Started | Selected inline mode as the first executable ARCH-022 child because its code is a contiguous self-contained flow. | Extract code and run validation. |
| 2026-06-27 | Complete | `mode_runners.rs` 1778→1500 lines; `mode_inline.rs` owns inline runtime and `/skills` handling; `cargo test -p talos-cli --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Return to the two-month plan at M3. |
