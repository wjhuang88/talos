# AUTO-HISTORY-001: Session-local rolling approval judgment history

Document status: Review. Claim #916 is effective; local implementation and validation passed.
Exact-head remote/native gates and delivery remain pending; no Complete claim.
Deliverable: subsequent approval-model requests include bounded, sanitized previous model judgments
and eventual fallback outcomes from the same live session, across every supported review family.
Distinguish observed human choices, legacy-host/unknown attribution and request cancellation.
User-facing documentation: README.md and README.zh-CN.md auto-approval lifecycle and limits.
Source requirement: [durable intake](../proposals/session-auto-approval-rolling-history.md).

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | AUTO-HISTORY-001: bounded live-session model/human judgment history, provider request integration for all supported auto-review families, correlated fallback outcomes, isolation/redaction tests and documentation; excludes permission eligibility expansion, reusable grants, disk persistence, background continuation, archive recovery and unrelated TUI changes |
| Claimed At | 2026-11-11 |
| Source Issue | None; explicit session request recorded in durable intake |
| Governance Claim PR | #916 |
| Authorization Mode | Independent review |
| Authorization Evidence | User requested completion of session-recovery handoff including this original requirement; independent security/design review required before activation and protected implementation delivery |
| Implementation PR | #964 (draft) |
| Last Updated | 2026-11-17 |
| Handoff / Release Condition | Effective claim on main, bounded-history implementation across all supported review families, current-head security review and CI, documented runtime acceptance and existing implementation commit evidence |

## Constraints and selected design

Hard: policy Deny and explicit human checkpoints remain authoritative; history is untrusted
reference data, never a grant, instruction or substituted current-request evidence. Preserve all
current digest/revision/expiry checks, permission admission and sandbox boundaries. Do not store
hidden chain-of-thought, raw provider errors or secrets. No cross-session copying or persistence.

Soft design for review: eight entries, at most 1024 serialized UTF-8 bytes per entry and 8192
serialized bytes for the whole envelope. Enforce limits after JSON escaping. Evict oldest first.
Reset on resolver replacement, session/fork change or explicit auto reset. Generation-bound
monotonic correlation tokens ensure concurrent/delayed human results cannot update another entry
or resurrect an evicted entry. Snapshot prior history before inserting the current request.

Record sanitized tool/action context, public model effect summary/decision points and result,
plus Pending/Allowed/Denied/Cancelled/Unavailable with independent outcome attribution.
ObservedHuman, LegacyHost/Unknown and request cancellation must remain distinguishable;
a host choice alone does not prove a human decision. Pending starts only when actual fallback
begins, including early evidence-unavailable branches. A dropped enclosing future settles only
its own token as request cancellation; explicit human cancellation requires a producer signal.
Unexplained channel closure means Unavailable. Missing history or lock failure keeps existing
safe fallback behavior. History receives no additional tools, retries or model authority.

Preserve ApprovalChoice and existing resolver signatures. Legacy message-only errors map to
Unavailable without parsing error text. Any additive SDK observation must preserve third-party
implementations and retain unknown attribution for legacy hosts. Desktop infrastructure failures
remain fail-closed without becoming human Denied. The concrete compatible observation API remains
a review candidate; API/ADR migration review precedes its implementation.

Read ADR-064 and its accepted successor decisions for each current review family, PERM-006,
PERM-007-H, auto_resolver and its action/network/invocation/file-mutation/prepared-command modules,
permission_pipeline, bounded_model, CLI/runtime composition and existing approval presentation.
Avoid public API breakage; any unavoidable change requires an ADR and migration plan first.

## Acceptance and validation

1. Captured actual provider requests cover all six construction boundaries listed in the intake:
   shell/context/script, shared ordinary/action builder (both entrypoints tested), file mutation,
   built-in invocation, network and prepared command. Prior history retains existing evidence,
   locale and payload bounds; early evidence-unavailable fallback is covered.
2. Real Terminal, TUI, Runtime SDK and Desktop fallback channels update the exact entry with
   independently attributed outcomes. Test actual user choices, explicit producer cancellation,
   unexplained channel closure, legacy SDK handlers and Desktop infrastructure failures.
   Never infer ObservedHuman from LegacyHost/Unknown or request cancellation. No pending or failed
   action is represented as allowed; recorded outcomes confer no reusable authorization.
3. Concurrent calls, late outcomes, eviction, reset(), control sync_reset(), replacement, rebind
   and session-ID reuse cannot cross-update or leak history. The current request is absent from
   its own prior-history snapshot; dropped old requests cannot update a new generation.
4. Oversized Unicode/JSON-escaped inputs remain within count/per-entry/aggregate byte limits;
   sanitization excludes credentials/control sequences while retaining useful bounded context.
5. Hostile previous approvals do not bypass explicit Ask/Deny, eligibility, current-request
   digest/expiry/revision checks or execution admission. Empty/unavailable history is fail-safe.
6. Run pinned locked focused real-resolver/provider/fallback regressions, standard
   ./scripts/release_preflight.sh, applicable native Windows gate and exact-head independent
   security review. Document live-session lifecycle in both READMEs; do not close on ring-buffer
   unit tests alone. No default background continuation or release authorization is implied.

## Selection and ownership inventory

Preparation base: 6c81390a (current origin/main when isolated governance worktree was created).
I249 Planned retained; I277/I302/I318/I320 Review retained; I319/I322 Active retained.
No Blocked owner selected. Historical Complete headers with embedded older Review notes
(I146/I147/I150/I162) are not reopened. Existing open implementation PRs #861/#907/#912
own search/storage work and do not transfer authority here. Recovery emergency, background
continuation and approval-presentation owners retain their scopes and dirty changes.
This is a separate requirement; it does not replace any published iteration baseline.

## Local preparation and residuals

Governance-only isolated worktree; no implementation branch/commit/PR created for this scope.
PR-number backfill is recorded. Both governance validators, independent design/security review,
merge-time main/owner CAS and main merge remain before the claim becomes effective.
Implementation, tests, native acceptance and delivery commits remain unimplemented.
Residual destination: this owner. No Completion Commit exists; no Complete claim is made.

## 2026-11-15 review correction — preparation only

Independent read-only review of PR916 at d0c7b2a identified missing provider/adapter coverage
and ambiguous human attribution. The existing intake source map and adapter correction are
synchronized here, with matching acceptance above. This refines the original Work Slice;
it adds no permission authority or new task. No implementation or claim activation is claimed.
Local proposal/owner diff passed independent design review and both repository governance
validators (zero warnings), plus git diff --check. Logs: recovery-approval-claim-sync-governance.log
and recovery-approval-claim-sync-claims.log under the primary target directory. Remote update,
remote exact-head review/checks, merge-time CAS and authorized target-branch merge remain required.
This local review does not replace future implementation security review. Implementation and
delivery remain open.


## 2026-11-16 effective claim and compatible observation implementation — Partial

Claim #916 merged e1cbee1e after maintainer authorization. Isolated implementation
branch fix/approval-history-20261116 starts at main5169d5f6; the primary mixed worktree
and sessions remain unchanged. Independent pre-implementation API/security review
selected an additive observation method preserving old resolver/SDK signatures and
legacy Unknown attribution; actual producer boundaries must prove human outcomes.
Design notes are retained in primary target/approval-observation-design-review-20261116.md.

Initial source now adds opaque ApprovalObservation and non-exhaustive attribution,
plus ApprovalResolver::resolve_observed default delegation to the exact old method
(with/without explanation). Unavailable and user-cancelled constructors always Deny.
Terminal private oneshots carry typed observations; actual displayed-input selection
marks human, UserInterrupt marks explicit cancellation, terminal/error/render failures
mark unavailable. Legacy public calls project the original ApprovalChoice unchanged.
Actual-channel/default-delegation/displayed-input regressions added; not yet executed.
This is in-memory advisory API data, with no serialized config/protocol change or
migration required for existing trait implementers. External consumer compilation and
exact-head API/security review remain required before delivery.

No rolling-history storage, six-family provider injection, correlation/drop guard,
SDK/Desktop/TUI observation or full lifecycle/redaction integration is implemented yet.
No authority is widened. Source format/whitespace checks only; heavy tests wait for
current UI/edit sequential gates. Remaining work stays owned by this full requirement;
these APIs are not a separate delivered feature or a completed history implementation.

### SDK observation adapter preparation

Added public observation re-exports and additive ApprovalHandler observed method,
delegating Some explanation to the old explanation method and None to old scoped
approval. RuntimeApprovalResolver forwards exact scope/presentation/explanation to
that override. Existing legacy methods and network/native-Pull refusal remain.
Extended actual SDK resolver/default dispatch tests for human versus legacy attribution
and both protected scope refusals. fmt/diff checked only; compilation and runtime
validation await the sequential edit gate. This is unvalidated partial implementation,
not rolling-history delivery; all provider/history/Desktop/TUI residuals remain open.

### Desktop attribution and real Terminal teardown preparation

Desktop scoped observation now projects legacy choice APIs from one typed request path.
Successful response from its ID-bound UI command is observed human; request-size limit,
busy pending slot, send/receive closure and lock/ID failures are Unavailable/Deny.
Existing actual oneshot/stale-ID/cancel tests now assert attribution; added oversized,
busy and lost-sender regression. Terminal test drives actual handle_event for interrupt,
completion and error, asserting response and lifecycle cleanup. These are source additions
only, not executed evidence. Independent read-only adapter review found no blocking
safety/API issue; full window-button integration, external SDK fixture and native gates
remain. Auto resolver does not yet consume observations: rolling history is not delivered.

External SDK fixture now also calls observed approval on a handler implementing only
old methods, using an actual runtime-produced GrantPreview. It checks None/Some
explanation dispatch and LegacyHostUnknown denial through talos-runtime exports only.
Both fixture feature configurations added to the prepared sequential validation runner;
not yet executed. Unrelated whole-fixture formatter change to main.rs was restored
from the clean branch HEAD; only intended safety.rs fixture additions remain.

### Observation adapter tests passed; history still unimplemented

Sequential pinned locked observation runner13398 exited0: Agent674,Runtime105,
Desktop117 tests and both external SDK fixture configurations passed. Nested test
subprocess summaries are not additional unique tests.2661manifest files unchanged,
1074.53s,memory2GiB/swap0. Logs/results in primary target/approval-observation-focused-20261116*.
Initial --lib selection omitted CLI binary tests; supplemental exec18463 ran
cargo test --locked -p talos-cli --bin talos and exited0:425passed/0failed/ignored,
125.29s,CPU110.126s,memory2GiB/swap0. Full test output observed in exec result;
systemd journal export retains only service lifecycle/resource lines, not test stdout.
This closes the adapter-focused execution gap, not whole-story preflight/native gates.
Rolling history, all six model construction boundaries and typed TUI transport remain
unimplemented. No complete delivery claim.

### Rolling container implementation started — not integrated

Private rolling_history module now bounds eight entries/1024serialized bytes each/
8192bytes complete envelope after JSON escaping. Captures prior snapshot before
insertion; generation+monotonic sequence tokens reject reset/evicted updates; terminal
fallback outcomes reject duplicate overwrites. Context/public summary uses existing
secret/control/sensitive-target rejection and UTF-8-safe trimming. No raw error/grant
fields. Focused unit gate exec71692 is live; source frozen until result. This is
container-only code, not provider injection or lifecycle integration; production
resolver, dropped-future guard and TUI remain open. Current unused-code warnings
are expected during this unintegrated stage, not waived for final Clippy.

Guard r3 exec35997 completed0:7passed/0failed/ignored,17.56s,CPU17.47s,
memory1.5GiB/swap0. Includes duplicate-guard ownership and compact JSON redaction
regressions after the failed r2 compile correction. Container/guard tests only;
production lifecycle/provider/TUI integration remains open.

## 2026-11-16 live-scope validation follow-up — Partial

Latest owner/live epoch/session/generation patch compiled under pinned Rust1.99.0,
locked Agent auto_resolver gate:81passed/0failed/ignored,18.541s,CPU18.041s,
1.5G/swap0 (exec77243; systemd talos-approval-history-live-scope-20261116).
Added direct task-local owner-isolation and live epoch-change late model/human
writeback regressions; subsequent gate83passed/0failed/ignored,17.882s,
CPU17.663s,1.5G/swap0 (exec70567; unit live-scope-r2). Both results observed
in exec output; no full stdout file claimed. fmt and diff --check passed.
These are scope/guard regressions, not complete nested production resolver or
provider acceptance. prior remains unused; six provider boundaries, useful safe
context/public summary, typed TUI transport and full/native/security gates remain.
Independent reviewer requested to examine check/write races and lease rebind before
provider injection. No completion, merge, install or background activation claimed.
#963 CI43888836462 remains in_progress: five jobs successful, main and Windows
workspace still running; no failed job observed at this check. Primary dirty tree,
session files, backups and migration content preserved.

Follow-up: moved history binding/generation validation and all scoped model/fallback
mutations into one history critical section (with_current); start/settle/drop now
use it. r3 exec38666 passed83/0failed/0ignored,19.904s,CPU17.720s,1.5G/swap0;
fmt/diff --check passed. Independent review correctly limits this claim: concurrent
control Atomic epoch change can still occur after the check, so strict reset-return
physical writeback exclusion requires a shared fence or an explicitly validated
read-isolation design. This remains open, not fixed by the lock consolidation.
Permission-backed rebind generates a new opaque permission Session ID; static leases
need resolver replacement/explicit reset. Avoid reverse permission/history lock order.
Provider prior is still unused, so no history payload exposure is introduced yet.
Current #963 CI remains five successful jobs/two running, no observed failures.

## 2026-11-16 reset fence and provider history integration — Partial

Added shared AutoPermissionControl history_fence to epoch reset, explicit resolver
reset, scope capture and scoped reads/writes; no lock spans await. Independent
source review found no reversed lock acquisition or reset-write race in that boundary.
Permission-session checks remain a separate linearization point; static leases require
resolver replacement/reset for a new incarnation. Captured provider requests belong
to their capture epoch; reset does not revoke an already constructed/sent request.

All six built-in provider construction boundaries now inject bounded prior history:
shell, shared action/ordinary, file mutation, invocation, network, prepared command.
System text explicitly treats history as untrusted reference, never authorization.
Current evidence/digests/locale are retained. For bounded16KiB payloads, evict oldest
history entries first and omit the field entirely if the empty envelope will not fit.
Shell/prepared retain their original request size behavior; added history <=8192bytes.
Public model summary/decision points are retained only after matching schema/digest
and live scope checks, sanitized and re-bounded after update. Closed review family
labels and sanitized tool name provide context without serializing raw arguments.
Shared assessor task-local scoping is reviewed as valid for current inline provider
construction; custom spawned tasks do not inherit scope and receive no history.

Focused gates: exec71043 83passed reset fence; exec86254 84passed actual shell
resolver/provider/fallback and reset/rebind; exec77286 85passed all constructors
(action and ordinary separately) with actual provider payload captures;
exec89091/68033 85passed nested two-resolver shared-assessor restoration, exact16KiB
fallback-to-original and public details/family changes. Latest before memory fix:
full Agent library exec58017 passed687/0failed/0ignored,90.677s,CPU49.839s,
1.5G/swap0. These are not workspace/preflight/native/TUI completion.

Reviewer found to_value-before-bounded-Writer allocated the whole oversized input.
Changed serializer to enforce original bounded serialization before parsing Value;
added streaming million-element regression checking early termination. Latest gate
exec35428/unit talos-approval-history-provider-memory-20261116 running; source frozen
until result. Full Agent687 gate predates this final memory fix. No full stdout file
claimed for these exec pipe observations. TUI typed transport, all-family end-to-end
lifecycle/hostile-prior coverage, final security/native/preflight, README and delivery
remain open. No PR or commit for this implementation yet. Background stays off.

#963 exact01973af9 CI43888836462 now seven jobs successful including native Windows
workspace; main Format/Check/Clippy/Test still running, no failures observed.

Memory-bound correction gate exec35428 completed0:87passed/0failed/0ignored,
18.475s,CPU18.169s,1.5G/swap0. Includes streaming early-stop and invalidated prior
omission regressions. Source no longer has the to_value-before-bound allocation
regression. Main CI43888836462 currently at Optional language Plugin acceptance;
seven other jobs succeeded. Task remains Partial; no local heavy job left running.

Independent reviewer approved this uncommitted provider/reset stage after checking the
memory fix and streaming regression; not final exact-head/full-story authorization.
English and Chinese READMEs now describe candidate memory limits, reset/capture
boundary, attribution and pending TUI/platform acceptance without claiming release.
Governance validator passed0warnings; Collaboration Claim validator passed with the
existing unrelated SEARCH-001-D3 warning. fmt/diff whitespace checks passed. Remaining
next implementation is typed TUI request/response + lifecycle binding, then broader
production all-family adverse-history and full/native/security acceptance. No product
merge/install/release or default-on background action occurred.

## 2026-11-16 typed TUI lifecycle implementation and verification

Uncommitted approval-history candidate now has additive typed TUI requests/responses,
unique generation/sequence binding and one oneshot per request, with no legacy placeholder.
Actual key choices alone produce Human; Esc/Ctrl-C produce explicit cancellation;
closure, expiry, publication invalidation and teardown produce Unavailable. Legacy APIs
remain available with unknown attribution. CLI creates/consumes requests under the
permission snapshot fence; session publication invalidates before sending any route,
including failed publication. New epochs cannot reactivate old tickets.

TUI handles active/queued invalidation, receiver replacement, session legacy cleanup,
output EOF, typed EOF, render errors and exit. Independent review found that cleanup
after await missed caller cancellation while retaining Tui; replaced with a safe Drop
guard. Actual modal-frame regression drops the run future and verifies both waiters
release before late input; EOF regression observes display, closure and retired frame.
CLI invalidation test retains responder until consumer completion, with outer timeout
shorter than request timeout, so responder drop or timeout cannot mask a broken wake.

Verification receipts (pipe output, not claimed as archived stdout): initial TUI
exec40591 failed101 due to test helper shadowing; corrected helper to make_request.
exec12529 full TUI703 passed. exec91976 full CLI428 passed/0failed/0ignored,
38.179s, CPU34.784s, peak2G/swap0. exec43879 full TUI704 passed/0failed/0ignored,
22.504s, CPU10.731s, peak882M/swap0, including cancellation Drop regression.
Final exec59898 TUI10 + CLI3 focused lifecycle tests passed/0failed/0ignored,
60.295s, CPU57.578s, peak2G/swap0, including new EOF and retained-responder test.
Independent source re-review found cancellation/EOF gap closed, no new blocker.
Removed a production dead-code warning by making test-only FallbackGuard::start cfg(test).
READMEs describe implemented candidate transport and retain incomplete integrated/platform
acceptance. No implementation commit/PR yet; full preflight, all-family adverse-history
integration, native and final exact-head review remain open. Status remains Partial.

Related #963 exact01973af90b70fa58bfb7699eaa7216a279619612 CI43888836462 completed
success, including all eight jobs. Independent technical review APPROVE confirms this
head changes only metadata from reviewed e4d8; manual terminal acceptance remains open.
No product merge/install/release, background enablement or session-data change occurred.

## 2026-11-17 full approval candidate preflight receipt

First standard preflight exec28500/unit talos-approval-preflight-20261116 exited101:
workspace check passed, Clippy rejected redundant closure at rolling_history.rs:402.
Replaced closure with FallbackGuard::start_scoped; no suppression or scope change.
Failure log retained: target/approval-preflight-20261116.log (primary workspace).

Rerun exec95975/unit talos-approval-preflight-r2-20261116 exited0, release preflight
passed. Full log: target/approval-preflight-r2-20261116.log (primary workspace).
148 test-result lines sum to5382passed/0failed/0ignored; both external Runtime SDK
fixture executions passed. Format, locked workspace check, strict Clippy and governance
passed; existing unrelated SEARCH-001-D3 Unclaimed metadata warning remains unchanged.
Elapsed13min15.001s, CPU9min6.624s, peak2G/swap0, sequential jobs/tests1,
pinned Rust1.99.0, incremental/debug0, migrated TMPDIR. No heavy local job remains.
This validates the uncommitted approval candidate, not installed delivery or all-family
adverse-history/native acceptance. Next: actual resolver hostile-prior/admission tests
for every review family, applicable Windows/native checks, final independent review,
stable staged-diff/commit/PR. Recovery real-provider/installed-host/derived-checkpoint
and UI/edit native acceptance remain open. Background continuation remains default-off.

## 2026-11-17 hostile-prior admission and correlated-result regressions

Added actual ProviderAutoPermissionAssessor/LanguageModel stream fixtures: initial shell
AllowOnce includes a hostile public summary instructing future approvals. Subsequent
shell resolver checks retain explicit-human flag and deterministic mutation/digest gates.
Separate PermissionPipeline cases cover ordinary exec, scheduler action, file creation,
file mutation, network and prepared Git command. Each uses current-digest allow control,
then independently seeded replay/explicit Ask/Deny cases with identical operation evidence.
Allow succeeds without fallback; replay and Ask deny with one fallback; policy Deny has
zero fallback. Captured provider payloads prove correct family schema, retained prior
AllowOnce, current exclusion and fresh digests. A subsequent provider probe proves exact
prior denied/legacy_host_unknown result, no invented human fallback for auto allow, and
no resolver-history entry for policy Deny. This is permission admission, not actual
execution; shell false-flag check alone is not described as policy-pipeline evidence.

Test construction failures retained as evidence: exec72438 failed compile because the
third new_nature parameter is ResourceKind, not tool name; switched to tool rule.
exec64655 exposed appended rules did not override scheduler defaults; use empty engine
with explicit rule to model effective configuration. exec47514/53516 found incomplete
Git facets bypassed review; added actual Read/root and Execute/git alongside capability,
matching existing command tests. No production admission conditions were relaxed.
exec45116 passed initial four-family subset; exec86322 passed six-family positive and
negative controls. Final exec94149/unit talos-approval-hostile-prior-r8-20261117:
89 auto_resolver tests passed/0failed/0ignored,18.555s,CPU18.344s,peak1.5G/swap0.
Independent reviewer approved evidence structure and absence of false-green controls;
review was source-only. Prior full5382 preflight predates these two new tests.

Windows test compilation started exec51342/unit talos-approval-windows-check-20261117:
cargo +1.99.0 xwin check --locked -p talos-agent -p talos-tui -p talos-cli
-p talos-runtime -p talos-desktop --tests --target x86_64-pc-windows-msvc.
Live log target/approval-windows-check-20261117.log in primary. Desktop default feature
does not include desktop-ui; explicit Desktop feature cross-check remains separate.
Do not restart this live job or treat cross-check as native Windows runtime evidence.

## 2026-11-17 SDK/Desktop direct observation chains and Windows limitation

Independent cumulative source review found no new blocking API/security defect, but
correctly identified owner acceptance2 was only segment-tested. Added Desktop actual
RuntimeHost two-tool conversation: real RuntimeCommand::ApprovalResponse(Deny) for
first identical tool request must appear as denied/observed_human in second provider
payload, with fresh request digest and current exclusion. Existing disabled/auto-allow
controls retained. exec84082 desktop-ui auto_tests3passed/0failed/0ignored,
14.120s,CPU13.235s,peak2G/swap0.

Added SDK bridge integration module approval_history_tests.rs: real RuntimeApprovalResolver
and public ApprovalHandler callback through oneshot, AutoPermissionResolver, permission
pipeline and actual provider capture for HumanDeny/channel-close/explicit-cancel/legacy.
Second provider payload distinguishes denied/observed_human, unavailable/request_lifecycle,
explicit_user_cancellation/observed_human and denied/legacy_host_unknown. No grants.
Initial exec12539 compile failed due test-only wrong type ApprovalPreview; corrected to
actual GrantPreview. exec61756 focused SDK chain passed1/0failed/0ignored,
4.832s,CPU4.709s,peak746.2M/swap0. Narrow-default feature build shows existing Tools
MemoryEvidence dead-code warnings; no unrelated source change. These tests do not
claim RuntimeBuilder full application coverage for SDK or Desktop cancellation chain.
Terminal/TUI production PTY chain harness prepared at target/approval-history-pty-20261117.py.

exec51342 ordinary Windows MSVC --tests check completed0,4min18.110s,
CPU3min14.527s,peak1.8G/swap0. Log target/approval-windows-check-20261117.log.
Existing Windows test-only History::prepare dead-code warning observed. This predates
new SDK/Desktop tests and did not enable Desktop UI feature.
Follow-up exec52494/unit talos-approval-windows-all-hosts-20261117 enabled desktop-ui,
but windows0.61.3 compilation incurred >14658memory.high events and host memory full
pressure~11%. Intentionally stopped rather than raising caps; terminal receipt code=killed,
status15/TERM,3min24.057s,peak2.1G/swap0. systemd says success/exec0 on requested stop,
which IS NOT a compile pass: no Cargo finished marker. Log retained at
 target/approval-windows-all-hosts-20261117.log. Explicit Desktop/native Windows gate
remains open for CI; do not restart this resource-heavy cross-check unchanged.

Current heavy job: exec22764/unit talos-approval-cli-build-20261117, pinned locked
production CLI debug build for isolated PTY acceptance. Poll before starting another.

## 2026-11-17 production CLI actual-key history acceptance

Production CLI build exec22764 completed0,18.140s,CPU16.231s,peak1.5G/swap0.
Binary SHA256 33c97870c9dd81502cc7dfa192517b2a64b660001ecefc267c0b50446e172dc5.
Isolated HOME/workspace with bwrap, local synthetic HTTP provider, actual production
REPL/TUI input loops and PTY ANSI screen: first real key3 Deny must reach second
provider payload as denied/observed_human, one previous entry, current excluded,
fresh digest. Both modes passed exec98378/unit talos-approval-cli-pty-r3-20261117,
4.261s,CPU1.416s,peak244.4M/swap0. Artifacts:
 target/approval-history-pty-20261117-r3/result.json
 target/approval-history-pty-20261117-r3/{repl,tui}-reviews.json and *.pty.log.
Harness target/approval-history-pty-20261117.py reuses retained PTY helper and pinned
pyte0.8.2 tooling from target/ui-pty-python-20261116. First launch failed before CLI
because system Python lacked pyte; reuse PYTHONPATH, no installation. r2 REPL passed,
TUI harness failed because expected capital Deny while actual modal renders lowercase
deny; corrected case-insensitive modal matching, r3 passed. Prior artifacts retained.

Four hosts now have direct denial observation→history evidence: production Terminal/TUI
PTY, actual Desktop RuntimeHost command channel, SDK callback through actual runtime
bridge. SDK also covers channel closure/cancellation/legacy end-to-end. This does not
claim every host's cancellation/teardown chain, manually observed native terminal,
real provider quality or native Windows acceptance. New tests need final preflight;
no heavy job remains at this checkpoint. Background remains default-off, source
sessions/primary mixed changes preserved, candidate remains uncommitted/Partial.

## 2026-11-17 final local candidate validation — Review

Final standard release_preflight.sh exec4774/unit talos-approval-final-preflight-20261117
completed with exit0 and explicit preflight passed marker. Pinned Rust1.99.0, locked
workspace checks/Clippy/tests and both external SDK configurations passed: 5385 summed
test passes,0failed/0ignored across148 result lines,2SDK fixture passes. This includes
the new hostile-prior and SDK tests; Desktop feature-specific direct-channel tests
remain evidenced separately by exec84082. Runtime8min53.509s,CPU5min7.182s,peak2G/swap0.
Archived log in primary: target/approval-final-preflight-20261117.log.

Scope remains AUTO-HISTORY-001; no Search/storage implementation or default background
continuation added. Four-host denial evidence and its limits remain as recorded above.
Prior cumulative independent review found no blocker; exact-commit review and native
Windows CI remain required. Desktop-ui local cross-check was intentionally stopped,
not passed. Product merge/install/release and overall handoff completion are not claimed.

Closure ledger: update this owner first, then derived Board and primary handoff; retain
all original execution notes, mixed primary changes and session data. Re-run both local
governance validators and whitespace checks after metadata synchronization. Residuals
remain in this owner and the primary session-recovery handoff.

Stable implementation commit: 3b0455a67cd24dd706eda9df594be125246da1bf, pushed as
draft PR #964. Independent exact-head Agent technical/API/security review APPROVE
verified all32 committed files against the reviewed staged snapshot; no known blocking
source finding. This is not native human acceptance or merge/install authorization.
Remote exact-head CI remains pending. Both local governance validators and the current
skill validator passed; collaboration validator retains one pre-existing unrelated
SEARCH-001-D3 Unclaimed/claim-metadata warning. No unrelated repair was included.
