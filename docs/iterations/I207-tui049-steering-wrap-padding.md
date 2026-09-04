# Iteration I207: Steering Wrap Padding Contract

> Document status: Active / Claimed (proposed; effective only after claim merge)
> Planned date: 2026-08-17
> Objective: implement TUI-049 so steering continuation lines use the shared horizontal padding
> contract at every supported terminal width.

## Selected Story

- `TUI-049` — `docs/backlog/active/TUI-049-steering-wrap-padding.md`

## Activation Gate

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I207 / TUI-049 only: steering wrapped-row padding and its focused/terminal validation. |
| Claimed At | 2026-09-04 |
| Source Issue | #267 |
| Governance Claim PR | #482 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested serial execution on 2026-09-04; implementation remains gated on this claim merge. |
| Implementation PR | Not started |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Both claim and Active state are ineffective until the finalized governance PR merges to main. |

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

Active / Claimed (proposed; effective only after the finalized governance PR merges). No implementation code exists yet.
