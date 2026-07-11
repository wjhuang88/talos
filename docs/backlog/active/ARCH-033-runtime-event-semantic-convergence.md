# ARCH-033: Runtime Event Semantic Convergence

| Field | Value |
|---|---|
| Story ID | ARCH-033 |
| Priority | P0 |
| Status | Complete (I115, 2026-07-11) |
| Source | 2026-07-10 semantic follow-up to ARCH-032 and user-reported dropped turn/content |
| Depends On | ADR-005, ADR-006, ADR-034, ADR-039 |

## Problem

ARCH-032 proved that Talos has no global broadcast bus and that its channels have explicit
single-consumer ownership. That channel-topology result did not prove semantic single-flow
behavior. The live runtime currently has independent ordering domains for `UiOutput` and nested
`StreamMessage` channels, treats provider-response `AgentEvent::TurnEnd` as a UI lifecycle event
alongside authoritative `SessionEvent::TurnCompleted`, and persists the same turn through multiple
CLI-owned paths. These conditions can drop queued text, complete a turn early, or make live and
resumed history diverge.

## Scope

- Flatten live text/reasoning/tool/status delivery onto one ordered UI event queue.
- Make `SessionEvent::TurnStarted`/`TurnCompleted` the authoritative user-turn lifecycle.
- Carry turn identity and deterministic sequence metadata across the session EQ.
- Drain steering only after authoritative turn completion.
- Move durable turn-message persistence to the session actor and remove CLI event/message writers.
- Converge TUI, interactive, inline, print, embedded runtime, and RPC on the session protocol.
- Preserve ADR-006: no global publish/subscribe bus or implicit tool/permission observer.

## Non-Goals

- No global event bus.
- No provider feature expansion or new retry policy.
- No permission, sandbox, credential, or storage-format default change.
- No removal of a semver-bound public compatibility variant without the ADR-039 migration window.

## Acceptance

- [x] Live TUI text, reasoning, tool, status, and terminal lifecycle use one FIFO queue; no runtime
      text is carried by a nested stream receiver.
- [x] Every canonical turn-scoped `SessionEvent` carries a stable `session_id`, `turn_id`, and monotonic `sequence`.
- [x] Steering drains only after `TurnEventPayload::Completed`, never after provider `TurnEnd`.
- [x] Tool-loop regression proves text before and after a tool is rendered without loss or reorder.
- [x] The session actor is the sole writer of successful turn messages; CLI modes do not separately
      append user/assistant/event copies.
- [x] Live execution and session reload reconstruct the same authoritative message sequence.
- [x] TUI, interactive, inline, print, embedded runtime, and RPC consume the same session lifecycle
      semantics.
- [x] `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, and
      `cargo test --workspace` pass.
- [x] Real `talos` binary MCP tool-loop and RPC scenarios reach authoritative completion and final
      text; the ordered Conversation regression covers thinking → text → tool → result → text.

## Documentation

- `docs/reference/ARCHITECTURE.md`
- `docs/decisions/039-runtime-event-semantic-single-flow.md`
- `docs/iterations/I115-runtime-event-semantic-convergence.md`
- `README.md` if user-visible runtime behavior or compatibility guidance changes.
