# ARCH-028: OpenAI Request Assembly Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-28
**Parent**: Two-month architecture optimization M8
**Selected iteration**: I073

## Problem

`crates/talos-provider/src/openai.rs` mixed provider construction, HTTP retry/error handling, SSE
stream parsing, request DTOs, secret redaction, and Chat Completions request body assembly. The
request assembly path is a maintenance hotspot because message mapping, tool schema serialization,
empty-content fallbacks, and debug snapshots all depend on it.

Keeping that logic in the provider root makes protocol changes harder to review and increases the
risk of accidentally changing request semantics while touching unrelated streaming or transport
code.

## Scope

- Move OpenAI Chat Completions request DTOs into a focused module.
- Move `build_request_body`, empty-content fallback constants, and `redact_secret` into that
  module.
- Preserve request body shape, secret redaction output, base URL handling, HTTP retry behavior,
  and SSE parsing behavior.

## Out of Scope

- Provider protocol changes.
- Reasoning/thinking fields.
- Retry, rate-limit, authentication, or server-error behavior changes.
- Streaming parser changes.
- Anthropic provider decomposition.

## Acceptance Criteria

- [x] `talos-provider/src/openai.rs` loses request assembly responsibility.
- [x] OpenAI request DTOs and body construction are centralized in a focused module.
- [x] `cargo test -p talos-provider --quiet` passes.
- [x] Workspace quality gates pass.
- [x] Governance validation passes.

## Duplicate-Logic Disposition

OpenAI request assembly remains centralized in `openai_request.rs`. No new duplicate request-body
builder, secret redactor, or empty-content fallback path was introduced.

## Execution Notes

- Added `crates/talos-provider/src/openai_request.rs`.
- Updated `crates/talos-provider/src/openai.rs` to import request assembly and redaction helpers.
- `crates/talos-provider/src/openai.rs` dropped from 1001 to 848 lines.
- `crates/talos-provider/src/openai_request.rs` is 169 lines.
- Validation passed: `cargo test -p talos-provider --quiet`, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and
  `git diff --check`.
