# Iteration I237: Canonical Work Domain And Todo Compatibility

> Document status: Complete / Closed
> Published plan date: 2026-08-30
> Planned objective: implement WORK-001-B/P1 as one canonical Work Domain with a tested Todo
> compatibility surface, preserving current persistence and permission behavior.
> MVP deliverable: a runnable domain/adapter test matrix proves existing Todo behavior through the
> canonical source of truth without a second independently mutable Todo repository.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session |
| Work Slice | WORK-001-B / I237 P1 only; canonical Work Domain plus Todo compatibility and migration evidence. |
| Claimed At | 2026-08-30 |
| Source Issue | #29 |
| Governance Claim PR | #440 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #440 exact head `4e2bd58dc5d54cea2846a2756b56b46438debbc9` passed CI `33312960052` and independent governance approval, then merged to `main` as `3f6f036dfcce612ca88dc57e654b23aedf761678`; I196 Completion Commit `779a4c71` and ADR-061 remain the boundary basis. |
| Implementation PR | #442 (final head `319a1f1576a2a47a49c987491c2c017376510bf6`; merged as `f2b0b5c7e5f5080c9c36f7b7a1993af4246f6f91`) |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Complete after exact-head CI, independent implementation review and merge-time CAS. P2 remains separately governed and unactivated. |

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

## Local Convergence Checkpoint (2026-08-30)

Initial domain, compatibility migration, deterministic graph projection and transactional Todo
mutation work is present locally on `feat-i237-work-domain-todo`. Focused locked-offline tests for
`talos-core` and `talos-session` pass (66 and 186 tests respectively). The candidate is not yet a
stable PR: full workspace validation, documentation closure, staged-diff audit, exact-head CI and
independent review are still required. The Published Baseline remains unchanged.

## Implementation Candidate Checkpoint (2026-08-31)

Implementation PR #442 was opened from `main@c1d0a126` with candidate commit
`7376349700f8c37a398f22a6be3e8ba42275ad9b`. Local release preflight, focused tests, clippy and
governance validators passed before push. Exact-head CI and independent implementation review are
pending; this checkpoint does not claim completion.

## Completion Evidence

Completion Commit: `f2b0b5c7e5f5080c9c36f7b7a1993af4246f6f91` (PR #442 implementation
merge, containing final implementation head `319a1f1576a2a47a49c987491c2c017376510bf6`).

## Completion Checkpoint (2026-08-31)

PR #442 final implementation head `319a1f1576a2a47a49c987491c2c017376510bf6`, based on
`c1d0a1262fa4fbab3c922cdeef14882a9f29add9`, passed exact-head CI run `33323509558`
attempt 2 with all five jobs successful. Independent Agent-role implementation review comment
`5470154364` approved the same head/base after checking WorkGraph invariants, Todo compatibility,
SQLite migration and fail-closed schema handling, CAS/revision semantics, stable edge identity,
permission routing and scope boundaries. Merge-time CAS found the PR MERGEABLE/CLEAN, no
overlapping non-archival main PR, unchanged head/base and a clean worktree; PR #442 then merged as
`f2b0b5c7e5f5080c9c36f7b7a1993af4246f6f91`.

The canonical Work Domain and Todo compatibility path are now complete. P2-P4 were not activated
or implemented by I237. A Todo-specific denial-to-unchanged-SQLite fixture remains an optional
test-strengthening follow-up; the accepted evidence combines Todo permission-facet tests with the
authoritative Agent pipeline's deny-before-execution tests, and the independent review found no
alternate model-facing writer.
