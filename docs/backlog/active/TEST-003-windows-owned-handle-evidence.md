# TEST-003: Test-Only Current-Process Handle Evidence

**Status**: Ready / Unclaimed; draft atomic selection for I302
**Type**: Spike / Technical Story
**Parent Epic**: None; related SEARCH-001-C/#643 and #716 CI blocker
**Source Issue**: #717
**Iteration**: [I302](../../iterations/I302-windows-owned-handle-evidence.md)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #717 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer accepted the bounded self-snapshot packet on 2026-10-14 at14:50 Asia/Shanghai; finalized ADR/claim/security review/CI/CAS still required. |
| Implementation PR | Not started; later #716 follow-up after effective claim refresh |
| Last Updated | 2026-10-14 |
| Handoff / Release Condition | No implementation before finalized atomic claim and ADR merge into main; no automatic original-defect closure or #716 merge. |

## Identity / Goal / Value

Maintainers need typed evidence for the cold Windows resource-growth failure blocking #716.
CI3175 shared counter151->162 and3176 isolated73->76 are retained. CI3177 samples show first
Pipe+1, first Assignment+2, otherwise stable through40 exact injected faults; +2 still failed.
Totals locate boundaries, not handle identity, legitimate lifetime or absence of a one-time leak.
Existing cleanup BOOL checks in #716 are a separate mechanical-maintenance checkpoint.

## Scope

Implement the accepted self-only observation contract in [ADR-091](../../decisions/091-test-only-current-process-handle-snapshots.md):
Windows PSS in a separate owned test worker, only current-process handle value/type copies,
checked resource ownership and bounded diagnostics. Preserve an independent uninstrumented cold
worker and original40/+2 assertions. Collect the snapshot worker safely even when the original
worker fails; retain that original failure after evidence collection. Reconcile recorded resource
types with known cleanup results, independent controls and primary facts, or explicitly conclude
the evidence is insufficient. TEST-003 is a diagnostic deliverable, not a promised production fix.

## Exclusions

No other PID/OpenProcess, process memory/VA clone/space/thread context/breakaway, privilege
enabling/escalation, external binary/driver, ObjectName/union inspection or raw dump. No search
activation/provider admission/parser work, launcher behavior/API/security-policy change,
dependency-version/lock change, threshold increase/warmup/skip/retry-to-green, #705 takeover or release.

## Dependencies / Decisions

Maintainer scope acceptance is recorded; ADR-091 and this claim are ineffective until the finalized
governance-only PR merges into main. Existing windows-sys0.61.2 is the sole binding package;
diagnostic feature unification must be checked against real production/test graphs. Implementation
code remains cfg(test,windows). New native ABI/unsafe sites require independent protected review.

## Acceptance

- Given the isolated cold counter worker, all40 fault checks and +2 remain; snapshot observation
  runs in another worker and cannot shift its baseline or suppress its failed outcome.
- Given only GetCurrentProcess, capture/walk bounded metadata and output only owned numeric handle
  value/type records. Document the NAME flag's internal object-name collection without reading,
  copying, formatting or persisting ObjectName or unions.
- Given invalid flags/pointers/lengths, capacity/API/release failures or timeout, report bounded
  diagnostic failure and finish all possible cleanup; do not claim zero/complete metadata.
- Given successful observations, distinguish helper effects/handle reuse/known cleanup results
  from original growth. Record evidence-supported attribution or precise remaining uncertainty.
- Native fixtures exercise valid type copies, limits/error/release/observer controls. Required
  full exact-head CI/protected review/CAS pass before delivery; the original defect and #716
  remain open if uncorrected, even if this diagnostic becomes independently deliverable.

## Planned Validation

Pinned1.99.0 locked check/Clippy/tests/full preflight, both governance validators, source/native ABI
review, Windows production/test Cargo feature graphs, real owned Windows worker receipts. Only the
previously authorized local Unix-socket filter may remain; native remote CI is unfiltered. No new
skip/ignore/lint/deadline waiver. Linux source/decoder checks do not prove native execution.

## Documentation / Residual Destination / Rollback

Operator-facing contract: ADR-091 and I302 receipts; update this owner, Board/index/matrix and #717.
No user product flow changes or README feature claims. I293/#643/#624 retain all product gates;
#705 retains latency diagnostics. Unknown resource attribution stays in this owner; actual repair
requires separately reviewed evidence/authorization rather than changing the accepted baseline.
Remove temporary observer code after evidence, or retain only with an explicit reviewed test
contract/cleanup result. Revert diagnostic helpers without changing production behavior.

## Required Reads

AGENTS.md; collaboration/start-iteration/change-control/testing/evolution SOPs; ADR-007/068/091;
I293 and #716/#717 evidence; process_boundary.rs; existing windows-sys manifest/lock; Microsoft
PSS capture/flags/entry/walk/free contracts linked in ADR-091.

## Execution / Completion

Governance preparation only. No implementation or Completion Commit. Claim/activation/ADR become
effective only on merge. #716 can later carry separate I293 and I302 commits after effective-main
refresh; sharing a PR transfers no ownership between the two slices.
