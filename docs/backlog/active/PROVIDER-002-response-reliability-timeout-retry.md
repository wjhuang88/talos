# PROVIDER-002: Provider Response Reliability, Timeout, And Retry

| Field | Value |
|-------|-------|
| Story ID | PROVIDER-002 |
| Priority | P1 |
| Status | Resolved — request-dispatch timeout fixed in I107 SBT111 (non-qualifying REL-002 evidence) |
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
- Add a **request-dispatch timeout**: maximum duration for HTTP request send/response-header wait
  before stream parsing begins. This is distinct from first-packet timeout and is still open.
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
- [x] Anthropic provider has equivalent first-packet/idle timeout and retry coverage. (UX103/UX104)
- [x] OpenAI-compatible and Anthropic providers bound HTTP request dispatch / response-header wait
      so `send().await` cannot hang forever before stream parsing begins. (#18)
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

## I102 D101 Cross-Reference (2026-07-07)

Source: `docs/iterations/I102-provider-runtime-reliability-gate.md` (D101).

- The OpenAI-compatible SSE fixture matrix was extended with six deterministic
  `parse_sse_stream_*` cases that lock protocol paths the original UX103/UX104 fixtures did not
  have an explicit case for: `finish_reason="length" → StopReason::MaxTokens`, role-only first
  chunk, SSE `: keepalive` / `retry:` / empty `data: ` passthrough, mixed content + tool_calls
  in one delta, and multi-byte UTF-8 round-trip.
- No provider behavior change; no new timeout/retry behavior. These fixtures are regression
  guards for the timeout/retry pipeline's downstream `StopReason` mapping and stream-idle
  passthrough. The full I102 evidence lives in `RUNTIME-002` and `I102`.

## 2026-07-08 Status Correction: Request Timeout Gap

The original I084 slice delivered first-packet and stream-idle timeouts, but GitHub Issue #18
identified a different gap: `reqwest::Client::new()` has no request-level timeout, and the
provider can hang in `send().await` before response headers arrive. That path is not protected by
the parser-level first-packet/idle timeout code.

This owner doc is therefore Partial until a request-dispatch timeout is implemented with tests for
both OpenAI-compatible and Anthropic providers.

## I107 SBT111: Dispatch Timeout Implementation (2026-07-09)

The #18 request-dispatch timeout gap identified in the 2026-07-08 Status Correction is now fixed. The fix adds a `dispatch_timeout_secs` field to `ProviderTimeoutConfig` (default 60s) and wraps the provider `send().await` call in `tokio::time::timeout`, so providers that accept the TCP connection but never return response headers now produce a terminal `ProviderError::NetworkError("request dispatch timeout: no response headers within Ns")`. The dispatch timeout is retryable via existing `classify_retry_with_backoff`.

Stream-idle semantics are preserved: the timeout does not cover the streaming body, which remains protected by `first_packet_timeout_secs` and `stream_idle_timeout_secs`.

Evidence:

- Provider layer: `test_dispatch_timeout_openai`, `test_normal_request_not_dispatch_timed_out_openai`,
  `test_dispatch_timeout_anthropic`, and `test_normal_request_not_dispatch_timed_out_anthropic`.
- Agent/runtime bridge: `run_streaming_emits_error_event_on_provider_dispatch_timeout` proves
  `Agent::run_streaming` converts provider dispatch failure into `AgentEvent::Error`.
- Conversation loop: `conversation_loop_clears_processing_on_dispatch_timeout_error` proves the
  terminal UI status has `is_processing=false` and `phase=TimedOut`.
- Runtime: glm-5.2 external -> non-qualifying for REL-002.
