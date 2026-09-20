# TUI-061: History Continuation Padding Regression

> Document status: Complete

| Field | Value |
|---|---|
| Story ID | TUI-061 |
| Type | Bug / TUI Layout Correctness Story |
| Priority | P1 corrective residual from I211 |
| Status | Complete |
| Source | [GitHub Issue #334](https://github.com/wjhuang88/talos/issues/334) |
| Selected Iteration | I279 |
| Depends On | I023/I142 three-column history continuation contract; I200 resize/reflow evidence |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I279 presentation-only delivery of this Story; no permission, provider, persistence or execution changes. |
| Claimed At | 2026-09-20 |
| Source Issue | #334 |
| Governance Claim PR | #579 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested all three Issues on 2026-09-20 under standing single-maintainer mode; independent Agent review and exact-head checks required before merge. |
| Implementation PR | #580 |
| Last Updated | 2026-09-20 |
| Handoff / Release Condition | Complete through #580; I279 final delivery ledger records passing acceptance, exact-head CI and independent Agent review. |

## Completion Evidence (2026-09-20)

Completion Commit: 52276f552496b160e064fd54ed7471bc84d3f559

PR #580 delivered this Story on main. Full local release preflight and all six
exact-head CI checks (35502971132) passed; independent Agent review 5749013208
and merge-time CAS 5749088682 bind the implementation candidate. Maintainer native
acceptance passed, including the final completed-tool-body dedup correction.
See [I279 final delivery ledger](../../iterations/I279-tui-history-and-live-activity.md)
for requirement-specific tests, observations, identity limits and ADR-080 migration.
No remaining acceptance in this Story. The original intake and dated selection
below are preserved as historical provenance, not unfulfilled activation gates.

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

## 2026-09-20 Unified Delivery Selection

I279 selects #298/#310/#334 as one locally converged TUI stage. PR #579 proposes this claim;
no implementation authority exists before merge. Historical intake text above remains provenance;
I279's explicit projection-only, per-entry, line-count and acceptance decisions govern execution.
Existing I277 deferred acceptance, I249 Planned and I164 Paused dispositions are preserved.
