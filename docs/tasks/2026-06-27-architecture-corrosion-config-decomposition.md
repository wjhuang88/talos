# 2026-06-27 Architecture Corrosion Config Decomposition Task

**Status**: Complete
**Owner story**: `ARCH-013`
**Iteration**: `I060`
**Requested outcome**: Continue architecture optimization beyond ARCH-012 by splitting another
oversized production module and registering remaining candidates.

## Success Criteria

- [x] A second concrete oversized module is selected from current evidence.
- [x] The selected module is split without public API or behavior changes.
- [x] Targeted tests pass.
- [x] Workspace quality gates pass.
- [x] Governance docs are synchronized.
- [x] Residual modules are recorded for follow-up.

## Implementation Record

| Area | Before | After |
|---|---|---|
| Crate root | `talos-config/src/lib.rs` 2083 lines with all concerns mixed | `lib.rs` 28 lines, docs + modules + re-exports |
| Error type | In root | `error.rs` |
| Public DTOs | In root | `types.rs` |
| Credentials I/O | In root | `credentials.rs` |
| `Config` behavior | In root | `config.rs` |
| Built-in providers | In root | `builtin.rs` |
| Env helpers | In root | `env.rs` |
| Tests | In root | `tests.rs` |

Current `talos-config/src` line counts after split:

| File | Lines |
|---|---:|
| `tests.rs` | 1065 |
| `model.rs` | 541 |
| `config.rs` | 462 |
| `types.rs` | 309 |
| `opencode.rs` | 271 |
| `agents.rs` | 244 |
| `builtin.rs` | 104 |
| `env.rs` | 60 |
| `credentials.rs` | 60 |
| `lib.rs` | 28 |
| `error.rs` | 27 |

## Validation Evidence

- 2026-06-27: `cargo test -p talos-config` passed: 89 unit tests and 1 doc-test.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- 2026-06-27: `git diff --check` passed.

## Residuals

- `crates/talos-cli/src/mode_runners.rs` is now the largest production module and should be the next
  architecture story if the user wants continued decomposition.
- `talos-tui` and `talos-agent` large modules remain candidates, but several are tied to rendering,
  prompt, permission, or compaction behavior and need dedicated owner stories.

## Resume Instructions

This task is closed. If continuing architecture optimization, start a new owner story for
`crates/talos-cli/src/mode_runners.rs` first, because it is now the largest remaining production
module.
