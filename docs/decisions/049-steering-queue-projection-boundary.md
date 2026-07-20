# 049: Steering Queue Projection Boundary

## Status

Accepted (2026-07-20)

## Context

TUI-026 requires the interactive viewport to show the content and FIFO order of queued steering
messages. The canonical queue is owned by `ConversationEngine`; existing `StatusSnapshot` projects
only `steering_count`. Letting the TUI mirror submissions would create a second, drift-prone queue
state across turn completion, cancellation, error, session switch and resume.

The existing cross-surface projection is the ordered `UiOutput` stream under ADR-039. Adding a
variant to its public Rust enum is semver-breaking for downstream exhaustive matches, despite being
additive in source. Adding a public field to `StatusSnapshot` has the same downstream struct-literal
risk.

## Constraint Decomposition

| Constraint | Type | Source | Decision impact |
| --- | --- | --- | --- |
| Engine is the sole owner of steering state | Hard | TUI-004, ADR-039 | TUI may not maintain a mirror queue. |
| One ordered UI projection, no global pub/sub | Hard | ADR-006, ADR-039 | Snapshot travels on `UiOutput`; no side channel/bus. |
| Public APIs are semver-bound | Hard | AGENTS.md #6 | New public enum variant requires migration guidance and a minor pre-1.0 release. |
| Bounded viewport and memory | Hard | TUI-026, AGENTS.md Simplicity First | Projection must have explicit entry and byte bounds. |
| Final history remains scrollback-only | Hard | ADR-035 | Queue preview is transient viewport state, never finalized history. |

## Decision

1. Add one canonical `UiOutput::SteeringQueueSnapshot` projection in `talos-conversation`. It is
   emitted by the engine whenever the authoritative steering queue changes: enqueue, dequeue,
   turn cancellation/error terminal handling, and session replacement/clear.
2. The snapshot is a value projection, not a mutable queue API. It contains:
   - `total_count`;
   - FIFO previews for at most the first 8 entries;
   - a per-entry UTF-8 byte cap of 4 KiB and an explicit `truncated` marker;
   - `omitted_count` for entries outside the projection bound.
   It contains no credential, provider request, tool result, persistence handle or permission data.
3. The TUI renders the snapshot transiently above the composer. Its own render budget remains at
   most 6 terminal rows; it may collapse previews further for terminal height, but it must retain
   exact total/omitted counts. It neither mutates nor reconstructs queue state.
4. The existing `StatusSnapshot.steering_count` remains for compact status consumers. New
   `SteeringQueueSnapshot` consumers must update for every queue mutation in the same ordered
   stream as the corresponding status update.
5. Because `UiOutput` is public, the release containing this change must be a new pre-1.0 minor
   version (not a patch) and release notes must instruct downstream consumers to handle
   `UiOutput::SteeringQueueSnapshot` or add a forward-compatible wildcard arm.

## Rejected Alternatives

- **TUI-local mirror of submitted text**: violates single ownership and drifts on terminal states.
- **New TUI-only channel/sidecar**: creates a second projection path contrary to ADR-039.
- **Global broadcast/event bus**: rejected by ADR-006.
- **Unbounded `Vec<String>` snapshot**: permits a large queued input workload to grow every UI
  consumer's memory without a user-visible rendering benefit.
- **Put preview strings in `StatusSnapshot`**: still breaks public struct literals while coupling a
  compact status type to a potentially multi-line payload.

## Migration

Downstream Rust consumers matching `UiOutput` exhaustively must add a
`SteeringQueueSnapshot` arm or a wildcard fallback. Consumers that only show status may safely
ignore the variant. The new type is transient and does not alter Session/TLOG formats, RPC input,
permission behavior, or turn drain timing.

## Reversal Trigger

Revisit only if a committed surface needs queue editing, cancellation, reordering, persistence or
more than bounded preview. Such a requirement needs a new state model decision; it must not expand
this projection into a queue-control API opportunistically.

## Related

- ADR-006: Event Architecture Boundary
- ADR-035: TUI Conversation History Scrollback Boundary
- ADR-039: Runtime Event Semantic Single-Flow Boundary
- TUI-004: State Model
- TUI-026: Queued Steering Message Display
