# TUI-061: History Continuation Padding Regression

| Field | Value |
|---|---|
| Story ID | TUI-061 |
| Type | Bug / TUI Layout Correctness Story |
| Priority | P1 corrective residual from I211 |
| Status | Ready / Unclaimed |
| Source | [GitHub Issue #334](https://github.com/wjhuang88/talos/issues/334) |
| Selected Iteration | None |
| Depends On | I023/I142 three-column history continuation contract; I200 resize/reflow evidence |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #334 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Select a runnable corrective iteration and establish an effective claim before implementation. |

## Identity / Goal / Value

Restore the shared blank three-column prefix on wrapped ordinary history rows so ASCII and CJK
continuations do not touch the conversation content boundary.

## Observed Failure

During I211's macOS real-terminal I200 matrix on integrated `main@a2f43248`, wide-to-narrow and
narrow-to-wide reflow preserved complete CJK glyphs and stable scroll anchors, but ordinary user
and assistant continuation rows began at the content-area left edge. I023/I142 require blank
three-column continuation padding. TUI-049/I207 covers steering only and explicitly excludes
unrelated history wrapping.

## Scope

- Preserve the shared blank three-column prefix on wrapped ordinary user and assistant history.
- Preserve left and right padding across ASCII/CJK wide-to-narrow and narrow-to-wide resize.
- Keep Unicode display-cell wrapping, history ordering, scroll anchors, selection/copy and fixed
  composer/status layout unchanged.
- Add focused layout regressions and real-terminal evidence.

## Exclusions

- No TUI-049/I207 steering implementation or global renderer redesign.
- No terminal scroll-policy, selection, permission, provider, persistence, dependency, release or
  publication change.

## Evidence And Required Reads

- Integrated validation head `a2f43248da6c2ae50266f8ce7811210a179e24ef`.
- I211 natural-person evidence in Issue #302 and PR #331.
- `docs/iterations/I023-tui-state-model.md`
- `docs/iterations/I142-composer-multiline-wrap.md`
- `docs/backlog/active/TUI-049-steering-wrap-padding.md`

## Residual Destination

This intake changes no renderer behavior. Select a separate iteration and effective claim before
implementation; I200 remains Review and I211 remains evidence-only.
