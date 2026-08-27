# TUI-048: Esc-Cancelled Steering Activates The Next Turn

| Field | Value |
|---|---|
| Story ID | TUI-048 |
| Type | TUI / Runtime State Story |
| Priority | P1 |
| Status | Active / Claimed (proposed; ineffective until claim merge) |
| Source | [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267); [Issue #408](https://github.com/wjhuang88/talos/issues/408) |
| Selected Iteration | I206 |
| Depends On | TUI-044 / I169 accepted steering custody; current Esc cancellation behavior |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-048 session |
| Work Slice | Implement only TUI-048/I206: after active-turn Esc cancellation reaches a terminal state, transactionally submit already-accepted Engine-owned steering to the same Session/generation as exactly one next runnable turn; preserve FIFO, durable submission identity, at-most-one active turn, retry/rollback semantics and existing queue projection. Prove provider switching remains blocked while queued work is authoritative and succeeds after the steering batch is actor-accepted and the queue is drained. No queue deletion/editing, provider-switch guard bypass, TUI-049/TUI-050, permission, persistence schema, provider lifecycle, release, Dashboard or Desktop changes. |
| Claimed At | 2026-08-27 |
| Source Issue | #267, #408 |
| Governance Claim PR | #410 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer directed implementation of Issue #408. Repository analysis maps the symptom to the existing TUI-048/I206 objective and ADR-049/ADR-056 custody contract; claim PR #410 remains ineffective until exact-head CI, independent Agent-role review and merge-time CAS pass and it merges to main. |
| Implementation PR | Not started |
| Last Updated | 2026-08-27 |
| Handoff / Release Condition | Merge this atomic claim/activation, then start implementation from that merge or later main. Completion requires focused bridge/engine/TUI tests and a real-terminal Esc -> queued turn -> provider switch trace. |

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

## Issue #408 Reconciliation Checkpoint (2026-08-27)

Issue #408 reports that provider switching remains blocked after Esc cancellation. The provider
guard is behaving correctly while `ConversationEngine` still owns queued steering; bypassing it or
dropping the queue would violate ADR-049 and the published I206 objective. I206 therefore owns the
correction: terminal cancellation must trigger transactional dispatch of the accepted steering,
and provider switching becomes legal only after actor acceptance drains the authoritative queue.
TUI-062 is absorbed as a duplicate symptom owner and grants no separate implementation authority.
