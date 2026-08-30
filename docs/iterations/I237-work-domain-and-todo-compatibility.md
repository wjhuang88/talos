# Iteration I237: Canonical Work Domain And Todo Compatibility

> Document status: Planned / Unclaimed — claim proposed; ineffective until merge
> Published plan date: 2026-08-30
> Planned objective: implement WORK-001-B/P1 as one canonical Work Domain with a tested Todo
> compatibility surface, preserving current persistence and permission behavior.
> MVP deliverable: a runnable domain/adapter test matrix proves existing Todo behavior through the
> canonical source of truth without a second independently mutable Todo repository.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | WORK-001-B / I237 P1 only; canonical Work Domain plus Todo compatibility and migration evidence. |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Proposed basis: WORK-001-A/I196 Completion Commit `779a4c71`; ADR-061 boundary; exact-base governance checks and independent review required |
| Implementation PR | Not started |
| Last Updated | 2026-08-30 |
| Handoff / Release Condition | Implementation starts only from claim merge or later current `main`; completion requires implementation evidence, exact-head validation, independent review and owner-first closeout. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WORK-001-B | WORK-001 | Ready / proposed claim | WORK-001-A/I196 Complete; RUNTIME-001; TODO-001; TODO-002; VALIDATION-001; ADR-061 | One canonical Work Domain and tested Todo compatibility path with no P2-P4 or Desktop scope. |

### Non-Terminal Inventory And Disposition

| State | Iterations | Disposition |
|---|---|---|
| Active | None on published base | This claim proposes I237 activation; it is ineffective before merge. |
| Review | None | No review iteration blocks P1 selection. |
| Planned | I207, I208 | Preserve unclaimed steering children; no overlap with P1. |
| Blocked | P2/P3/P4 child boundaries only (no owner/iteration yet) | Keep blocked until direct prerequisites complete; do not activate through I237. |
| Paused | I164 | Superseded historical target; do not resume. |

The current exact base is `main@d5ede746851a220aecb58b52f53192c0263dc615`; `origin/main` must be
refreshed and rechecked before claim review and before implementation. Other active or separately
owned work remains outside this slice, including I225 and any Dashboard/Desktop work.

## Scope And Exclusions

The scope and exclusions are identical to WORK-001-B. In particular, this iteration does not
implement Completion Claim/Evaluation, evaluator runtime, Mission Delivery, Desktop/Dashboard,
`/auto`, release or publication behavior.

## Validation And Handoff

- Governance-only claim candidate: project and Collaboration validators with explicit base, YAML/
  Markdown structural checks and `git diff --check`.
- Implementation candidate: focused Todo/domain tests, locked workspace checks, documentation and
  migration fixture validation, then exact-head CI and independent review.
- Merge-time CAS must confirm the claim head/base, no overlapping owner/PR, clean CI/review and
  unchanged P1 scope.
- After implementation merge, update this owner first, then WORK-001, backlog, Board, iterations
  README and manifest. Record an existing implementation SHA as Completion Commit; a status-only
  commit cannot self-certify completion.

## Resume Instructions

After this claim merges, refresh `main`, repeat the non-terminal and overlap inventory, create the
implementation branch from the effective claim merge (or later current `main`), and converge the
complete P1 design/code/tests/docs locally before pushing one stable candidate.
