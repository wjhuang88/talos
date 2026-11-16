# BG-CONTINUE-001: Default Background Completion Continuation

> Document status: Active / Claimed (activation merged in #893; implementation remains Partial)
> Historical checkpoints below describe their original revisions, not the current proposed disposition.
> Deliverable: background success, failure and timeout resume the originating live Session without another user message or opt-in flag.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Talos |
| Work Slice | Actor-owned terminal admission and default continuation, cancellation/consumption deduplication, bounded context, tests and docs |
| Claimed At | 2026-11-09 |
| Source Issue | None |
| Governance Claim PR | #893 |
| Authorization Mode | Independent review |
| Authorization Evidence | Exact-head activation/ADR-101 review: https://github.com/wjhuang88/talos/pull/893#issuecomment-6757010739; merged as bad94be8151253b7cc4806c3b4e9a449aa74eb69 |
| Implementation PR | #953 (default-off foundation; depends on #917) |
| Last Updated | 2026-11-09 |
| Handoff / Release Condition | Accepted successor decision, effective target claim, tested implementation and exact-head review/CI |

## Proposed atomic activation packet (historical; merged in #893)

This packet proposes atomic activation in #893 through the header and claim table
above. Both are ineffective while this PR is open. The proposed claimant
is @wjhuang88, executing agent Talos, using the Independent review authorization path.
claim PR is #893; implementation PR is not started. No implementation branch may
be created until the authorized claim reaches main.

Decision requested: accept ADR-101's default live-Session continuation policy,
including 128-slot backpressure, terminal retirement independent of process reads,
private Context provenance, no automatic provider retry and unchanged permissions.
Accept only the narrow successor to ADR-060's no-auto-continuation restriction;
no restart recovery, new process launch authority or Gemini replay ownership.

Runnable deliverable and verification scope: Actor mock-provider tests for idle,
busy, cancellation, consumption and recovery fencing; more than 128 sequential
success/failure continuations proving slot retirement; CLI background completion
smoke without a second user message. Update README.md, README.zh-CN.md and runtime/
process references with default continuation and additional provider usage.

This candidate supplies the proposed header, claimant fields and ADR-101 acceptance
request together. Historical checkpoints remain evidence of their original revisions.
Independent review must cover activation, not rely on the inactive e1c77bc2 re-review.

Before publishing that candidate: recheck target owner/ADR namespace and all relevant
claims; verify source-branch freshness against the collaboration SOP; preserve the
#838 scope separation and replay compatibility requirements below. At the latest
remote checkpoint target remains 4646c02ad839c8e45fc487c8a99d1bffb8131159,
recorded before the branch refresh. The user subsequently approved fetching main
and merging it into this candidate (not merging the PR). Fetch and ordinary merge
completed without conflicts; the local activation packet was preserved. The fetched
main also includes the clipboard ADR-100. No rebase or force-push was performed.
Run both governance validators and obtain exact-head independent activation review
and CI before any target-branch merge.

## Current activation candidate checkpoint

After the approved main merge, iteration headers were re-inventoried: retain I310
Active and I277/I302/I318 Review; defer I249 Planned; do not reopen I162 Complete
for its historical Review wording. TEMPLATE is excluded; no Blocked header matched.
The source integration and owner dispositions below remain the scoped baseline.
The header and claim table now propose activation, not effective ownership. Earlier
inactive checkpoints are historical. ADR-101 and its index explicitly request atomic
acceptance; independent review of this candidate is still outstanding. No production
implementation or runtime test evidence is supplied by these governance edits.

## Goal and scope

User explicitly requests default continuation and rejects per-job opt-in. Preserve
ADR-060 historical evidence; adopt a successor decision for its product boundary.
Actor owns admission and dispatch, never TUI. Retain permissions, live Session
identity/generation, shutdown, bounded output and queues. No process launcher,
unsafe, new dependency, process restart recovery, spinner or steering UI work.

## Required Reads

- docs/decisions/101-background-completion-continuation.md (Proposed; not effective)

- docs/proposals/background-continuation-actor-design.md
- docs/decisions/060-supervised-background-command-jobs.md
- docs/backlog/active/TOOL-024-background-command-jobs.md
- docs/sop/AGENT-COLLABORATION.md and docs/sop/TESTING.md
- crates/talos-agent/src/background_jobs.rs, scheduler.rs and session.rs
- crates/talos-core/src/submission.rs and session.rs

## Acceptance

1. Idle Session + authorized success/failure/timeout: mock provider receives bounded
   attributable completion context and continues without another user submission.
2. Busy Session: queue completion until current turn commits; never run concurrent
   provider turns or interrupt the active tool batch.
3. Duplicate notifications/uncertain receipt do not duplicate consumption. A terminal
   result already delivered to the model by process does not cause redundant work.
   Host reads alone are not proof of model consumption.
4. Explicit cancellation, stale generation and shutdown suppress pending wakeups,
   including notifications arriving after cancellation and before dispatch.
5. Queue saturation/output expiry have visible bounded outcomes, no silent loss or
   unbounded retries. Provider errors must not create automatic retry loops.
6. Completion is not user authority, privileged instruction or a second Tool result
   for the launch call. Subsequent tools retain permission checks.
7. Synchronous launch failure already returned does not generate a duplicate turn;
   explicitly decide and test asynchronous monitor failure.
8. Existing recovery fencing, runtime admission, manual compaction and pause states
   remain authoritative. Completion must not bypass unresolved Running custody.
9. Actor-only mock-provider tests and a real CLI smoke test demonstrate continuation;
   foreground behavior and cleanup regressions remain green.

## Selection and dependency checkpoint

Preparation base: 4adefbf4 (#887); isolated branch docs/bg-continuation-claim created
with user approval. Header inventory: I310 Active retained; I277/I302/I318 Review
retained; I249 Planned deferred; I162 Complete with historical Review wording not
reopened. No Blocked header matched; comprehensive owner inventory must be repeated
before activation. Remote open PRs #861/#849/#838 concern search and Gemini; inspect
actual file/claim overlap before merge rather than relying on titles alone.
TUI-ELLIPSIS-001 remains Review for deferred visual acceptance.

## Design readiness

ADR-101 selects private Actor input without extending public exhaustive enums.
It defines Context provenance, cancellation epoch, acknowledged terminal consumption,
bounded admission and exactly-once slot retirement. Shared preparation/finalization
must preserve existing custody/recovery fencing and #838 replay semantics if present.
If private integration cannot preserve these gates, stop and amend the decision
before changing public APIs. Design acceptance is requested in this same activation
candidate; it is not asserted as already granted.

## Verification and documentation

Pinned toolchain and locked focused agent/core/CLI tests, standard release preflight,
both governance validators, exact-head CI and independent runtime/security review.
README.md, README.zh-CN.md and applicable runtime/tool references explain default
continuation and additional provider usage. Residual destination: this owner.

## Review remediation

User supplied independent REQUEST CHANGES for head ba402e6d against main 4646c02a.
P2 slot lifecycle: ADR-100 now defines exactly-once retirement for successful
continuation without process, terminal provider failure, cancellation, worker
failure and pre-start/teardown paths, with capacity-exhaustion regression targets.
P2 activation contradiction: this candidate is Refinement/Unclaimed, ADR Proposed.
Merging this version would NOT activate implementation. Complete readiness and
atomic claim/activation in this same open PR before seeking activation approval;
do not claim the prior inventory is refreshed or create a redundant activation PR.
The reviewer attributed remote reconciliation failure to global matrix omission
#891, not this PR's runtime behavior. This is reviewer-provided evidence; retain
that separate residual without editing unrelated owners here. Both findings still
require independent re-review of the corrected head. No runtime tests were run.

## Owner disposition and remaining activation gate

Read the TOOL-024 owner directly at target
4646c02ad839c8e45fc487c8a99d1bffb8131159: it is Complete/Closed and explicitly
excludes automatic model continuation, reserving it for a later product decision.
BG-CONTINUE-001 therefore owns a new behavior rather than reopening TOOL-024 or
claiming its historical completion evidence. Preserve TOOL-024 and ADR-060 as
historical baselines; ADR-101 supplies the proposed narrow successor decision.
No Source Issue is assigned to this new scope merely by inheriting TOOL-024's #59.

Correction: shared source files with #838 establish an integration risk, not by
themselves a conflicting Work Slice or mandatory merge-order dependency. Proposed
scope separation keeps Gemini/private replay with #838 and completion admission,
deduplication and dispatch here. Preserve replay state and identity resets when
sharing preparation; require combined regression coverage when both coexist.
The coordination request is published at
https://github.com/wjhuang88/talos/pull/838#issuecomment-6744155268.
No owner agreement has been observed. Do not assert agreement, but do not treat
silence alone as proof of a scope conflict. Resolve any concrete conflicting change
before implementation. Independent authorization of ADR-101 and the atomic claim
remains required; inactive-remediation approval is not that authorization.

## Issue-index coordination resolution

Feedback at https://github.com/wjhuang88/talos/issues/891#issuecomment-6744246534
confirms #892 merged as 4646c02ad839c8e45fc487c8a99d1bffb8131159 and registered
#891's authoritative intake. This closes that coordination residual on main, not
browser feasibility or any implementation. It does not answer #838's replay
coordination request. Earlier reconciliation failures remain historical failures;
do not describe missing #891 registration as a current main defect or infer a
fresh candidate CI pass. Any remaining failure needs exact-head log attribution.
No duplicate intake or unrelated owner repair is needed in this slice.

## Target comparison and overlapping implementation slice

GitHub compare of candidate b378ce845192d0d9122007df06881363637aa479
against target 4646c02ad839c8e45fc487c8a99d1bffb8131159 reports divergence:
11 target-only commits and 6 candidate-only commits. Target-side changed paths
include permission/tool execution and session-owned worktrees, but no iteration
owner files and no session.rs/request_plan.rs/background_jobs.rs changes. Thus the
candidate iteration-header inventory remains applicable at that target SHA; it
is not a claim that all task owners or permission integration are unchanged.
Preserve target permission changes when implementation begins from the effective
claim merge; do not implement on the old design branch.

Open PR #838 has a real integration overlap, not merely a related title:
its session.rs patch adds generation/session-scoped PrivateReplayStore state and
passes it into prepare_session_turn_at_revision; request_plan.rs extends the
preparation signature and seeds request tokens from that store. BG-CONTINUE-001
owns completion admission/dispatch, not Gemini private replay. Any shared private
preparation extraction must preserve replay state, identity reset and pruning;
do not overwrite or duplicate #838's owner. Recheck merge order and add a combined
preparation regression when both implementations coexist. #861/#849 are search
slices with no agent source paths in the inspected file lists. This establishes
file-level overlap disposition, not authorization from another owner.

At this checkpoint b378ce84 Rust CI was still in progress; scope/installer passed,
remote reconciliation failed, and Windows/Desktop jobs were skipped. No complete
current-head CI pass is asserted. Activation and successor ADR acceptance remain
pending; this record does not turn remediation approval into implementation approval.

## ADR numbering resolution

The preceding checkpoints describe the historical ADR-100 candidate. This revision
renumbers this proposal to ADR-101, including its filename and required-read link.
Read-only main directory inspection found a highest numbered ADR of 099; all five
open PRs were inspected for decision paths: #895 proposes 100, #838 changes 094,
#861/#849 have none, and #893 held this proposal. No competing 101 was found.
This avoids the observed conflict without modifying another owner's PR. It is not
a permanent reservation: repeat target/open-PR CAS before merge. Earlier review
references to ADR-100 refer to the same proposal before this filename-only change.
ADR-101 remains Proposed and this owner remains inactive.

## Source integration checkpoint (candidate tree)

Inspection of `crates/talos-agent/src/session.rs` confirms the dispatch loop gates
pending turns on current turn, fork, manual compaction and pause state (lines
515–519), then calls runtime `commit_start` before `start_submission` (534–546).
`start_submission` currently takes a public `StructuredSubmission` (1251–1256)
and prepares messages from its items (1285–1295). Thus private completion input
cannot simply call this function unchanged without misrepresenting provenance.
Implementation must extract/reuse private preparation and finalization machinery,
not fabricate a user submission or add a parallel unguarded dispatch loop.
These are candidate-source observations, not evidence of implemented behavior or
proof that all target-branch integration seams have been audited.

Remote main decision-directory inspection currently ends at ADR-099; neither
competing ADR-100 is merged. A unique renumbering remains necessary before
activation. No number reservation or other PR modification has been performed.

## ADR namespace conflict discovered during follow-up

Open-PR file inspection found #895 (`fix/clipboard-image-input`) adds
`docs/decisions/100-native-clipboard-image-input.md` and edits the ADR index.
#893 independently proposes ADR-100. Therefore the provisional number is NOT
cleared for activation. Coordinate a unique successor number and update references
before proposing acceptance; recheck target and competing PRs at merge time.
Do not modify #895 or assume either candidate owns the number from PR order alone.

## Follow-up readiness checkpoint

User reports independent remediation re-review passed for e1c77bc2. This is
user-supplied review evidence; GitHub reviews API currently returns no reviews.
Read-only remote verification confirms PR #893 OPEN at
`e1c77bc20a169e99f00b2e9811225f330380820b`, base
`4646c02ad839c8e45fc487c8a99d1bffb8131159`. Exact-head Rust
Format + Check + Clippy + Test, scope classification and Windows installer fixture
passed; Windows workspace and Desktop jobs were skipped. Remote reconciliation
still failed; attribution to #891 remains reviewer-provided, not independently
revalidated here. No claim effectiveness or ADR acceptance follows from this check.

Candidate-tree iteration header inventory was repeated: I310 Active retained;
I277, I302 and I318 Review retained with their existing acceptance gates; I249
Planned deferred. I162 is Complete with a recorded historical Review outcome,
not reopened. TEMPLATE is not an iteration. No Blocked owner header matched.
This is a candidate-tree checkpoint, not yet a complete current-target inventory:
target-tree owner comparison and open-PR file/claim overlap remain activation gates.

ADR-100 already selects private Actor input without extending public exhaustive
enums, requires shared admission/finalization and specifies provenance, cancellation,
consumption and recovery fencing. Verify those integration seams against current
target source before replacing the historical Design readiness entry with a final
activation decision. Keep this candidate inactive until these checks and successor
ADR authorization are complete. Do not reinterpret remediation approval as approval
of a changed activation candidate.

## Execution ledger

Draft PR #893 publishes design commit c57f8f7245dddd9d4645d7c1c7172ccc1283f454.
Remote checkpoint: Issue/owner reconciliation failed; Rust validation still running;
no reviews/comments returned. Workflow marks remote reconciliation continue-on-error.
Cause remains unknown: full run logs were unavailable while the run was active;
annotations only reported exit 1. Track this residual here until scoped attribution
is known; do not infer it is caused by this slice or repair unrelated owners.
Local governance and collaboration validators passed with zero warnings.
This commit is design evidence only, not implementation completion evidence.

Only source characterization and intake/design drafts exist. No effective claim,
production changes, runtime tests, successor ADR approval or completion evidence.
Next: resolve design, prepare atomic claim/activation and successor decision in a
governance PR, obtain required authorization, implement from that merge or later.

## 2026-11-15 selected default-off foundation — Partial

The current user instruction prohibits default background continuation. This stage
implements and tests Actor-owned completion custody, bounded admission, cancellation,
retirement and provider-consumption bookkeeping with production activation disabled.
The published acceptance baseline above is retained; automatic production continuation
and its CLI smoke acceptance are not delivered by this stage. No production opt-in
flag or installation is introduced. Historical activation authorizes implementation,
not overriding the subsequent user instruction.

Selected source: ten Agent files relative to recovery candidate55ba5752. Recovery-only
diagnostic/temp-root test hunks and unrelated UI/approval changes are excluded.
The isolated locked Agent library gate passed729 tests,0failed/ignored; format passed.
Evidence: target/background-selected-agent-tests.log and
target/background-selected-validation-manifest.json in the primary workspace.
All2447 exported source files matched their recorded hashes before this documentation
update. Full preflight, exact-candidate review/CI and delivery remain pending.
Status remains Partial; this stage does not complete the default-continuation story.


Preparation-custody regression: the original candidate passed the full standard
preflight (5526 tests/doctests,0failed/ignored,two external SDK modes), but lacked
coverage for a noncancelled preparation error. A new actual Actor denial regression
failed on the original code at the Pending-custody assertion (exit101). The minimal
fix retains Pending and pauses for noncancelled errors while preserving retirement
for cancelled/revoked dispatch. The new test and three preparation/cancellation/fork
regressions passed (4passed,0failed/ignored). Original and corrected logs are
target/background-preparation-{red,green}.log in the primary workspace.
Independent source review found no blocker. Later user-driven retry is not covered
by this test. Corrected full preflight and delivery remain pending; default-off.


## 2026-11-15 committed foundation validation — Partial

Implementation commit: d50be16743cf16024f70d1fde870fb43fd6b3005 (Draft PR #953),
based on recovery PR #917 at55ba5752. The corrected standard preflight passed5527
tests/doctests,0failed/ignored,and both external Runtime SDK modes. All2447 committed
blobs matched the validated export. The preparation-failure regression demonstrably
failed on the old code, then passed with three related regressions after correction.
Exact-commit independent content review found no blocker and confirmed production
activation remains test-only. This supersedes earlier pending local-gate wording.

Evidence retained in the primary workspace: target/background-corrected-preflight.log,
target/background-corrected-preflight-result.json,
target/background-selected-corrected-manifest.json and
target/background-preparation-{red,green}.log. Remote/native CI, dependency merge,
source merge and installed-host acceptance remain open. Production default
continuation is not delivered or enabled; this owner remains Partial.


## 2026-11-16 user-driven retry regression and native results — Partial

The actual Actor preparation-failure regression now covers both shutdown and
user-driven recovery. After one hook denial it verifies retained Pending and no
automatic retry; a user submission then completes first, followed by the original
completion as Context, with two successful turns and zero retained slots. Both
focused cases passed with locked dependencies (2passed,0failed/ignored), log
`target/background-resume-regression.log` in the primary workspace. No production
source or activation changed. The 5527-test full gate above predates this additional
test and must not be presented as exact-head validation of the new test candidate.

Native run43731413997 atd50be167 finished FAILURE: macOS passed, while Windows
workspace failed the six-thread first-open binding regression with database locked.
Raw logs are `target/background-d50be167-{windows-failed,macos}.log`; terminal
run snapshot is `target/background-d50be167-ci-final.json`. Global Issue ownership
reconciliation also failed for existing937/947 drift. Recovery PR917 diagnostic
commit9b38f8e3 is undergoing native validation; cause and fix remain unproven.


## 2026-11-16 exact retry-candidate full local gate — Partial

Implementation/test candidate `3a7f1a7c11b79939d33d61f88e19fd610fd408d6` passed
the standard locked release preflight:5528 tests/doctests,0failed/ignored,and both
external SDK fixture modes. All2447 committed files matched the validated worktree.
Evidence in primary workspace: target/background-resume-preflight.log and
target/background-resume-preflight-result.json. Service exit0, runtime15min26.074s,
peak2G,swap0. Independent review of the retry test found no blocker and confirmed
shutdown coverage was preserved. This supersedes the local full-gate limitation
above for3a7f1a7c; this evidence paragraph itself is a later documentation-only edit.

Recovery diagnostic run43734019253 is still live. The recurring Windows binding
lock failure is unresolved; no merge, installation or default activation occurred.

## 2026-11-16 recovery dependency integration — Partial

Rebased four existing background patches from55ba5752 onto recovery2beac8f8;
range-diff confirms each patch unchanged. Original8f9dcc29 retained in
backup/background-custody-before-recovery-20261116. New tested source:
f2978d5008a1fba43297ec58ae118dcd8f0e09a0. This incorporates atomic binding-schema
publication and final logical identity-lease unlock without changing background
activation: completions defaults None and enable_completions remains cfg(test).
Independent integration review found no API/lifetime conflict or blocker.

Clean standard pinned1.99.0/locked release_preflight passed5533 unfiltered
tests/doctests,0failed/ignored, two extra filtered subprocess passes and both
external SDK modes. All2447 source hashes matched before/after. Session435
includes both new logical-lock lifetime regressions. Runtime12min47s, peak2G,
swap0 under existing resource limits and requested TMPDIR. Evidence:
target/background-recovery-preflight.log, -result.json, -source-manifest.json.

This supersedes old-source local validation only. Fresh exact-head native checks
are still required; the earlier macOS unavailable failure is not causally closed.
No default activation, merge, installation or session-data migration performed.
Foundation and complete handoff remain Partial; all existing residuals retained.
