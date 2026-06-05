# ADR-011: Guardian Approval Boundary

- **Status**: Accepted
- **Date**: 2026-06-05
- **Backlog**: #I010-S6

## Context

`#I010-S6` proposes a Guardian AI sub-agent that reviews tool calls and may
auto-approve low-risk operations. This touches Talos' most important safety
boundary: every write-capable tool must go through the permission pipeline, and
approval must remain auditable and fail-safe.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| All write-capable tools are gated by permissions | Hard | AGENTS.md hard constraint #4 | No |
| Sandbox/permission changes need security review | Hard | AGENTS.md hard constraint #5 | No |
| Guardian may improve ergonomics for low-risk calls | Soft | #I010-S6 product direction | Yes |
| Model judgment may be wrong or prompt-injected | Assumption | LLM behavior | Must be bounded |

## Reasoning

The dangerous version of Guardian is one where an LLM becomes a second,
implicit permission engine. That would make approval behavior harder to audit
and would let model errors or prompt injection bypass user intent.

The safer shape is to treat Guardian as a policy assistant inside the existing
permission flow. It can only resolve a bounded subset of `Ask` decisions when
explicitly enabled and when the tool/action is already classified as eligible.
The permission engine remains authoritative.

## Decision

- Guardian is disabled by default.
- Guardian must never bypass `PermissionEngine`; it can only participate after
  the permission engine returns `Ask`.
- Guardian must never auto-approve write-capable tools in the first
  implementation.
- Guardian may auto-approve only allowlisted read-only or metadata-only
  operations that have deterministic arguments and bounded output.
- Any uncertainty, model error, timeout, malformed response, prompt-injection
  signal, or circuit-breaker trip escalates to user approval or denial according
  to the caller mode. It must not silently approve.
- Guardian decisions must be logged with tool name, risk class, decision,
  reason code, and whether the decision was model-assisted. Do not log secrets
  or full sensitive arguments by default.
- Guardian prompt/context must exclude secrets and large raw file contents unless
  a future ADR explicitly expands the scope.
- A circuit breaker must disable Guardian for the session after repeated denials
  or malformed/unsafe responses.

## Reversal Trigger

Revisit this decision only after Talos has production evidence that read-only
Guardian approval is safe and useful, and a fresh security review accepts a
specific write-capable auto-approval class.
