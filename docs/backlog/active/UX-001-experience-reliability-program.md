# UX-001: Experience Reliability Program

| Field | Value |
|-------|-------|
| Story ID | UX-001 |
| Priority | P1 |
| Status | Planned |
| Origin | Maintainer feedback 2026-07-03 — thinking compatibility, long waits, timeout detection, and retry behavior directly affect daily experience |
| Owns | MODEL-003, PROVIDER-002, TUI reliability status surfaces |

## Problem

Talos has several user-visible reliability gaps that make an otherwise correct runtime feel stalled,
opaque, or broken:

- Thinking/reasoning preview plumbing exists, but provider-specific reasoning formats are not
  normalized into that path.
- A model request can wait too long before the first packet without a clear timeout or UI state.
- Streaming can become idle after a partial response without distinguishing "still working" from a
  dead connection.
- Transient provider/API failures do not have a documented retry/backoff policy.
- Users see provider failures as low-level errors instead of actionable status and recovery hints.

## Program Shape

Treat these as one experience-reliability program because they share the same boundary: the provider
request/stream pipeline must emit structured state that the conversation engine and TUI can present.

| Slice | Owner | Outcome |
|---|---|---|
| UX-A | MODEL-003 | Provider-specific reasoning/thinking request and stream formats normalize into the Talos preview boundary. |
| UX-B | PROVIDER-002 | First-packet timeout, stream-idle timeout, retry/backoff, and error classification are implemented in provider clients. |
| UX-C | TUI/conversation | The user sees clear states: connecting, retrying, thinking, generating, timed out, failed, cancelled. |
| UX-D | docs/tests | Runtime behavior, config defaults, and failure policy are documented and tested with mock streams. |

## Implementation Principles

- Keep provider-specific format knowledge in `talos-provider`; do not leak vendor JSON shapes into
  `talos-conversation` or `talos-tui`.
- Normalize user-visible stream semantics into `AgentEvent` variants and status metadata.
- Retry only when the request is safe to replay. Do not blindly retry after tool calls or after a
  non-empty final assistant answer has begun.
- Use bounded timeouts and bounded retry attempts by default.
- Surface progress and retry state to the TUI instead of relying on silent waits.
- Preserve the existing hidden-thinking boundary: visible preview is transient unless a future ADR
  explicitly changes persistence.

## Acceptance Criteria

- [x] MODEL-003 ADR and implementation plan are ready before provider request schema changes.
      (ADR-034 v3 accepted 2026-07-03 after architecture review; 7 dimensions resolved.)
- [ ] PROVIDER-002 defines and implements default first-packet timeout, idle timeout, max attempts,
      and exponential backoff with jitter.
- [ ] Provider clients classify HTTP 408/409/425/429/5xx, transport disconnect, DNS/connect
      failures, and malformed stream chunks into retryable/non-retryable categories.
- [ ] TUI/conversation surfaces connecting/retrying/timeout/failure states without duplicating
      assistant text or corrupting session history.
- [ ] Tests cover OpenAI-compatible, Anthropic, and mock-provider paths for thinking chunks,
      no-first-packet timeout, idle timeout, retry success, retry exhaustion, and cancellation.

## Required Reads

- `docs/backlog/active/MODEL-003-reasoning-thinking-support.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/backlog/active/TUI-020-thinking-preview-not-history.md`
- `docs/proposals/reasoning-thinking-field.md`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/lib.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-tui/src/app.rs`
