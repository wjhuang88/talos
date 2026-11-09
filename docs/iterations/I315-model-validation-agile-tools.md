# Iteration I315: Model Validation And Read-Only Agile Tools

## Completion — 2026-11-05

Completion Commit: `add669403734d4e2469eb479939066f57027c74d` (implementation PR #847).

T1 is Complete. Exact-head CI `41849101811` passed all eight jobs on
`d1b3c8e4fd1cd96de4d3f1b6b3e0285a90903cbd`, including native Windows workspace
tests and all seven explicitly enumerated validation lifecycle regressions.
Local standard preflight 5343 passed workspace check, warnings-denied Clippy,
workspace tests and both independent SDK modes. Independent API/process-security
review is PR comment `6633437001`; merge CAS is `6646259887`. PR #847 merged
2026-11-05 at 00:36:41 UTC. Historical checkpoints below do not override this state.

Delivered: host-bound agile summary and fixed-profile validation in shared product
composition; exact prepared permission consumption; bounded owned Unix/Windows
execution and operation cleanup; CLI migration; strict applicability/schema;
README and SDK/API migration documentation. Tests cover real denied admission,
approved Runtime dispatch with a harmless Cargo executable fixture, cancellation,
output overflow, startup failure, independent invocation cleanup, and native Windows
launch/preparation ownership. The fixture is not a live production Cargo build.

Limits remain explicit: descriptions guide model intent rather than implementing
a deterministic language router; non-Rust language detection does not invent host
adapters; Unix owned groups do not guarantee hostile tree containment; kernel calls
cannot be hard-interrupted; Desktop coverage uses shared composition/native CI,
not manual GUI acceptance. ADR-097 API removal ships only with the next minor,
not a 0.10 patch; this closure authorizes no release. T6 remains unimplemented and
the full handoff remains Active.

## Current validation checkpoint — 2026-11-04

Rebase completed onto `9ba327728` with reviewed CLI/Runtime conflict resolution.
Current implementation sequence is `34df43d51`, `15a2ae0f6`, `489c12d0b`;
earlier SHA references below describe pre-rebase evidence. New configured-search
and selected-workspace regression passed 1/1 (81910). Independent reviewer approved
the increment at head `aabad0ad39e7369121c977dcb6239420a5ea8cb8` against that base,
without running tests. Rebased full preflight 5343 exited 0, including workspace
check/Clippy/tests and both external SDK modes, at
`/private/tmp/i315-rebased-preflight.log`. Latest governance validators 31048/81162
exited 0 with zero warnings. Native CI and merge are still pending.

Latest gate: repaired standard preflight 66373 exited 0, including workspace
check/Clippy/tests and both external SDK modes. Governance validators 84281/31162
also exited 0. Independent staged review approved 36 files (diff SHA-256
`0b7d60e39f01800a8e7aa6685a5b3be439fb6e2e3d55235aeae854ad7f0e4dae`).
Implementation is now local commits `9bf942651`, `47dafbee9`, and `ae6ee2edc`;
the preflight covered their combined source plus these owner updates, not each
intermediate tree independently. Fresh fetch found I314 merged at main
`9ba327728bfc87775ed31740d58636ba73a5c68e`, overlapping CLI/Runtime composition.
Rebase/integration verification is required before stable publication; prior gates
do not certify the rebased tree. Native Windows, exact-head review/CI and merge
remain open. T1 remains Partial and T6 unimplemented.

Latest continuation supersedes the running/uncommitted snapshots below. Standard
preflight 41219 is terminal: workspace check, warnings-denied Clippy and workspace
tests passed, then the external SDK fixture refused its stale independent lockfile.
Both final governance validators passed with zero warnings. The reviewed sandbox
stage is committed locally as `9bf942651`; remaining implementation is uncommitted.
The fixture lockfile now includes the conversation dependency and its required
futures-executor/tokio-stream entries, without updating existing package versions.
Offline full metadata resolution could not download an uncached js-sys package;
that command is not a passing verification. Subsequent locked native default SDK
execution passed (97844, `/private/tmp/i315-sdk-default.log`). Both-mode standard
fixture script is running as 13793 (`/private/tmp/i315-sdk-both.log`). Its cache
routing now respects CARGO_TARGET_DIR; default/absolute/relative routing and both
locked command shapes passed a mocked-execution check, not a runtime substitute.
After confirming no active build, authorized cleanup removed only the duplicate
worktree target (3.6 GiB); shared cache, worktrees and stashes were preserved.
Both-mode standard SDK script 13793 subsequently exited 0, including coding mode.
Independent static follow-up approved the lockfile, cache routing and seven-test
native Windows CI gate; no Windows execution was performed by that reviewer.
Repaired complete preflight is running as 66373 at
`/private/tmp/i315-repaired-preflight.log`.
Full preflight success, native Windows validation, stable-head review/CI/merge,
T1 completion and T6 implementation remain outstanding.

Protected source review approved fingerprint refresh against HEAD 261602242:
process_boundary.rs SHA-256 `79c687203a36ccfac636aa4a0db37a929e9460079227c69d9b2f9d83f9dd2863`,
windows_foreground.rs SHA-256 `88a05da24cfacfdabc56a0603e86781512e71752a6ff31899bbdf14cbd3f2622`.
Local recomputation matched before updating only their FNV entries. Review checked
acquisition/transfer/release/unsafe and instrumentation paths; no native tests ran.
Initial focused retries 88476/95376 omitted coding features and ran zero tests,
so provide no verification. Corrected coding-feature retries 20607 (push with
process-observation permission) and 26045 (inventory) are running. Standard
preflight 41219 runs with approved unsandboxed process observation at
`/private/tmp/i315-final-local-preflight.log`; no full success claimed yet.
Corrected retries completed: 20607 actually ran and passed the push test (1/1),
26045 ran and passed all native inventory tests (3/3). Full preflight 41219 remains
running; those component passes do not substitute for its complete result.
All implementation remains uncommitted. T1 and T6 remain open.

Independent pre-commit protected review against base 261602242 approved the
current implementation for stable stage submission, not merge or T1 completion.
Reviewer `/root/i313_mem007_correction_review` inspected cumulative tracked diff
and new service/tool/platform/test/ADR files, ran no builds/tests, and retained
native Windows and complete preflight gates. Reviewed protected SHA-256 values:

| File under crates/ | SHA-256 |
|---|---|
| talos-sandbox/src/lib.rs | 8133f88e23e790040b17ca2b48ed95bf970ed52ab31096842564c199d804460c |
| talos-sandbox/src/supervisor.rs | 43dbcc2d03abda4d2682acdf469dd830875eff7a9753d6b4c2d64f05ef11623c |
| talos-tools/src/process_boundary.rs | 79c687203a36ccfac636aa4a0db37a929e9460079227c69d9b2f9d83f9dd2863 |
| talos-tools/src/process_boundary/windows_foreground.rs | 88a05da24cfacfdabc56a0603e86781512e71752a6ff31899bbdf14cbd3f2622 |
| talos-tools/src/project_management/validation_tool.rs | 4f8de7d792d4df8e7ddce654e4379cb455cfc83aa29e3dbea4f97a4699a53e04 |
| talos-tools/src/project_management/unix_validation.rs | e2ad1a220607332c8618016345dadf8528b0721fe85bf89164cff2afd72bc102 |
| talos-tools/src/project_management/windows_validation.rs | 79578e4c58748c6fc555eb4c778bb5c8eec6208bd338a5e1f2f6c6e3c83250cc |

Sandbox-only logical change is staged and staged diff reviewed, awaiting full
workspace verification per Git SOP. No implementation commit exists yet.

## Effective activation and initial implementation — 2026-11-03

Claim #845 merged as `261602242e49e6f921f5d3e4eac96eebbe92f8b3` after
independent exact-head review, both governance validators and current checks passed
on `cb22c269e6375f6e5f039d3290480399fcb33fed`. Run `41644693318` initially
failed remote owner reconciliation: first the open-Issue matrix omitted #844 and
retained closed #390, then the synchronization comment lacked the validator's
literal `Status reconciliation` marker. Matrix and comment were corrected; the
final remote job `125950460475` passed. Workflow-level success alone was not used
to certify a failed individual check. CAS: PR comment `6605283974`.

Implementation branch starts from that merged claim. Local uncommitted work adds
a shared asynchronous host-executor seam with explicit started/not-started failure,
caller cancellation preventing later checks, and two service regressions. It does
not yet wire platform executors or product tools, migrate the legacy synchronous
runner, or satisfy complete T1 acceptance. Formatting passed. Focused locked test
attempt (`cargo test --locked -p talos-conversation validation::host::tests`) failed
while rebuilding Wasmtime with ENOSPC before tests ran; no passing test is claimed.
Target was `/private/tmp/talos-i313-target`, single build job, debug=0 and
incremental=0. The failed process is terminal (exit 101). No active worktree or
target was deleted. Resume by resolving build capacity, verify the service tests,
then implement shared bounded Unix/Windows execution and real host registration.
T6 and the full handoff remain open.

Further local implementation binds Print/inline/RPC contribution composition to
the selected workspace and adds the shared read-only agile contribution. Discovery
uses a manifest or any two canonical governance files, independently of project
size or Active status; language-project validation discovery remains separate.
The model-facing description states positive/negative invocation examples and
does not grant mutation or execution authority. CLI inventory fixtures now bind
an explicit empty workspace instead of inheriting the test process directory;
the selected-workspace regression covers Print, TUI and MCP summary registration,
native ownership and returned source content. Fixed Cargo check/test profiles now
include --locked in both execution arguments and displayed commands, with a
regression for their consistency. These changes remain uncommitted and untested:
formatting and diff checks pass, but available disk remains approximately 372 MiB.
No new Cargo build was restarted under that unchanged capacity blocker. The
platform execution adapter, CLI runner migration and permission/runtime acceptance
remain incomplete; registration tests alone do not satisfy them.

The next uncommitted increment adds a strict typed validate_project adapter with
plan/read versus run/execute facets, required-check failure propagation, marker
rechecking and injected execution/aggregate cleanup from one Arc host adapter.
It is not yet registered in products. Direct-call fixture tests are written, not
run, and do not prove actual permission admission; evidence explicitly says the
outer host's actual decision was not supplied. Independent static review identified
and corrected misleading decision text and independently supplied cleanup ownership.
Per-invocation cancellation isolation and actual admission evidence remain required.

Unix OwnedCommandOperation now has an additive execute_with_start_evidence API;
existing APIs preserve their SandboxError return via the same supervisor. Typed
NotStarted/Started/Unknown distinguishes normal spawn refusal/failure, a retained
child, and uncertain spawn panic without parsing diagnostics. NotStarted means no
successfully started command, not proof that the OS never forked. Startup state
does not certify cleanup. Independent static security/API review found no new
cleanup-path defect, with adapter follow-through still required: Unknown is failed,
and unconfirmed operation receipts stop further checks. On current macOS, the
focused startup regression passed, then `cargo test --locked -p talos-sandbox`
passed 49 unit tests and 2 doc tests (sessions 8916 and 89323, exit 0). Both used
CARGO_TARGET_DIR=/private/tmp/talos-i313-target, CARGO_BUILD_JOBS=1,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0.
This small dependency graph avoided the blocked Wasmtime rebuild. Available disk
afterward is about 331 MiB; conversation/tools tests, Windows evidence, full
preflight, remote exact-head review/CI/merge and T1 completion remain outstanding.

2026-11-04 local continuation: UnixValidationHost now reuses the owned supervisor
with per-invocation operation/cancellation and aggregate shutdown ownership.
Cancellation drops execution before waiting for operation cleanup; unknown startup
is failed and unconfirmed cleanup stops the plan. ValidationHostFactory replaces
the shared invocation executor; factory panic is contained without claiming cleanup.
The panic hook remains visible in test logs; only returned tool evidence omits payloads.
Project-management minimal-feature tests passed 5/5 (session 52084); shared
conversation validation tests passed 21/21 (50153). Same locked/debug=0/job=1/target
settings as above; no cache was cleaned. Disk naturally recovered enough to build.

The finite-output overflow regression initially failed with macOS EPERM cleanup
both inside and outside the execution sandbox (71598/5337/57343/83491). Apple XNU
https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sig.c
filters SZOMB in killpg1 and can return EPERM with no signalable group member.
An exit-visibility race is a supported hypothesis, not a proven specific fallback
branch. Added at most 100ms asynchronous retry of the SAME complete ownership and
ADR-082 zombie-only proof, charged to the existing cleanup budget. EPERM, empty or
failed enumeration never independently means success. The finite regression then
passed. After the budget correction, full sandbox tests passed 49 unit + 2 doc
tests (6438). Independent review of this latest retry is requested. Persistent
failure injection and concurrent invocation isolation coverage remain outstanding,
alongside Windows, product permission wiring, CLI migration and full merge gates.

Further 2026-11-04 local evidence (all implementation remains uncommitted):

- Unix invocation concurrency and dropped-future shutdown regressions passed;
  project-management tests passed 7/7 in session 2488. Persistent Darwin EPERM
  and immediate ECHILD refusal injection passed in 13955. Minimal feature Clippy
  with -D warnings passed in 7508.
- CLI run now uses the owned adapter on Unix and Windows, with signal listener
  registration before execution, cancellation stopping later checks, retained
  JSON evidence on cleanup failure, and nonzero exit for Blocked/required failure.
  Native macOS CLI check passed in 17567; Windows CLI branch has not been natively
  verified. The public synchronous conversation runner is still unmigrated.
- CLI validation unit tests passed 2/2 in 33574. The first manual binary smoke
  timed out before the readiness marker; its startup delay remains unexplained.
  Subsequent diagnostic execution and the new regression
  `tests/validation_cancel.rs` passed in 7231: real SIGINT, nonzero exit, parseable
  JSON, only fmt started, later checks NotStarted, no delayed descendant write.
  The test source is uncommitted; this is local evidence, not merged delivery.
- CLI compatibility permission-wrapper tests passed 3/3 in 48673: Deny/headless Ask block
  host factory construction; mixed later-command denial prevents all execution;
  replace_policy workspace change revokes the validation session grants. This
  does not establish every product's workspace switch or actual admission receipt.
  Subsequent inspection found these wrappers are cfg(test), so the earlier "real
  CLI wrapper" characterization was too broad. They exercise real permission
  state but not production dispatch. A registered-tool/production PermissionPipeline
  regression is now written but unrun; full dispatch denial remains outstanding.
- Windows Job typed startup API and WindowsValidationHost are implemented locally.
  Native launch failures remain conservatively Unknown/cleanup unconfirmed;
  preflight refusal is NotStarted, returned process handles establish Started.
  Fixed cargo.exe resolution, explicit environment snapshot and invocation-local
  cleanup reuse the existing Job executor. Cross-target minimal-feature check
  passed in 2903; after feature-boundary corrections warnings-denied Windows
  target Clippy passed in 16139. No native Windows test ran. Synchronous executable
  discovery/canonicalization is outside the process deadline; complete preparation
  bounds and native failure coverage remain open.
- Shared Product/Runtime/MCP contribution composition now includes validation for
  eligible projects, alongside independently eligible agile summary. New Cargo-only
  discovery/plan test first ran zero cases without shared-composition (96650), which
  is not verification. Correct feature run 64286 failed linking with ENOSPC before
  the test executed. Available disk subsequently measured 153 MiB. No unchanged
  retry or unauthorized cache cleanup performed. Product registration, full
  permission/SDK/runtime acceptance and exact inventories still require validation.
- Model tool result classification now also treats Blocked findings as errors,
  matching CLI failure semantics. Formatting passed; this latest change has not
  been rebuilt under the capacity blocker.

Next: resolve build capacity, finish bounded preparation/public sync API migration,
validate actual registered permission paths and native Windows behavior, synchronize
user docs and inventory dispositions, then full preflight/review/CI/CAS/merge. T1
is not complete and T6 remains unimplemented; the full ordered handoff stays Active.

The next local increment contains adapter construction/poll panic containment,
invalid-success/oversized-evidence refusal and a regression ensuring no later
checks start after an adapter panic. Panic payloads are not copied into returned
evidence; this does not suppress the process panic hook or prove process cleanup.
Formatting and diff checks passed; tests remain unexecuted after the ENOSPC build.
Both inspected incremental-cache directories are empty; no space was reclaimed.
Requested explicit permission to clean only `/private/tmp/talos-i313-target`
rebuildable cache, preserving source/worktrees/stash/logs and the other target.
That request was subsequently explicitly approved. Cleaning only that target
removed 43,916 files / 20.3 GiB; source, worktrees and stash were preserved.
Rebuild session 53327 completed compilation, passed 8 CLI validation tests and the
Runtime Cargo-only contribution test, then exited 101 because the production
dispatch denial test expected the policy reason instead of the pipeline's public
`Permission denied: approval denied` text. The assertion is corrected locally;
follow-up session 50355 is running. Absence of build artifacts alone is not proof
of zero host execution; an execution marker remains required.

A new synchronous governance-only entry accepts no plan or commands and the
conversation `/validate` caller now uses it. A regression covers a Cargo workspace
without selecting host checks. Formatting/diff checks passed before the latest
assertion correction. The old public sync runner remains pending ADR/migration;
this additive step does not close that defect. All changes remain uncommitted.
Fresh fetch still resolves origin/main to `261602242e49e6f921f5d3e4eac96eebbe92f8b3`.

Follow-up 50355 passed 26 conversation and 2 Runtime tests. Session 75584 then
passed the same scope with a counter before the real validation tool execution
entry: actual Runtime Deny returns an error and the entry count remains zero.
This covers that Deny path, not Ask, all workspace grant behavior or cancellation.
Independent reviewer `/root/i313_mem007_correction_review` accepted ADR-097's
API/security contract on 2026-11-04, without running tests. Its cwd delegation
finding was corrected in the observation wrapper; authorized execution is also
forwarded explicitly. The old public sync runner and private direct subprocess
implementation have now been removed locally; the obsolete private-runner test
was removed while injected-host failure tests remain. CLI unsupported-platform
fallback now reports NotStarted through the shared service instead of spawning.
This breaking removal is next-minor-only, never a 0.10 patch. Session 30216 is
validating the migration across conversation, Runtime and CLI; no result yet.
README applicability and build-side-effect descriptions are updated. Full native
Windows, SDK, preflight and stable candidate review/CI/merge remain outstanding.

Migration session 30216 passed CLI 8, conversation 25 and Runtime 2 tests.
Both governance validators passed (60434/97167). New Unix pipe-boundary tests in
23341 passed all 6 adapter tests: concurrent stdout/stderr each at 256 KiB (exact
per-stream limit beyond pipe capacity), and deadline cleanup of a descendant
holding pipes after its parent exits, including no delayed marker. These extend
local runtime evidence, not remote/native-platform acceptance.

Independent Windows design review confirmed that synchronous filesystem and
CreateProcess calls cannot be interrupted by a Tokio timeout. The local Windows
adapter now charges preparation against the command budget and checks cancellation
and deadline around discovery/canonicalization and before execution. New preflight
regression is written; cross-check is running. This does not yet bound blocking
kernel calls, own a separate preparation worker, or add pre-resume native launch
cancellation checkpoints. Those remaining implementation gaps stay in T1; do not
claim full end-to-end deadline or native Windows validation. SDK migration and
tool applicability documentation now explicitly describe these candidate boundaries.

Windows preparation cross-check 34249 passed. Native launch now has cancellation/
deadline checkpoints at entry, before CreateProcess, after suspended process
ownership and before ResumeThread, using existing early-return cleanup. Added
four-stage refusal regression; no native execution evidence yet. Cross-check with
`--tests` (96989) failed in a native dependency build because
`x86_64-w64-mingw32-gcc` is unavailable, before the test target was verified.
This is not a test pass. Production-only Windows warnings-denied Clippy is running
separately; independent review of checkpoint ownership/races is requested.
Synchronous kernel calls, preparation worker ownership and native platform evidence
remain open; checkpoint additions alone do not establish hard wall-clock bounds.

Windows production-only Clippy 60556 passed with -D warnings. Independent static
review accepted checkpoint placement/ownership, while explicitly retaining the
check-to-ResumeThread race: cancellation observed at a checkpoint prevents resume;
cancellation after that check is handled by the Job supervisor. No claim of atomic
cancel-versus-resume ordering is made. Subsequent code carries one absolute deadline
through foreground encoding and supervisor scheduling, limits PATH discovery to
64 KiB/256 entries and environment preparation to 4096 entries/1 MiB, and checks
preparation cancellation during enumeration. Windows production Clippy again
passed (`/private/tmp/i315-windows-budget-clippy.log`). Added actual foreground
oneshot tests for pre-cancel, closed sender and expired deadline; native execution
and test compilation remain pending the Windows toolchain/CI gate. Those tests
retain Unknown/unconfirmed error evidence and do not certify native cleanup.

Windows preparation now runs in a blocking worker registered under the existing
operation/aggregate Scope before scheduling. A child cancellation token drop guard
closes preparation authority when the caller is dropped, cancelled or times out.
The worker retains its reservation until return; panic leaves sticky unconfirmed
cleanup. The current PATH/workspace/environment closure only constructs a Command;
the generic internal callback API is not a capability restriction on other closures.
The caller waits the operation receipt before accepting a prepared
command or returning cancellation; a still-blocked worker yields CleanupUnconfirmed.
Added a native regression that aborts the caller while a worker is deliberately
blocked, verifies aggregate wait stays pending, then releases and joins ownership.
Production Windows Clippy passed (`/private/tmp/i315-windows-worker-clippy.log`);
native regression is not run. Independent resource/security review accepted the
ownership design without running tests; its suggested checkpoint before workspace
canonicalization is applied. Pending ownership does not mean blocking I/O stopped.
Native launch itself still uses synchronous OS calls inside the supervisor; no
claim of hard wall-clock interruption is made. Full native validation remains open.

CLI/Runtime shared-composition all-target warnings-denied Clippy passed in 33705.
Project-management minimal-feature suite passed 10/10 in 57967, including new
governance-only internal execution and Node/Python/Go/Java prerequisite refusal
with zero host-executor calls. Both apply to actual contribution execution; they
do not prove model intent routing. Standard `release_preflight.sh` is now running
with the same locked toolchain, two jobs, debug=0 and existing target; its full
workspace and external SDK gates are not yet certified. Implementation remains
uncommitted and T1/T6/full handoff remain open.

The production Runtime denial regression now covers both Deny and headless Ask.
Its first extension used an invalid payload on the unit Ask variant (compile
failure 64644), then expected Deny's generic error for Ask (57746 failed because
Ask correctly reports no configured approval resolver). Both fixture mistakes
are corrected; 98418 is verifying separate exact refusal messages and zero
execution-entry counts. These failures are not successful execution evidence.
Standard preflight 88690 has passed governance/claims, workspace check and Clippy
and is building workspace tests; neither the full test result nor SDK gates have
completed yet.

I308 inventory disposition for the new contributions belongs to this T1 slice:
agile_summary and validation plan are Read facets; validation run has Execute
facets. No ordinary Auto command-review eligibility is added for validate_project
and no Pull/Git/Exec authority is reused. The existing ordinary-review allowlist
in permission_pipeline remains unchanged. Model-facing descriptions and SDK docs
state applicability without creating an automatic approval or intent-routing rule.

Preflight 88690 terminated with exit 101: CLI tests passed 402/403, and the
workspace wrapper fixture incorrectly assumed tree was the last contribution.
The fixture now locates tree/document_extract by name and preserves its wrapper
assertions; focused test 87329 passed. Full preflight has not passed or reached
SDK validation. Runtime Deny/headless Ask production dispatch test 98418 passed
both cases with zero execution entries before the following route change.

Independent design review accepted using existing PreparedNormalToolInvocation
for actual validation permission evidence. The local run adapter now binds input,
provenance, full facets and plan identity, consumes exact AdmittedInvocation before
factory construction, rechecks facets, and refuses bare run calls. Plan remains
ordinary Read. Evidence records only exact admission/consumption, not inferred
grant scope or approver. Initial check caught a preparation method-name mismatch,
corrected to prepare_normalized_with_provenance. Tests now obtain real permission
state approval/proof; 80894 is running. Actual CLI/Runtime prepared-route success,
failure and mismatched/missing proof coverage remain required; previous ordinary
dispatch tests do not alone certify this changed route. Changes are uncommitted.

Prepared tool tests 80894 passed 11/11. The next increment adds missing proof,
wrong input/plan identity and changed-facet refusal before executor construction;
Runtime's observation wrapper now forwards the prepared operation and counts its
actual execution entry. Combined tools/Runtime validation filter 54564 passed;
this includes real Runtime Deny/headless Ask through the new prepared route.
Admission proofs and prepared operations are consumed by value, with no Clone
implementation or replay API. Approved production host execution/provider-visible
results, CLI prepared-route verification and native Windows remain outstanding.

Runtime approved execution regression 83892 passed in isolated test subprocesses:
an explicit Allow policy admits the shared validation contribution, the real Unix
owned executor runs all three fixed Cargo commands through a harmless fake Cargo
fixture, and provider continuation contains its output. Both exit 0 and started
exit 7 produce correct per-host-check status/exit/admission evidence. The fixture
initially expected the wrong fmt argv (53949); corrected to `fmt --all -- --check`.
This is real dispatch/ownership evidence, not a real project build or native Windows.
CLI Agent plus actual Print registry prepared-governance regression passed 92067
after correcting test-only HookRegistry import and run_streaming signature errors
(80866/47708). It verifies internal execution and exact admission evidence, not
CLI host-command execution. Historical real CLI SIGINT fixture remains recorded
above and needs current integrated verification. New standard preflight is running
at `/private/tmp/i315-prepared-preflight.log`; final protected review is pending.

Protected review found PreparedValidation lacked the operation-local receipt
registered by Agent before prepared dispatch. Added a lazy per-operation receipt:
proof consumption remains before factory; NotStarted transitions to Unconfirmed
before factory entry, then Owned only after a real executor receipt is published
before any check await. Factory/receipt panic or missing receipt refuses checks
and stays Unconfirmed. Concurrent wait on NotStarted atomically closes the scope;
later execution cannot create the factory. No aggregate receipt substitutes for
operation-local ownership and no mutex is held over await. Unix/Windows adapters
provide their existing operation receipts through an additive executor method.
Updated project-management suite 95846 passed 15/15, including prior-wait closure,
factory panic and missing/panicked receipt refusal. Corrected-state independent
review is pending. Preflight 80252 continues workspace suites; it started before
these final receipt corrections, so it cannot alone certify the latest source.

Independent reviewer accepted the corrected lazy receipt state machine without
running tests. Standard preflight 80252 has now executed the real CLI SIGINT
integration test successfully (1/1, 1.79s), preserving JSON, stopping later checks
and preventing delayed descendant writes. Runtime fixture now also asserts overall
ToolResult success versus failure, not only individual host records; updated test
91621 passed both exit-0/exit-7 scenarios with latest receipt code. Latest Windows
production cross-Clippy is running separately; native tests are still outstanding.

Latest Windows production Clippy 99511 passed. Native Windows adapter now has a
real Cargo --version/output/operation-and-aggregate-cleanup regression written,
not executed on this macOS host. Fresh fetch retained main/HEAD at 261602242.
Preflight 80252 reached tools with 1067 passed/3 failed: two source-inventory
fingerprints changed with reviewed Windows lifecycle edits, and the prepared local
push test's process observation was denied by the execution sandbox (EPERM).
Full source-level independent review of both fingerprinted changed files is
requested before updating their baseline; no blind hash refresh. The push fixture
is rerunning with approved unsandboxed process observation. SDK gates were not
reached. Full T1 remains open.

> Document status: Complete (#847 merged; implementation evidence above)
> Published plan date: 2026-11-03
> Planned objective: complete handoff T1 using shared validation and supported read-only governance services.
> Baseline rule: preserve all published acceptance; no completion by documentation alone.
> MVP deliverable: a model discovers applicable tools, reads project status or plans checks, and executes supported validation with real permission, timeout, cancellation, bounded output and cleanup evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | VALIDATION-003 / handoff T1: shared model validation and read-only agile tools, CLI/Runtime host composition, bounded Unix/Windows execution and acceptance |
| Claimed At | 2026-11-03 |
| Source Issue | #844 |
| Governance Claim PR | #845 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized full ordered handoff delivery; independent API/process-security review, exact-head CI, validators and CAS required |
| Implementation PR | #847 |
| Last Updated | 2026-11-05 |
| Handoff / Release Condition | Claim and activation effective only after finalized governance PR merges; no release authority |

## Published Baseline

Selected child [VALIDATION-003](../backlog/active/VALIDATION-003-model-validation-agile-tools.md)
reuses VALIDATION-001 and GOV-003; it does not reopen the completed shared-service
slice or claim broader governance automation. All acceptance in that owner is binding.

### Preconditions and selection inventory

Target `58d8bf926beefe24df4e04c3d9c98f5ca62636cd`: I313/T5 implementation #837
is merged as `322574bc9707492ea6e9d5232794f73e687071d9`; closeout #843 merged,
Issue #390 closed. First five ordered handoff stages are complete. I315 was absent
from current owners and remote branch inventory. No overlapping validation PR found.

| Owner | State | Disposition |
|---|---|---|
| I310 / #838 | Active | Retain independent Gemini/authentication ownership; recheck composition overlap |
| I314 / #842 | Active | Retain independent Search ownership; no configuration/transport takeover |
| I302 | Review | Retain Windows self-only diagnostic owner; not launcher authorization |
| I277 | Review | Retain deferred human/device acceptance |
| I249 | Planned / Unclaimed | Retain deferred dependency pilot; no dependency upgrade |
| I164 | Paused | Retain superseded layout disposition |
| I313 | Complete / Closed | Consume merged T5 evidence; do not reopen |

No Blocked iteration header found. Nonstandard Status-table owners were also
checked; I263-I267/I274/I275 are terminal. Historical Active/Review references
inside completed owners are not new activation. T1 replaces this handoff's T5
execution slot and does not take over independently authorized I310/I314 work.

### Architecture and compatibility

Conversation retains project detection, plans, internal validation and evidence;
add a small injectable async host-check interface. Tools owns tool adapters and
process integration: Unix reuses sandbox OwnedCommandOperation; Windows reuses
existing Job-owned windows_foreground operation. No second subprocess runner or
conversation→tools dependency. Check tools→conversation dependency and feature
boundaries before implementation. Add a contribution factory without required
fields on public SharedToolContributions. Preserve bare RuntimeBuilder versus
explicit shared product composition. Migrate existing CLI host-run to the same
bounded adapter; internal /validate governance remains in-process. Record public
synchronous API compatibility; no silent loss of supported behavior. Any breaking
public API requires an accepted ADR/migration before coding that change.

### Validation and documentation

Pinned Rust 1.99.0, locked focused tests/check/Clippy, full standard
release_preflight.sh including external SDK modes, both governance validators,
independent API/process-security review, exact-head native CI and merge CAS.
Update README en/zh, tool reference, SDK/CLI usage, parent relationship notes,
owner/index/Board/handoff and Issue. Completion needs existing merged implementation
SHA and actual acceptance, not this claim. T6 remains next; full goal Active.

## Actual Activation And Execution

2026-11-03: governance-only draft from current main; no production implementation,
new dependency, implementation tests or effective activation in this record.

Finalized atomic proposal: #845 carries the actual PR number and proposed
Claimed/Active record. Both remain ineffective until this finalized head merges.
Independent review, validators, exact-head CI and CAS precede merge.
