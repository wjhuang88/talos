# Iteration I207: Steering Wrap Padding Contract

> Document status: Planned / Unclaimed
> Planned date: 2026-08-17
> Objective: implement TUI-049 so steering continuation lines use the shared horizontal padding
> contract at every supported terminal width.

## Selected Story

- `TUI-049` — `docs/backlog/active/TUI-049-steering-wrap-padding.md`

## Activation Gate

- Current-main inventory and effective Collaboration Claim are recorded before activation.
- The implementation branch starts from the effective claim merge point.

## Runnable Deliverable

Production width allocation and focused buffer/layout tests covering exact, narrow, ASCII and CJK
cases, with real-terminal evidence.

## Exclusions

No steering lifecycle/timing change, theme redesign, selection change or release work.

## Acceptance

- [ ] Wrapped steering lines honor both shared left and right padding boundaries.
- [ ] Narrow, exact-boundary and Unicode-width cases pass without overflow or edge contact.
- [ ] Locked validation and real-terminal evidence pass at exact head.
- [ ] User-facing layout documentation or residual ownership is recorded.

## Status

Planned / Unclaimed. No implementation branch or code authorization exists.
