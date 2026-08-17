# Iteration I208: Steering Boundary Insertion

> Document status: Planned / Unclaimed
> Planned date: 2026-08-17
> Objective: implement TUI-050 so steering is inserted at an explicit model-response or tool-call
> boundary rather than only after the outer turn completes.

## Selected Story

- `TUI-050` — `docs/backlog/active/TUI-050-steering-insertion-boundary.md`

## Activation Gate

- TUI-048 and TUI-049 contracts are accepted or their interaction is explicitly resolved.
- Current-main inventory and an effective Collaboration Claim are recorded before activation.
- The implementation branch starts from the effective claim merge point.

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

Planned / Unclaimed. No implementation branch or code authorization exists.
