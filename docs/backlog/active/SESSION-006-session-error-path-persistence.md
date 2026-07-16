# SESSION-006: Session Error-Path Tool Result Persistence

| Field | Value |
|-------|-------|
| Story ID | SESSION-006 |
| Priority | P1 |
| Status | Open — identified by TOOL-021 audit (I131, 2026-07-16) |
| Depends On | SESSION-002, TOOL-021 |
| Relates To | RUNTIME-002 |
| Origin | TOOL-021 audit finding (`docs/reference/TOOL-021-ERROR-PROPAGATION-AUDIT-2026-07-16.md`) FINDING-2 |

## Problem

When a provider error occurs mid-turn after tools have already executed, the canonical session
turn path (`talos-agent/src/session/turn.rs:188-200`) drops the agent's `new_messages` vector
without persisting it. Tool results that were already executed and pushed to the message vector
are lost. On the next turn or session resume, the model does not see those tool results.

The `Ok(Err(e))` branch sends an error event but never calls `persist_turn_messages`.

## Scope

- Modify `run_turn_with_forwarding` to persist partial turn messages (user message, assistant
  messages, tool calls, tool results) even when the agent returns `Err`.
- Ensure the persisted messages form a valid conversation prefix for resume.
- Add integration test proving tool results survive a provider error.

## Acceptance

- Given a turn where a tool executes successfully but the subsequent provider call fails
  When the session persists and resumes
  Then the tool result is present in the resumed conversation history.
- Given a turn that fails with a provider error
  Then `persist_turn_messages` is called with the partial messages before returning the error.

## Non-Goals

- Retrying the provider call automatically (that is RUNTIME-002/PROVIDER-002).
- Changing the agent's `run_inner` return type.
