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

## Implementation Facts (I145, 2026-07-20)

- `SteeringQueueSnapshot` and `SteeringQueueEntry` types live in `talos-conversation/src/types.rs`.
- `ConversationEngine::steering_queue_snapshot()` builds the bounded projection: 8 entries, 4 KiB per entry (ellipsis bytes reserved before char-boundary truncation).
- Snapshot emitted on: `enqueue_steering`, post-drain (success and empty), `cancel_turn`, `handle_turn_completed` (Success/Cancelled/Error), and session boundary (`/new`, `/resume`, `/fork` success paths in `session_handlers.rs`). Session error/cancel paths do NOT clear the queue.
- TUI `QueuePreviewComponent` uses a `plan()` helper shared between `height_hint` and `render`; hidden count = `total_count - entries_to_show`; truncation marker `⚠` width reserved in text budget; newlines normalized to `⏎`. For constrained terminals, `app.rs` reserves fixed rows before `compress_layout()` allocates the remainder to modal panels, composer, and queue in that priority order. Composer rendering and cursor scrolling share the allocated height.
- The `UiOutput::SteeringQueueSnapshot` variant is a pre-1.0 semver break. Release must be a minor bump.
- App-level layout reserves fixed rows first, then bounds the modal, composer, and queue allocations with `compress_layout()`. The composer retains one row whenever the remaining content budget permits it.
- Validation covers engine and bridge lifecycle snapshots, the shared session-boundary event helper, layout allocation, and Buffer+InlineFrame rendering. Exact suite totals are intentionally not recorded here because the workspace test count changes independently of this decision.
- README (EN + zh-CN) updated with user-facing queued steering documentation.

## Amendment (2026-07-22): ModelSwitchRequest.provider_hint pre-1.0 breaking change

### Context

P1-fix for MODEL-008-B / I148 added `provider_hint: Option<String>` to the
public `talos_conversation::ModelSwitchRequest` struct. Without this field,
the TUI bridge dropped the provider identity from `UserInput::SwitchModel`,
causing cross-provider duplicate model ID ambiguity in
`Config::set_active_model`.

### Decision

`ModelSwitchRequest` now has three public fields: `model_id`,
`provider_needs_credential`, and `provider_hint`. This is a pre-1.0
struct-literal breaking change in the same class as the ADR-049
`StatusSnapshot` additions and the ADR-050 `ContentPart::Image`
additions.

### Migration

Downstream Rust consumers constructing `ModelSwitchRequest { ... }` via
struct literal must add `provider_hint: None` (or `Some("provider")` if
they know the provider). Pattern matches are unaffected because the field
is additive to the struct (not an enum variant). Consumers that only
receive `ModelSwitchRequest` from the bridge need no change.

### Related

- MODEL-008-B: Model Discovery, Manual Fallback, And Immediate Activation
- I148: Discovery → selection → immediate activation closeout
