# Iteration I263: Interrupted Transcript Projection

| Field | Value |
|---|---|
| Status | Planned / Unclaimed |
| Source | SESSION-008 / Issue #45 |
| Deliverable | Expose a display-safe interrupted-turn marker in durable transcript projection without leaking hidden diagnostics or changing successful replay. |
| Depends On | I193 / SESSION-008-B, ADR-058 |
| Excludes | No new durable schema, no provider retry, no Desktop/TUI-specific implementation. |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Source Issue | #45 |
| Implementation PR | Not started |

## Problem

`finalize_turn` already persists a `Cancelled`/`Error` outcome marker, but the public durable
`transcript()` projection filters that marker. After restart, display-safe tool results remain while
the interrupted state is invisible. The fix must separate model-message replay from host-facing
status projection.

## Acceptance

- A cancelled turn with an admitted tool result replays the result exactly once and exposes an
  explicit interrupted status in the host-facing projection.
- Hidden terminal diagnostics remain excluded from ordinary message replay.
- Successful turns and existing TLOG/JSONL compatibility remain unchanged.
- Repeated reads and idempotent finalization do not duplicate markers.
- Focused durable/session tests pass and a restart walkthrough proves live/replay parity.

## Implementation Boundary

Add a typed, display-safe terminal status projection (or equivalent metadata) while keeping
`read_messages()` free of marker pseudo-messages. Preserve redaction and do not expose raw reason,
credentials, reasoning, or provider payloads.

## Completion Evidence

Completion Commit: pending
