# SESSION-UI-001 — Queue visibility and user-history continuity repair

Status: Review. Claim #952 is effective on main; local implementation and standard validation passed, remote review/CI and native acceptance remain open.
Owner: this document. Source: the maintainer's existing session-recovery handoff and
reported missing queued text / missing blank row after user input.
Primary-worktree source: `docs/tasks/2026-11-11-session-recovery-handoff.md`,
section 4; screenshot `/tmp/codex-clipboard-rjjsXe.png` and retained ANSI evidence
`target/recovery-user-separator-pty-result2/result.json`. These local artifacts
are provenance references, not files shipped by this governance-only candidate.

## Outcome and scope

Keep accepted but not yet projected user input visible, preserve a readable queued
body when the terminal has room, and separate a completed shaded user block from
live preview by one unshaded blank row. Verify animation continues through intermediate
tool/error progress until authoritative turn completion. These are existing handoff
items, not a new feature set. This task owns only their CLI/Conversation/TUI repair,
regressions and user documentation; history recovery remains PR #917's own slice.

Exclude approval history/policy, edit/diff line locations, ellipsis/theme redesign,
background enablement, provider retries, new storage formats, releases and deployment.
The completed TUI-026/TUI-044/TUI-049/TUI-060 and ellipsis owners retain their evidence;
this task does not reopen them or borrow their closed claims.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | SESSION-UI-001: accepted queue presentation, queue row allocation, external user-block separator, terminal lifecycle regressions and docs |
| Claimed At | 2026-11-15 |
| Source Issue | None |
| Governance Claim PR | #952 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer requested completion of the existing handoff; effective target-branch claim required before committed implementation |
| Implementation PR | #962 |
| Last Updated | 2026-11-16 |
| Handoff / Release Condition | Effective claim, isolated source validation, exact-head review/CI and native terminal acceptance |

## Hard constraints and implementation boundary

Actor acknowledgement transfers dispatch custody, not permission to erase pending
presentation. Any retained preview is bounded, correlated by submission/receipt and
session generation, and cannot become a second dispatch queue. Stale events cannot
clear current input; resolved terminals and runtime replacement remove stale previews.
No duplicate submission, reordered FIFO, provider-switch bypass or cancellation-policy
change is permitted. Additive public API must remain semver-compatible; any breaking
change requires its own ADR and migration plan before implementation.

Layout remains within terminal bounds. Required credential/approval controls and an
editable composer take precedence. Optional separator cannot displace queued body or
the last history row; extreme short-height fallback must be explicit in tests. Use
actual buffer colors: an empty row inside the user background is not an external gap.
The separator applies only at a FollowTail user-background tail before live preview;
anchored scroll positions and absent previews receive no extra gap. Selection mapping
and same-instance resizing must remain correct. Animation truth remains authoritative
completion, not temporary lack of output.

## Runnable deliverable and acceptance

1. During a long live preview, enqueue CJK/multiline input. Actual frames show queued
   body when space permits and preserve controls through narrow/wide and short/tall resize.
2. After Actor accepts but before projection, pending text stays visible. Test receipt/event
   order, paused/error/cancelled state, stale receipt, generation replacement and later queued
   input. AcceptedPending/PausedPending can remain visible while paused; matching
   terminal cancellation/error and Running user projection clear the corresponding preview.
   Exactly one intended provider submission occurs; preview never requeues work.
3. After direct or queued input enters history, the last shaded row is followed by an
   unshaded empty row before Connecting/live content when space permits. Real ANSI PTY
   evidence includes 40x24, 24x12, 80x24 and 40x12; short-height tests preserve required UI.
4. Feed actual bridge progress/error and terminal events through the production projection.
   Spinner remains active through intermediate states and stops after Success/Cancelled/Error
   completion. Native terminal observation supplements deterministic frame tests.
5. Update README.md and README.zh-CN.md with queue visibility, spacing and tiny-terminal
   fallback behavior, without claiming background continuation or changed execution timing.
6. Pinned/locked focused CLI, Conversation and TUI tests, standard release_preflight,
   independent exact-head review, relevant native CI and terminal acceptance pass before
   delivery. Existing experiments are not selective-candidate completion evidence.

## Baseline, inventory and preserved experiments

Target observed: 5b2fa435 (2026-11-15). Header inventory retains I249 Planned;
I277/I302/I318/I325 Review; I323/I324/I327 Active or proposed Active exactly as their
owners state. No Blocked iteration header was found; I164 remains Paused. No iteration
is selected/activated here and no unrelated owner state is changed. Scale harness
recommends high-risk, release-managed, required worktrees; existing policy is retained.

Nine uncommitted files are preserved separately from recovery commits, split into
geometry (five), accepted queue projection (three), and animation regression (one).
Primary artifact directory: target/recovery-ui-delivery-slices-20261115; its manifest
binds patches to recovery head55ba5752. Disposable-index application/whitespace checks
pass; originals and all mixed work remain retained. Prior 696 TUI tests and isolated
production CLI PTY screenshot evidence establish experiments only, not a new isolated
full gate or installed-host acceptance. Future extraction must recheck target overlap.

## Closure ledger and next step

Requested outcome: close these existing UI handoff defects with runtime evidence.
Artifacts: this owner and derived Board entry; implementation and both READMEs later.
Preserve: existing owners, claims, dirty code, original session data and patch artifacts.
Validate governance with both repository validators, link/whitespace checks and independent
scope review. PR #952 proposes claim/activation atomically with its actual number recorded;
then require target-branch merge before implementation branch/commits. No merge authority
is inferred from this document. All incomplete SESSION-UI-001 work remains owned here;
no Complete or implementation commit is asserted.

Local scope review found no blocker; follow-tail, selection/resize and correlated
terminal-cleanup acceptance were clarified before publication. Initial governance
validation passed with zero warnings; claim validation passed with one pre-existing
SEARCH-001-D3 documentation-intake warning outside this work slice.

## 2026-11-16 effective claim and isolated implementation — Partial

Maintainer explicitly authorized governance merge. Claim #952 became effective at
36eb752844e24a15ed295532d186d3bb6027c458. Implementation branch
fix/session-ui-continuity-20261116 starts from later main e1cbee1e in the isolated
worktree target/session-ui-implementation-20261116. The primary dirty worktree,
original session data, migration backup and retained patches remain preserved.

The candidate reserves queue header/body rows before optional spacing and adds an
unshaded separator only at a FollowTail user-background tail before live preview.
Accepted input remains in a display-only cache until correlated projection or terminal
resolution; runtime replacement clears it. The cache holds at most one prepared
batch (32 items / 256 KiB text, plus the items' existing attachment metadata), in
addition to the pre-existing queue. It is not zero extra storage. Rendered snapshots
remain bounded to 8 entries / 4096 UTF-8 bytes each and never dispatch the cache.
Public API additions retain existing signatures; no persistent format changes.

Locked local library tests: 195 Conversation and 691 TUI tests passed, zero failures
or ignored tests (target/ui-focused-20261116.log). Separate CLI binary tests passed
426 tests, zero failures/ignored (target/ui-cli-tests-20261116.log). The library-only
command did not include CLI bridge tests; the latter is the authoritative bridge
validation. Additional selection/anchored regression validation is in progress.
Both READMEs describe queue custody visibility, external spacing and tiny-terminal
priorities. Independent Agent local-diff review found no blocker; this is not final
exact-commit review or independent human/device acceptance.

Remaining: new geometry regression results, isolated actual CLI PTY (CJK/multiline,
resize, direct and queued-history colors, exactly-once requests), full standard
preflight, stable candidate/commit review and native CI, and native terminal
observation. No product merge, installation or background activation is authorized
by this claim merge. Handoff remains Partial; no Completion Commit asserted.

### Additional runtime and geometry evidence

Additional focused TUI run passed 9 user-related tests (683 filtered), including
same-instance resize/selection mapping and actual ScrollUp removal of the gap in
Anchored mode: target/ui-geometry-tests-20261116.log. Format and whitespace checks
passed. Production CLI build passed; SHA256
397ad5abcb4ce40331cfca058f6f6ec4978cc8788d99f1680e51c14842294cd8.

Actual CLI ANSI PTY acceptance passed at target/session-ui-pty-20261116-r2/result.json:
8 color-checked separator observations (direct and queued-history at 40x24, 24x12,
80x24, 40x12), plus 7 long-reasoning queue frames including 24x6 and 80x8. Bracketed
paste supplied two CJK lines; request capture verified the complete multiline input
appeared exactly once in the second request, with two provider requests total.
Queue body remained visible through same-instance resize. Raw ANSI output is retained
in that artifact directory. Initial harness startup failed before product execution
because system Python lacked pyte; isolated pinned pyte0.8.2/wcwidth0.2.13 wheels
were installed only under target/ui-pty-python-20261116 with PyPI SHA256 provenance.
No user session or system Python package was changed.

These are production-binary/controlled-provider/terminal-emulator observations,
not installed-host acceptance, a real provider quality claim, or native-terminal
visual approval. Full standard preflight and final candidate gates remain open.

### Full-gate locale correction

First full preflight exited101 after873.78s; all2649 manifest files were unchanged.
The Tools suite had1160 passed/1failed: hasconfig_non_path_key_still_triggers_url_prohibition
expected an English native Git diagnostic. The launcher omitted LC_ALL=C/LANG=C,
contrary to LOCAL-DEV.md's documented service-environment requirement. The exact
same compiled test passed under the prescribed locale, without source or assertion
changes (target/ui-git-locale-check-20261116.log). This focused result does not close
the full gate. Repeat full standard preflight with fixed locale and resource limits;
retain the initial failed log/manifest/result at target/ui-full-preflight-20261116*.

### Current candidate standard gate passed

Fixed-locale full standard preflight completed with exit0 in776.13s, including
5353 workspace/doc tests (zero failures/ignored) and both independent Runtime SDK
fixture feature configurations. Manifest2649files remained byte-identical throughout.
Evidence in the primary workspace: target/ui-full-preflight-20261116-r2.log,
-manifest.json and -result.json. Unit talos-ui-full-preflight-20261116-r2 reports
success, CPU482.718s, memory peak2GiB and swap0; prescribed CPU100%/MemoryHigh2G/
MemoryMax3G/TasksMax128/Nice10 and C locale were retained. Memory soft-limit reclaim
occurred without OOM; this is capped validation, not a performance measurement.

Source implementation is ready for staged-diff review and stable candidate creation.
Exact-commit independent review/native CI and native terminal acceptance remain open;
no product merge, installation, release, or whole-handoff completion is claimed.

### Stable candidate publication

Implementation commit d820d664c3059583baf1e47007b86532747694c4 was reviewed
as a staged scoped diff and pushed as draft PR #962. Only owner/Board evidence
changed after the full validation manifest; the product source remains identical.
Both governance validators passed after owner-first Review/Board synchronization
(existing unrelated SEARCH intake warning only). Exact-head independent review and
native CI are pending. No completion or product merge authorization is asserted.

### Upstream integration in progress

PR #962 could not run native CI while GitHub reported merge conflicts. Integrated
origin/main 5169d5f6 (Git status notification stage #960 and its receipt #961) into
the candidate without rewriting the already-published implementation evidence.
Conflicts were append-only tests in app_tests.rs and Board insertion; both sides'
tests/owner entries are retained. Upstream watcher source/dependencies remain intact.
Formatting and whitespace checks passed. Because upstream changes affect the TUI,
the prior full pass is historical for this integration: rerun standard preflight
before pushing the integration candidate and refresh exact-head review/native CI.

Integration full preflight r3 completed exit0 in513.80s:5360 successful test-summary
passes,0failed/ignored and both Runtime SDK fixture configurations passed. All2661
manifest files unchanged. Unit CPU274.948s,memory peak2GiB,swap0; same documented
resource/locale envelope. Evidence target/ui-full-preflight-20261116-r3.log,
-manifest.json and -result.json. This proves the integrated source candidate;
exact final-head review/native CI and native terminal acceptance remain open.

## 2026-11-17 committed-reasoning separator follow-up — Partial

Actual isolated Ghostty+niri observation of the combined production CLI found that
completed Thinking history directly followed a user background block even though
user→Connecting already had a separator. Both reasoning ingestion paths bypassed
normal stream opening separators. This is the original user-history spacing scope.

The combined candidate adds a regression through actual UiOutput and draw_frame:
both typed/live reasoning paths, pending/committed user tails, folded/expanded/folded,
80→24→80 resize, assistant response and promoted queued input. Red failed as expected;
minimal correction adds one default-background logical blank only after a user-colored
tail. Full combined TUI library passes713/0failed/0ignored. Independent source review
APPROVE. Actual Ghostty rerun confirms the committed Thinking gap plus direct/queued
Connecting gaps and queued body; evidence in primary
`target/handoff-native-ghostty-r3-20261117/` and visual-review.json. Binary SHA256:
`c224a5a651cbfb42b8be7aedbcb98d4d9b81dd80db30e082ddcf6bd67658b07c`.
Native rendering uses live nested-PTY output and scripted input; no installed-host,
physical-key, all-size or GPUI acceptance is claimed.

Only the two TUI files are selectively ported here from that correction. Source patch
applied cleanly; the append-only test was extracted byte-for-byte because earlier
combined tests differ. Provenance in primary
`target/user-reasoning-gap-selective-20261117.json`. No recovery/approval/background
source is copied. Standalone focused/full gates remain pending behind the running
combined preflight81360; no PR update or final completion claimed yet. Keep original
source and failure evidence; background remains default-off.

## 2026-11-17 tool boundary regression and combined preflight result

Reasoning-only combined preflight exec81360 completed successfully (9min2.875s,
peak2G/swap0); log target/user-reasoning-gap-preflight-20261117.log. This evidence
predates the generalized tool-boundary correction. Native edit rendering in
target/handoff-native-edit-20261117/ passed line-location assertions, but its
rendered.png exposed the remaining user-to-tool separator defect. This is original
SESSION-UI-001 scope; spacing is not complete.

Standalone actual UiOutput-to-draw_frame tool regression reproduced failure:
header row14 instead of15, exec99521 exit101, log target/tool-gap-red-20261117.log.
The correction moves separation to commit_pending_transcript, covering typed
reasoning/tool blocks while preserving existing blanks and continuous user padding.
Full standalone TUI validation exec18218 is in progress; no PR update or deployment.
Primary session data, original work and migrated TMPDIR contents remain retained.

Unified separator full TUI run exec18218 completed exit0:701passed,0failed,
0ignored;22.033s,peak815M/swap0 (target/tool-gap-green-20261117.log). Added
negative controls for existing whitespace gap, continuous user padding, plain
non-user output and same/cross-commit behavior afterward. Final standalone
preflight exec27916/unit talos-ui-boundary-preflight-20261117 is now running;
log target/ui-boundary-preflight-20261117.log. Combined source has received the
same bounded separator delta and both tool/negative regressions, preserving its
independent integration fixture; it still needs validation and native rerender.
Remote recovery head15bdc03f CI43936276699 is valid and now in Desktop explicit
feature validation; no cancellation or replacement is warranted.

Final standalone generalized-boundary preflight exec27916 passed exit0,8min33.615s,
peak2G/swap0; log target/ui-boundary-preflight-20261117.log. Independent review
APPROVE: no blocker, typed header indices/anchors preserved, existing gaps and
user padding retained. This includes the new negative controls. Combined final
preflight is now running (target/combined-boundary-preflight-20261117.log);
new native screenshots and exact-head publication CI remain pending. No Complete claim.

## 2026-11-17 final native boundary acceptance

Both actual Ghostty runs passed on binary SHA256
0d746296e1c3316cd0fcfd6ef2ac7f0547b677001836d5aeb38100aabedc9334:
target/handoff-native-ghostty-r5-20261117/ (direct/queued/promoted) and
target/handoff-native-edit-r2-20261117/ (actual edit/diff). Screenshot review
confirms external blank rows before Thinking, Connecting and tool header, visible
queued CJK body, correct old/new diff columns and absence-side blanks. Hashes and
limits are in visual-review.json. Input remains scripted through a nested PTY;
synthetic provider/isolated workspace, not installed-host/physical-key/GPUI evidence.
The r4 attempt failed before CLI launch due to missing pyte; preserved screenshot
and reran using the existing dependency directory, without installing dependencies.
Standalone final preflight totals5363passed/0failed/0ignored. Combined gate remains
running. Recovery exact15bdc03f CI43936276699 is now completed success.

## 2026-11-17 Windows watcher CI failure reproduced and corrected locally

CI43946467676 failed Windows missing_watch_root_reports_failure waiting5s for
process-global BUSY. UI healthy-watch test holds that same watcher9s under
TEST_SCAN_LOCK; five direct watcher tests omitted that lock. Original source
reproduced with full TUI8threads:697passed/5failed, all five watcher start timeouts
(exec2883, target/watch-isolation-before-parallel-20261117.log). Adding the existing
shared test lock to those five tests passed702/0/0 under identical8thread execution
(exec79412,target/watch-isolation-parallel-20261117.log). Production behavior and
timeouts unchanged; no broad retry or ignored test. Independent review/full gates
and new exact-head Windows evidence still required. Failure is not claimed resolved
on Windows solely from Linux reproduction.


Watcher isolation follow-up: independent review APPROVE; standard pinned/locked
release_preflight.sh passed (5363 passed, zero failed/ignored), exit0,8min31.998s,
peak2G/swap0. Log: target/ui-watch-preflight-20261117.log. Eight-thread red/green
evidence above remains applicable; fresh exact-head Windows CI remains pending.


## 2026-11-17 main integration validated

Integrated main e1f8c693 into UI8bcb7b47. Only conflict was appended app_tests:
retained three separator tests and numbered edit/diff resize test with distinct
#[test] attributes. Independent integration review APPROVE; no API/security or
default background behavior expansion. Standard pinned/locked preflight passed
5588/0/0, exit0,15min48.319s,peak2G/swap0; external SDK and SQLite API
fixtures passed. All2681tracked file hashes unchanged during validation.
Evidence: target/ui-main-integrated-preflight-20261117.log and
target/ui-main-integrated-source-manifest-20261117.json.

Old CI43992079353 Windows workspace tests passed, including five watcher tests
and TUI702/0/0. Its later WAL path diagnostic failed because old UI8bcb7b47 lacked
wal_path_probe requested by target workflow. This integration includes that main
example. Raw failure retained; new exact-head CI required, no whole old-run pass.


## 2026-11-17 approval integration final gate

Merged main0cf52249 into UI101687c2 locally. Fixed the stale EventLoop test literal
(remove workspace_root, add runtime_generation:1/input_started:false); real compile
failed E0560 before fix and4approval_queue_tests passed after. Independent review
APPROVE; causal cancellation/cleanup assertions preserved. Production source matches
validated combinedff068667; only a test's ordering differs within crates.
Full pinned/locked preflight passed5622/0/0,exit0,11min52.695s,peak2G/swap0,
external SDK/SQLite fixtures passed. All2686tracked file hashes unchanged. Evidence:
target/ui-approval-integrated-preflight-20261117.log and source-manifest JSON.
Old CI43998984637 was superseded; normal cancel remained live, authorized force-cancel
confirmed completed/cancelled. New exact-head CI still required. No installation,
source-session rewrite, default continuation or Complete claim.

## 2026-11-26 recovery regression follow-up

Commit dfd2736b adds four CLI/TUI regressions under the existing queue, separator and terminal lifecycle scope. The CLI case covers running and cancelling turns. The TUI cases cover history spacing, preview allocation and animation through intermediate tool events. [Reconciliation record](2026-11-11-session-recovery-handoff.md)
