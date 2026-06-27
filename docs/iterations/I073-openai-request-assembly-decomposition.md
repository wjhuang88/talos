# Iteration I073: OpenAI Request Assembly Decomposition

> Document status: Complete
> Published plan date: 2026-06-28
> Planned objective: Continue the technical-debt-zero architecture cycle by extracting OpenAI
>   request assembly from the provider root without changing request or stream behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: OpenAI request DTOs/body construction live outside `openai.rs`, behavior remains
>   unchanged, and provider/workspace gates pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-028` | Two-month architecture optimization M8 | In Progress | M0-M7 complete | Split OpenAI request assembly and request DTOs out of `openai.rs`. |

## Scope

- Move OpenAI request DTOs, request body builder, redaction, and empty-message fallbacks into
  `openai_request.rs`.
- Keep OpenAI HTTP send/retry, endpoint URL, SSE parser, usage extraction, text-tool fallback, and
  public provider API unchanged.
- Preserve existing provider tests.

## Acceptance

- [x] `openai.rs` is materially smaller than the 1001-line baseline.
- [x] Request assembly and DTO logic are isolated in `openai_request.rs`.
- [x] Duplicate request assembly logic is not introduced.
- [x] `cargo test -p talos-provider --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-28 | I073 opened as the M8 provider adapter slice after ARCH-027/I072 completed and was pushed. |
| 2026-06-28 | Extracted OpenAI request DTOs, `build_request_body`, empty-content fallbacks, and `redact_secret` into `openai_request.rs`. |
| 2026-06-28 | `openai.rs` reduced from 1001 to 848 lines; `openai_request.rs` is 169 lines. |
| 2026-06-28 | Targeted validation passed: `cargo test -p talos-provider --quiet`. |
| 2026-06-28 | Full validation passed: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check`. |

## Validation Plan

- `cargo test -p talos-provider --quiet`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Closure State

I073 is complete. No residual OpenAI request assembly extraction or duplicated request-body builder
work is left in this slice.
