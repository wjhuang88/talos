# Iteration I208: Steering Boundary Insertion

> Document status: Active / Claimed (proposed; ineffective until claim merge)
> Planned date: 2026-08-17
> Objective: implement TUI-050 so steering is inserted at an explicit model-response or tool-call
> boundary rather than only after the outer turn completes.

## Selected Story

- `TUI-050` — `docs/backlog/active/TUI-050-steering-insertion-boundary.md`

## Activation Gate

- TUI-048 and TUI-049 contracts are accepted or their interaction is explicitly resolved.
- Current-main inventory and an effective Collaboration Claim are recorded before activation.
- The implementation branch starts from the effective claim merge point.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I208 / TUI-050 only: insert accepted steering at explicit model-response or tool-call boundaries, preserving FIFO, Session/generation identity, exactly-once custody and existing transcript semantics. Excludes layout/padding, arbitrary token preemption, parallel model execution, global event bus, persistent cross-Session queues, permission, release and CAP-001 text seam work. |
| Claimed At | 2026-09-04 |
| Source Issue | #267 |
| Governance Claim PR | Pending draft PR number (backfill before merge) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I207/TUI-049 is Complete / Closed on main at `2edb914f`; maintainer directed serial execution of I207, I208 and I246. |
| Implementation PR | Not started |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Claim and activation are ineffective until this governance record merges. Implementation must start from its merge or a later `main`; independent exact-head review remains required. |

## Runnable Deliverable

An event-boundary implementation with deterministic ordering tests, error/cancel/restart coverage,
and real-terminal timing evidence.

## Exclusions

No arbitrary token preemption, parallel model execution, global event bus, or release work.

## Acceptance

- [ ] Steering is inserted at the selected model/tool boundary with published ordering semantics.
- [ ] Multiple boundaries, late arrivals, errors, cancellation and restart reconcile exactly once.
- [ ] Locked validation and real-terminal evidence pass at exact head.
- [ ] User-facing steering timing documentation is updated.

## Status

Active / Claimed (proposed; ineffective until claim merge). No implementation branch or code
authorization exists before the claim merge.
