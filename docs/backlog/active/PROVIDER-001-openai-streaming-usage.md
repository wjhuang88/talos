# PROVIDER-001: OpenAI-Compatible Streaming Usage Accounting

| Field | Value |
|-------|-------|
| Story ID | PROVIDER-001 |
| Priority | P1 |
| Status | Review |
| Source | [GitHub Issue #12](https://github.com/wjhuang88/talos/issues/12) |
| Relates To | TUI-017, MODEL-004 |

## Requirement

OpenAI-compatible streaming providers must request and parse usage data so status bar token counts,
exit summaries, and cost estimates are non-zero when the provider returns usage.

## Scope

- Add `stream_options.include_usage = true` to OpenAI Chat request payloads.
- Parse usage-only streaming chunks before skipping empty `choices`.
- Remove duplicate or unreachable usage extraction paths.
- Add regression tests for request payload and usage-only chunks.

## Acceptance Criteria

- [x] Streaming request payload includes `stream_options: { include_usage: true }`.
- [x] Usage-only chunks with empty `choices` update input/output token counters.
- [x] Status bar and exit summary receive non-zero usage for compatible providers.
- [x] `cargo test -p talos-provider` passes.

## Execution Notes

- 2026-07-01: Activated in I076/T101. Implementation is in progress; verification pending.
- 2026-07-01: Moved to Review. `parse_sse_stream_retains_usage_only_chunk` verifies usage-only chunks survive the empty-choices path and reach `TurnEnd` usage.

## Verification Evidence

- 2026-07-01: `cargo test -p talos-provider` passed: 48 unit tests, 4 integration tests, 2 doc tests.
- 2026-07-01: `cargo check --workspace` passed.
- 2026-07-01: `cargo clippy -p talos-provider -p talos-tui -- -D warnings` passed.

## Required Reads

- [GitHub Issue #12](https://github.com/wjhuang88/talos/issues/12)
- `crates/talos-provider/src/openai_request.rs`
- `crates/talos-provider/src/openai.rs`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
