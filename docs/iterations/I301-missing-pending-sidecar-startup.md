# Iteration I301: Missing Pending-Sidecar Startup Recovery

> Document status: Complete
> Published plan date: 2026-10-12
> Planned objective: eliminate database initialization during recovery of an absent pending sidecar.
> MVP deliverable: new/legacy Runtime admits commands without a recovery-only SQLite side effect.

Completion Commit: 98fa04cbdfdb7b30ff4a32d06036427872a41703

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Missing pending-sidecar recovery read fast path, storage/Actor regressions and startup documentation only; no existing-store durability or custody change. |
| Claimed At | 2026-10-12 |
| Source Issue | None |
| Governance Claim PR | #710 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer repeatedly authorized continuing the intermittent-timeout investigation and repair through GitHub CI. No separate maintainer reviewer is available; independent Agent storage/API review, exact-head CI, both validators and merge-time CAS remain required. |
| Implementation PR | #711 (merged as 9cad344b8f7adef855ca6b858e6ccc7723103767) |
| Last Updated | 2026-10-16 |
| Handoff / Release Condition | Bounded startup storage acceptance complete; #705 retains broader timeout investigation. No release. |

## Published Baseline

Selected Story: [RUNTIME-010](../backlog/active/RUNTIME-010-missing-pending-sidecar-startup.md),
Ready; no parent. Preserve all scope, exclusions, Given/When/Then acceptance, validation and
documentation targets in that owner. Accepted ADR-008/056/058 and this claim merge are dependencies.
The final Active/Claimed proposal becomes effective only when merged into main.

### Selection Inventory And Dispositions

Selection baseline: main `f036111f6e969feea47ad784f3cb3effd7e20a33` (includes #708).
All iteration document headers were inventoried on 2026-10-12:

| Owner | Current state | Disposition |
|---|---|---|
| I293 | Active / Claimed | Retain other-session Search router scope; no transfer or overlap. |
| I294 | Active / Claimed | Retain other-session Search provider evidence; no transfer or overlap. |
| I300 | Active / Claimed (#704 merged) | Retain T7 Git permission/diff repair; no overlap in behavior or implementation files. |
| I277 | Review | Retain deferred human/device acceptance. |
| I249 | Planned | Retain dependency-pilot deferral; no dependency version change here. |
| I164 | Paused | Retain superseded layout target; no reactivation. |
| DEPENDENCY-003-A | Review / Claimed | Retain separately owned native dependency evidence and platform gates. |

No Blocked iteration header was found. I290/I291/#590 and I296-I299 remain terminal.
Existing explicit non-overlap authorization plus the maintainer's continued direction governs this
independent Runtime repair; this record does not activate another owner's work.
Only open PR found at selection was #705, whose bounded diagnostic exception excludes production
storage changes. RUNTIME-008/009 and SESSION-011 are unclaimed future semantic/API work; this repair
preserves their contracts and does not claim them. Recheck inventory/overlap before merge.

### Risks And Rollback

Filesystem absence must preserve real I/O errors; concurrent creation cannot turn a read snapshot
into custody authority. Existing journal schema validation and WAL/FULL remain unchanged. Revert only
this repair if regression evidence shows loss, hidden error or altered Actor recovery; retain all
historical failing CI evidence. Windows runner timing is variable and cannot prove universal latency.

## Actual Activation And Execution

2026-10-12: governance-only draft prepared; no production/test/dependency edit or implementation
branch. Official Rust 1.99.0 toolchain is available locally. Final PR number, exact-head checks,
independent review and merge-time CAS must be recorded before claim becomes effective.

## Verification Evidence / Completion Evidence

Pending governance checks and implementation. No Completion Commit exists; not Complete.

## Variance And Residuals

#705 retains PowerShell latency, old-failure attribution and existing-store startup cost. This
iteration does not claim the entire intermittent-timeout investigation is closed.

## Atomic Claim Proposal — #710

The Active/Claimed record is a proposal only until finalized #710 merges into main.
No production implementation or implementation branch exists. Both validators, relevant exact-head
CI, independent Agent storage/API review and merge-time CAS are required before merge.

## Implementation Checkpoint — 2026-10-12

Claim #710 merged as `42b861125d78aab0707528fc5fdec5e14ede0d19` after CI3156,
independent Agent review6127973689 and merge-time CAS6128063256. Implementation branch starts
from that main commit, including other-session #709. Claim is now effective; proposal paragraphs
above are preserved publication history. Delivery remains Review, not Complete.

The candidate adds an error-preserving absence check under the existing store guard. An existing
final directory entry takes the original SQLite route. Absent entries require a verified existing
parent directory; dangling ancestors, loops and permission failures remain errors. A negative read
is a snapshot only; normal durable admissions and subsequent acknowledged-work recovery are unchanged.

Conservative path exception: any `..` component with an unavailable final entry takes the original
SQLite/directory-initialization route. A real regression proved that `missing/../session.tlog`
otherwise returns zero records despite an acknowledged journal at the canonical target (0 vs 1).
Creating the missing intermediate directory can reveal that existing journal. Preserve this legacy
path behavior rather than normalize paths or claim zero side effects for this ambiguous case.

Validation so far: 231 session tests passed before the final ParentDir regressions; one exact real
Actor Submit/Success/no-sidecar test passed. Final storage, full preflight and exact-head remote
Windows/Unix CI are pending. Initial Actor fixture type errors were corrected locally before
publication. Default-debug baseline full preflight ran out of the local 32GB disk during test
linking; only owned generated cache was cleared, interrupted test writes restored from Git, and
local builds now use environment-only debug=0/incremental=0/jobs=2. No dependency/lock/pin/assertion
or remote CI policy change. Nonprivileged `runuser` verification was blocked by unavailable group
switch capability. The initial suite did not capture an individual branch result; inferring
privileged NotFound solely from process identity was unsupported. Later exact fixture output
explicitly observed PermissionDenied locally; remote CI must independently prove the denied branch. Full baseline/preflight failures and the
pre-correction ParentDir test failure are retained, never replaced with retry-to-green evidence.

Source/test/doc review and final checks remain required. #705 owns PowerShell active-child latency
and attribution of earlier failed samples; no overall timeout-risk closure or release is claimed.

Final storage-path validation checkpoint: the corrected ParentDir candidate passed all 233
session unit/integration tests (206 unit + 27 integration), zero failures/ignored. Both prior
ParentDir failure runs remain retained. The existing macOS full-validation CI chain now repeats
only the exact permission fixture with ErrorKind-only output and requires one passed test plus
PermissionDenied. No new workflow/job/permission/deadline or dependency is introduced; this is
associated storage-regression validation, not production policy. Final full preflight and remote
implementation review/CI remain pending.

Local environment boundary: full preflight reached workspace tests but failed the unchanged
`talos-runtime::tests::runtime_evidence_rejects_directory_symlinks_and_socket_entries` at its Unix
socket fixture (`Operation not permitted`). This sandbox does not permit Unix socket creation.
No assertion was removed/ignored and no success is claimed for that run. Final locked format,
workspace check/Clippy/governance and affected-crate tests are rerun after source convergence;
remote exact-head full CI must execute every workspace test, including this fixture, before merge.

Stable source evidence: `cargo test --locked -p talos-session -p talos-agent` passed 733
unit/integration/doctests, zero failed/ignored, including all 11 new storage regressions and the
real Actor case. Standard stable-source preflight passed fmt, locked workspace check/Clippy,
project/claim validators and CI-classifier tests; workspace tests again failed solely at the
unchanged Unix-socket fixture above (64 other runtime unit tests passed). It is still a failed
full preflight, with no skipped test or local waiver. Remote exact-head full workspace CI is the
required environment-capable gate. Runtime SDK external fixture validation is in progress.

Runtime SDK fixture completed: `python3 scripts/validate_runtime_sdk_fixture.py` passed both
independently locked consumer modes (default and coding), with one direct Talos dependency.
Root and fixture lockfiles/manifests remain unchanged. Final owner/derived diffs were staged and
reviewed; project and collaboration validators passed with zero warnings after Review synchronization.
Candidate publication is for exact-head CI/review, not a local full-preflight success claim.

Pre-merge Windows fixture correction: the two ParentDir regressions incorrectly required the
intermediate `missing` directory to exist on every platform. Pinned Rust 1.99 Windows path/FS
code can normalize that cancelled component before filesystem access; this Unix side effect is
not a Windows contract. Limit only those two new directory-side-effect assertions to Unix;
all-platform recovered receipt/full submission and canonical database assertions remain. No fixture
is ignored, no production byte/deadline changes and no original test assertions are removed.
References: rust-lang/rust 1.99.0 `library/std/src/sys/path/windows.rs`,
`library/std/src/sys/fs/windows.rs` and `library/std/src/path.rs`. Fresh exact-head review/CI is
required; prior #342e candidate evidence remains historical and cannot approve the corrected head.

Actual CI3160 Windows failure retained: job `114917790858` failed those two new directory
assertions (200 session unit tests passed, 2 failed, 0 ignored). Real fresh Actor passed; recovered
receipt/full submission checks in the existing-ack case passed before its directory assertion,
while the empty case's later canonical-DB assertion was not reached. Corrected local session233
passes. Old exact342e review is historical; new exact-head full CI/review is still mandatory.

The batched platform correction passed all 733 affected tests/doctests locally. Additional
focused `clippy --all-targets` caught the test permission literal's decimal notation; use the
identical octal `0o000` value. Production source remains byte-identical to the reviewed candidate.
CI3160 completed with five successful jobs and the retained Windows fixture failure; no rerun
of that unchanged head replaces it. Publish one corrected stable head for fresh full CI/review.

Corrected stable-stage checks: affected733 tests pass; format and focused session all-targets Clippy
pass. Exact permission fixture passed one case and printed `filesystem_error=PermissionDenied`
locally; this supersedes the earlier unsupported root-identity inference. Both governance validators
passed. No production/dependency byte changed in this correction; full new-head CI remains required.


## Implementation Merge And Startup Acceptance — 2026-10-16

Current status is Complete / Closed for RUNTIME-010 only. Published objective/acceptance/exclusions
and all earlier failures remain intact; earlier Review/pending paragraphs are historical. This
closeout names an existing implementation commit, not itself, and follows effective claim#710.

#711 exact98fa04cbdfdb7b30ff4a32d06036427872a41703/tree9d3c18e5d7202156fdb47f81f66778705d1335d9
merged as9cad344b8f7adef855ca6b858e6ccc7723103767. CI3161/run38283574722 completedSUCCESS,
attempt1/all6jobsSUCCESS; independent exact-head review6129237320 and merge CAS6129422889.
Root freshly reread completed Windows114926496803 and macOS114926496894 logs: actual fresh Actor
admission and conservative ParentDir recovery pass; macOS original Unix-socket/full release
preflight, both independent external SDK executable modes and actualPermissionDenied pass.
The retained failed local preflight and failedCI3160 Windows fixture candidate are not passes.

### Requirement-To-Evidence Mapping

Source Story's first four acceptance rows are covered by the11storage regressions and actual
fresh_actor_admits_submit_without_recovery_sidecar_creation in the reviewed implementation.
missing_sidecar_recovery_is_read_only_and_admission_stays_durable checks structural absence and
later persisted admission; existing_sidecar_recovery_preserves_running_pending_and_schema_errors
covers actual Running/Pending and schema behavior; absent_read_then_cross_instance_ack_is_recoverable
checks acknowledged concurrent custody. Invalid parents/permissions, links/loops and ParentDir
regressions preserve conservative behavior. Original WAL/FULL/schema/query/receipt authority and
public API remain unchanged. README2/runtime SDK contract changes shipped with #711.

Current main7b33bc9f2a7c446781b2311e7b171f86cf35b4a4 contains the separately closed I304 repair.
Source comparison98fa04cb→main shows pending_submission.rs production differences only in
pause_unstarted and a helper comment (read→operation snapshot); recover_states/sidecar_exists
behavior is unchanged. No startup logic or published acceptance is broadened by this closeout.

### Post-Merge Original-Budget Comparison

Retained CI3153 successful target parallel5/PID9060/thread72 had actor-entry→reconcile-return
4.5720454s, followed by recover63.7013ms/restore574.9591ms; these include scheduling/observation.
Post-#711 CI3163 at48ed767e first-recovery60targets median0.2307ms/max69.1622ms, Actor reconcile
median0.2905ms/max69.2491ms. Its actual120/2220 result remains2205pass15fail; baseline Auto15s
no-result and instrumented3Auto+10other host failures are retained. CI3182 remains2219pass1fail
(evaluator5s composite branch/phase unknown). Earlier checkpoint6129850034 kept this slice Review
pending evidence-bearing closeout and separately investigated shutdown cost; I304 now closes
only that distinct pause optimization, never these historical failures.

CI3206/run38832452061 exactff84ec7064edb54ace27ba7c2700f028e5dfaef6 includes actualmainc869a9af,
with only3temporary diagnostic files and unchanged I301 startup code. All8jobsSUCCESS attempt1;
fixed120processes/2220pass0fail0ignored, independently validated in#705 comments6209342583 and
6209379440. Two65-file ZIPs, expireOct30:
- baseline12017369938 SHA256
  1faed5ecf6261634cb26b97f82a1360d61d80dedb86260855470f7a7fd412f8e;
- instrumented12018356958 SHA256
  2fcf78709cd7614c4f5f3e004a4c316506121ec122ab54dbfa85a599d52b36cb.
Both manifests match exactff84ec7/image20261007.264.1/4CPU; normalized production hashes and
all12WindowsCRLF original/restored raw hashes match actual production sources. No original
Auto15s/cleanup10s/tool120s/evaluator5s assertion or deadline changes, warmup or retry-to-green.

Fresh root recount of the60unique matched original-Auto PID/thread groups:
- First recovery only60samples: median0.19485ms/max136.6005ms.
- All three recovery calls per target180samples: median0.137ms/max140.7649ms.
- Logged actor_reconcile_elapsed_ns60samples: median0.24525ms/max145.0001ms.
  Outer actor-entry→reconcile-return markers instead measure median0.29335ms/max145.0726ms;
  these include additional observation and are a separate metric.
The all180population must not be described as60first-startup timings. Timers do not themselves
label absence, establish exact I/O cost or eliminate scheduling. Structural no-storage/admission
regressions plus reviewed production branches establish the bounded optimization. Different
production baselines/runner images/VM/probe overhead prevent a controlled causal speedup claim.

### Scope, Retrospective And Residuals

SOP3a: this existing slice is SDK/library-only pending-store/Actor startup with ParentNone and no
new CLI capability/binary-entry acceptance. Real external SDK executables plus Actor/Runtime
execution establish this layer; Auto test executables are budget comparisons, not actual talos
CLI walkthroughs. Broader CLI/PowerShell integration and timeout acceptance remains open in#705.
No behavior-facing parent is marked Done by this storage closeout.

Structural absence/reopen/custody tests prevented a timing-only conclusion. The ParentDir
counterexample showed why naive NotFound is insufficient; platform-specific side effects required
a corrected fixture with all-platform custody assertions preserved. Separate startup/pause/child
spans allowed bounded completion while retaining the unexplained failures. No new ADR/dependency/
release or runtime implementation is introduced here. I304 staysComplete independently; other
Search/Git/PSS/Skill owners and terminal#590 are unchanged.

#711 CI3160 remains retained here. #705 remainsOPEN/UNMERGED with oldCI3163/3182 failures,
native-child waiting (current
successful assigned→waitreturn4.9122951s sample incomment6209483437), old10s/15s attribution,
evaluator5s phase uncertainty, existing-journal costs and prior Plugin uncertainty. Neither a
successful sample nor this closeout guarantees universal latency or closes the full investigation.
Applicable exact-closeout-head CI/review, both validators and CAS remain mandatory before merge.


### Closeout Evidence Review And Validation

Fresh root acceptance reconciliation: #711 comment6211705932. Independent Agent evidence/local
six-file closeout review6211716953 APPROVE confirms the exact implementation/native gates, first60/all180
populations, logged Actor field and library-only acceptance. Root freshly rehashed bothZIPs,
reparsed raw60target logs and both1110test summaries, and matched all12normalized/CRLFsource
restoration receipts to current production. Earlier raw failures remain retained, not new passes.

Local pinned Rust/Cargo1.99.0 project governance0warnings (SQLite17fixtures/delivery12),
Collaboration Claims0warnings, classifier14/14 and staged scope/whitespace review PASS.
Only six governance/docs files change; manifest audit date is already2026-10-16 and only its
scope comment includes I301 alongside I304. No profile/claim/source/dependency/lock/CI change.
This documentation-only closeout does not claim a fresh full local Rust preflight; runtime
acceptance comes from the existing exact implementation/native CI and bounded comparison.
Exact published-head review, applicable CI and merge-time CAS remain gates before main update.
