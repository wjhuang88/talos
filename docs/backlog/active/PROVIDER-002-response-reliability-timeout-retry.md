# PROVIDER-002: Provider Response Reliability, Timeout, And Retry

| Field | Value |
|-------|-------|
| Story ID | PROVIDER-002 |
| Priority | P1 |
| Status | Complete (I084/UX103-UX105, 2026-07-03) |
| Origin | UX-001, maintainer feedback 2026-07-03 |
| Relates To | UX-001, MODEL-003, PROVIDER-001, TUI-020 |

## Requirement

Provider calls must not leave the user staring at an indeterminate "processing" state when the
network, gateway, or model is slow or failing. Talos needs bounded timeout detection, retry/backoff,
and user-visible retry/failure states.

## Scope

### Timeout Policy

- Add a **first-packet timeout**: maximum duration from request dispatch to the first usable stream
  event. Default target: 30 seconds, configurable later if needed.
- Add a **stream-idle timeout**: maximum duration between stream packets after the first packet.
  Default target: 90 seconds for provider streams, with tests using small durations.
- Treat user cancellation as higher priority than timeout/retry.
- Convert timeout into a structured provider error and TUI-visible state; do not panic or silently
  drop the stream.

### Retry And Backoff Policy

- Retry only before irreversible user-visible completion:
  - Safe: no provider event has been emitted beyond `TurnStart`, or only transient status events have
    been emitted.
  - Unsafe by default: tool calls started, assistant text emitted, or final usage emitted.
- Retryable failures:
  - HTTP 408, 409, 425, 429, and 5xx.
  - Connect timeout, request timeout, broken stream before first payload, and temporary transport
    failures.
- Non-retryable failures:
  - HTTP 400/401/403/404, schema/config errors, missing credentials, unsupported model, permission
    errors, malformed request body.
- Use exponential backoff with jitter:
  - Default attempts: 3 total attempts.
  - Base delay: 500 ms.
  - Max delay: 8 seconds.
  - Jitter: +/- 20% deterministic-testable helper.
- Every retry emits structured status so the TUI can show the attempt number and short reason.

### User-Facing Status

Provider runtime should support these states without requiring TUI-specific code in provider crates:

- `connecting`: request sent, no first packet yet.
- `retrying`: retry scheduled or in progress, with attempt counter.
- `thinking`: provider emitted normalized thinking/reasoning content.
- `generating`: answer text is streaming.
- `timed_out`: first-packet or idle timeout reached.
- `failed`: non-retryable error or retry budget exhausted.
- `cancelled`: user interrupted the active request.

## Proposed Implementation

1. Add provider reliability config types in `talos-core` or `talos-config` only after checking
   existing config boundaries:
   - `first_packet_timeout_ms`
   - `stream_idle_timeout_ms`
   - `max_attempts`
   - `backoff_base_ms`
   - `backoff_max_ms`
2. Add a small retry classifier in `talos-provider`:
   - `ProviderFailureKind`
   - `RetryDecision`
   - `BackoffPolicy`
3. Wrap OpenAI-compatible and Anthropic request execution with the classifier and timeout guards.
4. Keep stream parsing provider-specific, but normalize retry/timeout output into `AgentEvent` or a
   sibling status event decided by the implementation slice.
5. Add mock response fixtures:
   - no first packet
   - idle after first text
   - 429 then success
   - 500 then exhausted
   - 401 no retry
   - malformed stream chunk continues or fails according to current parser policy
6. Add TUI/conversation tests verifying status changes do not create durable history entries.

## Acceptance Criteria

- [x] OpenAI-compatible provider has first-packet timeout, idle timeout, retry classification, and
      backoff tests. (UX103/UX104)
- [x] Anthropic provider has equivalent timeout/retry coverage. (UX103/UX104)
- [x] Retrying is not attempted after assistant text/tool-call output has begun unless a later ADR
      explicitly introduces resumable streams. (Retry only in send_request, before streaming starts)
- [x] TUI/conversation can display retry and timeout states without duplicating assistant messages.
      (UX105: TurnPhase states)
- [x] Configuration defaults are documented and do not require users to tune values for normal use.
      (ProviderTimeoutConfig with defaults; config.reference.toml documented)
- [x] `cargo test -p talos-provider -p talos-conversation -p talos-tui` passes. (1497 workspace tests pass)

## Non-Goals

- No multi-provider failover in this story.
- No resumable generation protocol.
- No automatic model switching.
- No retry of write-capable tool execution.
- No change to permission defaults.

## Required Reads

- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/lib.rs`
- `crates/talos-provider/src/openai_request.rs`
- `crates/talos-core/src/provider.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-tui/src/app.rs`
- `docs/backlog/active/MODEL-003-reasoning-thinking-support.md`
- `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`
