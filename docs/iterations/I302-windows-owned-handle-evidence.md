# Iteration I302: Windows Current-Process Handle Evidence

> Document status: Review
> Published plan date: 2026-10-14
> Planned objective: qualify cold-worker resource growth with bounded self-only native metadata.
> MVP deliverable: reproducible operator-visible handle type/cleanup evidence or explicit uncertainty.

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
| Handoff / Release Condition | No PSS code before this finalized claim/ADR reaches main; diagnostic completion does not close the original failure or permit #716 merge. |

## Published Baseline

Selected Story [TEST-003](../backlog/active/TEST-003-windows-owned-handle-evidence.md), Ready,
no parent. Preserve all scope/exclusions/acceptance/validation/rollback in that owner. ADR-091 is
the new native/unsafe contract; ADR-007/068 ownership remains. The final Active/Claimed/Accepted
proposal is ineffective until atomic governance merge. No native implementation is in this PR.

### Selection Inventory And Disposition

Selection main d66e380ad144329dbea49634122505a5a3075e95. All iteration headers inventoried:

| Owner | State | Disposition |
|---|---|---|
| I293 | Active / Claimed | Retain Search scope; #716 form and existing mechanical checks remain I293; I302 owns only new self-snapshot diagnostics. |
| I294 | Active / Claimed | Retain provider evidence; E/F gated, no takeover. |
| I300 | Active / Claimed | Retain independent Git scope; no transfer. |
| I301 | Review / Claimed | Retain published startup/storage acceptance and residuals. |
| I277 | Review / deferred device acceptance | Retain, not a diagnostic blocker. |
| I249 | Planned | Retain dependency-pilot deferral; no version upgrade. |
| I164 | Paused | Retain superseded layout disposition. |
| I162 | Terminal with historical Review wording | Retain publication disposition; do not reactivate from text matching. |

No Blocked iteration header found; other terminal owners remain terminal. DEPENDENCY-003-A retains
its independent native evidence. Open PRs #705/#716: #705 workflow/latency diagnostics remain
independent; #716 is the existing search/fixture candidate and may carry new I302 diagnostic
commits only after claim merge and refreshed base/head. Maintainer's explicit accepted adjacent
diagnostic authorizes this bounded selection without pausing or claiming another active slice.

### Dual-Owner Delivery Contract

No rule requires one implementation PR per Story. If #716 carries this deliverable, preserve
separate logical commits/work slices and synchronize both owners. I302 PSS work starts in a
worktree derived from the effective claim merge or later main; refresh #716 to include that main
before publishing PSS. Obtain a complete fresh protected review/CI of the combined head.
Diagnostic-native evidence is not I293 Search acceptance or original-defect repair.

### Risks / Rollback / Documentation

PSS can allocate/perturb observations; names are collected internally under the bounded authorized
flag but excluded from copied output. Numeric handles can be reused. Neither differences nor
stable totals prove object identity or lifetime. ABI strings require bounded copies under marker
ownership, checked releases and no double free. Retain an uninstrumented cold worker and original
failure after observation. Revert/remove helpers or explicitly review their retention. Operator
docs are ADR-091/this owner/TEST-003; no changed user feature or release docs.

## Actual Activation And Execution

2026-10-14: accepted proposal selected for governance preparation. Draft claim/ADR are not
effective; PR number backfill, validators, exact-head protected review/CI and CAS remain.
No PSS code, new dependency feature or unsafe has been implemented.

## Verification / Completion / Residuals

Governance checks pending. Implementation/native evidence Pending; no Completion Commit exists.
Keep I302 Active/Review as appropriate until diagnostic acceptance and existing evidence are
recorded. Keep #643/#624 open and original #716 failure unresolved until actual repair gates pass.

## Final Atomic Claim Proposal — #718

This Active/Claimed record and ADR-091 acceptance are proposed only until finalized #718 merges.
Draft PR number backfill is complete; independent contract preview found no blocker. First local
validator misread a terminal sibling's inventory cell as I302 completion; terminal phrasing was
corrected without adding false I302 Completion Commit or changing another owner. Corrected draft
governance(SQL17/delivery12)/claims0warnings/diff pass; final exact-head checks/review/CI/CAS remain.
No PSS implementation, dependency feature, unsafe or runtime change exists in this candidate.

CI3178/run38524228263 on #716 head5c49a8bd also failed cold73->76 with the identical first-use
trace. All40 iterations reached the final sample after the new per-operation cleanup mask checks,
so the known CloseHandle/TerminateProcess calls returned success. This excludes failed BOOLs at
those sites, not unaccounted handles or a one-time leak. Five other CI jobs passed. Preserve this
failed run as evidence; it makes the accepted type observation useful, not an existing root cause.

Final governance checkpoint: pinned locked-metadata governance(SQL17/delivery12), claims0warnings,
classifier14, diff and scale assessment pass. Independent full draft native/security contract
preview found no blocker; release-error no-retry and synthetic-pointer rejection limits are
included. Eight changed files are documentation only; native code/manifests/locks/workflows are
unchanged. Trusted target-branch classifier determines the applicable governance CI route; no
native execution/full implementation validation is claimed here. Fresh exact-head protected
review, applicable CI, current-main/head/tree/Issue/inventory CAS still precede merge.

## Effective Main / Recovery Candidate — 2026-10-15

#718 merged f4313237c4212e59f57202e22df2ed9b10f78681; protected contract review6163000640,
applicable CI3180 and CAS6163082933 establish the effective ADR-091/claim. Earlier proposal
paragraphs are preserved history. Current main9b7ba281 also contains independent #720/I303;
#721 skill implementation and #705 latency work retain their owners. New worktree starts from
this main and locally merges original7165c49a8bd in a separate baseline-refresh commit.

Workspace maintenance removed unpublished local candidate d1d818e2/treeeed12027, source,
toolchain and logs before remote publication. Reviewer retains its historical local APPROVE,
not restorable complete source or a remote commit. Prior3705primary+1child local passes,
feature graphs and cross-Clippy therefore do not qualify this reconstructed candidate. Old
Issue comment6163468017 records that old unpublished checkpoint, not delivered code.

Reimplementation follows the same accepted boundary with fresh fingerprint/evidence: PSS only
cfg(test,windows), current process, fixed HANDLES|NAME/zero context, bounded owned value/types;
no ObjectName/union/reserved-field inspection. Successful OS walk/live marker establishes type
buffer provenance; null/alignment/even128-byte UTF16 checks alone are not arbitrary-pointer
safety. Checked marker-before-snapshot release consumes ownership before calling free, attempts
both cleanups even on error and never retries uncertain descriptors. Collection panic is caught
before checked cleanup; fallback releases only resources never attempted.

Three exact-test children use existing assigned-before-resume/kill-on-close outer Jobs,60s
event wait, failure termination/10s terminal drain,16KiB retained output with continued draining.
Cold mode0 is explicitly set in its owned environment; no observer warmup, original40 exact
faults/baseline/independent final/+2 remain. Mode1 records four snapshots: baseline, no-launch
observer control, first Pipe and final40; counts around each observer expose observer effects.
Mode2 alone exercises four native captures: valid owned copy, precise post-capture error,
precise post-marker error, caught post-marker panic with exact error/no cleanup suffix.
Shared-harness tests make no PSS call. All launch/exit/receipt failures are Results recorded
before attempting later workers and printed before final assertions; original failure stays.
Every worker uses a fresh owned directory/receipt.json with checked child_pid and mode receipt.

Caps remain1024entries/4captures per worker/32changed entries/16KiB output;12KiB JSON reserves
4KiB for harness/controls/trace. Cap/API/invalid-data failures are not zero or complete evidence.
Pure fixtures exercise rejection-only fabricated pointers, real owned valid buffer, invalid UTF16,
unknown type, ordered dual-release errors/no retry, entry/change/output caps and bad receipts.
Existing grandchild-reap fixture loses only its redundant let-unit binding, retaining wait,
deadline and expect; no lint waiver/production launcher change.

New local protected preview found no blocker. Pinned1.99 toolchain/Windows target and verified
local MinGW/cache are being restored. Fresh check/Clippy/tests/full preflight, both validators,
actual Windows production/test graphs, remote protected review and unfiltered native CI are
Pending until recorded below. Only prior exact local socket filter is authorized. CI3175-3178
remain failed history;3178 all known cleanup BOOLs passed but cold73->76 unresolved. No release,
default Search activation, diagnostic completion, root-cause closure or716 merge follows.

### Fresh Local Checkpoint

Reconstructed candidate pinned1.99 locked release_preflight passed: workspace check/Clippy/tests,
public-site/installer/text/classifier/governance/claims and both Runtime SDK fixture modes.
Harness receipts:3705 primary passes plus1 existing exact-selection child pass;0failed/ignored.
The457 sibling filters in that child are not additional parent-suite skips.
The temporary external cargo wrapper adds ONLY the previously authorized exact Unix-socket
filter to the workspace test command; source scripts/CI remain unchanged and remote tests are
unfiltered. Root Cargo.lock unchanged. The old failed unfiltered run is preserved history,
not represented as a new unfiltered pass. Windows coding test-target Clippy(-D warnings)
cross-compiled anew with restored Windows GNU std and six SHA256-verified local MinGW archives.
Cross-compilation is not native execution. Actual Windows coding and full-workspace/all-feature
normal+build graphs exclude ProcessSnapshotting; coding dev/test graph includes it.
Both governance validators pass with0warnings; diff/fmt/scale checks pass.

Owned child environments explicitly set RUST_BACKTRACE=0 so the intentional caught-panic fixture
does not inherit an unbounded backtrace; no process-global mutation. Fallback diagnostic writes
ignore broken-pipe errors to avoid a second panic after resource cleanup. Fresh exact-head
protected publication review and new full CI/native observations are still required. This is a
stable diagnostic stage, not an existing defect repair or merge-ready candidate.

## CI3190 Result And Bounded Stage Follow-up — 2026-10-15

Published a39ee9590428630a28ea8deb1781608e76ac9abb/tree797191a0 has exact protected remote
publication review6184162949 (all20 changed blobs matched). CI3190/run38679585476 FAILED:
Windows tools453passed/1failed/0ignored/0filtered; all five other jobs passed. Cold mode0 still
fails73->76, first Pipe+1 and first Assignment+2 then stable through40 exact faults. Mode1
observer and mode2 native fixtures succeeded despite the preserved original failure. Four
snapshots baseline/observer_control/first_pipe/final_40 had sizes73/73/74/76, with matching
counts before/after each observation. First_pipe adds/retypes File\u0000 value168; final_40
adds/retypes unknown value328 and Key\u0000 value332. Raw type names retain trailing NUL;
unknown remains unknown. Values/reuse/observer effects cannot establish object identity, leak
or legitimate initialization. Parent cleanup Ok and terminal true are not complete lifetime proof.

Source short-circuiting proves the injected Assignment path does not execute actual
AssignProcessToJobObject. Its preceding preparation/CreateProcess/failure-cleanup interval is
still unresolved. New independent modes3/4 narrow that interval using existing operations:

- Mode3: control_baseline, first_pipe_cleanup, attributes_prepared_then_rejected using the exact
  existing quote error after UpdateProcThreadAttribute, suspended_process_cleanup using exact
  existing Assignment error.
- Mode4: control_baseline, program_resolved, environment_built, suspended_process_cleanup.
  The existing builder reads/copies worker environment; immediately drop its result, with no
  extra observational copy/inspection/log/persistence. No new PSS/environment capture or output.

Both have four existing bounded captures, known phase labels/count caps checked in the parent,
unchanged Jobs/60s wait/10s terminal drain/output caps. A pure fixture rejects reordered or
another mode's phase labels. All five outcomes are attempted/retained before final assertion;
original mode0/cold40/+2, modes1/2, native helper/unsafe/features/lock remain unchanged. Independent
protected local preview found no implementation blocker; exact candidate validation/review and
new native evidence remain pending. No retry of old failed heads or root-cause/fix claim.

Baseline refresh bc5d4ea68342377b2ebba7edfe5f6c14cf4852da includes mainbdea322f and retains
independent #721/#723 I303/SKILL-006 Complete/Closed (implementation6d72975e), I302Review/Claimed
and all I293/#643/#624 product gates. Earlier old-main/recovery receipts above are historical.

A further baseline refresh includes main23e659e2403739b74bc018ec46305346e8e2c7ad (#722),
retaining I304/RUNTIME-011 independent claim and all unrelated owner state.

### Fresh Stage-Control Local Validation

Pinned1.99.0 locked release_preflight passed on the refreshed source: workspace check/Clippy,
3717 primary test passes plus1 existing exact-selection child pass,0failed/ignored; both Runtime
SDK fixture modes passed. Only the previously authorized exact local Unix-socket filter is added
by an external wrapper; source scripts/CI are unchanged and native CI remains unfiltered. The
457 sibling filters belong to that existing child invocation. Root Cargo.lock is unchanged.
Windows GNU coding test-target check and Clippy(-D warnings) passed on final source; this is
cross-compilation, not native execution. Actual Windows coding and full workspace/all-feature
normal+build graphs exclude ProcessSnapshotting; coding test/dev graph includes it. Governance,
claims(0warnings), text boundary23, classifier14, fmt/diff and scale checks pass. Independent
protected local previews found no blocker; final exact local/remote fingerprint review and new
native CI remain required. CI3190 remains failed evidence; stage controls are not a repair.
