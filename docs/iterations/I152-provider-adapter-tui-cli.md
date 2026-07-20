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

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ exit 0 |
| `cargo test --workspace --locked` | ✅ all pass |
| `scripts/validate_project_governance.sh .` | ✅ 0 warnings |
| `git diff --check` | ✅ clean |

## Remaining: Real Terminal Acceptance

- TUI attachment UX (attach/list/remove/cancel/pre-send/capability-gating) — implementation plumbing.
- CLI `--attach` parameter or safe rejection.
- End-to-end integration with `image_validation` module.
- Real-terminal walkthrough (requires human verifier).
