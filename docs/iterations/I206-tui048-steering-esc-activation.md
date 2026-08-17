# Iteration I206: Esc-Cancelled Steering Activation

> Document status: Planned / Unclaimed
> Planned date: 2026-08-17
> Objective: implement TUI-048 so accepted steering becomes a runnable Session turn when `Esc`
> cancels the active turn, without changing the accepted I169 custody contract.

## Selected Story

- `TUI-048` — `docs/backlog/active/TUI-048-steering-esc-activation.md`

## Activation Gate

- Current-main inventory records all non-terminal iterations and open PRs/issues.
- An effective Collaboration Claim exists on target `main`.
- A fresh implementation branch starts from the claim merge point.

## Runnable Deliverable

Lifecycle implementation and focused tests proving cancellation, Session admission and exactly one
subsequent turn, plus real-terminal evidence.

## Exclusions

No TUI-049/TUI-050 implementation, queue redesign, permission policy change or release work.

## Acceptance

- [ ] Esc cancellation admits accepted steering into the same Session.
- [ ] Exactly one continuation turn becomes runnable; empty and repeated-cancel cases are covered.
- [ ] Focused, locked workspace validation and real-terminal evidence pass at exact head.
- [ ] User-facing steering/cancellation documentation is updated.

## Status

Planned / Unclaimed. No implementation branch or code authorization exists.
