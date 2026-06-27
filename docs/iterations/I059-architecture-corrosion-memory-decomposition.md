# Iteration I059: Architecture Corrosion And Memory Module Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Run a focused architecture-corrosion audit and close one oversized-module
>   decomposition slice without changing user-visible behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `talos-memory` no longer has a 2000+ line crate root, public API imports still
>   work, and memory tests pass.

## Published Baseline

### Non-Terminal Iteration Inventory

| Iteration | Current State | Disposition |
|---|---|---|
| I011 | Paused | Not reopened. |
| I018 | Planned | Not reopened. |
| I019-I020 | Review | Not reopened; memory/exploration behavior remains unchanged. |
| I028 | Planned | Deferred. |
| I047 | Review | Awaiting release workflow evidence; no release action in I059. |
| I048 | Planned | Not activated. |
| I049-I058 | Review | Preserve execution records; no status rewrites. |
| R27 | In Progress | Not granted personal approval authority; I059 is a separate architecture task. |

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-012` | Architecture residual cleanup | Promoted from new audit evidence | ARCH-011 discipline; ADR-016 | Split `talos-memory` crate root into focused modules with tests preserved. |

### Corrosion Rubric

- **Hard constraint**: no public behavior or schema change.
- **Soft constraint**: prefer a module split that preserves `talos_memory::*` imports over a deeper
  API redesign.
- **Assumption**: line count is not sufficient alone; promotion requires responsibility mixing and
  likely future change pressure.

### Audit Result

Top oversized source files before this slice included:

| File | Lines | Disposition |
|---|---:|---|
| `crates/talos-memory/src/lib.rs` | 2141 | Selected: concrete boundary corrosion after memory feature growth. |
| `crates/talos-config/src/lib.rs` | 2083 | Residual candidate; needs separate owner story. |
| `crates/talos-cli/src/mode_runners.rs` | 2062 | Residual candidate; already had one I046 extraction. |
| `crates/talos-agent/src/tests.rs` | 1861 | Watch-only unless test maintenance degrades. |
| `crates/talos-tui/src/scrollback.rs` | 1614 | Remains under ARCH-011 promotion rule. |

## Scope

- Split memory domain types, store implementation, entity extraction, prompt formatting, and tests.
- Keep all public re-exports in `lib.rs`.
- Run targeted memory tests and workspace verification.
- Sync backlog, board, and task records.

## Non-Goals

- No memory schema migration beyond existing schema v2.
- No changes to prompt content, hidden-output filtering policy, or retention dry-run semantics.
- No decomposition of `talos-config`, `mode_runners`, TUI files, or agent files in this iteration.

## Acceptance

- [x] `crates/talos-memory/src/lib.rs` is under 100 lines and acts as a re-export root.
- [x] No touched production module exceeds the previous 2141-line root; responsibilities are
  separated by file.
- [x] `cargo test -p talos-memory` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | Promoted ARCH-012 after audit found `talos-memory/src/lib.rs` at 2141 lines with storage, prompt, entity, type, and test responsibilities mixed. |
| 2026-06-27 | Split `talos-memory` into `types.rs`, `store.rs`, `entities.rs`, `prompt.rs`, `tests.rs`, and a 39-line `lib.rs` re-export root. |
| 2026-06-27 | `cargo test -p talos-memory` passed: 48 tests, 0 failures. |
| 2026-06-27 | Workspace gates passed: `cargo fmt --all -- --check`; `cargo check --workspace`; `cargo clippy --workspace -- -D warnings`; `cargo test --workspace`; `scripts/validate_project_governance.sh .`; `git diff --check`. |
| 2026-06-27 | First `cargo test --workspace` run hit a transient MCP client e2e evidence assertion; immediate targeted rerun passed, and second full workspace test passed. |

## Validation Plan

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Closure State

I059 is Complete. No runtime behavior, schema, or public memory API change was intended. Residual
oversized modules are recorded in ARCH-012 and should not be split without their own owner stories.

## Documentation To Update

- `docs/backlog/active/ARCH-012-memory-module-decomposition.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- `docs/iterations/README.md`
- `docs/tasks/2026-06-27-architecture-corrosion-memory-decomposition.md`
