# TUI-048: Esc-Cancelled Steering Activates The Next Turn

| Field | Value |
|---|---|
| Story ID | TUI-048 |
| Type | TUI / Runtime State Story |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Source | [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267) |
| Selected Iteration | I206 |
| Depends On | TUI-044 / I169 accepted steering custody; current Esc cancellation behavior |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #267 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Establish an effective claim and preserve I169 custody and cancellation semantics. |

## Identity / Goal / Value

When a user cancels an active turn with `Esc`, steering text already accepted for the next turn
must be admitted into the Session and make that turn runnable without requiring a second submit.

## Scope

- Define the exact cancellation-to-session admission and turn-activation event sequence.
- Preserve FIFO ordering, Session/generation identity, durable custody and at-most-one active Turn.
- Keep modal, approval, idle-exit and repeated-cancel behavior unchanged unless explicitly covered.

## Exclusions

No queue editing, cross-Session persistence, automatic retry of a started terminal turn, permission
policy change, or broad scheduler redesign.

## Acceptance For Future Implementation

- Given an active turn and accepted steering input, when `Esc` cancels the active turn, then the
  steering input is admitted to the same Session and exactly one subsequent turn becomes runnable.
- Given no accepted steering input, `Esc` does not create an empty turn.
- Repeated `Esc`, cancellation errors, restart and Session generation changes do not duplicate or
  lose the input.
- Focused lifecycle tests and a real-terminal acceptance trace prove the event ordering.

## Required Reads

- `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`
- `docs/iterations/I169-batched-steering-turn.md`
- `docs/decisions/049-steering-queue-projection-boundary.md`
- `docs/decisions/056-transactional-steering-submission-boundary.md`
