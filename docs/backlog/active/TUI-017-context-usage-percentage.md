# TUI-017: Context Usage Percentage In Status Bar

| Field | Value |
|-------|-------|
| Story ID | TUI-017 |
| Priority | P2 |
| Status | Review |
| Source | [GitHub Issue #9](https://github.com/wjhuang88/talos/issues/9) |
| Relates To | TUI-011, PROVIDER-001 |

## Requirement

Show context usage percentage in the status bar when `context_limit` is available.

## Scope

- Compute `(input_tokens + output_tokens) / context_limit * 100`.
- Render the percentage next to token usage in the status bar.
- Hide or degrade cleanly in compact layouts.

## Dependency

OpenAI-compatible streaming usage must be fixed first via PROVIDER-001; otherwise affected
providers will continue reporting zero tokens.

## Acceptance Criteria

- [x] Status bar shows percentage when context limit is known.
- [x] Percentage uses input plus output token usage.
- [x] Compact mode remains readable.
- [x] Unit tests cover normal, missing-limit, and compact cases.

## Execution Notes

- 2026-07-01: Activated in I076/T103 after PROVIDER-001 implementation started. Status bar implementation is in progress; verification pending.
- 2026-07-01: Moved to Review. Status bar now renders context usage percentage from input plus output token counts when `context_limit` is known.

## Verification Evidence

- 2026-07-01: `cargo test -p talos-tui status_bar` passed: 14 status-bar tests.
- 2026-07-01: `cargo test -p talos-tui` passed: 180 unit tests, 2 doc tests.
- 2026-07-01: `cargo clippy -p talos-provider -p talos-tui -- -D warnings` passed.

## Required Reads

- [GitHub Issue #9](https://github.com/wjhuang88/talos/issues/9)
- `docs/backlog/active/TUI-011-status-bar-exit-polish.md`
- `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`
- `crates/talos-tui/src/scrollback_status.rs`
- `crates/talos-conversation/src/types.rs`
