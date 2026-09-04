# TUI-050: Steering Inserts At Model And Tool Boundaries

**Status**: Active / Claimed (proposed; ineffective until claim merge)

| Field | Value |
|---|---|
| Story ID | TUI-050 |
| Type | TUI / Runtime Scheduling Story |
| Priority | P1 |
| Status | Active / Claimed (proposed; ineffective until claim merge) |
| Source | [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267) |
| Selected Iteration | I208 |
| Depends On | TUI-044 / I169 accepted queue custody and Turn lifecycle |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I208 / TUI-050 only: insert accepted steering at explicit model-response or tool-call boundaries, preserving FIFO, Session/generation identity, exactly-once custody and existing transcript semantics. Excludes layout/padding, arbitrary token preemption, parallel model execution, global event bus, persistent cross-Session queues, permission, release and CAP-001 text seam work. |
| Claimed At | 2026-09-04 |
| Source Issue | #267 |
| Governance Claim PR | #486 (proposed; ineffective until merge) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I207/TUI-049 is Complete / Closed on main at `2edb914f`; maintainer directed serial execution of I207, I208 and I246. |
| Implementation PR | Not started |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Claim and activation are ineffective until this governance record merges. Implementation starts from its merge or a later `main`; independent exact-head review remains required. |

## Identity / Goal / Value

Accepted steering should become eligible at a defined model-response or tool-call boundary instead
of waiting for the entire outer turn to finish, while retaining deterministic ordering and custody.

## Scope

- Decide and implement the smallest insertion boundary after one model response or one tool call.
- Preserve exactly-once ownership, FIFO order, Session/generation identity and transcript semantics.
- Define behavior for multiple boundaries, errors, cancellation and no-op tool calls.

## Exclusions

No arbitrary mid-token preemption, parallel model execution, global event bus, persistent
cross-Session queue, or change to I169's accepted durable transfer protocol without change control.

## Acceptance For Future Implementation

- Given accepted steering during an active turn, when the selected model/tool boundary completes,
  then the input is inserted at that boundary according to the published ordering contract.
- Inputs arriving after a boundary belong to the next eligible boundary and are not reordered.
- Error/cancel/restart paths retain or reconcile pending input exactly once.
- Event-sequence tests and a real-terminal trace prove insertion timing and visible ordering.

## Required Reads

- `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`
- `docs/iterations/I169-batched-steering-turn.md`
- `docs/decisions/049-steering-queue-projection-boundary.md`
- `docs/decisions/056-transactional-steering-submission-boundary.md`
