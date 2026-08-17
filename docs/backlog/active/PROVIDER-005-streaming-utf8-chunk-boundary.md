# PROVIDER-005: Streaming UTF-8 Chunk Boundary Integrity

| Field | Value |
|---|---|
| Story ID | PROVIDER-005 |
| Type | Provider / Runtime Reliability Fix |
| Priority | P0 Emergency |
| Status | Review — emergency containment implemented in PR #271 |
| Source | [GitHub Issue #270](https://github.com/wjhuang88/talos/issues/270) |
| Selected Iteration | None — Emergency override |
| Depends On | Existing OpenAI-compatible and Anthropic SSE adapters |
| Implementation Commit | `1d31847a` |
| Completion Commit | Pending |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 emergency provider containment session 2026-08-17 |
| Work Slice | Preserve valid UTF-8 code points split across provider transport chunks in OpenAI-compatible and Anthropic SSE while retaining terminal errors for invalid bytes and incomplete EOF. No retry, protocol, permission, persistence, dependency, release, model configuration or TUI layout change. |
| Claimed At | 2026-08-17 |
| Source Issue | #270 |
| Governance Claim PR | #271 |
| Authorization Mode | Emergency override |
| Authorization Evidence | Maintainer supplied a live TUI failure screenshot and explicitly requested urgent handling on 2026-08-17. Issue #270 records the incident, exact scope, validation and rollback. |
| Implementation PR | #271 |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Exact-head CI, workspace preflight and independent review must pass before merge; rollback is revert of implementation commit `1d31847a`. |

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
- [ ] Full workspace release preflight and exact-head CI pass.
- [ ] Independent review confirms both adapters preserve fail-closed behavior.

## Completion Evidence

Completion Commit: Pending. Implementation commit `1d31847a` predates any future status-only
closeout, but this Story remains Review until merge and required exact-head evidence complete.

## Residual Destination

Reproduce and route the Markdown table border/wrapping observation to a TUI owner separately if it
persists after the provider fix.
