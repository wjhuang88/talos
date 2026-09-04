# Iteration I207: Steering Wrap Padding Contract

> Document status: Review / Claimed (implementation merged; closeout pending)
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
| Authorization Evidence | Maintainer requested serial execution on 2026-09-04; claim became effective through PR #482 before implementation. |
| Implementation PR | #483 |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Implementation started from main at or after claim merge `8ff4c6f186e299d1ca179baeeedc19971eeaa75f`; implementation PR #483 is merged, while closeout still requires the recorded real-terminal acceptance trace. |

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

- [x] Wrapped steering lines honor both shared left and right padding boundaries.
- [x] Narrow, exact-boundary and Unicode-width cases pass without overflow or edge contact.
- [ ] Locked validation and real-terminal evidence pass at exact head.
- [ ] User-facing layout documentation or residual ownership is recorded.

## Status

Review / Claimed. Claim and activation became effective with governance PR #482 merge
`8ff4c6f186e299d1ca179baeeedc19971eeaa75f`. Implementation PR #483 was independently reviewed,
validated and merged as `ca3b2fa7ffb1ca14b82d1acf6af6be147368e6fe`; closeout remains pending the
real-terminal narrow-width trace and its independent closeout review.

## Implementation Checkpoint (2026-09-04)

I207 changes only the steering queue preview: long queued entries are wrapped to the available
display width, continuation rows retain the shared `↳` content-column padding, and truncated
entries preserve their warning marker. Existing short-entry, summary, follow-up and transcript
behavior is unchanged. Focused TUI tests and the full `talos-tui` library suite pass locally.

## Verification

- `cargo test -p talos-tui --lib`: 570 passed, 0 failed.
- `cargo clippy -p talos-tui --lib --locked -- -D warnings`: passed.
- `cargo check --workspace --locked`: passed.
- `cargo fmt --all`: passed.
- `./scripts/release_preflight.sh`: passed.
- Exact-head CI `33861265618`: 5/5 successful for implementation head
  `0998b10581867dca92c2b37919825f8d72134766`.
- Independent implementation review APPROVE: comment `5539172892`, bound to the same exact head.
- Remaining gate: run and record a real-terminal narrow-width trace through the actual `talos` binary;
  this evidence is not inferred from unit tests or CI.

## Review Correction Checkpoint (2026-09-04)

Independent review comment `5538822655` on PR #483 identified silent viewport loss when one
wrapped entry exceeded the five-row queue budget. The implementation now marks the final visible
row with a display-width-safe `…`, while retaining the separate storage-truncation `⚠` marker;
tests cover both a single over-budget entry and a later entry that only partially fits. The next
stable candidate must use fresh exact-head CI and review evidence.
