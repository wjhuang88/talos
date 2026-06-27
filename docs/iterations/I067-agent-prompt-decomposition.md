# Iteration I067: Agent Prompt Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue the architecture debt burn-down by decomposing
>   `talos-agent/src/prompt.rs` into focused modules without changing prompt output or cache
>   semantics.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: public prompt imports remain compatible, prompt tests pass, and the root prompt
>   module no longer owns assets, public DTOs, section internals, builder behavior, and tests.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-020` | Architecture debt burn-down | Promoted by T7 | ARCH-019 complete; long-task T7 complete | Split `prompt.rs` into focused child modules. |

## Scope

- Move prompt assets, public DTO/cache marker types, private section metadata, builder
  implementation, and tests into child modules.
- Preserve public import paths through `talos_agent::prompt` re-exports.
- Run agent targeted tests and workspace gates.

## Acceptance

- [x] `prompt.rs` is reduced to module docs, child module declarations, and re-exports.
- [x] Existing `talos_agent::prompt::*` public imports remain compatible.
- [x] `cargo test -p talos-agent --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | I067 opened from the architecture debt burn-down long task after T7 identified a behavior-preserving split across assets, types, sections, builder, and tests. |
| 2026-06-27 | Split `prompt.rs` into `assets.rs`, `types.rs`, `sections.rs`, `builder.rs`, and `tests.rs`, preserving public re-exports through `talos_agent::prompt`. |
| 2026-06-27 | `prompt.rs` reduced from 1232 to 64 lines. |
| 2026-06-27 | Full gates passed: `cargo test -p talos-agent --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace --quiet`. |

## Validation Plan

- `cargo test -p talos-agent --quiet`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residual Work

After this slice, continue the long task with session decomposition.

## Closure State

I067 is complete. This slice intentionally changed only structure; prompt text, prompt section
order, cache marker byte ranges, stable/dynamic prefix behavior, hook behavior, and memory section
placement remain unchanged. Session decomposition continues under the parent long task.
