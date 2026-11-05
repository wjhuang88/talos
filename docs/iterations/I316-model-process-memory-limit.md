# Iteration I316: Model-Selectable Process Memory Limit

> Document status: Active / Partial (claim #854 effective at 18aa4a58f)
> Published plan date: 2026-11-05
> Planned objective: complete handoff T6 across existing shell/exec execution modes.
> Baseline rule: preserve full acceptance; platform gaps cannot be closed by wording.
> MVP deliverable: model-provided per-process limits with tested inheritance, enforcement/failure evidence and unchanged 2048 MiB default policy.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6-sol |
| Work Slice | PROCESS-001 / handoff T6: typed process memory policy, shell/exec foreground/background/composition propagation, native enforcement and acceptance |
| Claimed At | 2026-11-05 |
| Source Issue | #853 |
| Governance Claim PR | #854 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized the ordered handoff; independently reviewed claim #854 merged as 18aa4a58f (review 6646977525, CAS 6647020075); final implementation exact-head review remains required |
| Implementation PR | Not started |
| Last Updated | 2026-11-05 |
| Handoff / Release Condition | No implementation before finalized claim/ADR merge; no release authority |

## Published Baseline

Selected Story [PROCESS-001](../backlog/active/PROCESS-001-model-process-memory-limit.md)
owns every T6 acceptance item. [ADR-098](../decisions/098-model-process-memory-limits.md)
defines proposed native policy and next-minor migration; independent acceptance is
required before implementation. No model/conversation-memory scope is included.

### Inventory and dependencies

Baseline `cc36fb1bc5f2d495ddaea472642896701e98dca5`: I315 Complete/Closed,
implementation #847 `add669403`, closeout #852, source #844 closed. Full predecessors
I309 → I308 → MODEL-014 → T4 → T5 → T1 are complete; only T6 remains in this queue.

| Owner | State | Disposition |
|---|---|---|
| I310 | Active | Independent Gemini wire/authentication; retain ownership |
| I302 | Review | Self-only Windows PSS diagnostics, no production launcher claim; retain |
| I277 | Review | Deferred human/device acceptance; retain |
| I249 | Planned / Unclaimed | Deferred dependency pilot; no dependency upgrade |
| I164 | Paused | Retain historical superseded layout disposition |
| I315 | Complete / Closed | Consume merged lifecycle/API evidence; no reopening |

All iteration status headers were inventoried; no Blocked header was found.
Open search intake PRs #849/#851 remain Proposed/Unclaimed and non-overlapping;
their temporary registration in #852 grants no implementation authority. Refresh
inventory and open PR/Issue overlap during claim CAS, not from this snapshot alone.

### Implementation and validation

Map actual final spawn owners before coding: Unix shell, direct exec (currently
missing hardening), background duplicate hardening, and Windows Job-based paths.
Propagate one resolved typed policy with omission/explicit evidence and per-step
inheritance. Reuse lifecycle owners; no new parallel launcher or permission bypass.

Run schema/invalid input, actual permission proof, default/override/inheritance,
foreground/background/sequential/parallel/pipeline, native allocation denial,
parent isolation and cancellation/cleanup regressions from PROCESS-001. Pinned
locked workspace check/Clippy/tests and standard preflight with both SDK modes;
native Linux/Windows CI, macOS exact-host evidence, independent security/API review,
exact-head CAS/merge and owner-first Completion Commit are required.

### Documentation / risks / rollback

Update README en/zh, tool descriptions/schemas, SDK input migration, ADR-098,
story/iteration/index/Board/handoff and #853. Explicitly distinguish RLIMIT_AS and
Windows committed memory, omitted best-effort policy and confirmed enforcement.
Windows default enforcement tightens historical behavior; next-minor migration
must disclose it. Roll back policy via reviewed commit, never by removing permission
or cleanup safeguards. No release, provider probing or aggregate tree quota.

## Original activation proposal (historical)

Proposed atomic activation in #854; ineffective until target-branch merge. Independent
API/native-security design review approved the complete contract on 2026-11-05,
including explicit inheritance and pre-exec safe evidence requirements. Exact-head
protected review and CI remain required. No code implementation, tests or Completion
Commit exists. Full handoff remains Active.

### Local implementation checkpoint — 2026-11-05

Claim #854 merged as `18aa4a58f269b441ff8df7cc030352c718f7e60a` after
exact-head CI `41947908272` (four applicable jobs successful; native jobs skipped
for documentation scope), independent protected review comment `6646977525`, and
merge CAS comment `6647020075`. Implementation starts from this effective merge.

Local, unpublished work adds strict optional memory fields to BashInput, ExecInput
and ExecStep, plus a typed policy preserving explicit request inheritance. The
locked shell-feature policy unit tests passed; the two actual tool-parser regressions
passed with `cargo test -p talos-tools --locked --features shell --lib memory_`.
These are parsing/policy tests only: final launch owners, native enforcement,
visible evidence, permission-proof regression, full mode/platform coverage and
required workspace/review/CI/merge gates remain unimplemented or unverified.
Do not publish this intermediate API as working memory-limit support.

Unix local increment: shell/direct-exec foreground and supervised background now
pass the resolved policy to resource hardening; the duplicate exec-background
default hook is removed. Explicit setup errors refuse exec and inherited hard
limits are never raised. Request values propagate into sequential/parallel/pipeline
steps while preserving explicit intent. Five focused tests pass via
`cargo test -p talos-tools --locked --features shell --lib memory -- --nocapture`,
including an actual child probe accepting only the exact configured bound or
macOS setup rejection with no payload marker. The aggregate result does not identify
which macOS branch executed and is not allocation-denial evidence. Windows wiring,
visible policy/setup evidence, full operation/mode/permission/native allocation
coverage, reviewed native inventory and delivery gates remain outstanding.

Windows local increment wires shell foreground/background and exec background
policy into the existing Job setup, using checked `usize` conversion and
`JOB_OBJECT_LIMIT_PROCESS_MEMORY` before assignment/resume. Non-shell foreground
consumers retain their prior contract. Windows foreground exec is still unwired;
no Windows compilation or runtime success is claimed by the five macOS focused
tests repeated after this change. All native changes remain unpublished and await
independent implementation review and inventory acceptance.

Windows foreground exec now adapts through the existing owned Job supervisor,
including inherited environment materialization, bounded input and memory policy.
`cargo check -p talos-tools --locked --features shell --target x86_64-pc-windows-gnu`
passed locally with feature-specific warnings (partially corrected afterward).
This is cross-compilation only, not Windows runtime evidence. Adapter timeout/output
semantics still need preservation: the existing owned supervisor reports output
overflow as an error rather than exec's historical truncation, and timeout must
retain structured step output. These are implementation defects to fix before
submission, not accepted behavior changes. Full gates remain open.

Subsequent local correction adds per-stream bounded capture and total byte counts
to the existing Windows supervisor, returning structured timeout plus retained
output only after confirmed cleanup. Non-tool callers retain overflow-error policy.
Cross-check compiled; native truncation/timeout regression added but not executed.
Visible memory-policy descriptions now distinguish default Unix best-effort from
explicit setup requirements and Windows committed-memory semantics, including
background launch results and per-step output. Shell-feature unit suite initially
reported 121 passed / 7 failed: five command-header regressions were corrected by
preserving the first command line; two native-inventory gates intentionally remain
unaccepted pending independent review. No full-suite success is claimed.

Native evidence refinement: the macOS explicit-4096-MiB probe was rerun with
branch-specific output and observed `Invalid argument (os error 22)` before payload
startup. This proves fail-closed setup on this host, not successful enforcement.
Linux-only tests now cover actual `try_reserve_exact` allocation denial under a
128-MiB child address-space limit, unchanged parent `/proc/self/limits`, and
background child observation of the explicit 128-MiB override. These Linux tests
are authored, not executed locally. Independent protected implementation review
has been requested against the current local diff; inventory acceptance remains open.

Independent review found two blockers: omitted Unix policy could raise an inherited
hard limit, and Windows exec created per-step aggregate owners outside the tool
lifecycle. Local corrections now inspect inherited hard limits for all policies
(omission takes the minimum; failed inspection leaves inherited limits untouched)
and retain a shared Windows executor on ExecTool, exposing its aggregate cleanup
owner to the registry while steps use separate operations. Windows cross-check
passes with four remaining feature warnings. Cancellation/runtime teardown proof
and reviewer confirmation remain required. New Linux real-child tests cover
request/step overrides across sequential/parallel steps and pipe chains; these are
not run on macOS. All 25 macOS exec tests, including invalid-later-step zero launch,
passed before these final ownership corrections; no broader pass is implied.

Cancellation acceptance authored: a Windows ExecTool test waits for a real payload
marker, aborts its caller, seals/waits the exposed aggregate owner, checks cleanup
and rejects subsequent launch after sealing. Local Windows `--tests` cross-check
cannot reach these tests: dev dependencies arborium/ring require missing
`x86_64-w64-mingw32-gcc`. No test pass is claimed; native Windows CI remains required.
Feature-gating warnings were corrected separately without adding lint suppressions.

Correction after maintainer feedback: the GNU/MinGW attempt above was the wrong
validation route, not a project blocker. LOCAL-DEV already requires cargo xwin
with MSVC. Installed cargo-xwin 0.22.0 is now running
`cargo xwin check --locked -p talos-tools --all-features --tests --target x86_64-pc-windows-msvc`
using the existing SDK cache after sandbox access approval. Its result is pending;
native Windows execution is still separate. AGENTS/EVOLUTION capture the missed
pre-command SOP check so target discovery cannot override the documented route.

Correct-route result: the above all-feature MSVC test cross-check completed exit 0
in 1m15s. This supersedes the incorrect MinGW blocker; native Windows execution
is not yet validated. A real Windows child allocation probe is included: under a
128-MiB Job policy, a small allocation must succeed and a 512-MiB allocation must
fail, followed by confirmed cleanup. Its runtime result remains pending.

Further acceptance work: real ordinary Auto admission now includes a same-call-ID
replay changing only `memory_limit_mib`; the assessor must see the changed field
and execution must refuse reuse. The focused agent regression is compiling.
Independent review confirms the two prior fixes but retains a blocker: child-side
effective-limit evidence must report a lower inherited hard limit rather than only
the requested/default policy. A bounded pre-exec evidence-channel design is under
review; no parent-query substitute or inventory approval is claimed.

Latest local evidence increment: Unix shell/exec foreground and background launch
now use a fresh nonblocking fixed-record channel per spawn. Child-side getrlimit
after setter reports actual soft/hard bounds and setter status; explicit failures
refuse exec, missing/invalid records report unknown. Parent collects after spawn
handshake without waiting for EOF and releases Command-held writer afterward.
Pipeline metadata remains separate from child stdout/stdin (regression passed).
Four record-validation tests, 25 exec tests and background observed-bound tests
passed. Ordinary Auto same-ID changed-memory regression also passed after the
ENOSPC retry with incremental compilation disabled. Pinned Rust b940084d7 socket
source shows Linux MSG_NOSIGNAL but macOS pair uses flag zero; retained-reader
invariant is under independent review. Full inventory approval remains withheld.

Channel-focused independent static review accepted the retained-reader invariant
and observed-bound protocol; this is not full inventory approval. A real Unix
timeout/descendant-pipe regression first failed, then passed after signaling the
owned process group before leader reap on timeout. A separate cancellation
regression still fails: aborting foreground exec permits a delayed descendant file
write. This is a confirmed unresolved implementation defect. Required next action:
adapt foreground exec to the existing owned Unix supervisor with host-visible
cleanup receipts, preserving truncation/timeout output and child-bound evidence;
do not substitute an untracked drop task or claim full cleanup from timeout alone.
Linux inherited-hard-limit/default-observation regression is authored but unrun.

Owned lifecycle correction: Unix foreground exec and shell now reuse the existing
sandbox supervisor and expose aggregate cleanup receipts. Captured execution
retains bounded streams while draining excess bytes, preserves timeout output only
after confirmed cleanup, and counts final cleanup reads with saturation. Unix shell
retains at most 1 MiB per stream and reports truncation. The cancellation defect
above is fixed locally: both tools pass real abort → seal/wait receipt → no delayed
descendant-write regressions. Shell's 49 existing tests pass; the additional large
output/timeout-descendant regression passes. Sandbox's 53 tests pass (a deprecated
atomic method warning was subsequently corrected to pinned Rust's `try_update`).
New sandbox capture API and shell lifecycle adapter remain under independent review;
this does not approve source inventory or complete T6. Windows MSVC all-feature
test cross-check passed again before the latest Unix shell changes; native Windows
and Linux execution, full preflight, final review/CI/merge remain open. Latest fetch
still places origin/main at effective claim merge `18aa4a58f269b441ff8df7cc030352c718f7e60a`.

Further local convergence: Windows foreground shell now uses the same owned Job
adapter as exec and exposes an aggregate receipt. Its new abort/seal/wait/late-write
regression compiles with xwin MSVC, but is not yet executed natively. Independent
static review accepted this wiring and the native policy/evidence files. Reviewed
SHA256s: process_boundary.rs `319a4114ad853431ba511d2dd41a7d26bbc58252da4744c7e39d73f96185362d`,
windows_foreground.rs `f1c8dbfe11428cde41e02896a186083bcdbb7d94b5fde047a2aec3167e797e04`,
memory_evidence.rs `064ccb34bb9d05e948b7565f240ff55fe3a68ffd2a3b5a7eb643137063661d99`.
The corresponding inventory fingerprints are refreshed and the new evidence source
is included as the sixth protected file; this is static acceptance, not runtime proof.

Review also found the pipeline's outer timeout could return before owned cleanup.
That timer is removed: the same owner now enforces the minimum of the step timeout
and request/default cap. Its regression verifies retained output, confirmed cleanup,
no descendant late write and no next-step launch. Full locked shell-feature library
suite passes 139 tests, zero failures/ignored/filtered, including inventory. An
orphaned helper warning was removed afterward. Both shell platforms now retain
1 MiB raw bytes per stream and display stdout then stderr (no interleaving guarantee).
Sandbox capture is locally committed as `0a5287604`; remaining work is unpublished.
Tools Clippy initially failed ENOSPC; scoped cargo clean of CLI/runtime artifacts
released 3.7 GiB and the check is being retried. Full preflight, native CI and final
exact-head review/merge remain required; no completion is claimed.

Clippy retry now passes with `cargo clippy -p talos-tools --locked --features shell
--all-targets -- -D warnings`. Corrections remove an orphan helper, group step limits,
use explicit test failure messages and simplify an equivalent native condition.
Independent follow-up approved the new process_boundary.rs SHA256
`173948adc836a87b2b3c1c2dad37123a182c41092e1d1eef1320f7cfa5c37d11`
(FNV `e746535786e3d0ff`); the other two protected source hashes above are unchanged.
Both governance validators passed with zero warnings. These remain local evidence,
not final exact-head CI or native Windows/Linux runtime acceptance.

### Current local checkpoint

Scope is unchanged: complete PROCESS-001/T6, preserving published acceptance.
Local implementation commits are `ccc4f838c` (typed inputs), `0a5287604` (owned
capture), `82d658545` (native enforcement/evidence), and `29bc76056` (tool modes,
lifecycle and permission regression). None is a Completion Commit or merged
implementation. Latest shell-feature suite passes 139 tests without warnings;
xwin all-feature MSVC test compilation and strict tools Clippy pass. Linux fixture
now consumes upstream pipe input before observing limits to avoid early-exit races.
First standard preflight stopped at formatting; that one-line issue was corrected.
The retry log is `/private/tmp/i316-preflight-after-format.log`; this run later
failed ENOSPC during test compilation, as recorded below.

Synchronization scope: this owner → PROCESS-001 → iteration/backlog indexes,
Board and handoff, plus README/SDK migration and Windows xwin lesson/rule. Existing
historical snapshots and full acceptance are preserved. Required structural checks
are both project validators and the skill validator; native/full workspace evidence,
independent exact-head review/CI, publication and merge remain owned here. This is
a partial checkpoint, not delivery or a governance-model/profile change.

Standard preflight retry completed workspace check and warnings-denied workspace
Clippy, then failed during test compilation with ENOSPC. No workspace-test pass is
claimed. After explicit authorization, only `/private/tmp/talos-i313-target/debug`
was removed; both worktrees and the MSVC target/SDK cache were preserved. The next
standard-script run uses `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0` (no test/assertion exclusions), with log
`/private/tmp/i316-preflight-compact-build.log`. Its outcome is pending. Added Windows
composition regression checks real 192-MiB allocation refusal under inherited
128-MiB policy and success under a 256-MiB step override across sequential/parallel
steps and pipe chains; its later xwin result is recorded below and native execution
remains required. The skill governance validator passed; final project-validator evidence
will be refreshed after document convergence.

Windows composition test compilation subsequently passed via xwin/MSVC (log
`/private/tmp/i316-windows-composition-check.log`). A further authored background
launcher regression uses an explicit 128-MiB policy and requires its real child
to accept a small allocation and reject 512 MiB; its cross-check passed, followed by
a successful check of the reviewed cleanup-on-test-error correction
(`/private/tmp/i316-windows-final-check.log`).
Both added Windows regressions remain natively unexecuted. Source/claim refresh
confirms #854 merged at `18aa4a58f` and no implementation PR exists yet.

Windows regressions are locally committed as `1b0244272`. The compact-build
preflight reached actual workspace tests and failed in talos-agent: 654 passed,
one failed (`session_process_cancels_only_its_own_running_job`). Added memory text
had corrupted the background receipt's JSON envelope. The local fix adds memory
metadata as JSON fields instead of prefix/suffix text and extends the real session
status/cancel integration test to require those fields while retaining `job_id`.
This correction is under independent review and focused validation; earlier xwin
results do not certify the new production diff. Full preflight remains incomplete.

The real session receipt regression now passes with JSON metadata assertions and
cross-session status/cancel checks. Correction `b9ea2fceb` also preserves observed
child evidence in non-JSON error diagnostics. Independent static review accepted
the JSON-field approach and identified that error-path evidence requirement, now
implemented with a regression. The standard retry
`/private/tmp/i316-preflight-json-receipt.log` has passed workspace check/Clippy
and is rebuilding tests. Latest Windows cross-check is separately queued in
`/private/tmp/i316-windows-json-check.log`. Remaining gates include completed
standard preflight/SDK modes, all-feature workspace checks, native Linux/Windows
CI, exact-head protected review and merge/closure; T6 and the full goal stay open.

The JSON-receipt preflight retry subsequently stopped at MCP server E2E configuration
loading with sandbox `Operation not permitted`; no bypass/skip was introduced. The
authorized unsandboxed standard retry is `/private/tmp/i316-preflight-unsandboxed.log`
and has passed that formerly failing boundary, with remaining tests/SDK modes pending.
The latest production xwin check passed (`/private/tmp/i316-windows-json-check.log`).
All-feature workspace check and warnings-denied Clippy both passed; the dependency
`block 0.1.6` still emits its existing future-compatibility notice. All-feature tests
are running separately (`/private/tmp/i316-all-features-tests.log`). Final source
review is anchored to implementation `b9ea2fcebf32a848572e14327165b941db93676f` and
base `18aa4a58f269b441ff8df7cc030352c718f7e60a`; the latest fetch still matches that
base. No implementation PR, remote native evidence, or completion is claimed yet.

Final independent code review APPROVE on base `18aa4a58f` / head `b9ea2fceb`
covered the full T6 native/API/permission/lifecycle scope and found no remaining
source blocker. Reviewed `git diff base head -- crates` SHA256 is
`70bdae46faadfe55276fa49e33998ad819c248883c173b8fd8a0c947ccab0500`.
Reviewer did not run tests and did not approve dirty documentation or delivery.
The unsandboxed standard run has completed workspace tests and doctests (tools
unit suite 1115 passed); SDK fixture modes are next and waiting for the all-feature
build lock. All-feature tests and native remote acceptance remain outstanding.

Standard preflight completed exit 0, including both external SDK modes, in
`/private/tmp/i316-preflight-unsandboxed.log` (fixture pass lines 6254/6497;
preflight pass line 6498). All-feature tests first failed the existing long-history
source binding regression (654 agent tests passed, one failed). The same test
passed alone under the same all-feature configuration in 2.29s; the production
binding has a fixed two-second budget. Parallel load is a hypothesis, not a proven
root cause. A full all-feature retry with `--test-threads=2` is running in
`/private/tmp/i316-all-features-tests-bounded.log`, without changing the product
deadline or excluding tests. Preserve the original failure evidence; native CI
and final exact-head publication gates remain open.

The two-thread retry passed long-history binding but stopped at plugin-acceptance
because `TALOS_LANGUAGE_BUNDLE_ROOT` was absent. Prior temporary bundle directories
contained no usable artifacts. Rebuilt Rust/Python guests separately with pinned
locked plugin workspace and LLVM WASM compiler, packaged fresh bundles under
`/private/tmp/i316-real-language-bundles`, and verified digest installation,
activation, real context, trusted path and stop using `verify_installed_languages`.
Rust artifact SHA256: `2d67f75e9cdf7531ae43d1fc65a2d65db249b1220658d1597996fc561420f54b`;
Python: `040bd1ab7c8fa07b0743b5bcf16293797b220570212a0691a95a65e0d204a1b6`.
Full all-feature retry now supplies that verified bundle root, retaining two test
threads and all tests; log `/private/tmp/i316-all-features-real-bundles.log`.
