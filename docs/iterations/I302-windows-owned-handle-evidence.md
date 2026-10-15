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

## CI3197 / Root-Cause Experiment — 2026-10-15

CI3197/run38685760669 on770d856e/treeabef7bb2 FAILED cold73->76, Windows454passed/1failed/
0ignored/0filtered, five other jobs successful. Exact protected remote review6184988997 approved
publication only after all20 remote blobs matched. Modes1/2/3/4 succeeded. Mode3 sizes/counts
73/74/74/76 at baseline/Pipe/attribute rejection/suspended-process cleanup; mode4 73/73/73/76
at baseline/path/env/suspended-process cleanup. Snapshot before/after counts match. Mode3 adds/
retypes File\u0000 value176 at first Pipe, unknown336/Key\u0000 value340 at final. Mode4 final
adds/retypes File\u0000 value188, unknown336, Key\u0000 value340. Values are local to each
worker and reusable, not shared identities. This still does not prove a resource leak or cache.

Success criteria for the next experiment: distinguish growth in the API-return interval from
growth during cleanup, and compare API-returned numeric values with observed survivors. No new
native API/unsafe/rights/features or fault is necessary. Optional concrete test-only observer
in existing hooks is None for original workers. Modes5/6 use existing Pipe/Assignment faults
and exactly four checkpoints each: before call, successful API return, before cleanup, after
cleanup. Copy only seven existing returned handle slots into bounded receipts; no dereference,
duplicate, close or ObjectName lookup. Post-cleanup values mean historical returns, not owners.
Each checkpoint's counts/PSS/copy/storage are guarded together; error/panic records incompleteness
and lets original cleanup finish. Explicit output/receipt assertions follow cleanup, although an
ordinary caught-panic hook may output earlier. Original mask and cold40/+2 assertions remain.

Fixed four slots and existing4captures/1024entries/32differences/12KiB JSON+4KiB reserve caps
remain; final JSON cap includes the new call records. Parent validates exact four checkpoint
order/roles/counts/seven numbers; pure fixtures reject missing, wrong-mode and malformed records
without native calls. All seven worker launch/exit/receipt outcomes are attempted before final
assertions. Total parent waiting budget increases with workers; individual60s wait/10s failure
drain is unchanged. Independent protected design and implementation previews found no blocker.

Primary API docs establish returned-resource responsibilities, not Windows internal cache identity:
[CreatePipe](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-createpipe),
[CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw),
[CloseHandle](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle).
No authoritative source found establishing this runner's +3 as legitimate initialization. PSS can
change first-use behavior, so these controls alone cannot replace original cold evidence. A
surviving returned numeric value may have been reused; a non-returned value is not automatically
an OS-owned legal cache. Keep uncertainty until direct evidence supports a specific conclusion.

Maintenance pruned the previous source/toolchain/cache again. Recovered source starts from exact
remote770d856e and main23e659e2; old local passes are historical. Fresh pinned locked preflight,
Windows coding test-target cross-check/Clippy, graphs and governance checks are being repeated.
No repair, Complete state, production activation, #716 merge or #643/#624 closure is claimed.

Interpretation checks for the next receipt:

- Growth already present at successful API return, beyond its recorded returned slots, supports
  acquisition inside that instrumented call interval; it does not prove a legal OS cache.
- Growth first appearing during cleanup redirects investigation to that interval. Successful
  known closes and observed numeric disappearance still do not identify every resource lifetime.
- Any before/after-snapshot change exposes observer effects. Missing/error/oversized receipts
  fail collection, and matching numbers alone cannot exclude all first-use observer effects.
- Original cold failure remains authoritative alongside the instrumented evidence; no changed
  baseline, warmup, skip or threshold follows from a diagnostic success.

### Local Failures And Separate Ownership Findings

First Windows cross-check failed on the new fixture's unsupported json!([0;6]) syntax. Fixed
to a Rust vec! expression; fresh fixed-source coding --tests check and Clippy(-D warnings) passed.
Native helper/features/lock unchanged; actual Windows coding/full-workspace all-feature normal+
build graphs exclude ProcessSnapshotting, while coding test/dev graph includes it. Governance
(SQL17/delivery12), claims0warnings, text23, classifier14, fmt/diff/scale checks passed.

Fresh full local release_preflight FAILED, not passed. Workspace check/Clippy passed; tools unit
target reported457passed/1failed/0ignored/0filtered. Failure was untouched Linux test
search_backend::transport::tests::enclosing_deadline_covers_dns_and_response_wait_and_drops_socket,
tests.rs:953, "owned connection closes: Elapsed(())". No rerun-to-green or new filter was used;
only the prior authorized exact Unix-socket filter was in the external wrapper. The failed
script stopped before remaining targets/SDK; their independent checks must not be combined into
a claimed full-preflight pass. This Windows-only change has no established causal link to that
Linux failure. Preserve the failed log/receipt and merge gate.

Source review shows a separate fixture ambiguity: second request has150ms total budget without
first proving accept/request arrival, yet requires peer EOF within2s afterward. Deadline before
connect and non-timeout read errors both leave closed==0; the current failure cannot distinguish
those paths from non-release. These are hypotheses/contract gaps, not this run's proven cause.
Independent protected review found no blocker for diagnostic publication with this explicit
failed local gate, original native failure and fresh exact-head review/full CI retained. No
merge or all-validation approval follows. Separate I293 test-only terminal telemetry is added after this failure; no transport behavior,
deadline, EOF success condition or skip changes. A later pass is not proof of repair.

Another confirmed API-contract gap is outside I302 production authority: CreatePipe documents
indeterminate output parameters on return0, but the launcher passes owned slots directly and
later closes any non-null value. Initial null is not proof of ownership after a failed call.
This needs separately claimed ADR-068 production launcher/security maintenance with success-only
commit of temporary outputs. It is NOT the observed +3 cause: current Pipe injection is after
successful return, and all40 exact fault checks reached their intended errors. Independent
protected source review agrees; no production repair or ownership transfer is made in this slice.

The adjacent I293 fixture snapshot is non-atomic: committed shapes exclude in-flight request
buffers; accepted==0 lacks an accept receipt but does not exclude a queued TCP connection. EOF
counts reads, not unique connections. errors_unrepresented can reflect in-flight recording or
the four-kind cap. These receipts aid the next failure diagnosis; they are not atomic proof.

### Fresh Combined Diagnostic-Source Checkpoint

After adding the bounded I293 terminal receipts (a materially new diagnostic version), pinned
locked full release_preflight PASSED:3717 primary+1 existing exact-selection child,0failed/
ignored, workspace check/Clippy and both SDK modes. Original rootcause-preflight FAILED and
its457/1 EOF receipt above remain evidence: the later pass is not a repair or scheduling-cause
proof. Only the already-authorized local exact Unix-socket filter is used externally; scripts
and remote CI remain unfiltered. The457 sibling filters belong to the existing child invocation.
Independently continued TUI609 and normal tools-integration/workspace-doc71 receipts passed;
those overlap later validation and are not added to its totals. Standalone SDK modes also passed.
An initial continuation command lacked required feature gates and executed no tests; corrected
to coding/normal targets, without retrying the failed unit target as a pass substitute.

Combined final-source Windows coding --tests check/Clippy(-D warnings) passed, with no native
execution claimed. Both governance validators0warnings/text23/classifier14/fmt/diff/scale and
actual production/test feature graphs passed. Root lock/manifest/native helper/ABI/unsafe sites
are unchanged; native call-site inventory/order is unchanged apart from cfg(test) observations.
Independent protected design/code/combined previews found no blocker. Stable local and remote
fingerprint review plus new unfiltered native CI still precede interpretation/acceptance. No
rootcause closure, local-failure erasure, Complete state, production repair or merge authority.

## CI3198 Call Boundaries And Installed-File Identity Stage — 2026-10-15

Published d90676f7a8f423f48c6fc0809d69616a5f27dad5/tree9d14f9d8 received exact-remote
protected publication review6188161308 after all20 changed blobs matched. Full CI3198/run38707880538
FAILED: Windows tools455pass/1fail/0ignored/0filtered, five other jobs succeeded. Cold40 remains
73->76; modes1–6 succeeded. Native job116331772442 receipts are retained in #717 comment6194344099.

Mode5 counts73/76/76/74 at before_pipe/pipe_created/before_cleanup/after_cleanup. Successful
CreatePipe returned308/312; these disappeared after cleanup. Extra File304 was already present
at pipe_created. Mode6 counts79/83/83/76 at before_process/process_created/before_cleanup/
after_cleanup. Successful CreateProcessW added returned Thread312/Process316 plus unknown328/
Key332. All seven API-return values (four pipes, Job, process, thread) disappeared after cleanup;
unknown328/Key332 remained in the copied table. Counts before/after each PSS observation matched.
Independent protected read-only interpretation verified these raw receipts and exact source.
For these instrumented executions, extra growth exists in the successful API call intervals,
not first in later cleanup. This is not object/lifetime identity, cold causation, all-resources-
released proof or elimination of PSS/concurrent OS effects. Actual Assignment API remains
short-circuited. Original failure is retained; no cache qualification, repair or merge acceptance.

Offline read-only analysis downloaded the public Microsoft symbol-server kernelbase.dll sample:
https://msdl.microsoft.com/download/symbols/kernelbase.dll/4370855D401000/kernelbase.dll
SHA256 bffbb14b005b7b2764dbde1a95512aeb95662215fc33c780618435cde6661d6a,4254808bytes,
FileVersion10.0.26100.9549. Official download/hash is not a completed Authenticode trust validation.
Third-party Winbindex was only a locator, not primary authority. PE exports/imports and disassembly
show CreatePipe RVA1007f0 testing global RVA3ac9d0, opening the static Device/NamedPipe root via
NtOpenFile, publishing with compare-exchange, closing a losing candidate, then using the cached
root in NtCreateNamedPipeFile. Independent reviewer verified this path/hash. This establishes a
cache mechanism in this sample and a concrete candidate for the +1, NOT that CI's File304 is that
cached handle. Installed runner module bytes/version have not been matched. This cannot explain
CreateProcess unknown+Key. Matching-GUID public PDB has InfoAge2 versus PE/DBIAge1; do not claim
strict symbol identity. Export/import/static-string evidence above does not require PDB names.
No downloaded Windows PE executed, process memory read, additional PID or observer API used.

Next bounded evidence stage reads only the three fixed installed System32 files kernelbase.dll,
kernel32.dll and ntdll.dll before the unchanged Windows workspace command. PowerShell uses a
read-only stream (sharing read only), rejects reparse points and empty/>=32MiB files, hashes in
16KiB chunks with an actual32MiB read cap, validates observed length, and disposes each stream/hash.
Output is three fixed-shape basename/phase/status/byte-count/SHA256 records, under1KiB total;
no contents, full paths, exception bodies or environment values emitted/saved. Hashing necessarily
reads bytes. No file download, PE execution/parser, version API, process/module enumeration,
PInvoke, Talos ABI/unsafe/dependency, PSS or production behavior change. Identity collection failure
is explicit and all original workspace tests are still attempted; a test failure retains its exit,
otherwise incomplete identity fails the step. Existing20min step budget now also covers this small
prelude. #705's other workflow steps are unchanged. The identity is an on-disk sample at collection
time, not proof of the module mapped into the later test process; no atomic loaded-file claim.
Independent protected scope assessment accepted this as existing I302 evidence, not an ADR/claim
expansion. Fresh workflow review/checks/exact-head native CI remain required. No Completion Commit.

Local convergence for this workflow/document-only delta: official PowerShell7.6.7 Linux asset
SHA2566896f9e3a6b0a354bf470835a96da76d2f801cd50dcf047ad7c4563bf93bf431 verified. Parser
accepts the extracted exact step. Disposable real-file fixtures independently match all three
SHA256/length records, reject reparse points and >=32MiB, preserve cargo execution and exit1/7
priority when identity is incomplete. A protected draft review found unguarded Dispose could
skip tests; both releases now have independent catches, mark the receipt incomplete and attempt
all remaining releases. Fault fixtures verify both throwing releases/all six attempts and no
exception-body output. Initial exit fixtures used incorrect -Command/-File wrapper propagation;
corrected explicit wrapper propagation passes, without treating those harness failures as native
Windows outcomes. No fixture/mock code is added to CI or the repository. Linux PowerShell cannot
prove Windows System32 behavior. Text23/classifier14/claims0warnings/diff checks pass.
Workspace maintenance removed the previous checkout/toolchain; fresh checkout is exact d906.
Governance first failed missing cargo, then cargo-only recovery failed missing rustc; neither is
reported as a pass. Restore pinned1.99 tool executables and rerun. Rust/native sources are unchanged
from failedCI3198 and its recorded local preflight; no new Rust/full-preflight pass is claimed for
this workflow-only delta. Fresh exact-head unfiltered CI remains the native check.

Recovery completed: official pinned Cargo/Rustc archives verified; two truncated extracted native
libraries were detected by archive sizes and restored with checked chunked extraction. Rustc1.99
and actual locked dependency metadata now run. Final governance passes SQLite17/delivery12,
0warnings; claims0warnings/text23/classifier14/diff and exact-step PowerShell parser/fixtures pass.
This is workflow/document validation, not a new Rust build or native Windows pass. The revised
independent protected draft review found the Dispose blocker fixed and no remaining publication
blocker; final fingerprint review and new full CI remain outstanding.
