# 039: Runtime Event Semantic Single-Flow Boundary

## Status

Accepted (2026-07-11)

## Context

ADR-005 and ADR-006 establish a bounded SQ, an unbounded single-consumer EQ, and reject a global
publish/subscribe bus. ARCH-032 verified those channel shapes, but runtime behavior still split one
logical turn across independent ordering domains: `UiOutput` events and nested `StreamMessage`
receivers. Provider-response `AgentEvent::TurnEnd` also competed with session-level
`SessionEvent::TurnCompleted`, while product modes persisted or reconstructed messages differently.

The result is topology compliance without semantic single-flow: a later tool or lifecycle output
can be selected before earlier queued text and close its receiver, and different surfaces can reach
different conclusions about the same turn.

## Decision

1. `AppServerSession` remains the canonical runtime seam. This ADR does not introduce a global bus.
2. Session-level `TurnStarted` and `TurnCompleted` are the only authoritative user-turn lifecycle.
   Provider response boundaries are progress details and cannot complete or drain a user turn.
3. A turn-scoped session envelope carries `session_id`, `turn_id`, and a monotonic sequence number. Downstream
   projections preserve that order.
4. Live UI text, reasoning, tool, status, and lifecycle outputs share one FIFO `UiOutput` queue.
   In-tree runtime producers must not carry live text through nested stream receivers.
5. The session actor owns successful turn-message persistence. Renderers and CLI bridges are
   projections, not durable writers.
6. TUI, interactive, inline, print, embedded, and RPC surfaces consume the same session protocol;
   only input, rendering, and approval policy may differ.
7. Because `UiOutput::Stream` is a public API, it remains temporarily as a deprecated compatibility
   input. All in-tree producers migrate in I115. Removal requires a semver-major release or an
   explicit public migration decision.

## Consequences

- Ordering becomes inspectable and testable with one queue and sequence identity.
- Steering and terminal status are driven by whole-turn completion.
- Persistence/replay has one authoritative message sequence.
- Compatibility code remains temporarily, but it is not used by canonical runtime paths.
- No new subscriber or implicit permission/tool-event sink is created.

## Reversal Trigger

Revisit only if a concrete surface cannot preserve ordered projection from the session EQ. The
remedy must remain typed and explicitly wired; it must not be a global pub/sub bus.

## Related

- ADR-005: Canonical TUI Event Architecture
- ADR-006: Event Architecture Boundary
- ADR-034: Provider Reasoning / Thinking Boundary
- ARCH-032: Single Data Flow Audit
- ARCH-033: Runtime Event Semantic Convergence
