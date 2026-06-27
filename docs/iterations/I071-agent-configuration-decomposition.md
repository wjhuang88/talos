# Iteration I071: Agent Configuration Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue the technical-debt-zero architecture cycle by extracting Agent
>   constructors/configuration setters and centralizing repeated prompt-builder mutation logic.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `Agent` construction/configuration lives outside `lib.rs`, duplicate setter
>   boilerplate is centralized, and agent/workspace gates pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-026` | Two-month architecture optimization M5/M6 | In Progress | M0-M4 complete | Split Agent configuration and remove duplicate setter mutation boilerplate. |

## Scope

- Move `Agent::new`, `Agent::with_security`, `Agent::with_security_and_hooks`, prompt/tool/skill
  setters, `build_system_prompt`, and `cancellation_token` into a focused module.
- Add a helper for prompt-builder mutation plus optional stable-prefix invalidation.
- Preserve public API and behavior.

## Acceptance

- [x] `lib.rs` is materially smaller than the 914-line baseline.
- [x] Duplicate prompt-builder mutation logic is centralized.
- [x] `cargo test -p talos-agent --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | I071 opened after the user raised the bar from oversized-module cleanup to technical-debt-zero cleanup, including duplicated local logic. |
| 2026-06-27 | Extracted Agent constructors and configuration setters into `configuration.rs`. |
| 2026-06-27 | Centralized repeated prompt-builder mutation plus stable-prefix invalidation into one helper. |
| 2026-06-27 | `lib.rs` reduced from 914 to 655 lines; `configuration.rs` is 242 lines. |
| 2026-06-27 | Targeted validation passed: `cargo test -p talos-agent --quiet`. |
| 2026-06-27 | Full validation passed: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check`. |

## Validation Plan

- `cargo test -p talos-agent --quiet`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Closure State

I071 is complete. No residual Agent configuration or duplicated prompt-builder mutation work is
left in this slice.
