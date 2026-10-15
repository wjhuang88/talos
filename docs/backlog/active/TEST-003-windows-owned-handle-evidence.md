# TEST-003: Test-Only Current-Process Handle Evidence

**Status**: Review / Claimed; #716 diagnostic candidate, native evidence/acceptance pending
**Type**: Spike / Technical Story
**Parent Epic**: None; related SEARCH-001-C/#643 and #716 CI blocker
**Source Issue**: #717
**Iteration**: [I302](../../iterations/I302-windows-owned-handle-evidence.md)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | TEST-003 self-only Windows PSS handle metadata, bounded copies/cleanup/observer controls and evidence; no I293 search or production launcher ownership. |
| Claimed At | 2026-10-14 |
| Source Issue | #717 |
| Governance Claim PR | #718 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer accepted bounded self-snapshot scope 2026-10-14 at14:50 Asia/Shanghai and authorized continued GitHub delivery; no distinct maintainer reviewer available. Independent protected Agent native/security review, exact-head CI, validators and CAS are mandatory; no natural-person approval claim. |
| Implementation PR | #716; separate I302 diagnostic work after effective #718 main refresh |
| Last Updated | 2026-10-15 |
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

Finalized #718 proposes Active/Claimed I302 and accepted ADR-091 atomically; all remain ineffective
until merge. Independent complete contract preview found no blocker. No PSS code is in this PR.

## Effective Claim And Recovery — 2026-10-15

#718 merged f4313237c4212e59f57202e22df2ed9b10f78681 after protected contract review
6163000640, applicable CI3180 and CAS6163082933; proposal statements above are historical.
Current main9b7ba281 retains this claim. Recovered implementation starts from that main and
merges original7165c49a8bd as a separate baseline commit. #721/I303 and #705 remain independent.

Workspace maintenance removed the unpublished d1d818e2/treeeed12027 source, toolchain and logs.
Old local validation/review receipts are historical, not evidence for reconstructed code. This
candidate is reimplemented/reviewed/validated anew; see I302 execution journal. Three owned
workers retain cold40/+2, four observer snapshots and four native cleanup-fixture captures.
Diagnostic publication is not original-defect repair, Complete state or716 merge authority.
No Completion Commit or native observation is claimed. #643/#624 remain open.

## Native Evidence And Stage Controls — 2026-10-15

CI3190/run38679585476 on exact published head a39ee9590428630a28ea8deb1781608e76ac9abb
failed the unchanged cold worker73->76: Windows453passed/1failed/0ignored/0filtered; five other
jobs passed. The observer and native cleanup fixtures both succeeded. Four observer phases had
counts73/73/74/76, with unchanged before/after observation counts. Added/retyped metadata:
first_pipe File\u0000 value168; final_40 unknown value328 and Key\u0000 value332. These raw
type strings include a trailing NUL. Values are process-local, may be reused, and do not identify
objects or prove a leak/legitimate cache. Successful controls do not eliminate every observer effect.
The known cleanup BOOLs passed; parent cleanup/terminal success does not establish all lifetimes.
Exact protected remote review6184162949 approved diagnostic publication only, not repair/merge.

Source confirms the injected Assignment branch short-circuits before AssignProcessToJobObject.
That API therefore cannot explain the +2 through its execution on this fault path. Two further
independent owned workers use existing launcher operations/rejections without a new hook/native
call/unsafe/rights: mode3 observes baseline, first Pipe cleanup, attribute preparation followed by
the exact existing quote rejection, then suspended-process failure cleanup; mode4 observes
baseline, existing program resolution, existing environment construction, then suspended-process
failure cleanup. The environment builder itself reads/copies the worker environment; immediately
discard its result, with no extra observational copy/inspection/log/persistence of its contents.
Each worker retains four snapshots and all existing caps/Jobs/deadlines. All five outcomes are
collected before asserting; original cold40/+2 remains uninstrumented and unchanged.

Main23e659e2403739b74bc018ec46305346e8e2c7ad (including bdea322f) is included in separate
baseline refreshes. I304/RUNTIME-011 retains its independent effective #722 claim.
I303/SKILL-006 remains Complete/Closed with its own evidence, while I302 stays Review/Claimed.
New controls require fresh local validation, exact-head protected review and unfiltered native
CI. They narrow a phase interval, not root cause or resource identity. No Completion Commit.

Fresh pinned locked local preflight passed3717primary+1existing-child tests/0failed/ignored,
workspace check/Clippy and both SDK modes, with only the previously authorized local socket
filter. Final-source Windows test-target cross-check/Clippy, feature graphs, governance/claims,
text/classifier/fmt/diff/scale pass; no new native execution claimed. Full receipts are in I302.
Stable exact-head protected review and unfiltered native CI precede evidence acceptance.
