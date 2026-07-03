# 2026-07-03 Experience Reliability Plan

**Status**: Planned  
**Owner area**: Provider and TUI user experience reliability.  
**Created**: 2026-07-03  
**Priority**: P1, ahead of plugin/distribution work unless maintainer explicitly restores the prior order.

## Objective

Make Talos feel responsive and trustworthy during model calls. The first reliability slice must solve
the user-visible gaps around provider thinking formats, long waits with no packets, transient API
failures, retry/backoff, and clear TUI state.

This is a plan and implementation authorization for a focused UX reliability series. Provider request
schema changes for reasoning/thinking still require the ADR gate named in MODEL-003 before code.

## Why This Moves Up

The current frontline plan improves extension and distribution surfaces, but daily use depends first
on whether the active model call is observable, bounded, and recoverable. A stalled request, missing
thinking preview, or raw provider error is experienced as a broken agent even if plugins and docs are
well designed.

## Tracks

| Track | Theme | Outcome |
|---|---|---|
| UX-A | Reasoning/thinking compatibility | Provider-specific thinking fields normalize into Talos preview events. |
| UX-B | Provider response reliability | First-packet timeout, idle timeout, and retry/backoff protect the request path. |
| UX-C | TUI status clarity | Users see connecting/retrying/thinking/generating/timeout/failure states. |
| UX-D | Evidence and closeout | Mock streams and provider tests prove failure behavior without real network calls. |

## Execution Matrix

| ID | Iteration | Track | Deliverable | Dependencies | Validation | Status |
|---|---|---|---|---|---|---|
| UX100 | I084 | UX-A | ADR-034 draft/decision for provider reasoning/thinking boundary: request fields, stream normalization, persistence, TUI/RPC exposure. | MODEL-003, ADR-013 | ADR review + owner doc sync | Planned |
| UX101 | I084 | UX-A | Normalize Anthropic `thinking_delta` and OpenAI-compatible `reasoning_content`/provider-specific fields into Talos thinking preview events. | UX100, TUI-020 | provider stream fixture tests; conversation tests | Planned |
| UX102 | I084 | UX-A | Request-side reasoning config mapping for Anthropic, OpenAI o-series, and OpenAI-compatible nested `options.thinking`, gated by model capability/config. | UX100, MODEL-001/003 | request-body snapshot tests; config validation tests | Planned |
| UX103 | I084 | UX-B | Provider first-packet timeout and stream-idle timeout, with structured timeout errors and cancellation precedence. | PROVIDER-002 | timeout fixture tests | Planned |
| UX104 | I084 | UX-B | Retry classifier and exponential backoff with jitter for retryable HTTP/transport failures. | UX103 | 429/5xx/timeout retry tests; 401 no-retry test | Planned |
| UX105 | I084 | UX-C | Conversation/TUI status bridge for connecting, retrying, thinking, generating, timed out, failed, cancelled. | UX101-UX104 | TUI state/render tests | Planned |
| UX106 | I084 | UX-D | Documentation and closeout: README/reference config notes, residuals, and full targeted validation. | UX100-UX105 | `cargo test -p talos-provider -p talos-conversation -p talos-tui`; governance | Planned |

## Detailed Design Notes

### Reasoning/Thinking Compatibility

- Provider adapters own vendor JSON:
  - Anthropic: request `thinking: {type: "enabled", budget_tokens: N}`; parse `thinking_delta`.
  - OpenAI o-series: request `reasoning_effort`; account for `max_completion_tokens` behavior.
  - OpenAI-compatible gateways: support configured nested options such as `options.thinking` and
    stream fields such as `reasoning_content`.
- Talos-internal stream should remain provider-neutral:
  - thinking/reasoning preview content is transient;
  - final assistant text is separate;
  - persistence remains stripped unless ADR-034 chooses a separate durable representation.
- Hidden chain-of-thought must not be exposed by default when a provider marks it hidden or only
  returns reasoning token metadata.

### Timeout And Retry

- First-packet timeout protects the no-feedback period after dispatch.
- Stream-idle timeout protects dead streams after partial progress.
- Retry is allowed before irreversible user-visible output; after text/tool output begins, fail
  visibly instead of replaying a request that could duplicate side effects.
- Backoff defaults should be conservative and testable: 3 total attempts, 500 ms base, 8 s max,
  jitter helper separated from wall-clock sleeps for unit tests.

### TUI Status

- The preview/status area should distinguish:
  - waiting for provider;
  - retrying with attempt count;
  - thinking preview;
  - answer generation;
  - timeout/failure with a short actionable reason.
- Status events must not create durable conversation messages unless the failure is finalized as a
  visible error block.

## Non-Authorizations

- No remote plugin install, browser automation, release tag, crate publish, or permission-default
  change.
- No hidden chain-of-thought exposure by default.
- No provider failover or automatic model switching.
- No retry of tool execution or write-capable operations.

## Required Reads

- `docs/backlog/active/UX-001-experience-reliability-program.md`
- `docs/backlog/active/MODEL-003-reasoning-thinking-support.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/proposals/reasoning-thinking-field.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/lib.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-tui/src/app.rs`

## Recovery Instructions

1. Read this plan, UX-001, MODEL-003, and PROVIDER-002.
2. Start with UX100; do not implement provider reasoning request schema changes before the ADR.
3. After each slice, update owner docs before `docs/BOARD.md`.
4. Run targeted provider/conversation/TUI tests for implementation slices.
5. Close I084 only after governance validation and recorded residuals.
