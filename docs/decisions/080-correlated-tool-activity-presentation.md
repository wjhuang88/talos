# 080: Correlated Tool Activity Presentation

## Status

Accepted by the maintainer on 2026-09-20 during I279: add invocation-identified
tool events and migration documentation while preserving existing events.
Implementation, independent review and delivery remain pending.

## Decision

Add `UiOutput::ToolActivity` to the existing ordered FIFO projection. Requested
events carry the structured call ID and name; Finished events carry the result's
`tool_use_id` and error flag. Both carry the existing display body (complete request
arguments or returned result text), not a new execution stream. Tool names are never
correlation keys. Width-dependent counts measure this available body only; they do
not claim streamed process-output progress when no such event exists.

`ResponseStarted` resets the transient invocation namespace at the actual
`AgentEvent::TurnStart` boundary. Compatible providers may reuse fallback IDs such
as `call_0` on their next response. This reset is not user-turn completion and is
never inferred from Connecting/Reconnecting status. Completed durable history is
untouched; new calls with the same ID in a new response are independent activities.

Requested means complete arguments were observed, not that execution started.
Finished reports only the structured result. No renderer may infer permission,
retry, timeout, cancellation or success from arbitrary output text. The existing
events do not identify approval requests by call ID, so per-call approval state
must not be guessed from a matching name.

Keep existing ToolCall and ToolResult outputs as the history path; the new event
is transient presentation metadata, not another tool execution or persisted
message. No provider, permission, storage or scheduling behavior changes.

## Migration And Compatibility

Adding a variant to the public Rust enum breaks exhaustive external matches.
External consumers must add a ToolActivity arm (ignore it when they do not render
live activity). Consumers rendering activity must still process legacy display
events exactly once and must not append the metadata as duplicate history.

This change requires the next semver-compatible pre-1.0 minor release, not a
patch release of the existing API. I279 does not authorize a version bump or
publication. Keep legacy events during migration; no constructor fields are added
to ToolCallDisplay or ToolResultDisplay.

## Verification

Verify FIFO ordering, same-name concurrent calls with out-of-order results,
unknown result IDs, turn/session cleanup, unchanged history and absence of
fabricated lifecycle states. Independent API review remains required.

## Related

- [ADR-039](039-runtime-event-semantic-single-flow.md)
- [ADR-052](052-sdk-publication-and-composition-boundary.md)
- [I279](../iterations/I279-tui-history-and-live-activity.md)
