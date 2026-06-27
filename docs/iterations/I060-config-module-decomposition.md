# Iteration I060: Config Module Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue the architecture optimization task by splitting the next concrete
>   oversized production module, `talos-config/src/lib.rs`, without changing config behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `talos-config` no longer has a 2000+ line crate root, public API imports still
>   work, and config tests pass.

## Published Baseline

### Non-Terminal Iteration Inventory

| Iteration | Current State | Disposition |
|---|---|---|
| I011 | Paused | Not reopened. |
| I018 | Planned | Not reopened. |
| I019-I020 | Review | Not reopened. |
| I028 | Planned | Deferred. |
| I047 | Review | Awaiting release workflow evidence; no release action in I060. |
| I048 | Planned | Not activated. |
| I049-I058 | Review | Preserve execution records; no status rewrites. |
| I059 | Complete | ARCH-012 memory split is complete; I060 is a separate follow-up. |
| R27 | In Progress | Not granted personal approval authority. |

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-013` | Architecture residual cleanup | Promoted from remaining oversized-module audit | ARCH-012 complete; ADR-023 | Split `talos-config` crate root into focused modules with tests preserved. |

## Scope

- Split config errors, DTOs, credentials, config behavior, built-in providers, env helpers, and tests.
- Keep `talos_config::*` public re-exports stable.
- Preserve ADR-023 inline API-key persistence and Debug masking behavior.
- Run targeted and workspace validation.

## Non-Goals

- No config schema or provider behavior change.
- No changes to README/user docs; behavior is intentionally unchanged.
- No decomposition of CLI/TUI/agent modules in this slice.

## Acceptance

- [x] `crates/talos-config/src/lib.rs` is under 100 lines and acts as a re-export root.
- [x] Production responsibilities are separated by module.
- [x] `cargo test -p talos-config` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | Promoted ARCH-013 after user correction that ARCH-012 covered only one oversized module. |
| 2026-06-27 | Split `talos-config` into `error.rs`, `types.rs`, `credentials.rs`, `config.rs`, `builtin.rs`, `env.rs`, `tests.rs`, and a 28-line `lib.rs` re-export root. |
| 2026-06-27 | `cargo test -p talos-config` passed: 89 unit tests and 1 doc-test. |
| 2026-06-27 | Workspace gates passed: `cargo fmt --all -- --check`; `cargo check --workspace`; `cargo clippy --workspace -- -D warnings`; `cargo test --workspace`; `scripts/validate_project_governance.sh .`; `git diff --check`. |

## Validation Plan

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Closure State

I060 is Complete. No config schema, provider behavior, API-key persistence, Debug masking, or public
`talos_config::*` import change was intended. Remaining oversized modules are residual candidates
and need their own owner stories before further decomposition.

## Documentation To Update

- `docs/backlog/active/ARCH-013-config-module-decomposition.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- `docs/iterations/README.md`
