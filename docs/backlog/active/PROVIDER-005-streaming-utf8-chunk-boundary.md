# PROVIDER-005: Streaming UTF-8 Chunk Boundary Integrity

**Status**: Complete (2026-08-17)

| Field | Value |
|---|---|
| Story ID | PROVIDER-005 |
| Type | Provider / Runtime Reliability Fix |
| Priority | P0 Emergency |
| Status | Complete |
| Source | [GitHub Issue #270](https://github.com/wjhuang88/talos/issues/270) |
| Selected Iteration | None — Emergency override |
| Depends On | Existing OpenAI-compatible and Anthropic SSE adapters |
| Implementation Commit | `1d31847a` |
| Completion Commit | `1d31847a01f482f3d832b6a935e6d4f23fda555d` |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 emergency provider containment session 2026-08-17 |
| Work Slice | Preserve valid UTF-8 code points split across provider transport chunks in OpenAI-compatible and Anthropic SSE while retaining terminal errors for invalid bytes and incomplete EOF. No retry, protocol, permission, persistence, dependency, release, model configuration or TUI layout change. |
| Claimed At | 2026-08-17 |
| Source Issue | #270 |
| Governance Claim PR | #271 |
| Authorization Mode | Emergency override |
| Authorization Evidence | Maintainer supplied a live TUI failure screenshot and explicitly requested urgent handling on 2026-08-17. Independent review comment `5313112992` approved exact head `f51051c8495accca5292bc90157908bc16e0d6ff`; merge-time CAS produced merge commit `89523dbc2b667ca83587aaf3d2825e69efa18f58`. |
| Implementation PR | #271 |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | None — closed. Rollback remains revert of implementation commit `1d31847a`. |

## Identity / Goal / Value

A valid Chinese, emoji or other multibyte character may cross an arbitrary HTTP transport chunk
boundary. Provider adapters must reconstruct it exactly instead of aborting the active turn with an
invalid UTF-8 error.

## Scope

- One provider-internal incremental decoder retains only an incomplete UTF-8 suffix.
- OpenAI-compatible and Anthropic SSE paths use the same decoder.
- Valid prefixes continue streaming without waiting for unrelated later data.
- Truly invalid bytes and an incomplete code point at EOF remain terminal errors.

## Exclusions

- No lossy decoding, replacement characters or silent byte removal.
- No TUI Markdown table/layout correction; the screenshot's table rendering is a separate display
  observation unless it reproduces after this containment.
- No provider retry, timeout, terminal-outcome, tool-call, permission or persistence change.

## Acceptance And Evidence

- [x] Chinese and emoji code points split across multiple byte chunks round-trip exactly.
- [x] Invalid byte sequences remain rejected.
- [x] Incomplete UTF-8 at EOF remains rejected.
- [x] `cargo test -p talos-provider --locked` passed: 127 unit, 4 integration and 2 doctests.
- [x] `cargo clippy -p talos-provider --all-targets --locked -- -D warnings` passed.
- [x] `cargo fmt --all -- --check` and `git diff --check` passed.
- [x] Full workspace release preflight and exact-head CI `32002811484` passed.
- [x] Independent review comment `5313112992` confirmed both adapters preserve fail-closed behavior
  at exact head `f51051c8495accca5292bc90157908bc16e0d6ff`.

## Completion Evidence

Completion Commit: `1d31847a01f482f3d832b6a935e6d4f23fda555d`. This pre-existing implementation
commit contains the shared incremental decoder and both provider integrations; this status-only
closeout does not cite itself. PR #271 merged as `89523dbc2b667ca83587aaf3d2825e69efa18f58`
after exact-head CI `32002811484`, independent approval `5313112992`, and merge-time CAS confirmed
the reviewed head as the merge commit's second parent.

## Residual Destination

Reproduce and route the Markdown table border/wrapping observation to a TUI owner separately if it
persists after the provider fix.

Reviewer observations N2 (an additional adapter-level split fixture) and N3 (more specific invalid
versus incomplete diagnostic wording) are optional maintenance suggestions, not unmet acceptance
or provider-safety residuals. They do not keep Issue #270 open.
