# PROVIDER-001: OpenAI-Compatible Streaming Usage Accounting

| Field | Value |
|-------|-------|
| Story ID | PROVIDER-001 |
| Priority | P1 |
| Status | Planned |
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

- [ ] Streaming request payload includes `stream_options: { include_usage: true }`.
- [ ] Usage-only chunks with empty `choices` update input/output token counters.
- [ ] Status bar and exit summary receive non-zero usage for compatible providers.
- [ ] `cargo test -p talos-provider` passes.

## Required Reads

- [GitHub Issue #12](https://github.com/wjhuang88/talos/issues/12)
- `crates/talos-provider/src/openai_request.rs`
- `crates/talos-provider/src/openai.rs`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
