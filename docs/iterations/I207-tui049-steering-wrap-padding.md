# Iteration I207: Steering Wrap Padding Contract

> Document status: Review / Claimed (local convergence; implementation PR pending)
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
| Implementation PR | Stable local candidate; PR pending |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Implementation starts from main at or after claim merge `8ff4c6f186e299d1ca179baeeedc19971eeaa75f`; independent review remains required for the implementation candidate. |

- Current-main inventory and effective Collaboration Claim are recorded before activation.
- The implementation branch starts from the effective claim merge point.

## Activation Checkpoint (2026-09-04)

Governance PR #482 merged to `main` as `8ff4c6f186e299d1ca179baeeedc19971eeaa75f` from exact
head `8cb3161d95f21fe8fc5b8a1a6b326f1fbe0938c3` on base `c1f7e83db2723cbec140ba6629342bff7d746a9e`.
Exact-head CI `33849472054` passed, with documentation-only routing; independent review was
recorded on the PR issue thread. I207 is now Active / Claimed. No implementation authorization
extends to I208, I246, Dashboard, Desktop, or release work.

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

Review / Claimed locally. Claim and activation became effective with governance PR #482 merge
`8ff4c6f186e299d1ca179baeeedc19971eeaa75f`. The implementation is locally converged; the stable
candidate is pending its implementation PR, exact-head CI and independent review.

## Implementation Checkpoint (2026-09-04)

I207 changes only the steering queue preview: long queued entries are wrapped to the available
display width, continuation rows retain the shared `↳` content-column padding, and truncated
entries preserve their warning marker. Existing short-entry, summary, follow-up and transcript
behavior is unchanged. Focused TUI tests and the full `talos-tui` library suite pass locally.

## Verification

- `cargo test -p talos-tui --lib`: 568 passed, 0 failed.
- `cargo fmt --all`: passed.
- Workspace locked checks remain in progress before the stable candidate is pushed.
