# Iteration I152: MODEL-009-D Provider Adapter And TUI/CLI Interaction

> Document status: Review
> Published plan date: 2026-07-20
> Activated: 2026-07-20

## Published Baseline

- Selected Ready story: MODEL-009-D, under ADR-050.
- OpenAI-compatible adapter emits protocol-native image request content (data URL).
- Anthropic-compatible adapter emits protocol-native image request content (base64 source).
- Fixture tests prove multi-part text/image ordering and request shape.
- `catch_unwind` at file read boundary in both adapters (AGENTS.md Hard Constraint #9).
- TUI attachment UX: not yet wired (implementation plumbing for future session).
- CLI: safe rejection with documented pointer to TUI path (not yet implemented).

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Implementation | OpenAI adapter: `OpenAIMessage.content` changed from `Option<String>` to `Option<Value>`; Multimodal arm constructs array of `image_url` parts with data URLs. Anthropic adapter: Multimodal arm constructs `image` content blocks with base64 source. Added `base64` dependency to `talos-provider`. |
| 2026-07-20 | Safety | Added `catch_unwind` wrapping `std::fs::read` in both adapters — handles I/O errors and panics gracefully (empty bytes + tracing::warn). |
| 2026-07-20 | Tests | 3 wire mapping fixture tests: OpenAI image_url data URL shape, OpenAI text-only array, Anthropic image base64 source shape. |
| 2026-07-21 | Security rework | Added capability fail-closed gating, SEC-001/ADR-047 attachment authorization, real decode and pixel limit, content-digest verification at provider read, `/attachments`/`/detach`, print-mode `--attach`, and safe multimodal scrollback summaries. |
| 2026-07-22 | Owner acceptance | P1-A/P1-B code paths accepted after canonical-path authorization and bounded byte-snapshot remediation. Commit `17e3fef` adds regressions for approved-symlink drift and actual-snapshot oversize rejection. Real-terminal evidence remains required before Complete. |

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ exit 0 |
| `cargo test --workspace --locked` | ✅ all pass |
| `scripts/validate_project_governance.sh .` | ✅ 0 warnings |
| `git diff --check` | ✅ clean |

## Remaining: Real Terminal Acceptance

- Real-terminal walkthrough (requires human verifier): Supported/Unsupported/Unknown attachment gate; external-path approval; attach/list/detach/send; OpenAI-compatible and Anthropic-compatible configured-provider behavior; history safe summary; text-only regression.
- A live credential/provider check is maintainer-owned and remains separate from deterministic mock coverage.
