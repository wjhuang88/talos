# TUI-020: Thinking Preview Without History Pollution

| Field | Value |
|-------|-------|
| Story ID | TUI-020 |
| Priority | P2 |
| Status | Review — I078/T124 |
| Source | [GitHub Issue #15](https://github.com/wjhuang88/talos/issues/15) |
| Relates To | TUI-004, SESSION-002 |

## Requirement

Model thinking content should be visible in the live preview area while streaming, but should not be
inserted into scrollback history or persisted as normal conversation history.

## Scope

- Keep active thinking state separate from finalized history.
- Clear thinking preview when the assistant response finalizes.
- Ensure persisted sessions contain final assistant output, not transient thinking text.

## Decision

Implemented the smallest explicit transient-message boundary:

- `AgentEvent::ThinkingDelta` is a stream-only event for UI preview.
- `UiOutput::ThinkingPreview` carries active preview state to the TUI.
- The conversation engine keeps thinking text separate from `current_turn_text` and clears it on
  turn finalization, error, or cancellation.
- Session JSONL persistence ignores `ThinkingDelta`, so resume history cannot replay old thinking
  content.

## Acceptance Criteria

- [x] Thinking is visible during active streaming.
- [x] Thinking does not appear in finalized history.
- [x] Thinking is not persisted as normal session history.
- [x] Resume does not replay old thinking content.
- [x] Tests cover stream, finalization, persistence, and resume.

## Review Evidence

- Implementation: `crates/talos-core/src/message.rs`, `crates/talos-conversation/src/engine.rs`,
  `crates/talos-conversation/src/types.rs`, `crates/talos-tui/src/app.rs`,
  `crates/talos-tui/src/state.rs`, `crates/talos-session/src/jsonl.rs`,
  `crates/talos-cli/src/mode_runners.rs`.
- Tests: conversation preview/finalization, agent final-history separation, session persistence,
  and resume-history exclusion.
- Validation passed on 2026-07-02:
  - `cargo fmt --all -- --check`
  - `cargo test -p talos-core`
  - `cargo test -p talos-conversation`
  - `cargo test -p talos-session`
  - `cargo test -p talos-agent`
  - `cargo test -p talos-cli`
  - `cargo test -p talos-tui`
  - `cargo clippy -p talos-core -p talos-agent -p talos-conversation -p talos-session -p talos-cli -p talos-tui -- -D warnings`
  - `cargo check --workspace`

## Required Reads

- [GitHub Issue #15](https://github.com/wjhuang88/talos/issues/15)
- `docs/backlog/active/TUI-004-state-model.md`
- `docs/backlog/active/SESSION-002-session-integrity-lifecycle-hardening.md`
- `crates/talos-conversation/src/`
- `crates/talos-tui/src/`
- `crates/talos-session/src/`
