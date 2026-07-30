# OBS-002: Turn Pipeline Boundary Observability

| Field | Value |
| --- | --- |
| Story ID | OBS-002 |
| Type | Product / runtime diagnostics story |
| Priority | P1 |
| Status | Refinement — ADR decision required before Ready |
| Source | Maintainer feedback after session `9945f9b4-9c93-41d3-9a9b-a7b33814f130` stream-idle timeout (2026-07-28) |
| Depends On | ADR-006; RUNTIME-002 and PROVIDER-002 resolved timeout foundations; SESSION-006 durable error-path boundary |
| Iteration | Unselected; RUNTIME-003/I168 owns the narrower P0 terminal-outcome correction |

## Problem

The current terminal state can report a terminal phase such as `timed out`, but
it cannot reliably tell a user which boundary of a multi-step turn was reached.
For example, a tool can finish successfully and its result can be retained, yet
the following provider continuation can stall after it receives initial stream
traffic. The resulting error is currently surfaced as a generic unexpected
event/timeout without a correlated, user-readable account of whether the tool
ran, its result entered the next model request, the request was dispatched,
response headers arrived, or the stream subsequently became idle.

This story makes those existing runtime boundaries explicit and correlated. It
does not change tool, provider, retry, permission, or session semantics.

## 2026-07-29 Scope Split

Maintainer-provided TLOG evidence exposed a narrower correctness defect that cannot wait for this
Story's broader public progress-contract ADR:

- unknown finish/stop reasons are normalized to `EndTurn`;
- terminal-frame-less EOF is normalized to `EndTurn`;
- text-only `MaxTokens` is indistinguishable from ordinary success;
- interactive TLOG does not retain the terminal cause.

That bounded repair is owned by
`RUNTIME-003` / `I168`. It uses existing provider/session event semantics and forbids a breaking public
event contract. I168 completed that P0 correction on 2026-07-30 at Completion Commit `2eac5b05`.
OBS-002 remains Refinement for the larger live timeline covering tool completion, result acceptance,
dispatch, headers, first packet, retry, and terminal correlation. Completing RUNTIME-003 does not
complete OBS-002.

## Goal

Expose one bounded, ordered turn-attempt timeline through the existing typed
session/event seam so an interactive user can distinguish:

1. tool execution started and completed (including success/failure);
2. the completed tool result was accepted for the next model continuation;
3. that continuation request was dispatched;
4. response headers and the first usable provider event were received; and
5. terminal completion, cancellation, dispatch timeout, first-packet timeout,
   or stream-idle timeout.

The timeline must identify the boundary that failed without displaying request
bodies, credentials, raw hidden tool output, or duplicate transcript content.

## Proposed Boundary Model (Requires ADR)

Before implementation, decide an additive, typed `TurnProgress`/attempt-event
contract at the existing `AppServerSession` SQ/EQ seam. It must preserve
ADR-006's single-consumer architecture: no global pub/sub bus, no implicit
observer, and no second renderer.

The ADR must decide:

- whether the contract is an additive `AgentEvent` variant or a named sibling
  carried through the existing session seam;
- a stable `turn_id`, `attempt_id`, and `tool_call_id` correlation model;
- which transition is authoritative for `tool result accepted for continuation`;
- how provider dispatch, response-header, first-packet, retry, and idle states
  are normalized across OpenAI-compatible and Anthropic providers;
- the bounded retention/display policy, redaction rules, and whether any
  diagnostic record is durable. Default: session-local and non-durable;
- compatibility and migration requirements for the semver-bound public event
  API; and
- the TUI/dashboard presentation surface and accessibility of a concise
  current-stage/failure explanation.

## Scope

- Emit only state transitions that correspond to real runtime boundaries.
- Correlate tool execution and its following continuation without inferring
  success from rendered text.
- Make the current stage and terminal failure boundary visible in TUI status
  and diagnostic output; retain only a bounded per-turn timeline.
- Preserve existing `TurnPhase` terminal-state behavior, timeout values,
  retries, cancellation priority, tool permission checks, and transcript facts.
- Add deterministic scripted tests for the pipeline through the actual
  conversation/session bridge, not helper-only tests.

## Explicit Exclusions

- No global event bus, open subscription API, or background polling watchdog.
- No tool retry, provider failover, resumable streaming, or automatic turn
  restart.
- No raw request body, credential, hidden tool result, reasoning, or provider
  payload in the timeline.
- No change to `.tlog`/session export format unless the ADR explicitly approves
  a redacted, bounded durable diagnostic record.
- No changes to the Active I165/TUI-039 layout work.

## Acceptance Criteria

- [ ] A successful tool execution emits correlated started/completed evidence,
      including its outcome, without exposing its raw result unnecessarily.
- [ ] The runtime emits a distinct event when a completed tool result is
      accepted into the next provider continuation.
- [ ] Each provider attempt distinguishes dispatch begun, response headers
      received, first usable event received, retry scheduled, and terminal
      failure/completion where applicable.
- [ ] A scripted path `tool succeeds -> continuation dispatches -> first event
      arrives -> stream becomes idle` renders an unambiguous explanation such
      as: `tool succeeded -> continuation request sent -> response started ->
      stream idle timeout (90s)`.
- [ ] A dispatch timeout before response headers, a first-packet timeout, a
      tool failure, cancellation, retryable failure, and normal completion each
      identify their final reached boundary deterministically.
- [ ] The TUI/dashboard never represents a failed boundary as a successful
      subsequent boundary; a terminal status remains compatible with current
      `TimedOut`, `Failed`, and `Cancelled` handling.
- [ ] Timeline storage is bounded and redacted; it does not grow provider
      context, transcript facts, TLOG records, or exported conversation text
      unless explicitly approved by the ADR.
- [ ] Cross-crate bridge tests and workspace locked validation pass.

## Required Reads

- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/backlog/active/SESSION-006-session-error-path-persistence.md`
- `docs/backlog/active/RUNTIME-003-provider-terminal-outcome-integrity.md`
- `crates/talos-core/src/provider.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-provider/src/anthropic_stream.rs`
- `crates/talos-provider/src/openai_sse.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-tui/src/app.rs`

## Initial Evidence

The source session reached successful read-only tool results before the
provider reported a 90-second stream-idle timeout. Existing persistence retains
the completed safe prefix but does not persist the terminal failed exchange,
which is consistent with SESSION-006/ADR-042. This is sufficient to establish
the observability gap, but not to infer that a provider continuation was never
sent. The new contract must make that distinction explicit at runtime.
