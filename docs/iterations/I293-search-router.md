# Iteration I293: Talos-Owned SearchRouter

> Document status: Active
> Parent: SEARCH-001-C / #643 / SEARCH-001 / #624
> Objective: implement the accepted ADR-085 first-valid-success router after B, without changing the model-facing contract.

| Field | Value |
|---|---|
| Story | SEARCH-001-C |
| Source Issue | #643 |
| Depends on | Accepted ADR-085; SEARCH-001-B / #642 complete in #656; NET-001 boundary coordination |
| Claim State | Claimed |
| Implementation PR | #681 / #685 / #696 / #706 / #707 / #708 / #709 (merged partial evidence); #712 governance amendment; #713 merged carrier; #714 merged direct transport; #715 merged TLS/CONNECT subset; #716 merged bounded form/diagnostic stage at `b5b8241e`; #732 merged owned payload at `8fa2a50`; #733 merged Lite/HTML parsing at `dea3edc`; #734 merged Instant Answer at `45cf9eb9`; #736 merged Bing parsing at `2b015a6b`; #737 merged fixed requests at `ae6a0791`; #738 merged bound execution at `2d7f4b36`; #739 merged fixed headers at `91c330ba`; #741 merged result admission at `4e20ee95`; #742 merged fixed chain at `9f998392`; #743 merged direct plan at `181d7584`; prepared-invocation candidate tracked from #643; full C/EP and default activation Pending |
| Completion | Pending |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-C SearchRouter; search-specific core/permission/Agent prepared invocation and endpoint authority; controlled existing-protocol compatibility internalization and optional-adapter transport; CLI/SDK/MCP dispositions, migration and validation under ADR-085/090 |
| Claimed At | 2026-10-06 |
| Source Issue | #643 |
| Governance Claim PR | #712 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer accepted bounded production scope 2026-10-13; independent protected security/API contract review, exact-head CI, governance validators, Issue reconciliation and final CAS required for #712 merge. No release or native admission authorized. |
| Last Updated | 2026-10-21 |
| Handoff / Release Condition | Original #664 claim effective at `d801790454e252699684253166984662ea9dab6a`; expanded scope effective only after #712 accepted-contract merge. Start implementation from that merge or later main; activation requires ADR-090 transport/parity/D gates. |

## Scope

- Add Talos-owned first-valid-success routing with bounded hedging and cancellation/deadline propagation.
- Preserve backend identity and typed failures, while keeping health state bounded and query-free.
- Keep paid providers explicit opt-in, Wikipedia as compatibility knowledge fallback, and rust-websearch as rollback adapter.

## Non-goals

No model-facing schema/output change, GeoIP or country routing, startup probe, generic retry/circuit-breaker system, new public crate, or provider admission.

## Acceptance

Deterministic fixtures cover first-valid-success, fast-error/slow-success, all-fail, invalid result, timeout, cancellation and bounded hedging. Locked tests, governance validation, focused review and rollback evidence pass.

## Test-Only Characterization Stage — 2026-10-09

Base: `270911c071961a79c85af742a4daf62d4ad6735a`. This stage preserves production search
behavior and the model-facing contract. Both new tests are inside the existing test module:

- Empty, empty-URL and non-HTTP URL responses returned as `Ok` can preempt a later valid response
  in the current select pattern. This records a gap against ADR-085, not first-valid-success.
- A successful or error winner drops the owned pending loser future. This is local cleanup
  evidence only; it does not prove HTTP cancellation or abortion of dependency-spawned work.

An existing backend test uses an array instead of a fixed vector to satisfy pinned Clippy's
`useless_vec` lint. No runtime wiring, provider admission, manifest or lockfile changes are made.
The older disposable two-future specification is not used as production evidence.

The previous unpublished local checkpoint `d0c8906dcca9f98b01d563818f395e2e4e3774d5` passed
search tests 25/25, tool-library tests 113/113, targeted Clippy including network tests, workspace
check/Clippy and both governance validators. Its first full preflight exhausted the 32 GiB
filesystem; compact local builds then reached workspace tests but failed in the unchanged runtime
Unix socket fixture with `PermissionDenied / Operation not permitted` (64 passed / 1 failed).
These are historical results, not a full-preflight pass for the reconstructed candidate. The
unpublished checkout/cache was subsequently lost during workspace maintenance. This stage is
reconstructed and freshly verified; the previous commit is not claimed as pushed evidence.

The maintainer explicitly authorized skipping that single environment-blocked fixture locally
on 2026-10-09 and retaining its complete PR CI coverage. The local workspace test command is:

```bash
cargo test --locked --workspace -- --skip tests::runtime_evidence_rejects_directory_symlinks_and_socket_entries
```

Local build settings are `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`
to bound storage. These do not change repository profiles. No `#[ignore]`, test deletion, permission
policy change or CI skip is introduced. Merge still requires exact-head full CI without this skip
and applicable review. Local validation results are recorded below before submission.

Fresh reconstructed-tree validation: `cargo test --locked --workspace` with the single explicit
skip above exited 0: 111 suite summaries, 3455 passed, zero failed/ignored, one filtered test.
This is not an unskipped full-workspace pass. Format, whitespace and both governance validators
also passed.
Workspace check and workspace Clippy (`--locked`, `-D warnings`), targeted network-feature test
Clippy, and the external Runtime SDK fixture in both default and coding modes subsequently passed.
Public-site, installer, text-boundary and CI-classifier checks passed as well. The local socket
skip was the only deferred test at submission; its final CI/merge disposition is recorded below.

This stage cannot complete C: actual routing, typed failures, invalid-result filtering, bounded
hedging, enclosing deadline/cancellation, bounded query-free health, NET-001 coordination and
binary integration/rollback evidence remain pending. I293 remains Active / Claimed, completion
pending; #643 and #624 stay open. D business correspondence is not required for offline tests.

## Merged Test Evidence And Integration Plan — 2026-10-09

PR #681 merged at `41381f8055b66d54de42988e2b936a12e5af3ac3`, after exact-head
`6474eb4fae47984ef4c0785ea694b57bac76874a` CI3057 / run `37878864053` completed success.
Full macOS release preflight (including the unchanged Unix socket fixture), Windows workspace
tests and governance validation passed; Linux Desktop validation also passed. The preflight runner
is macOS, not Linux. Independent agent technical review APPROVE, shared-GitHub-identity limitation,
single-maintainer reason and merge-time CAS are recorded in #681 comment `6074280675`.
Issue #643 synchronization is recorded in comment `6074282962`. This is existing partial test
evidence, not a C completion commit or transport-cancellation proof.

[Router integration and acceptance plan](../reference/SEARCH-001-C-ROUTER-INTEGRATION-PLAN.md)
pins the merged baseline and audits the private seam, AgentTool dispatch, Session task-abort path,
per-request timeouts and unclaimed NET-001 boundary. It proposes concrete routing transitions,
typed-error/health ownership, caller-context gates and C-V01 through C-V11 acceptance rows.
All those rows remain pending. No production implementation, numeric policy, public-API change,
optional-provider activation or native admission is delivered by the plan.

The documentation-only candidate passed both governance validators (including locked Cargo
metadata/SQLite consumer validation), public-site and installer checks, the 14-case CI classifier
suite, changed-document/source link verification and diff whitespace validation locally. No new
workspace-test execution is claimed for this prose-only stage; exact-head documentation CI and
technical review remain its remote gates.

Next: converge the caller deadline/cancellation and explicit-selection seams with their owners,
then implement the router only within a production-authorized stage. Until then, architecture
and offline characterization can proceed; generic retry/circuit ownership stays with NET-001.
I293 remains Active / Claimed, Completion Pending; #643 and #624 remain open.

## Offline Scheduling Experiment — 2026-10-10

This stage continues the effective #664 claim, without activating production routing. The runnable
deliverable is `crates/talos-tools/tests/i293_router_policy_experiment.rs`, a disposable experiment
with twelve paused-clock tests. Its affected reference documentation is the router integration
plan. This is an explicit infrastructure-only exception: no user-visible behavior is delivered.

Dependency correction to the preserved published table: canonical B owner
`docs/backlog/active/SEARCH-001-B-compatible-search-backends.md` is Complete / Claimed with
Completion Commit `7728f7681df11a9d715a0e479590c1002c53a26f` (#660), and its iteration is
`I292-search-compatible-backend-boundary.md`. The separate planned/unclaimed compatible-adapters
document is an alias, not evidence that the effective B dependency remains incomplete. Global
alias reconciliation is outside this C slice.

Selection inventory on base `e63e0ee3793e2e8fd914f676224f982c6cae5593`: I293 continues C;
I294 remains Active under independent D ownership (#693 merged partial evidence); I296 remains
Active under TOOLCHAIN-001; I290/I291 remain Review under locale owners; I277 remains Review with
deferred human validation; I249 remains Planned and is not activated. TEMPLATE is excluded.
No new iteration, claim or overlapping implementation is introduced.

The experiment owns lazy candidate futures directly, takes already-eligible candidates with
already-classified synthetic outcomes, and tests first-valid success, immediate replacement,
delayed bounded hedging, finite all-fail/no-eligible outcomes, one enclosing monotonic deadline,
pre/in-flight cancellation, external future drop, fast primary success and slow-primary survival
after hedge failure. An adversarial candidate poll signals cancellation while returning failure,
remaining pending at the hedge boundary, or returning success: no replacement or success escapes
the checked control boundary. Started and dropped futures and peak concurrency are observed.

The two-future cap, timing values, oneshot sender-drop semantics and deadline-versus-success tie
are fixture parameters, not shipping policy. Simultaneous cancellation/deadline ordering is not
established. No production SearchBackend, WebSearchTool, AgentTool, network/socket, environment
key, config, manifest, lockfile or CI policy changes occur. Future drop does not prove transport,
DNS or dependency-spawned work termination. Full C-V01 through C-V11 remain Pending; this supplies
only partial C-V01/C-V03 through C-V07 model evidence.

Recovery provenance: unpublished local checkpoints `7023c98ff4b75eefe4f03cf10e48cbb01eee1c90`
and `7c41d6ec` were lost with the temporary worktree. They are not pushed implementation or
completion evidence. The restored test blob is byte-identical to the final prior source:
`a58ddfa0b698d6f2c9634ec280adbb51568da2be`. Historically observed local results were twelve
targeted tests/strict Clippy, sequential workspace tests (3579 passed, zero failed/ignored, one
authorized socket filter), both Runtime SDK modes and zero-warning governance/claim validation.
An earlier cross-checkout shared target produced StableCrateId/crate-resolution failures; an
overlapping workspace run produced five existing Session elapsed-time failures. Isolated cache
and sequential rerun passed; the precise timeout cause is unknown. These are historical results,
not fresh validation for the restored candidate.

Fresh recovery uses the current merged main above. Old registered Rust 1.97 executables failed
with SIGBUS; a separate rustup home was installed and actual `rustc`/`rustfmt` execution verified
against the unchanged repository pin. Compact local profiles, bounded jobs and an isolated target
are retained. The standard preflight and fresh results are recorded below before submission.
Only the previously authorized unchanged Unix socket fixture may be filtered locally; full exact
head CI must run it. I293 stays Active / Claimed, Completion Pending; #643/#624 stay open.

Fresh restored-tree observations: standard release preflight passed site/installer/text/classifier,
format, locked workspace check/strict Clippy and both governance validators (zero warnings), then
stopped at the unchanged socket fixture (runtime 64 passed / 1 OS-1 PermissionDenied). The first
filtered workspace run reached the new twelve tests (all passed), then failed one existing TUI
grammar assertion (594 passed / 1 failed). Its exact targeted rerun passed; the fallback has a
500-ms boundary, but the precise original branch is unknown. A following cached workspace run
failed five existing Session waits; single-test-thread diagnosis failed four Session assertions
or waits. None is represented as a full-workspace success.

Source inspection established that these Session fixtures write `/tmp/.talos/runtime` sidecars
using process ID and instance number. Repeated cached sandbox invocations reused PID 16;
read-only SQLite state counts showed prior pending/terminal rows. Only the 66 identified
`runtime_16_*.pending.sqlite*` current-run artifacts (mtime within this validation window) were
reversibly archived. Without changing source, assertions, thresholds or default test concurrency,
the next locked workspace run exited 0: 114 suite summaries, **3588 passed, zero failed/ignored,
one filtered socket test**. This supports stale fixture-state reuse for this fresh sequence;
it does not establish the cause of historical failures. No additional CI filter is permitted.

Final local convergence: network-feature Clippy for the new integration test passed with
`-D warnings`; the external Runtime SDK fixture passed in both default and coding modes.
Both governance validators passed with zero warnings, format and diff-whitespace checks passed,
and changed-document local links resolved. An initial final governance check rejected three
dependency file labels as missing repository paths; explicit external-dependency labels corrected
the prose before the successful rerun. Independent local Agent review found no blockers, including
the pinned dependency source audit. It did not execute tests and is not remote exact-head approval.
The offline stage enters Review on submission; overall C remains Active / Claimed, incomplete.
Remote exact-head full CI, independent technical review and merge-time CAS remain required.

### Rust 1.99 baseline refresh and second checkout recovery (2026-10-10)

The accepted toolchain update #695 and owner closeout #697 advanced the relevant baseline to
`63c3ab658592e51fe19786547d1e6da597bbffc1` (Rust 1.99.0). This stage incorporates that accepted
main, rather than treating the old Rust 1.97 CI as final evidence. The subsequent main commit
`e58e26783e2f48416b871a394b17ac79b642bb1f` adds two unrelated session-handoff documents only.
Those upstream changes are preserved; the SEARCH slice remains the same five test/doc files.

Before the second temporary-checkout loss, actual Rust 1.99.0 execution was observed. Standard
preflight passed check, strict workspace Clippy, format and zero-warning governance/claims, then
stopped only at the unchanged OS-1 socket fixture. The authorized local-filter workspace run
passed 3588 tests, zero failed/ignored, one filtered, across 114 summaries. Target-test strict
Clippy and external SDK default/coding modes passed. Twenty identified prior-run PID-16 sidecars
were reversibly archived within their verified run window; no test or CI policy was changed.
These are historical observations from the lost checkout, not freshly recovered logs or a new
full-preflight success. The unpublished refresh commit/tree cannot be verified from that checkout.

Recovery restores the published #696 head `540f98091f1057cd7a8ee4981c818ce01bcb228b` and merges
current main cleanly. The experiment blob remains `a58ddfa0b698d6f2c9634ec280adbb51568da2be`.
Old-head CI run 38030606949 is Cancelled and cannot authorize merge. Fresh local governance,
fresh exact-head unfiltered CI and independent technical review are required for this refreshed
candidate. C/I293 stays Active / Claimed / incomplete; every full C-V01..C-V11 stays Pending.

Fresh recovered-checkout validation: actual rustc 1.99.0 execution passed. Standard preflight
passed site/installers, text/classifier, format, locked workspace check and strict workspace
Clippy, plus both governance validators with zero warnings, then stopped at the unchanged
runtime socket fixture (64 passed / 1 OS-1 PermissionDenied). The authorized local-filter
workspace command exited 0: 114 summaries, 3588 passed, zero failed/ignored, one filtered;
all twelve new experiment cases passed. Network-feature experiment Clippy (`-D warnings`) and
external Runtime SDK default/coding modes passed. No fixture archive was needed for this fresh
run. Current logs are `/tmp/i293-recovery199-preflight.log`, `/tmp/i293-recovery199-workspace.log`,
`/tmp/i293-recovery199-clippy.log` and `/tmp/i293-recovery199-sdk.log` (temporary, not durable links).
Independent source review also reread restored locked dependency source and found no blockers.
Final exact-head remote review, unfiltered CI and merge-time CAS are still pending.

### Concurrent main integration checkpoint (2026-10-10 UTC)

Head `40d9f66146327cbc3cddb8aae061fc2bad7546f3` CI3111/run38061417205 initially passed five jobs,
including unfiltered macOS preflight, the original socket, twelve new cases, both SDK modes and
zero-warning governance/claims. Windows failed the unchanged Desktop Auto allow-once fixture:
105 passed / 1 failed, with a 15-second watchdog after the decision/report and tool-request stages,
but no tool result. Exact cause is unknown. One bounded same-head Windows rerun passed that
Desktop stage and full workspace tests; the remaining governance/audit/claims/smoke steps then
completed successfully. CI3111 is six-job SUCCESS after that one rerun. The original failure
remains recorded, and this successful old-head result is not refreshed-head merge evidence.
No skip, assertion, timeout or CI budget was changed by SEARCH.

During that rerun, independent #682 and its closeout advanced main to
`45892e7ddf5985aba8fdd5804b364611e1cc7173`. Final CAS found an actual EVOLUTION merge conflict.
The refresh preserves upstream PTY lesson 58 and both SEARCH lessons (renumbered 59/60), including
all three full lesson bodies. Other accepted upstream changes are incorporated unchanged; the
SEARCH diff remains five test/doc files and the experiment blob remains byte-identical. Prior
exact-head review/CI are historical for the refreshed head; fresh validation/review/CI are required.
C/I293 and Epic remain incomplete/open, with no production search change or native admission.

Fresh integrated-main checkpoint: actual Rust 1.99 execution, workspace check and strict Clippy
passed. Standard preflight passed site/installers, governance/claims, text/classifier and format,
then began workspace tests before the execution approval layer rejected the action over a request
to unverified `api.example.com`. The exact intercepted test/request was not established from the
captured partial log; do not call this a completed workspace run or an ordinary test failure.
No retry, alternate network route, added filter or CI workaround was used after that rejection.

Read-only source audit found existing CLI provider-registration fixtures using explicit sample
endpoints and literal test keys, isolated HOME, and model discovery that sends GET /models with
authentication headers and no request body. This identifies a concrete fixture exposure, not proof
of the specific intercepted request or approval to send real credentials/repository data. Resuming
that blocked execution requires a narrow maintainer decision; any broader fixture repair needs
its own scope rather than changing unrelated production code in SEARCH.

Unaffected checks passed: twelve network-feature scheduling tests, target strict Clippy, external
SDK default/coding modes, and final zero-warning governance/claims plus format/diff checks.
The conflict-resolved candidate remains unsubmitted while that authorization is unresolved. The remote
PR head stays `40d9f661`; do not move its ref or use the successful old-head CI to merge this refresh.

### Authorized fixture remedy and recovered local candidate (2026-10-11)

The maintainer authorized an independent, bounded test-only local-mock remedy for the
example-endpoint discovery exposure. Maintenance PR #702 has head
`2faca9af7ac5e4db9a3908bab78eabd3cdc839c8`, based on main
`2f9420f95cd2db3654de463c928debe592a89845`. It changes only the CLI test module:
four discovery fixtures own ephemeral loopback endpoints with deterministic HTTP 503;
original assertions remain, and production, dependency and CI policy are unchanged.
This is authorization for that remedy, not permission to contact the rejected external host.

Fresh maintenance validation on actual Rust 1.99: eight registration tests passed; standard
preflight passed workspace check, strict Clippy and zero-warning governance/claims, then stopped
only at the unchanged OS-1 Unix socket fixture (64 passed / 1 failed). The previously authorized
local-only exact socket filter passed 3579 tests, zero failed/ignored, one filtered, across 113
summaries. External Runtime SDK default and coding modes passed. Independent Agent exact-head
technical review found no blockers; this is not natural-person approval. Full unfiltered CI3123 /
run38116500414 and merge-time CAS remain pending. No fixture-fix merge is claimed here.

Recovery restored GitHub checkpoint `69bbe5377fd8b9182e2b86b7d225869946be3570` and incorporated
accepted main `2f9420f9` locally. The SEARCH diff remains five test/doc files; experiment blob
`a58ddfa0b698d6f2c9634ec280adbb51568da2be` is unchanged. Fresh network-feature experiment tests
passed all twelve cases; target Clippy with `-D warnings` passed. These are focused checks, not
full validation of the refreshed SEARCH candidate. #696 still publishes head `40d9f661`.

Resume only after #702 review/CI/CAS and merge: integrate the actual accepted fixture-fix main,
refresh complete local validation, converge owner evidence, then publish one stable #696 candidate
with fresh exact-head review/unfiltered CI and final CAS. C/I293 remains Active / Claimed /
incomplete; C-V01..C-V11 remain Pending, and #643/#624 stay open. No production search behavior,
native admission or paid selection is changed.

### Accepted fixture repair and refreshed SEARCH validation (2026-10-11)

Maintenance #702 merged as `72c1c0daef3597df43cf192d39bb3a2381059249` after CI3123 /
run38116500414 completed all six jobs successfully. Inspected macOS logs confirm full unfiltered
preflight, all eight registration tests, the original socket test and SDK default/coding; Windows
passed workspace tests, governance/claims, audit and rebuilt CLI smoke. Both platforms' governance
validators reported zero warnings. Independent Agent exact-head review and a separate audit of
accepted #701/current main `69092675` found no fixture-callpath or assertion overlap; GitHub
reported mergeable with no blocking reviews. A conflict-free local prospective merge passed all
eight registration tests before final head/main CAS and squash merge. Shared identity was not
represented as a second natural-person approval; the authorized single-maintainer reason is on #702.

The optional prospective local build first stopped before tests because the remote object was not
loaded locally, then reported StableCrateId/rlib failures in the shared multi-root target. Neither
attempt is passing evidence. Cleaning only generated Talos-package artifacts, without source,
lockfile, assertion or policy changes, preceded the successful eight-test preview. This repeats
the existing local-build-integrity lesson in EVOLUTION; use one checkout root for final validation.

The refreshed local SEARCH candidate incorporates actual accepted main `72c1c0da`, including
independently accepted #701 and #702 changes unchanged. Against that main it still changes only
five SEARCH test/doc files, with experiment blob `a58ddfa0` unchanged. Fresh complete SEARCH
validation and exact-head remote review/CI/CAS are required; prior #696 evidence remains historical.
C/I293 and all full acceptance rows remain incomplete/Pending, and #643/#624 stay open.

Fresh single-root convergence: actual Rust 1.99 standard preflight passed site/installers,
text/classifier, format, locked workspace check and strict workspace Clippy, plus zero-warning
governance/claims. It stopped only at the unchanged OS-1 runtime socket fixture (64 passed / 1
failed). The first authorized-filter workspace run had five existing Session elapsed-time failures;
a prematurely started rerun after the archive guard rejected an unexpected companion-file count
had six existing Session failures. Neither failed run is passing evidence.

Read-only source/state inspection established PID/instance-derived fixture paths, reused PID 14,
and prior pending/terminal rows. File mtimes matched this session's earlier 05:51 UTC validation
and current failure/audit windows. Exactly 21 identified SQLite fixtures and two WAL/SHM companions
were reversibly archived with a size/mtime/SHA256 manifest; no other PID/data was touched. The
subsequent unchanged default-concurrency locked workspace run exited 0: **115 summaries, 3612
passed, zero failed/ignored, one authorized socket filter**. All twelve scheduling experiments
passed. This supports stale fixture-state reuse for this sequence, not a universal historical or
production cause. No assertion, timeout, additional skip or CI policy changed.

Network-feature experiment Clippy with `-D warnings` passed; external Runtime SDK default and
coding modes both passed, and format/diff checks passed. Fresh exact-head remote review and
full unfiltered CI are still required before merge; no full C acceptance follows from these checks.

### Merged scheduling stage and next result-admission experiment (2026-10-11)

#696 merged at `444a8137f054b33802f26bffdb55fb0f83a7a861`, reviewed head
`f26cc53ae1e3c2b73d292da832dc6d5089095f96`, after all six exact-head CI3129 /
run38118573173 jobs succeeded without reruns. Complete macOS and Windows network-feature
workspace logs each show twelve experiment tests passing. macOS also passed the original socket
test, both Runtime SDK fixture builds and release preflight; Windows passed workspace, governance,
claims, audit and CLI smoke. Both governance validators reported zero warnings. Independent Agent
technical review and the single-maintainer reason are in #696 comment `6106314437`; final CAS is
`6106425480`. Shared identity is not a second natural-person review. The accepted merge preserves
unrelated #703 documentation; tree `36227f9c2721ba17fa41198a18dfee4288a205c8` matched the preview.
This supersedes earlier pending-stage instructions, not the preserved historical failures.

The next bounded stage starts from that merge under the same effective #664 claim. Runnable
deliverable: `crates/talos-tools/tests/i293_result_admission_experiment.rs`, an eight-case disposable
offline experiment. It supplies partial C-V02 and validation-boundary C-V05 evidence, not full
acceptance. Already-normalized synthetic hits use the locked reqwest URL parser; no adapter,
HTTP, DNS, key/config, permission, router, dependency, public API or CI behavior changes.

The experiment distinguishes empty/invalid-only responses, filters before truncation, preserves
backend identity/order/display fields, and checks fixture input-count, URL-byte and output-count
bounds. An injected monotonic clock exercises pre/post-parse budget checks, including expiry on
the final parse and between entries without returning a partial winner. It does not measure parse
duration or interrupt synchronous parsing, bound transport allocation or title/snippet bytes,
compose with the scheduling model, propagate caller cancellation, or qualify backend eligibility.
Limits (8 input hits / 256 URL bytes / 1..3 output hits) and terminal ordering are experimental.

The WHATWG parser's repair behavior is explicitly characterized: parse success can accept strings
such as `https:example.invalid/path`, extra slashes or embedded newline, and loopback result URLs.
Preserving their original display text is not strict raw-syntax validation or permission to fetch.
Production must separately review strictness/canonicalization and endpoint/redirect/SSRF controls;
this stage does not change those policies or promote the proposed predicate to accepted policy.

Affected documentation: router integration plan and C Story. Infrastructure-only/test-only stage;
no user-visible delivery. C/I293 remains Active / Claimed / incomplete, all full C-V01..C-V11
Pending, #643/#624 open; D remains independent, E/F gated.

Fresh local convergence on pinned Rust 1.99: eight focused network-feature tests passed without
skips; target Clippy with `-D warnings`, format and whitespace checks passed. Standard preflight
passed site/installers, governance/claims (zero warnings), text/classifier checks, workspace check
and strict Clippy, then stopped only at the unchanged OS-1 socket fixture (64 passed / 1 failed).
It is not reported as a full preflight pass. Under the existing exact local-only deferral, locked
workspace tests passed across 116 summaries: 3620 passed, zero failed/ignored, one socket filter;
all eight new experiments passed in that run. Independent Runtime SDK default and coding builds
both passed. No fixture archive, extra skip, assertion/timeout or CI-policy change was needed.

Independent read-only Agent preview found no blockers, but is not final exact-head approval or
natural-person review. Before merge obtain fresh exact-head technical review, full unfiltered CI,
authorized single-maintainer reason and final current-main/claim/overlap/review CAS. Source and
validation limitations above remain effective even after this partial test stage merges.

### Caller-boundary characterization stage (2026-10-12)

#706 merged at `4b03149ca452bbaf4e96b580ab74135bc4ac670d`, reviewed head
`02c8d9780cfd7fd82e786e5162b555d4b0077a2d`, after CI3136 / run38134968930 completed all
six jobs successfully without reruns. Full macOS and Windows logs each execute eight admission
experiments; macOS original socket, both SDK modes and preflight passed, as did Windows workspace,
governance/claims (zero warnings), audit and CLI smoke. Independent exact-head review, current-main
compatibility review, single-maintainer reason and final CAS are on #706 comments `6108458526`,
`6108514280`, `6108453822`, `6108605266`. These supersede the preceding pending-stage instructions;
they do not complete C. Unrelated #704 governance was preserved at merge.

Workspace maintenance removed the prior local checkout and Cargo cache. This session re-cloned
authoritative main at the #706 merge and reinstalled pinned Rust 1.99.0; actual rustc execution
was verified. Old logs are historical evidence, not freshly recovered validation. No dependency,
toolchain pin or repository build profile is changed.

Runnable deliverable: `crates/talos-agent/tests/i293_caller_cancellation_characterization.rs`,
three test-only cases through real Agent permission dispatch and AppServerSession interruption.
An independently authored Network tool stand-in owns a counted waiting future and performs no
network I/O. A bounded, buffered mock model emits native tool calls without spawning producer
tasks; each test owns a unique temporary workspace. This stays under effective #664 and the
current architecture/evidence/test-only execution boundary, with no production/API/permission
policy, config, provider, transport or CI changes. Open #705 remains separately owned diagnosis.

Focused assertions (three passed): Network Ask without a resolver denies before tool start;
an admitted tool can outlive the permission approval deadline, and cancelling the separately
created Agent token does not stop that active call; Session Interrupt drops the admitted tool's
owned future before the matching Turn's Cancelled event, without a returned tool result. Paused
Tokio time exercises the approval/execution distinction, not a new search execution budget.

Initial local compilation rejected direct equality on TurnCompletionStatus (no PartialEq); variant
matching corrected the test. The first execution passed one rejection assertion but failed both
start waits. Diagnostic observations established that the stand-in lacked a safe typed resource:
the rejection came from permission-state validation, not unavailable approval. That initial
passing rejection is not Network-Ask evidence. The stand-in now explicitly declares a synthetic
Domain resource, verifies approval-related denial rather than resource-state failure, and records
the resolver invocation before cancellation. All three focused tests then passed. No production
permission policy or existing timeout/assertion was changed. Actor shutdown/join followed by an
event drain additionally rejects delayed tool results.

This fixture's explicit Domain resource is not evidence that WebSearchTool has a suitable grant
profile: its own implementation inherits the default Network facet with no typed resource, and
its query schema supplies no URL for generic resource extraction. Actual registration/wrapper and
approval behavior needs a separate audit/characterization before C-V10; do not infer a production
failure or repair from the stand-in. Coordinate any policy repair with the permission owner.

This supplies caller-carrier characterization toward C-V06/C-V07/C-V10 only. It does not invoke
WebSearchTool or SearchBackend, propagate an enclosing search deadline/token, stop HTTP/DNS or
dependency-spawned work, validate permission wrappers outside Agent, or prove binary/rollback
acceptance. No returned SearchBackendError::Cancelled is inferred from future destruction.
All full C-V01..C-V11 remain Pending; C/I293 Active / Claimed / incomplete, #643/#624 open,
D independent and E/F gated. Documentation target: router integration plan and C Story.
Infrastructure-only exception: no user-visible delivery. Local validation and fresh exact-head
review/full unfiltered CI/final CAS are required before submission/merge respectively.

Fresh Rust 1.99 local convergence: all three focused cases passed with no skips, target strict
Clippy passed. Standard preflight passed site/installers, governance/claims (zero warnings),
text/classifier, format, locked workspace check and strict Clippy; it stopped only at the
unchanged OS-1 socket fixture (64 passed / 1 failed), not a full preflight pass. Existing exact
local-only socket deferral: locked workspace 117 summaries, 3623 passed, zero failed/ignored,
one filtered; all three new cases passed. Independent Runtime SDK default and coding builds
both passed. No fixture archive, extra skip, dependency/lockfile, assertion/timeout or CI-policy
change was needed. The early fixture failures above remain distinct from these successful runs.
Updated independent Agent preview found no blockers; it acknowledged missing the original typed
resource prerequisite and verified the corrected cause assertions and post-join drain. This is
not final exact-head approval or natural-person review. Fresh exact-head CI/review and CAS remain
merge gates; every full C acceptance row is still Pending.

### Registered-search permission characterization stage (2026-10-12)

#707 caller stage merged at `7e817ab59364bf282c49941ec501f82b0c6c40fb`, reviewed head
`03e9aec27d68ca442de39436c5080afcb3dbf690`, after CI3141 / run38191125400 passed all six
jobs without reruns. Complete macOS and Windows logs execute all three caller cases; macOS
original socket, both SDK modes and full preflight passed, Windows workspace/audit/CLI smoke
passed, governance/claims reported zero warnings. Exact-head independent Agent review and final
CAS are recorded in #707 comments `6116027416`, `6116181798`; merge synchronization in
`6116188215`, #643 `6116188388`, #624 `6116188616`. Single-maintainer reason and exact-tree
publication verification are in `6116021786`. These supersede prior pending merge instructions,
without completing any full C acceptance row.

This stage starts from that merge under effective #664. Selection disposition: I293 continues
its existing bounded slice; I294 remains independent; I300 owns Git repair, PERM-007-H is
Refinement / Unclaimed; I288/I292 retain Complete / Claimed records and are not reactivated;
I296-I299 retain their recorded completion states; I277 deferred device
acceptance and DEPENDENCY-003-A retain their own scopes. Open #705 is separate Windows timeout
diagnosis and is not edited. Board-wide historical row drift is a separately owned governance
residual, not changed in this implementation batch.

Runnable deliverable: `crates/talos-tools/tests/i293_registered_search_permission.rs`, three
synchronous tests using the actual exported network contributions and ToolRegistry. They validate
the public query input, unchanged untyped Network facet, default Ask proposal compilation failure
in interactive and headless diagnostic contexts, and a narrowly configured policy Allow admission
control. Constructors may read ambient configuration; no execute/search/adapter, HTTP or DNS call
is made, and no credential value is printed. No policy grant is committed by the fixture.

Default PermissionEngine Ask evaluates successfully but begin_invocation and both Once/Session
proposal compilation return exactly GrantError::MissingResource(0), with zero grants. Explicit
fixture policy Allow instead admits the unchanged profile without an approval grant. This is a
resource compiler boundary observation, not universal failure across runtime configurations or
real binary dispatch proof. Allow is a characterization control, not a recommended workaround.

The router integration plan pins the actual registration and Agent pipeline installation audit.
The CLI PermissionAwareTool and SDK RuntimePermissionAwareTool are cfg(test), not production
wrappers that can repair WebSearchTool metadata. Runtime permission configuration may differ
from PermissionEngine defaults; the test does not claim those hosts all default to Ask.
Any resource/profile/enforcement repair needs permission-owner coordination and separate
production authorization; PERM-007-H is not activated here and I300's Git slice is not broadened.

Infrastructure-only/test-only exception: affected documentation is the router plan and C Story,
with no user-visible delivery. No production/API, dependency/lockfile, config, provider-admission,
CI or permission-policy change. C/I293 remains Active / Claimed / incomplete, all full C-V01..11
Pending, #643/#624 open; D remains incomplete and E/F gated. Fresh validation, exact-head
independent Agent review, unfiltered full CI and merge-time CAS remain required.


Fresh pinned Rust 1.99 local convergence: three focused network-feature cases passed without
skips; target Clippy with -D warnings passed. Standard preflight passed site/installers,
governance/claims (zero warnings), text/classifier, format, locked workspace check and strict
Clippy; it stopped only at the unchanged OS-1 socket fixture (64 passed / 1 failed), not a full
preflight pass. Under the existing exact local-only socket deferral, locked workspace tests
passed across 118 summaries: 3626 passed, zero failed/ignored, one filtered; all three new cases
executed successfully. External Runtime SDK default and coding builds both passed. No extra skip,
fixture archive, old assertion/timeout, dependency/lockfile or CI change was needed.
Independent read-only Agent preview found no blockers; it is not final exact-head merge approval
or natural-person review. Fresh unfiltered CI, exact-head review and CAS remain merge gates.
Final governance rerun initially omitted the local Cargo PATH and failed its SQLite validator
with cargo not found. Restoring the same verified toolchain environment produced zero-warning
governance/claim passes; the invocation error is not reported as a source failure or a pass.

Merge-time evidence review found the new selection sentence "remain closed" ambiguous: I288
and I292 retain Complete / Claimed records, and GitHub #625 is still open. The sentence was
corrected without closing that Issue or changing any claim. Independent review withdrew its
initial APPROVE pending this correction. Candidate head 102338cd / CI3144 is superseded, not
final merge evidence; corrected exact-head review and full CI must pass. Test/production code
is unchanged from the locally verified tree; fresh docs governance/claims and diff checks apply.

### Endpoint-authority architecture packet (2026-10-12)

#708 merged at `f036111f6e969feea47ad784f3cb3effd7e20a33`, corrected reviewed head
`ad70b76e7696d19d73b0d3c5588b19219430601d`, after all six CI3146/run38195338735 jobs
succeeded. Complete macOS and Windows logs each execute the three registered-search cases with
zero failed/ignored/filtered; macOS original socket, both SDK modes and full preflight pass.
Windows workspace/audit/CLI smoke pass, both platforms governance/claims report zero warnings.
Corrected independent Agent approval is #708 comment6116601192, final CAS6116824003;
merge synchronization comments6116831474/6116831628/6116831778. These supersede prior pending
merge instructions; the initial withdrawn review and candidate/validator failures remain history.
No full C acceptance row is completed by #708.

This stage starts from that merge under unchanged effective #664, after refreshing open PRs and
permission owners. I293 continues architecture/evidence only; I294 remains independent/incomplete,
I300 retains its Git slice, PERM-007-H remains Refinement/Unclaimed. Completed owners retain their
recorded completion/claim states without reactivation, I277 deferred device acceptance and I249
Planned retain their scopes; I164 remains Paused/superseded, not reactivated. #705 Windows timeout
diagnosis is separate and untouched. No new
selection, activation, claim, production behavior or permission repair occurs.

Deliverable: [endpoint-authority plan](../reference/SEARCH-001-C-ENDPOINT-AUTHORITY-PLAN.md),
linked from the router plan and C Story. Fresh source audit finds that Domain/Remote proposal
compilation is distinct from execution authority: current execution_authorizations emits Path
capabilities only, and default execute_authorized delegates to execute. A Domain facet alone
cannot establish outbound endpoint enforcement. This is source evidence, not a new transport test
or exploit claim. The packet proposes an owned invocation-bound destination plan, a carrier/API
decision gate, compound compatibility coverage and ten planned acceptance rows. It chooses no
public API and does not generalize the browser prepared route.

Infrastructure-only architecture/evidence exception: no user-visible delivery. All modified files
are Markdown; no Rust test or production source, Cargo/lockfile, config, permission, provider or CI
change. Repository documentation-only validation routing applies; previous full #708 Rust/SDK
results are historical, not newly run validation. Workspace maintenance removed checkout/cache;
the repository was re-cloned and pinned Rust 1.99 restored for current documentation validators.
Fresh local reduced checks, independent exact-head architecture/permission-evidence review,
trusted-base CI routing and final CAS remain submission/merge gates. C/I293 Active/Claimed and
incomplete, all full C-V01..11 and all EP rows Pending; #643/#624 open, D incomplete/E-F gated.

Fresh documentation-only convergence on actual Rust 1.99 execution: public-site and installer
checks pass, governance/claims report zero warnings, classifier fixture matrix passes 14 cases,
format and whitespace checks pass. The SQLite consumer validator used fresh locked Cargo metadata
and passed its 17-case fixture matrix; delivery workflow validation passed 12 cases. No full Rust
workspace test/preflight, Clippy or SDK build is claimed for this Markdown-only stage. Independent
read-only Agent preview found no blockers and confirmed source evidence/proposed API boundaries;
it is not exact-head merge approval or natural-person review. Repository trusted-base routing,
exact-head applicable review/CI and final CAS remain required before merge.

### Authorized Production Scope Addition — 2026-10-13

#709 merged as `bac4cce938da10d051fbafc562e2cba19e9557bd`, reviewed head
`ec4a6112e992fb4196c7e43976c5caa8bd074387`, after independent architecture/permission-evidence
APPROVE6127945569, CI3154/run38274655324 and final CAS6128010518. Four documentation-route jobs
passed; Windows Rust/Linux Desktop were policy-selected skips, not Rust test passes. Complete logs
show governance/claims0 warnings, SQL17, delivery12 and classifier14 checks. Merge synchronization
is in comments6128036145/6128036514/6128036835. These supersede pending-stage merge instructions.
All full C/EP acceptance remains Pending; no production API/behavior was delivered by that packet.

The maintainer explicitly accepted the expanded bounded scope on 2026-10-13, after reviewing its
description: invocation-bound endpoint authority and actual search wiring under ADR/claim,
independent safety/API review and CI gates. This supersedes the session's architecture/test-only
execution restriction for the authorized production phase once its scope amendment is effective;
it does not pre-approve an unknown API or waive any gate. No further generic confirmation of the
same scope is required. Concrete contract acceptance is through the reviewed ADR, not this note.

Change-control classification: explicitly accepted scope addition required for I293's published
SearchRouter/permission integration objective, not a separately selected general network or Auto
feature. Preserve published scope/acceptance and historical restriction paragraphs above. Amend
the existing single C claim, rather than creating duplicate C ownership or a new Issue for a local
prerequisite. Proposed scope covers search-only core/permission/Agent prepared invocation and live
authority, controlled compatibility/optional-adapter transport, associated CLI/SDK/MCP disposition,
tests and migration docs. PERM-007-H's all-built-in Auto policy, I300 Git, D/E/F provider admission,
NET-001 generic retry/breaker, new crate/dependency, unsafe/native exceptions and release stay out.

Scope amendment PR: #712. The target #664 claim and its previous Work Slice remain
effective until this finalized amendment merges. This branch contains governance/decision docs only;
no production implementation, implementation test or implementation branch exists. ADR-090's
Accepted contract status is an ineffective draft until #712 passes independent exact-head
permission/API/security review, CI and merges.
Implementation
starts from the amended-claim/accepted-contract merge or later main, never this draft.

Selection source: main `9cad344b8f7adef855ca6b858e6ccc7723103767` (I301 #711 repair merged).
Fresh inventory retains I293/I294 Active search, I300 Active Git, I301 Review/Claimed (no closure
inferred from merge), I277 Review/deferred device acceptance, I249 Planned and I164 Paused.
PERM-007-H remains Refinement/Unclaimed; open #705 owns separate Windows latency diagnostics.
Other completed owners retain their exact recorded completion/claim states. No other owner is
reactivated, taken over or closed. Target claim/overlap and ADR identifier must be rechecked at CAS.

The concrete draft is [ADR-090](../decisions/090-invocation-bound-search-endpoint-authority.md).
An independent read-only design preview identified complete Session grant binding, Deny projection,
public-struct-literal compatibility, live state authority and transport ownership as hard gates;
it is not exact-head approval. The pinned rust-websearch archive was freshly downloaded, SHA256
verified (`06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee`) and inspected:
no injected client, private HTML parsers, internal limited(5) redirects. The candidate rehosts only
the existing compound protocol under guarded clients inside talos-tools, keeps the same dependency
and reusable network-free helpers/types with MIT provenance, and counts no native E/F admission.
Protocol internalization is explicitly within this proposed scope and requires bounded D
coordination for terms/privacy/parser and migration differences before activation. The pinned
upstream root MIT license/notice was retrieved; preserve the complete notice and per-file source
and modification records before copying. Protected invocation failures cannot automatically
fall back to opaque/legacy execution; rollback is a controlled version/deployment revert or a
separately accepted migration. Preparation errors use a closed, content-free error contract.
No live search, key usage or transport proof is claimed by this source audit.

Infrastructure-only governance deliverable: reviewed scope/ADR and owner/index synchronization.
Production acceptance remains real Agent/CLI/SDK/transport evidence for EP-01..10 and C-V01..11,
locked full checks, source-compatible external implementer/wrapper fixtures, independent protected
review, compatibility rollback and final CAS. No default switch before controlled compatibility
and connection/redirect parity gates pass. C/#643/#624 stay incomplete/open, D incomplete/E-F gated.

### Endpoint Carrier Implementation Stage — 2026-10-13

#712 merged 989fec2e00124e86667a3519f82527e2e75074ad after independent exact-head
Agent protected security/API APPROVE6136536739, CI3165/run38335038737 and CAS6136603322.
Four documentation jobs passed; two Rust/platform jobs were policy-selected skips.
Synchronization6136612802/6136613235/6136613625 records effective ADR-090/amended scope.
Earlier draft restrictions above are historical; implementation starts from this merge.

This carrier-stage candidate adds opt-in core prepared types, complete manifest binding through
proposal/preview/Session match/admission, live permission-state authority, owned Agent dispatch
and complete CLI/TUI preview. SDK legacy defaults deny new network scope; real SDK actor fixtures
exercise scoped approval and legacy rejection. Public request literals/Path APIs remain compatible.
No WebSearchTool, transport, selection, dependency or lockfile changes. This is partial
infrastructure, not binary WebSearch activation, full C/EP acceptance or HTTP cancellation proof.

Independent local preview identified and corrected host-only TUI preview and an overbroad test
name: shared full formatter/channel fixture and real final-hook Skip/revocation, ledger replay
and caller-drop fixtures now exist. Preview is not exact-head PR approval. Source audit corrected
ADR-090's first-match description to existing matching Deny precedence/non-Deny first-match;
the evaluator is unchanged.

See [carrier migration/API inventory](../reference/SEARCH-001-C-NETWORK-CARRIER-MIGRATION.md)
for storage bounds, callback/wrapper/root dispositions and activation gates. Eight permission
lifecycle fixtures, seven Agent dispatch fixtures, core bounds, TUI channel and SDK callback/actor
fixtures are available. Final locked validation, protected review and CI remain publication/merge
gates. Carrier stage is Review; Implementation PR not assigned before first stable push. I293/C remain Active /
Claimed / incomplete; #643/#624 open, D incomplete/E/F gated. No Completion Commit.

Local stable checkpoint (Rust1.99.0 / cargo1.99.0, locked): workspace check and Clippy passed;
affected core/permission/Agent/CLI/runtime all-target Clippy passed with -D warnings. Workspace
tests passed3657 across119 summaries,0 failures; only the already maintainer-authorized exact
tests::runtime_evidence_rejects_directory_symlinks_and_socket_entries was filtered locally.
Code CI must run the unfiltered workspace/platform gate; no new skip or ignore was introduced.
The independently locked single-direct-Talos consumer passed both minimal and coding runs,
including the new prepared network implementation/scoped approval and existing API surfaces.
Formatting (workspace/external fixture), whitespace, text boundary23, classifier14, governance
(SQL17/delivery12), Collaboration Claim, public site and installers passed. Initial Clippy's
needless borrow and private preview enum-size findings were corrected locally; the preview stores
boxed scope rather than changing an unrelated event enum. Existing models.toml compression
build-script information remains; no new Rust lint allowance or dependency changes.

No whole release_preflight.sh success is claimed: equivalent required local steps use the
pre-authorized exact socket deferral above. Remote exact-head CI/security/API review and final
CAS remain mandatory before merge. Transport/default activation and full C/EP rows remain Pending.

Late actual-URL canonicalization audit: #713's first head8b5e4146/treea1fa475e was independently
approved in comment6138564512, but the new mapped-IPv6 regression failed (actual parsed
::ffff:7f00:1 versus std dotted display). The first head is not the corrected merge candidate;
comment6138722441 records the hold. Core now validates mapped hosts using HTTP URL hexadecimal
serialization, with a real parsed-URL -> Domain proposal -> admission/check regression.
No Deny/facet relaxation, dependency or transport change. Fresh stable validation and exact-head
security/API review/CI are required; the earlier3657 checkpoint remains historical.

The correction refreshed locked workspace check/Clippy, affected all-target Clippy,3658 tests
across119 summaries (0fail/only the same authorized socket filter), both independently locked
SDK runs and formatting/whitespace. Eight permission fixtures now pass, including the formerly
failing actual-URL case. Initial CI3167/run38348784946 completed successfully on8b5e4146, but is
historical evidence only and cannot approve the corrected head. Fresh governance/claims, review
and CI bind the corrected candidate. Recorded lesson61 promotes the real boundary regression.

### Merged Carrier And Controlled Direct Transport Stage — 2026-10-13

#713 merged at `69e1e1166943e7e9b39acab04ade9e362cccb5d3`, exact head
`fc9286b430bebfef77891d95d9b1c992d6249faa`, tree `39d1809d664ce7685821ee692b6f0dccae5a10f6`.
Protected independent Agent APPROVE6138838842, full CI3168/run38350762077 (six jobs) and
CAS6139059609 passed. The original Unix socket fixture ran unfiltered in macOS CI. Issue
synchronization6139067980/6139068201 retains C/Epic open; receipt6139068432 records the merge.
This source update records existing carrier evidence, not a completion commit for C/EP.

The next library-only infrastructure stage starts from that merge, within effective #712 scope.
It adds a private `search_backend::transport` under the existing staged seam, without adding a
lint allowance, dependency, lockfile, exported API or WebSearch/default wiring. There is no new
provider, copied upstream protocol/parser, paid request, external search or regional observation.
Runnable deliverable: actual reqwest -> owned loopback server fixtures and real Once permission
admission -> transport. See [controlled transport contract/evidence](../reference/SEARCH-001-C-CONTROLLED-TRANSPORT.md).

The direct client derives exact endpoints from the actual URL/method; checks authority before
DNS and after DNS before logical send; rejects mixed/over-limit/nonpublic answer sets unless
each nonpublic IP has an exact approved exception; pins all accepted addresses; disables ambient
proxies, redirects, retries, Referer and decoding; creates a new HTTP/1 client per hop. Fixed
request headers cannot be overridden. Explicit proxies return UnsupportedProxy with no direct
fallback. Scoped GET/HEAD redirects are bounded; POST supports only same-origin307/308 and
does not change method or forward its body to another origin. HTTPS downgrade is rejected.
One shared execution deadline/request/body budget covers candidates and hops; explicit deadline
fences supplement timeout_at. Storage, header acceptance and query/body limits are documented.

Independent local source preview identified a deadline blocker: Tokio Timeout polls a Ready
inner future before its timer. No failing pre-fix test run is claimed; corrected explicit fences
and paused-clock expired/late-ready DNS fixtures cover the finding before publication. Negative
socket counts now use terminal worker join/drained accept queues, not an instantaneous counter.
This is a design/implementation preview, not remote exact-head approval.

Focused corrected checkpoint:26 transport fixtures pass (plus the separate ambient-proxy child
fixture invocation). Actual HTTP receipts cover Host identity, System DNS, exact IP exceptions,
denied destination/redirect zero connections, real Once admission/revocation, concurrent shared
budgets, body/header/encoding bounds, deadline/caller abort peer EOF and no hidden HTTP retry.
Local TCP sockets are available; the unchanged Unix socket fixture remains environment-blocked. No new fixture is skipped. Full standard preflight and fresh
exact-head independent security review/CI/CAS remain candidate gates; results follow below.

Infrastructure stage is Review; I293 remains Active/Claimed/incomplete, no C Completion Commit.
HTTPS certificate/SNI connection fixtures, explicit proxy and ambient-config migration, upstream
form/request/parser/probe/retry/source parity, bounded D terms/privacy/MIT review and actual
WebSearch/Agent/CLI/SDK/MCP integration remain default activation gates. Do not interpret direct
local HTTP sockets as TLS, arbitrary-network SSRF, OS sandbox, resolver-task cancellation or
regional reachability proof. D remains independently incomplete and E/F gated; #643/#624 open.

Final local checkpoint (pinned Rust/cargo1.99.0, locked): workspace check/Clippy and network
all-target Clippy pass; formatting, whitespace, governance(SQL17/delivery12), claims0warnings,
text boundary23, classifier14, site/installers and both independent SDK modes pass. Workspace
tests exit0 with3684 passes across119 primary suite summaries, no failures/ignores, and exactly
one primary filter: the already authorized unchanged Unix socket fixture. The isolated ambient
proxy child adds one pass/one summary; its --exact filters436 sibling tests only in that child,
while the parent tool suite runs all437. Raw output therefore has120 summaries/3685 passes,
which is not3685 distinct workspace tests or437 waived tests. All26 new fixtures run locally.

The initial standard unfiltered release_preflight.sh did run and failed at the original Runtime
socket fixture (66pass/1fail, PermissionDenied/Operation not permitted). TCP availability does
not imply Unix socket availability. Preserve that failure; no whole local preflight pass is
claimed. The equivalent required steps above use only the maintainer's prior exact local filter;
full remote CI must run unfiltered. No production/test waiver or CI change was introduced.
Source-based deadline finding, late-ready DNS completion observation and bounded failed-DNS
hop reservations are reconciled locally before first submission. Independent protected exact-
head review, CI and final CAS remain the remote gates for this stable stage.


## Direct Transport Merge And TLS/CONNECT Subset — 2026-10-13

#714 merged `d78dc75c4b52fe7a2f708b752090c17e1f5f7bca` after protected independent Agent
review6140200974, full CI3170/run38360092833 (six successful jobs) and CAS6140498436.
Issue receipts6140512149 (#643),6140512432 (#624) and6140512783 (#714) retain partial status.

The next library-only stage is Review under the same effective claim, based on that merge.
[Transport contract](../reference/SEARCH-001-C-CONTROLLED-TRANSPORT.md) records verified direct
TLS/SNI and limited explicit CONNECT: unauthenticated HTTP proxy, HTTPS IPv4-literal targets,
proxy+target authority, pinned proxy DNS, two reservations per compound hop, no direct fallback.
Hostname/HTTP/IPv6 targets and HTTPS proxies fail closed. Proxy `/` is a configuration locator;
CONNECT uses literal IP:port. Compound-start revocation cannot undo already-started work, and
trusted-proxy remote routing is not proven. No default WebSearch/API/dependency/lock change.

Thirteen new fixtures plus26 prior fixtures pass in the focused network run. Test-only synthetic
CA verification remains enabled. Python stdlib is an explicit temporary fixture host utility;
missing utility fails, no new skip. Initial fixture33pass/3fail and corrected callback/None-SNI
receipt handling are preserved in the contract and EVOLUTION. Independent implementation
preview found no new blocker, but is not exact-head remote approval. Full C/EP remain Pending,
I293 Active/Claimed, D independently incomplete, E/F gated, #643/#624 open. Broader proxy and
ambient migration, compatibility form/parser/probe/retry/source parity, bounded D review and
actual official root/router acceptance remain activation gates.


Fresh final local checkpoint: pinned1.99.0 locked workspace check/Clippy, network all-target
Clippy, fmt/diff, Python AST, governance(SQL17/delivery12), claims0warnings, text boundary23,
classifier14, site/installers and both independent SDK modes pass. Fresh rebuilt workspace
suite exits0:3697 primary passes/119 summaries, zero failures/ignores and exactly the prior
Unix-socket filter. Its isolated ambient-proxy child adds1pass/1summary and449 sibling filters;
parent tools run450 including all13 new TLS cases. Raw3698/120/450 filters must not be mistaken
for3698 distinct tests or450 waived tests. No new skip or unfiltered local preflight claim.
Independent final local protected preview found no blocker; remote exact-head review/CI/CAS
remain required. Resume from feature/I293-search-tls-proxy; publish this stable stage, verify its
remote tree equals the local tree, obtain independent protected review and full unfiltered CI,
then repeat claim/base/head/tree/non-overlap/Issue CAS. Keep #643/#624 open and record actual PR.

## Cross-Platform TLS Fixture Correction — 2026-10-14

#715 original head `6c3bdb0f5bd6dc859c11c3055fdead753dd52140`, treebd8eb29e, received
independent protected Agent review6141466921. Full CI3172/run38368995942 was NOT successful:
five jobs passed, but macOS release preflight failed445pass/5fail in tools; all13 added TLS
cases passed on Windows. The five positive TLS failures were closed Request errors, not
deadline evidence or proof of a single underlying cause. Original CI remains failed; no rerun,
test skip, verification bypass, natural-person approval or merge is claimed.

Read-only certificate inspection found positive leaf2020..2045, contrary to Apple's documented
825-day maximum for TLS leaves issued after2019-07-01 (https://support.apple.com/en-sg/103769).
The public bundle is regenerated together: root2020..2045; positive/host-only leaves365days
(2026-10-01..2027-10-01); expired2020..2021. SAN/EKU/verification negatives remain. CA key exists
only in offline generator memory and is discarded; public leaf key remains unchanged. SKI/AKI
make issuer linkage explicit. The offline cryptography generator/checker verifies signatures,
CA constraints, P-256/SHA-256, key correspondence, SAN/EKU/date/lifetime; it is not a CI/runtime
prerequisite. Renew/review before2027-10-01. No production Rust/API/dependency/trust change.

This eliminates a confirmed fixture incompatibility, not yet proof of complete macOS recovery.
Independent correction design preview found no new blocker. Fresh corrected-head review and
full unfiltered macOS/Windows CI remain mandatory. Checkout/toolchain/cache were removed by
workspace maintenance; checkout restored from submitted head and pinned1.99.0 reinstalled.
Do not reuse old local outputs as new-head evidence. Validation/resume receipt follows.

Corrected local checkpoint (pinned1.99.0, locked): fresh focused39pass and rebuilt workspace
3697 primary passes/119 summaries +isolated ambient child1 (raw3698/120), zero failures/ignores.
All13 TLS cases execute; only the previously authorized unchanged Unix-socket primary filter.
The child filters449 siblings, while parent tools run450. No unfiltered local preflight claim.
Workspace check/Clippy, network all-target Clippy, fmt/diff, both independent SDK modes, AST,
governance(SQL17/delivery12), claims0warnings, text boundary23, classifier14 and site/installers
pass. An incomplete redirected tail was not used as SDK/governance evidence: direct independent
rerun receipts verify both SDK modes and the remaining validators. No host-log root cause inferred.
Offline metadata/signature/key/SAN/EKU/CA checks and OpenSSL positive hostname/IP verification
pass; host-only IP and expired verification fail as intended. Checker rejects Python -O rather
than silently suppress assertions. Independent local correction preview found no blocker.

Resume on #715 / feature/I293-search-tls-proxy: push the coherent corrected checkpoint, verify
remote/local tree equality, obtain fresh independent protected exact-head review and all six
unfiltered CI jobs, then repeat main/head/tree/claim/non-overlap/Issue CAS. Preserve failedCI3172
and do not use old-head approval as new-head evidence. C/EP Pending, D independently incomplete,
E/F gated, #643/#624 open; broader proxy/ambient/protocol/D/official-root gates remain.

## TLS/CONNECT Merge And Bounded Form Transport — 2026-10-14

#715 merged `d66e380ad144329dbea49634122505a5a3075e95` after corrected exact-head protected
Agent review6158318325, full six-job CI3173/run38491794838 and CAS6158600236. Complete logs
show all13 TLS fixtures passing on macOS and Windows; macOS unfiltered preflight includes the
original Unix-socket case and both SDK modes. FailureCI3172 remains historical; the corrected
head recovered on these runners without verification bypass or skips, not a lifetime-only causal
experiment or proof of universal portability. Partial receipts: #6436158606822/#6246158607177/
#7156158607444. C/EP Pending, D independently incomplete, E/F gated; #643/#624 stay open.

Next stage on feature/I293-search-form-protocol starts from that merge under the unchanged
effective #712 claim. Private execute_form owns ordered fields, predicts exact UTF-8 form byte
expansion using checked arithmetic before encoded allocation, then uses the existing URL
serializer.16KiB encoded cap implies <=8192 fields (2N-1 minimum); individual raw fields also
receive an early16KiB rejection. JSON execute signature/behavior stays intact, and internal fixed
Json/Form selection never exposes caller headers. POST307/308 retains encoded bytes/type only
same-origin; other POST/cross-origin redirects fail. Existing DNS/pinning/proxy/TLS/authority/
shared request/deadline/body budgets remain the only send path. No new dependency/lock/API,
parser copy, actual upstream route or default WebSearch activation.

Eight new fixtures cover golden Unicode/reserved/control/duplicate/empty/order encoding,
bounded generated round-trip properties, real wire16KiB exact body/type, expansion/field-count
overflow before DNS/reservation, same-origin redirect semantics, scoped cross-origin refusal,
denied method/expired entry fence, and actual CONNECT verified-TLS form/CONNECT isolation.
The crate has no direct proptest dev dependency: bounded generated properties reuse the
standard decoder without adding a Git/test dependency; do not claim proptest execution.
The Rust-native existing URL serializer replaces the need for a host encoder/manual escape
implementation. Initial compile-only fixture Value/string accessor error was corrected locally;
corrected focused47/47 and network all-target Clippy pass. No failing runtime or remote candidate
is claimed for that compile error. Independent design preview found no blocker; full local
checkpoint and protected implementation review/CI/CAS remain before publication/merge.

This is a transport prerequisite, not full upstream form/request/parser/probe/retry/source parity.
Form input ownership here does not prove real prepared invocation/request fingerprint binding or
official root/router integration; those remain full C/EP gates. Broader proxy/ambient migration,
bounded D terms/privacy/MIT and official-root/rollback acceptance remain activation gates.

Final local form checkpoint: independent protected implementation preview found no blocker.
Its bounded-property limitation was addressed with actual wire UTF-8 escape-heavy exact16KiB
and one-byte-overflow negatives, alongside the existing ASCII boundary; no new dependency.
Fresh final focused47/47, network all-target Clippy and full workspace tests pass. Workspace
3705 primary passes/119 summaries +ambient child1 (raw3706/120), zero failures/ignores; exactly
the previously authorized unchanged Unix-socket primary filter. The child's457 exact-only
sibling filters are not waived tests: parent tools run458 including all eight additions.
Locked workspace check/Clippy, both SDK modes, fmt/diff, governance(SQL17/delivery12), claims,
text boundary23, classifier14 and site/installers pass. No full unfiltered local preflight claim.
Fresh exact-head protected review/full CI/CAS remain the remote gates. Resume on
feature/I293-search-form-protocol from #715 merge; verify local/remote tree equality before
publication and keep full C/EP, #643/#624 and D/E-F activation dispositions unchanged.

## Form Candidate CI And Isolated Handle Fixture — 2026-10-14

#716 original head925e46a89a8dce7e8177b9e1f681657974f2f9e2/tree2085eb5e9069c9cf574fd59f89abb8ac6b74e777
received protected independent Agent review6158827975. CI3175/run38495718642 passed five
jobs but failed Windows workspace: tools448pass/1fail, pre-existing
windows_partial_setup_failures_release_every_parent_handle before151/after162 (+11).
Its GetProcessHandleCount observes the entire concurrent harness. That measurement scope
permits interference from other tests; this is not proof that interference solely caused +11
or that no actual leak exists. Failed CI remains evidence, not a retry-to-green candidate.

Bounded maintenance under the existing collaboration SOP exception repairs only that Windows
test in the same PR. Production process source is byte-identical; no unsafe/native binding,
process rights/policy, dependency/lock, production deadline, skip or waiver changes. The outer
test uses existing WindowsBackgroundLauncher assignment-before-resume/kill-on-close Job
ownership to launch the same binary with exact test selection and one test thread. Descendants
created before inner assignment remain under the outer Job; no breakaway is introduced.
The sole control Arc stays owned through terminal handling and receipt validation; last drop
provides panic cleanup. The worker initializes its Tokio runtime before the baseline, then
executes the original synchronous five faults times eight attempts and original +2 threshold.
Exact injected-error checks prevent an earlier launch failure from masquerading as a later fault.

Success requires a real successful Exited event and a <=1024-byte JSON receipt in an owned
canonical temporary directory, reporting all40 attempts, u32 before/after and a nonzero worker
PID distinct from the parent. Exact invocation/receipt checks reject zero-test success or EOF.
Captured diagnostics retain <=16KiB while continuing to drain. Event waiting is bounded60s;
failure requests force termination and drains terminal events for up to10s before assertion.
These bounds do not claim a hard limit on every OS/native cleanup tail. Actual nested-Job
compatibility and resource assertions must pass native Windows CI; unsupported-host failure
is not skipped. #705 retains its separate workflow/diagnosis/script scope.

Restored pinned1.99.0 local locked check/Clippy and workspace tests pass:3705 primary/119
summaries +ambient child1 (raw3706/120), zero failures/ignores; only the previously authorized
unchanged Unix-socket primary filter. Child457 exact-only sibling filters are not waivers;
parent tools458 execute them. External SDK fixture passed. Windows-only correction is not
compiled/executed by this Linux checkpoint; fresh final validation, independent protected
implementation/exact-head review and all six unfiltered native CI jobs remain before CAS.
I293/C remain Active/Claimed/incomplete; #643/#624 open, D separate and E/F gated.

Final correction checkpoint: independent protected implementation preview found no blocker;
not stable-head approval or native execution. Fresh fmt/diff/workspace Clippy, both SDK modes,
governance(SQL17/delivery12), claims0warnings, text boundary23/classifier14 and site/installers
pass. Initial shell scoping omitted cargo from later chained commands; validation was rerun
under a shared explicit toolchain environment. A mistaken extra SDK --offline invocation was
rejected by argparse after the standard validator had passed both actual SDK modes; no offline
SDK claim is made. Remote main is stilld66e380a, #643/#624 open and only #705/#716 open PRs;
#705 changes workflow/diagnostic docs/script, not this committed Rust test fixture.
Publish the coherent checkpoint to the existing #716 branch, verify tree equality, obtain fresh
exact-head protected independent Agent review and complete unfiltered CI, then repeat CAS.

## Isolated Worker Failure And Bounded Trace — 2026-10-14

Fresh #716 protected review6159740360 approved heada30279c1c53f52c8b2448d85e5940539557ba552/
tree6ae44a0b2ab5096d646585bd20e193c78892cf07, conditional on native CI/CAS. CI3176/
run38502637067 FAILED Windows workspace, five other jobs passed. The isolated worker reached
the original final resource assertion: before73/after76 (+3),0pass/1fail/448 exact-only sibling
filters. Parent tools448pass/1fail,0ignored/0filtered. Outer cleanup=Ok(())/terminal=true;
no timeout or output overflow. These observations prove the worker/Job launched and the exact
fault checks reached the final sample, not that all supervision tasks joined or that no leak
exists. Isolation alone did not resolve the failure. Mac full unfiltered release preflight,
original Unix-socket case and both SDK modes passed; form/TLS fixtures passed on both hosts.
Windows downstream governance/audit/smoke steps were skipped by failure, not passed evidence.

Source review finds attempted cleanup of thread/process/job/pipes and attribute list, but its
unchecked native cleanup results do not prove actual closure. Neither a true handle leak nor
cold initialization is yet established. Do not warm up the launch path to hide a first-use delta,
widen +2, ignore/skip the case, retry the failed head to green or activate/merge the candidate.

The next stable candidate is explicitly diagnostic, not a completed correction: a fixed stack
[u32;41] records the original baseline and the count after each of the original40 fault attempts.
It reuses the existing read-only count helper and adds no unsafe/binding/handle/rights/dependency
or production-code changes. No diagnostic I/O or dynamic trace allocation occurs in the sampled
interval. A bounded trace prints after the independent original final sample and before its
unchanged assertion, so failures reveal growth by fault/attempt. Index0 is baseline; indices1..8
Pipe,9..16 Job,17..24 AttributeList,25..32 Assignment,33..40 Resume. Retain both counts[40] and
the separate original after value. Original exact errors,40 attempts,+2,outer ownership,
deadlines/drain and success-only receipt remain. Totals can locate growth boundaries but cannot
identify handle types or establish a unique cause. Native CI is required; even a green diagnostic
run alone is not root-cause closure or merge authorization. No new scope/claim/lifecycle change;
bounded fixture maintenance stays within #716 and does not take over #705 diagnosis.

Local diagnostic checkpoint: independent protected implementation preview found no blocker for
diagnostic publication only. Restored pinned1.99.0 fmt/diff, governance(SQL17/delivery12),
claims0warnings, text boundary23 and classifier14 pass. Source-prefix comparison proves production
process bytes unchanged; only10 Rust lines in the Windows test module are added. No new local
workspace compile/runtime pass is claimed for this Windows-only trace, and no new local test
filter was used. Original Linux and CI3176 Mac results remain historical receipts for unchanged
non-Windows code; fresh full native CI is mandatory. Main remainsd66e380a, open PRs #705/#716
are separate; I293 claim/lifecycle and all activation gates remain unchanged. Publish the stable
diagnostic tree, verify remote identity, obtain fresh independent protected exact-head review,
then interpret the trace before proposing any correction or merge.

## First-Use Boundaries And Checked Existing Cleanup — 2026-10-14

CI3177/run38520756375 failed Windows while all five other jobs passed. The diagnostic worker
recorded73 baseline,74 after first Pipe,74 through all eight Pipe/Job/AttributeList attempts,
76 after first Assignment,76 through remaining Assignment/Resume, independent after76. All40
exact injected-error checks reached the unchanged +2 assertion; parent448pass/1fail, worker
0pass/1fail/448 exact-only sibling filters, no ignored tests. This run exhibits two first-use
growth boundaries and no repeated accumulation; it establishes neither a legitimate lifetime
nor a one-time leak's absence. Preserve3175/3176/3177 as actual failed evidence.

Within the same bounded fixture maintenance, capture BOOL results from the existing close and
TerminateProcess calls and aggregate a test-only failure mask. Bits0 thread close,1 termination,
2 process close,3 Job close,4..7 stdout-read/write and stderr-read/write close. Only assert after
all cleanup operations have been attempted, so a failed check cannot skip a later release.
The original calls/conditions/order and production failure result remain; no new native call,
binding/unsafe block/handle/rights/manifest/lock is introduced. Source bytes in this production
function do change to capture existing return values; previous byte-identity claims apply only
to earlier heads. No production return checking or behavioral fix is claimed. The new checks
can establish known calls' success, not process-wide resource closure or the +3 cause.

Independent local protected preview found no blocker for these stronger checks. A proposed
[current-process type snapshot packet](../proposals/WINDOWS-HANDLE-COUNT-SELF-SNAPSHOT.md) records
the next concrete observation contract. It is Proposed/Unclaimed and requires maintainer scope
acceptance followed by separate effective ADR/claim before adding a native feature or unsafe
helper. Current I293 scope/claim and security decisions remain unchanged; no PSS code is added.
No warmup, threshold increase, skip or merge authorization follows from either diagnostic.

Final local checkpoint: independent seven-file protected preview found no publication blocker.
Pinned1.99.0 fmt/diff, governance(SQL17/delivery12), claims0warnings, text boundary23 and
classifier14 pass. Comparison of the touched function's native call-site inventory/order and
unsafe block count is unchanged; this is source evidence, not compiled-binary equivalence or
native cleanup execution. No new local workspace/native pass is claimed. Fresh stable-head
protected diagnostic review and native full CI remain necessary. Proposal is not scope approval;
Cargo feature unification must be verified before any test-only feature-isolation claim.

2026-10-15 adjacent diagnostic: #718 / ADR-091 / TEST-003 / I302 claim is effective via
f4313237. Current main9b7ba281 and original7165c49a8bd are combined before reconstructed
I302 diagnostic publication. Unpublished d1d818e2 was lost in workspace maintenance; its local
receipts are historical, not current evidence. Cold40/+2 and CI3175-3178 failures remain;
3178 known cleanup BOOLs succeeded without resolving73->76. Three owned workers qualify
bounded types/observer/cleanup evidence, not a production fix or716 merge. Fresh review/full
native CI pending;643/624 open, D/activation/rollback gates unchanged,705/721 independent.

CI3190/run38679585476 on a39ee959 failed Windows cold73->76 (453passed/1failed), with five
other jobs successful. I302 observer/native fixtures succeeded but did not repair the original
failure. Exact protected publication review6184162949 does not authorize merge. I302 now owns
two additional bounded stage controls using existing launcher operations; see its owner journal
for metadata/uncertainty and fresh validation requirements. Latest main23e659e2 is included,
preserving I303 Complete and I304's independent claim. I293/C remains Active/Claimed/incomplete;
#643/#624 open and activation/full-chain gates unchanged.

CI3197/run38685760669 on770d856e FAILED Windows454passed/1failed, five other jobs passed.
I302 modes1–4 succeeded, narrowing but not resolving the original cold73->76 failure. Direct
CreatePipe/CreateProcessW return/cleanup observation belongs to I302 and adds no Search or
production launcher behavior. I293 remains Active/Claimed/incomplete and #643/#624 stay open.

C-fixture residual from the next local preflight: untouched Linux deadline/EOF test failed
with457tools passes/1failure. It requires EOF without first proving request/accept arrival and
cannot distinguish read errors from no EOF. This is a source-level ambiguity, not proven cause;
retain the failure and original2s/150ms bounds. I302 journal records exact receipt and separate
validation. No retry-to-green/full-preflight pass or transfer to #705 is claimed.

Bounded Linux fixture telemetry now records up to4 read ErrorKinds and, only at failed EOF
assertion, accept count, up to4 request(length,header-complete) shapes and EOF count. No request
content/path is logged; no error becomes EOF and all150ms/2s/10s bounds remain. The next full
preflight tests materially new diagnostic source; retain the old failure and make no repair or
scheduler-rootcause claim from a later pass. Independent protected review remains required.

The failure snapshot is non-atomic: committed request shapes exclude any still-in-flight buffer,
so an empty list is not proof that no bytes arrived; accepted==0 is absence of an accept receipt,
not proof that no TCP connection reached the listen backlog. EOF counts read events, not unique
connections. Error total and saved kinds are sampled separately; errors_unrepresented may
reflect in-flight recording as well as the four-kind cap. Preserve these interpretation limits.

Fresh pinned locked full preflight on the new combined diagnostic source passed3717primary+
1existingchild/0failed/ignored and both SDK modes (only prior exact local socket filter).
Windows cross-check/Clippy, graphs and governance pass; these are not native observations.
The earlier457/1 Linux EOF failure remains unresolved evidence; a later pass does not prove a
fixture repair or scheduling cause. Production transport and all original deadlines/assertions
are unchanged. Fresh stable-head protected review and unfiltered full CI remain required.

## Merged Bounded Form Stage — 2026-10-17

#716 merged as b5b8241e2d8fa72d6e469bd96ccbdf54ff851c69 after exact198cb751/tree29ac46f2,
all6CI3221 jobs, protected full-stage review and CAS6233550377. This adds only dormant private
bounded form transport and its tested TLS/redirect/authority constraints; no default WebSearch or
provider admission. Full C/EP/protocol parity, prepared-binding, D/privacy/root/rollback and actual
activation gates remain Pending; this owner stays Active/Claimed, #643/#624OPEN.
TEST004/I305 closes only its known-resource test criterion with existing merged evidence. #717
retains unexplained native resources and prior failures. Passing captured assertions are not raw
numeric receipts or cache/global-leak proof; #705 and completed I301/I303/I304 remain separate.

## Owned Transport Request Stage — 2026-10-17

Selection base: main `891f6c4992c12afdc91f702b4c85d2316065b541`. Effective #712 claim and
ADR-090 cover this existing controlled-transport prerequisite before branch creation.
Inventory: I293 Active/Claimed continues this slice; I294 Active retains independent D evidence;
I300 Active retains structured Git and PERM-007-H is not claimed here; I302 Review retains #717
native attribution; I277 Review retains deferred device/human checks; I249 Planned stays deferred;
I164 Paused stays superseded. I301/I303/I304/I305 stay Complete/Closed. Only open PR #705 owns
Windows timeout diagnostics; no overlap, takeover, retry-to-green or acceptance relaxation.
Historical status-change notes in completed I146/I147/I150 and I162's completed review outcome
are not new nonterminal work. No unrelated owner or inventory status is changed.

Bounded runnable deliverable: private immutable PreparedTransportRequest constructors and consuming
execute_prepared, with deterministic/local-socket fixtures. Own actual parsed URL/method/encoded
body/fixed type before execution, validate scope locally, reject cross-invocation consumption before
DNS or budget reservation, and preserve all live authority, address/TLS/redirect/deadline fences.
Pure preparation performs no authority check, DNS, send or budget consumption. Raw entry points
become test-only adapters. No new API/dependency/lock/configuration or default-search behavior.
Independent protected design preview approved this boundary, with exact-source review still needed.

This is explicitly library-only. An object cannot be mutated or cloned through its API and is
consumed once; the same manifest may prepare multiple objects. This is not global exactly-once
execution, proof of preparation before approval, request-fingerprint integration, or official Agent
root binding. Future prepared tool invocation must retain this same object across admission.
Session scope remains query-independent; query/body and digests never enter approval scope.
Affected reference: SEARCH-001-C-CONTROLLED-TRANSPORT.md. Full C/EP/parity/D/root/rollback and
default activation remain Pending, with #643/#624 open and #717 independently unresolved.

Historical local convergence before workspace maintenance: pinned Rust1.99.0, locked focused
transport tests52/0; full release_preflight PASSED (3734 primary workspace tests plus1 existing
exact-selection child, 0failed/ignored; independent Runtime SDK default and coding modes pass).
Only the prior authorized Unix socket fixture was filtered locally; the child selects its one
existing proxy-isolation test, filtering465 sibling tests covered by the parent. Remote CI stays
unfiltered. Workspace check/Clippy, format/site/installers, governance(SQL17/delivery12)/claims
0warnings, text23/0 and classifier14 passed. Scale remains high-risk/release-managed/on-demand.
Prior Linux EOF457/1 and Windows/global-count failures remain historical; this pass is not a repair.

Independent working-source technical/security review APPROVE found no blocker; source blobs:
transport46b2855d039c1e5d8ce95c8bcab9559404dd973f,
fixtures6f2cf1e2d93a78c5445e3d45046a5c5d852ad4e2,
prepared25efbdcb3587baba17d4ef418852de073ba82ffa. Shared Agent/account identity is not distinct
human approval. Stable exact-head protected review, unfiltered CI and final CAS remain required
before deliberate merge. The source Issue delivery comment links the resulting candidate PR.
I293 stays Active/Claimed, Completion Pending. No release or default activation is authorized.

### Workspace Recovery — 2026-10-18

Automated workspace maintenance removed the unpublished candidate, local logs and toolchain/cache.
Main remains891f6c49. The candidate is reconstructed from the recorded patch and source fingerprints;
the historical pass above does not substitute for fresh candidate validation. The prior final
test-Clippy/governance commands had not been collected before maintenance, so no result is claimed
for them. The already-collected full-preflight governance/claims results above are separate from
those later standalone commands. Fresh pinned checks, exact-source review, stable publication and
remote gates follow.

Fresh reconstructed-candidate validation PASSED: pinned Rust1.99.0 locked full
release_preflight, 3734 primary workspace tests plus1 existing exact-selection child,
0failed/ignored, both independent Runtime SDK modes, focused transport52/0 and network-feature
test-target Clippy with -D warnings. The sole local socket filter and existing exact-child
selection remain as described above; no new skip or remote CI filter. Governance(SQL17/delivery12),
claims0warnings, text23/0, classifier14, site/installers and format pass. No source change was needed
after reconstruction. Three reviewed source fingerprints remain exact; this is fresh local evidence,
not a reclassification of any earlier failed run. This prerequisite's candidate is Review;
I293 remains Active/Claimed and Completion Pending. Exact remote CI/review/CAS remain pending.

## Merged Owned Payload / Dormant DDG Parser Stage — 2026-10-18

#732 merged as `8fa2a50016143b7c12f4fe7652d75078ab30f006`, source headb0ae0827/tree7c741724.
CI3227/run39053154860 attempt1 all6jobs SUCCESS; Windows tools468/0failed/0ignored/0filtered,
new5prepared fixtures/realOnce clear pass; macOS unfiltered original Unix socket, new fixtures,
SDK2modes/fullpreflight and subsequent gates pass. Protected full-stage review/CAS6237834927 and
delivery6237844190 record the evidence. This backfills actual #732; it is partial C evidence,
not a Completion Commit for this still-incomplete parent. Historical failures remain unchanged.

Selection base: main8fa2a50. The effective #712 claim already covers controlled existing-protocol
internalization under ADR-090. Inventory before branch: I293 Active continues; I294 Active retains
independent provider evidence; I300 Active retains Git/PERM-007-H boundaries; I302 Review retains
#717; I277 Review keeps deferred device/human checks; I249 Planned stays deferred, I164 Paused
stays superseded. I301/I303/I304/I305 remain Complete/Closed. Only #705 was open, owning Windows
timeout diagnostics, with no overlapping parser scope. No separate owner/claim or new dependency.

Runnable deliverable: private dormant Lite/HTML response parser plus synthetic fixtures, existing
network-free rust-websearch helpers and bounded selection/text collection. This is explicitly
library-only; no live query/response acquisition, request factory, default WebSearch switch,
official root integration, new API/provider/dependency/lock or release. Affected reference:
SEARCH-001-C-DDG-RESPONSE-PARSER.md. Full C/EP/parity/D/privacy/root/rollback/activation remain pending.

Before deriving source, downloaded rust-websearch0.1.2 and verified archive06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee.
VCS metadata pins b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch. Lite/HTML/html_utils
archive files exactly match those independently retrieved from that GitHub revision. Full upstream
MIT notice was retrieved before copying and retained inside the tools crate for package inclusion;
per-file provenance and changes are recorded in the reference. No service-use admission follows.

Protected design preview APPROVE scoped this to parser-only. Candidate outputs remain untrusted;
javascript/relative/malformed links are deliberately not silently converted into approved URLs.
Existing helper quirks, snippet alignment and Lite/HTML challenge differences remain characterized.
Strict UTF-8/BOM rejection, 1MiB input,16KiB field and1..20 result bounds are explicit migration
differences. Talos scraper0.27 versus upstream0.22 malformed-DOM parity remains unproven.
Selector iterators do not make the DOM parser streaming; bounds do not prove hard process-memory
or CPU deadlines, and catch_unwind does not catch abort/OOM or suppress global panic hooks.

Workspace maintenance removed the prior checkout/cache; main was cloned and pinned Rust1.99.0
restored. No prior check is relabeled as current validation. Fresh parser fixtures12/0 pass;
full checks, crate license-inclusion validation, exact candidate review/CI and final CAS follow.
I293 stays Active/Claimed, Completion Pending; #643/#624/#717 remain open and #705 stays independent.

Fresh local convergence PASSED on pinned Rust1.99.0: locked full release_preflight,
3746 primary workspace tests plus1 existing exact-selection proxy child, 0failed/ignored;
independent Runtime SDK default and coding modes pass. Only the authorized socket fixture is
filtered locally; the exact child filters477 sibling cases covered by the parent. Remote CI stays
unfiltered. Focused parser12/0 and network-feature test-target Clippy with -D warnings pass.
Package --list includes the full MIT notice and both parser source/fixture files. Workspace
check/Clippy, format/site/installers, governance(SQL17/delivery12), claims0warnings, text23/0
and classifier14 pass. Scale remains high-risk/release-managed/on-demand. No prior native/EOF
or Windows failure is reclassified by this pass.

Independent protected working-source/provenance review APPROVE found no blocking finding:
parser blobf36a11a5df341347c5d9634d2740f333b70f64db,
fixtures e4041ee25ad285f7731e60456abaae6a6f4aaaff,
MIT notice a9159334333b7406e04f5b5e765402b41cdcce95. Reviewer independently checked the pinned
GitHub source hashes and complete license, not a separate archive download or test execution.
Shared Agent/account identity is not distinct-human approval. This dormant prerequisite is ready
for stable candidate Review; exact-head protected review, all native CI and final CAS remain
required before deliberate merge. The source Issue delivery index will name the actual candidate.
Parent I293 remains Active/Claimed, Completion Pending; no release or default activation.

## Merged Lite/HTML / Dormant Instant Answer Stage — 2026-10-18

#733 merged as dea3edccb163420806e08ddeef51c65f977bac59, sourcehead3dce62bb/tree9a5382fb.
CI3230/run39096940347 attempt1 all6jobs SUCCESS. Windows tools480/0failed/0ignored/0filtered,
macOS tools478/0/0/0 include all12parser fixtures; original Unix socket/SDK2modes/fullpreflight
and subsequent gates pass. Protected source review6245623527, final full-stage review/CAS6245774420
and source delivery6245778297 preceded/recorded the verified merge. This backfills actual #733,
as partial C evidence only. Parent I293 stays Active/Claimed, Completion Pending.

Selection base: main dea3edc. Before branch, verified effective #712/ADR-090 claim and no
overlapping PR; only #705 open, retaining Windows timeout diagnostics. Inventory: I293 Active
continues its existing bounded protocol internalization; I294 Active retains D research,
I300 Active retains Git/PERM-007-H boundaries; I302 Review retains #717; I277 Review retains
deferred device/human checks; I249 Planned deferred, I164 Paused superseded. I301/I303/I304/I305
remain Complete/Closed. Historical I146/I147/I150 review notes and I162 completed review outcome
do not establish new nonterminal work. No owner takeover, new claim, dependency or activation.

Runnable deliverable: private dormant Instant Answer JSON parser with synthetic fixtures, reusing
the common bounded/redacted DDG candidate boundary and existing network-free entity helper.
Explicitly library-only: no HTTP/request factory/header change, live provider query/response,
Agent root wiring, new backend/API/lock/dependency/permission or default-search behavior. Affected
reference: SEARCH-001-C-DDG-RESPONSE-PARSER.md. Full C/EP/D/privacy/parity/root/rollback remain pending.

Before copying, reverified retained0.1.2 archive06b90d9a/VCS b30180a0/crates/websearch and freshly
retrieved pinned Instant Answer source SHA256b3aad75760fcad2e079bb9d15ec155e072748482d6ca1906b3ad19e109ca6ced.
Archived and retrieved source match exactly; retained full MIT notice matches freshly retrieved
pinned root notice (trailing newline only). This is not another fresh archive download or D admission.
Independent protected design preview APPROVE required direct typed schema, required-unused fields,
raw topic first-sentence splitting, limited cleanup, query encoding and explicit migration limits.

First local focused run16passed/1failed: a fixture incorrectly expected general numeric entity
decoding for &#46;. Inspection of the pinned network-free helper confirms only seven fixed
replacements, excluding that entity; corrected the fixture to retain it, with no implementation
repair or acceptance relaxation. Also corrected a duplicate-field fixture's JSON spelling locally
before stable publication. Fresh focused/full checks, source review, package inclusion, native CI
and final CAS follow. No historical failure is reclassified; #643/#624/#717 stay open, #705 independent.

Fresh local convergence PASSED: pinned Rust1.99.0 locked full release_preflight,3763primary
workspace tests plus1 existing exact-selection child,0failed/ignored; independent SDK default
and coding modes pass. Focused Instant Answer17/0 and network-feature test-target Clippy
with -D warnings pass. Package --list includes Instant Answer source/fixtures and full MIT notice.
Only the previously authorized socket fixture is locally filtered; the exact proxy child filters
494siblings covered by the parent. Remote CI remains unfiltered, with no new skip or ignored test.
Workspace check/Clippy, site/installers/format, governanceSQL17/delivery12, claims0warnings,
text23/0 and classifier14 pass. Scale remains high-risk/release-managed/on-demand, sources489,
packages31/workflows4. No version/lock/dependency/public API or permission change.

Independent protected working-source/provenance review APPROVE found no blocking finding;
parser blobf13f4360f3e6cc8965b1f64d0846035b34c4b194 and fixture
a55fc7026c9d6fa9df16e7d176db01ef3430d7fe. Reviewer independently fetched/hash-checked pinned
source, helper and root license; did not build or independently redownload the archive. Shared
Agent/account identity is not distinct-human approval. This prerequisite is ready for stable
candidate Review; exact-local/remote review, native CI and final CAS remain merge gates.
I293 stays Active/Claimed, Completion Pending; full C/EP/parity/D/privacy/root/rollback/activation
remain gated. The source Issue index will record the actual candidate PR and exact evidence.

## Merged Instant Answer / Dormant Bing Response Stage — 2026-10-19

#734 merged45cf9eb98a0a90a1fc73d0d62ce75dbe78469519, sourceheadb1582f8a/treefd5d4058.
CI3232/run39099268100 attempt1 all6jobs SUCCESS: Windows tools497/0failed/0ignored/0filtered,
macOS tools495/0/0/0 include all17new fixtures; original socket/SDK2modes/fullpreflight and
later gates pass. Protected source approval6245925720, final full-stage review/CAS6257354451
and source delivery6257359977 record actual merged evidence. No historical failure is reclassified.
This backfills actual #734 as partial C evidence, not parent completion.

Selection base: main45cf9eb9. Effective I293/#712/ADR-090 claim verified before branch. Inventory:
I293 Active continues existing-protocol internalization; I294 Active retains D research; I300
Active retains structured Git/PERM-007-H boundaries; I302 Review retains #717; I277 Review retains
deferred device/human checks; I249 Planned deferred, I164 Paused superseded. I301/I303/I304/I305
remain Complete/Closed; historical I146/I147/I150 notes and I162 completed review outcome are not
new nonterminal work. Open #705 owns timeout diagnostics; #735 owns I300/#704 Git and does not
touch search_backend/I293. No claim/owner takeover or approval of either independent PR.

Runnable deliverable: private dormant Bing CN response parser with synthetic HTML, closed errors,
redacted candidates and bounded DOM/text traversal. Explicitly library-only: no HTTP/request
factory/header/retry/Agent root/default change, new backend/API/dependency/lock/permission or release.
Existing DuckDuckGo ID denotes the compound compatibility family, not Bing attribution or native
admission. Affected reference: SEARCH-001-C-BING-RESPONSE-PARSER.md; DDG reference links it.

Before deriving code, freshly downloaded0.1.2 archive06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee
and verified VCS b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch. Archived src/bing.rs
matches freshly retrieved pinned source SHA25673fe17e6156599bd0e0e109762f88bacf4b47d18024f5d7a3eb7e78dfb3334d0.
Retained full tools-package MIT notice matches freshly retrieved pinned root notice (final LF only).
Independent protected design preview APPROVE confirmed first-element/ordering/status/helper
semantics, raw-href-before-trim bounds and family-ID limitation. No live provider query or result
fixture was acquired, and source licensing is not D service-use/privacy admission.

Workspace maintenance removed prior toolchain/cache; pinned Rust1.99.0 was restored from official
manifest-checksummed packages. New candidate checks are required; no prior run becomes fresh
validation. Focused fixtures, full preflight/test-Clippy, package inclusion, protected source/head
review, native CI and final CAS follow. Full C/EP/parity/D/privacy/result admission/root/router/
rollback/default activation remain pending; I293 stays Active/Claimed, Completion Pending.
#643/#624/#717 stay open, #729 closed, #705/#735 remain independent.

Fresh local convergence PASSED on pinned Rust1.99.0: locked full release_preflight,
3777primary workspace tests plus1 existing exact-selection child,0failed/ignored; independent
Runtime SDK default and coding modes pass. Focused Bing14/0 passed on its first run; network-feature
test-target Clippy with -D warnings passes. Package --list includes Bing source/fixtures and the
full MIT notice. Only the previously authorized socket fixture is locally filtered; the exact
proxy child filters508siblings covered by the parent. Remote CI remains unfiltered, no new skip
or ignored test. Workspace check/Clippy, site/installers/format, governanceSQL17/delivery12,
claims0warnings,text23/0 and classifier14 pass. Scale remains high-risk/release-managed/on-demand,
sources491/packages31/workflows4. No version/lock/dependency/API change or historical repair claim.

Independent protected working-source/provenance review APPROVE found no blocking finding;
parser blobd782bfdf16aabcc0b1ba2ff9e1ad6f5610d87745, fixtures134712d7d9325122528aa1ded502a1343db41368.
Reviewer independently fetched/hash-checked pinned Bing source and root license; did not build or
independently redownload the archive. Shared Agent/account identity is not distinct-human approval.
The parent-module overview now accurately describes dormant infrastructure rather than only typed
contracts; existing executable behavior is unchanged. The reference clarifies href bounds apply
after DOM entity decoding, before trim. This prerequisite is ready for stable candidate Review;
exact-local/remote review, native CI and final CAS remain required before deliberate merge.
Parent I293 remains Active/Claimed, Completion Pending; the source Issue index will record the
actual candidate PR/head/tree. No default activation, D admission or release follows.

## Merged Bing / Dormant Fixed Request Stage — 2026-10-19

#736 merged2b015a6bba10e646ac4b022514177c4a92ba372c, sourcehead78780d93/tree709e6bbf.
CI3245/run39184232215 attempt1 all6jobs SUCCESS: Windows tools511/0failed/0ignored/0filtered,
macOS tools509/0/0/0 include all14new fixtures; original socket/SDK2modes/fullpreflight and
later gates pass. Protected source approval6257545314, final full-stage review/CAS6257754612
and source delivery6257761924 record actual merged evidence. This backfills #736 as partial C
evidence. No historical failure is reclassified; I293 stays Active/Claimed, Completion Pending.

Selection base: main2b015a6bba10e646ac4b022514177c4a92ba372c. Effective #712/ADR-090 claim
verified before implementation branch. Inventory: I293 Active continues existing-protocol
internalization; I294 Active retains D research; I300 Active retains Git/PERM-007-H; I302 Review
retains #717; I277 Review retains deferred human/device checks; I249 Planned deferred, I164
Paused superseded. I301/I303/I304/I305 Complete/Closed; historical I146/I147/I150 and completed
I162 review notes are not new nonterminal work. Open #705 remains timeout diagnostics; #735
head0b42e580 is I300/#704 Git and does not touch search_backend/I293. No takeover or approval.

Runnable deliverable: private pure fixed request factories for Lite/HTML/Instant Answer/Bing,
returning the existing immutable PreparedTransportRequest; exact encoded limits and manifest
membership/invocation binding remain mandatory. Explicitly library-only: no production caller,
HTTP/DNS/authority/budget effect, root/router/default or model behavior, new backend/public API,
dependency/version/lock/permission change. Affected reference:
SEARCH-001-C-COMPATIBILITY-REQUESTS.md; controlled transport reference links it. Full protocol
headers/UA/parity/D/privacy/parser binding/result admission/root/rollback/activation stay pending.

Independent protected design preview APPROVE requires actual fixed-destination derivation,
ordered fields, complete encoded GET4096/POST16384 limits before serializer allocation,
raw bound before blank check, preserved nonblank bytes and adversarial scope/invocation fixtures.
No approval seal/global once/query fingerprint is claimed. Shared Agent/account identity is not
distinct-human approval; exact candidate and terminal gates still require separate review.

Fresh archive SHA25606b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee
and VCS b30180a0/crates/websearch verified. All four archived sources match freshly retrieved
immutable pinned sources byte-for-byte; hashes and complete MIT notice are in the reference.
An initial local comparison reconstructed trailing newlines incorrectly for Bing and asserted;
rechecking the exact retrieved strings confirmed identical source bytes and the complete retained
MIT notice (final LF only). No source mismatch, live query, D admission or test failure is claimed.

Workspace maintenance removed the old toolchain/cache. Official manifest-checksummed Rust1.99.0
was restored and executed; fresh focused tests, full preflight, network-test Clippy, package
inclusion, protected exact-source/head review, native CI and final CAS follow. Only the existing
authorized local socket filter is allowed; remote CI remains unfiltered. #643/#624/#717 stay open,
#729 stays closed and #705/#735 remain independent. Parent completion and default activation Pending.

Fresh local convergence PASSED on pinned Rust1.99.0: standard locked release_preflight.sh
exited0, including workspace check/strict Clippy/tests and independent SDK default/coding modes.
The redirected preflight journal retained only an early fragment; no aggregate suite count is
claimed from that fragment. The command's terminal exit0 is observed. Native exact-head logs
must independently establish unfiltered tests and the original socket fixture before merge.
Focused request fixtures11/0 passed first run; network-feature test-target Clippy -D warnings,
format/whitespace, reference links and package inclusion of source/fixtures/full MIT all pass.
Governance SQL17/delivery12 and claims0warnings were observed. Only the existing authorized
local socket filter is used, no additional skip/ignore/retry or threshold change. Scale remains
high-risk/release-managed/on-demand, sources493/packages31/workflows4. No prior failure is repaired.

Independent protected working-source/provenance review APPROVE found no blocking finding:
transport blob2ff93a7187030968a805d20b7b3669ea54618af9,
factory8f3ecd98e9f039568a4814620b680459605d57c5,
fixtures6a56c07be70911d263df5f07909e1e9c3e5a58a1. Reviewer independently retrieved/hash-checked
all four pinned sources and complete root MIT; did not independently download the archive or
build. Shared Agent/account identity is not distinct-human approval. This prerequisite enters
stable candidate Review; exact-local/remote protected review, native terminal CI and final CAS
remain required. Actual candidate PR/head/tree will be indexed from #643; I293 remains Active/
Claimed, Completion Pending, with full C/EP/parity/D/privacy/root/rollback/default activation gated.

## Merged Fixed Requests / Dormant Bound Execution — 2026-10-19

#737 mergedae6a0791c6317ff7e384295ac9af74a6571572a7, sourcehead271322c9/treeb10ae124.
CI3249/run39189980467 attempt1 all6jobs SUCCESS. Windows tools522/0failed/0ignored/0filtered,
macOS tools520/0/0/0 include all11new fixtures; original socket/SDK2modes/fullpreflight and later
gates pass. Protected sourceapproval6258352299, terminal full-stage review/finalCAS6258556389
and source delivery6258560230 record verified merged evidence. This backfills actual #737 as
partial C evidence only; no historical failure or local incomplete journal is reclassified.

Selection base: mainae6a0791c6317ff7e384295ac9af74a6571572a7. Effective I293/#712/ADR-090 claim
verified before branch. Inventory: I293 Active continues bounded compatibility internalization;
I294 Active retains D research; I300 Active retains Git/PERM-007-H; I302 Review retains #717;
I277 Review retains deferred human/device validation; I249 Planned deferred, I164 Paused
superseded. I301/I303/I304/I305 Complete/Closed; historical I146/I147/I150 and completed I162
notes are not new nonterminal work. Open #705 retains diagnostic ownership. #735 drafthead
9d5fd3bbc5dc07349c44f5a971bcfb59f5f5902c has120files across two pages, no search_backend/I293/
SEARCH-C overlap. No owner takeover, new claim or approval of independent work.

Runnable deliverable: private non-Clone PreparedCompatibilitySearch retaining protocol, original
query, max and existing immutable request; consuming execute uses controlled transport then only
the captured parser, with unconditional existing-budget checks before/after parsing. Unified
redacted candidates remain untrusted; errors wrap existing closed content-free transport/DDG/
Bing classes. Explicitly library-only: no official caller, new router/winner/result admission,
default behavior, public API/backend/dependency/version/lock/permission change. Affected reference:
SEARCH-001-C-COMPATIBILITY-EXECUTION.md, linked from request and controlled-transport references.

Protected independent design preview APPROVE requires count validation before request prep,
preserved original query, no replacement execute inputs, no new transport trait/destination seam,
saved parse result before unconditional post-check and explicit synchronous non-preemption limit.
Test-only relocation starts from the real fixed factory, revalidates an explicit loopback scope
through the existing constructor and preserves method/path/query/body/type plus captured metadata.
This proves controlled loopback dispatch into parsing, not official-host HTTPS or full parity.
Private deterministic live-check injection and paused Tokio clock tests establish deadline
ordering without relying on HTML parse duration or introducing a production clock override.

Ten new synthetic tests cover four real request/response formats with count1/2, fixed wire bytes,
captured query/fallback/count, wrong parser, status/challenge/empty/whole-response failures,
unsafe unadmitted candidates, redaction, same-scope cross-invocation zero effects and shared
pre/post deadline boundaries. No official provider request or live result fixture is acquired.
The new orchestration is Talos-authored; existing parser/helper source and pinned full MIT
provenance remain unchanged. Workspace maintenance removed the previous checkout/toolchain;
official manifest-checksummed Rust1.99.0 was restored and executed for fresh validation.

Focused tests, full locked release_preflight, network-test strict Clippy, package inclusion,
protected exact-source/head review, native unfiltered CI and final CAS follow. Only the existing
authorized local socket fixture may be filtered; no new skip/ignore/retry/threshold relaxation.
Full C/EP/headers/UA/probe/retry/source/parity/D/privacy/result admission/official roots/router/
health/cancellation/rollback/default activation remain Pending. I293 stays Active/Claimed,
Completion Pending; #643/#624/#717 stay open, #729 stays closed, #705/#735 remain independent.

The initial fresh focused command failed during dependency compilation, before test execution:
icu_locale_core2.2.0 archive creation could not map a zero-length object. Read-only inspection
confirmed icu_locale_core-d87cf6bcfd13873d codegen0 and autocfg-4891372e8b3278fb codegen2 objects
were0bytes;28GiB free space was observed. The precise corruption cause is unknown. The command
terminated with exit101; this is not a test pass/failure
or a new source/dependency issue attribution. Recovery cleans only those two packages' build
outputs and uses bounded CARGO_BUILD_JOBS=2, with source/lock/pin/assertions/test concurrency and
CI coverage unchanged. The existing EVOLUTION lesson "Isolate and verify local build environments"
applies: preserve the failed observation separately and repair artifacts instead of weakening
acceptance. Fresh focused/full results below must independently pass; no native failure is repaired.

That recovered build also exited101 before tests, now at jsonschema0.58.6 archive creation with
the same zero-length object diagnostic. Lower build concurrency did not eliminate it; no precise
cause or successful test outcome is inferred. The next recovery moves build output outside the
workspace into an isolated temporary target, preserves compatible cached artifacts and cleans the
failed package outputs. Repository source/lock/pin/test assertions/concurrency/CI remain unchanged.
This is artifact/environment recovery, not rerunning a failed test to obtain a green result.
The jsonschema codegen09 object was confirmed0bytes; cleanup removed20files/23.8MiB for that
package. Installed libLLVM and librustc_driver SHA256 both match the official pinned archive,
so no compiler-binary mismatch is established. The target is now outside the workspace, with
an ignored compatibility symlink for existing build-script/SDK paths. This isolates artifact
placement; it is not proof that workspace synchronization caused either prior failure.

## Bound Execution Recovery Checkpoint — 2026-10-20

After that artifact recovery, all10 new focused tests actually passed (0failed/0ignored,
290filtered) and strict network test-target Clippy exited0. Independent working-source review
found no blocking finding. The subsequent standard preflight lost its execution connection;
both owner and independent read-only reviewer received409 environment_offline. No terminal
receipt or completed preflight log was obtained. Its outcome remains unknown, not a pass or a
test failure. Source Issue#643 checkpoint6263051529 preserves this boundary; no implementation
commit, branch publication or PR was created for this stage.

The execution environment is available again. All six local paths survive, with unchanged
compatibility source blobc2bb701bcd8405cdaa907efed1e2c6c8c8686794 and test
blob41bd59c8f22a72cb95fed055d420a13b1cc5d88e. The toolchain/cache and isolated physical target
are absent; official manifest-checksummed Rust1.99.0 was restored and executed. The standard
preflight is restarted with the same bounded build settings and sole authorized local socket
filter; a new journal streams each line and records the terminal exit separately. This is
recovery of an interrupted validation with unknown outcome, not a retry of a failed test.

Fresh remote main remainsae6a0791c6317ff7e384295ac9af74a6571572a7. Independent draft#735 moved
to b9c01b0d1ed1928d9a09d0db6efcb77f0d7a1bb7 (124files across two pages); diagnostic#705 moved
to703cb39b2eb9d6f14f71d84afe5154e4f0d85ec2 (3files). Complete changed-path inventories contain
no search_backend/I293/SEARCH-C overlap. Neither independent candidate is approved or taken over.
Fresh full-preflight/SDK/package/governance and exact-head review/CI/CAS receipts remain required.

The resumed standard locked release_preflight.sh now exits0 with both terminal markers:
release preflight: passed and TALOS_PREFLIGHT_TERMINAL_EXIT=0. Workspace check, strict workspace
Clippy, tests/doctests and independent Runtime SDK default/coding both pass. Tools530/0failed/
0ignored/0filtered includes all10 new bound-execution tests. Only the previously authorized
local runtime socket fixture is skipped; no new filter, ignore, threshold, warm-up or test retry
is introduced. Remote native CI must still run unfiltered, including that socket fixture.
SQLite17/delivery12, governance0warnings, claims0warnings, text-boundary, CI classifier and format
checks pass. The streamed journal SHA256 is
854753b3d1c89540fc09759181c5abfe9d492c1f5a798f5509953e94edf825aa.
This new terminal success does not reclassify the interrupted old run or either compile failure.

Locked offline package-list verification includes compatibility.rs, both compatibility fixture
modules and the complete rust-websearch MIT notice. Scale494sources/31packages/4workflows remains
high-risk/release-managed/on-demand. Independent refreshed working-source review APPROVE names
the same base and both source blobs above; it is not exact-commit, publication or merge approval.
The six-file staged diff contains only the bounded implementation, synthetic tests and owner/
reference synchronization. Full C and every remaining activation gate stay Pending.

Fresh network-feature test-target Clippy also exits0 with -D warnings, after the resumed full
preflight. Final owner synchronization again passes governance (SQLite17/delivery12/0warnings)
and claims0warnings. Format/whitespace and packaged provenance remain intact. This bounded
library prerequisite enters stable-candidate Review; exact local/remote protected review,
unfiltered native terminal CI and final fresh CAS are still required before merge. Actual PR/
head/tree delivery will be indexed from #643. I293 itself remains Active/Claimed, Completion
Pending; no parent completion, new provider admission, default activation or release is claimed.

## Merged Bound Execution / Dormant Fixed Headers — 2026-10-20

#738 merged2d7f4b36c17ae17f5f74d34b167987876db8add2, source362215a0/tree1a5fa0b13c32ffd39608c41ff996ad183f7c1a10.
CI3271/run39351058934 attempt1 all6jobs SUCCESS. Windows tools532/0failed/0ignored/0filtered,
macOS tools530/0/0/0 include all10 new fixtures; original macOS socket/SDK2modes/fullpreflight
and later gates PASS. Protected source6280849789, terminal review/freshCAS6281336287 and merged
source delivery6281349020 establish this partial stage only. No prior interrupted/failed run is
reclassified; full C/EP and default activation remain Pending.

Fresh selection base/main2d7f4b36c17ae17f5f74d34b167987876db8add2. Effective #712/ADR090 actor,
claim and Work Slice remain intact before branch creation. Inventory dispositions: I293 Active
continues controlled compatibility internalization; I294 Active retains D; I300 Active retains
Git/PERM-H; I302 Review retains #717; I277 Review retains deferred human/device rows; I249 Planned
deferred, I164 Paused/superseded. I301/I303/I304/I305 Complete/Closed stay closed; historical
I146/I147/I150 and completed I162 notes are not new activation. Independent #735 drafthead
77a8cf8412f5c4d01785454311a5580ef60f6ffb has125files across two pages; #705head703cb39b has3
diagnosticfiles. Full changed-path checks find no search_backend/I293/SEARCH-C overlap. No other
owner is taken over or approved.

Runnable library-only deliverable: fixed compatibility factories retain a private static UA/
Bing-language snapshot inside the same invocation-bound PreparedTransportRequest; controlled
execution and every scoped redirect apply that frozen profile. Generic JSON/form constructors
retain None. No caller header map, credentials/cookies/referer facility, public API/model schema,
dependency/lock/version/permission relaxation, official root/default or provider activation.
Affected documentation: SEARCH-001-C-COMPATIBILITY-HEADERS.md, linked request/execution/transport
references. I293 remains Active/Claimed, Completion Pending; #643/#624 stay open.

Protected design preview and working-source review APPROVE: validate count/query/encoded bounds/
actual scope before one existing pinned random_user_agent helper call, retain &'static str,
catch panic into closed InvalidRequest with no reroll/fallback, bound nonblank visible ASCII
UA<=512 and validate HeaderValue. Private suppliers permit deterministic error/order fixtures;
no exposed override or global hook exists. Transport only applies the typed fixed profile after
existing hop checks and before final authority fence. Existing identity encoding/content type,
POST redirect/body isolation, DNS/address/TLS/proxy/budget/deadline controls remain authoritative.

Seven new tests cover bounds/control/injection, panic closure/no retry, redaction/protocol
language, invalid preparation zero helper calls/valid exactly one, generic wire header absence,
scoped same/cross-origin GET snapshot retention and denied target zero sends. Existing four-format
execution fixtures now compare actual wire UA with captured snapshot for count1/2, verify only
Bing language, identity encoding and absent sensitive headers. Test-only relocation preserves
the added field and revalidates an explicit literal-loopback scope; no production URL override.
There is no assertion that random requests must differ, uniform draw proof or official-host query.

Fresh archive06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee/VCSb30180a0
verified; archived anti_detection/mod.rs, user_agent.rs and bing.rs exactly match freshly fetched
immutable source bytes. Full MIT text matches the pinned root, with one final LF added by Talos.
The root notice is1083bytes without LF; the retained notice is1084bytes, not byte-identical.
Source hashes and modifications are recorded in the new reference. The existing public helper/12-entry static
pool is reused, without copying a second production pool. Independent reviewer separately fetched
and hashed pinned sources/full notice, but did not independently download the archive or build.
Shared Agent/account review is not distinct-human approval.

Preparation now includes local randomness/entropy access, while performing no network/authority/
budget work. Snapshot-per-preparation differs from upstream client/source/retry scheduling, so full
draw timing/distribution and protocol/service parity are not established. catch_unwind does not
suppress panic hooks or recover abort/OOM. Charset/probe/retry/source/proxy parity, D/privacy/
result use/admission, official CLI/SDK/MCP roots/router/health/cancellation/rollback/default
activation and release remain gated. #705/#717/#735 retain owners; #729 remains closed.

Workspace maintenance again removed the toolchain/cache/physical isolated target. The checkout
survived unchanged at #738; official manifest-checksummed Rust1.99.0 was restored and executed.
Focused transport tests, strict network-test Clippy and standard locked release_preflight run
serially with existing bounded settings and streamed terminal journals. Only the authorized local
runtime socket filter is permitted; remote native CI remains unfiltered. Full actual validation,
package inclusion, exact-local/remote protected review, terminal native CI and final CAS follow.

Fresh focused cargo test --locked -p talos-tools --features network --lib search_backend::transport
exits0: primary suite80passed/0failed/0ignored/227filtered, with the existing separate child
re-execution1passed/306filtered. All7new tests and enhanced four-format execution PASS; no test
failure or retry occurred. Locked offline package-list includes both new headers modules and
the complete MIT notice. Scale496sources/31packages/4workflows remains high-risk/release-managed/
on-demand. Strict network-test Clippy and full standard preflight/SDK receipts follow.

Local convergence PASSED on pinned Rust1.99.0: strict network-feature test-target Clippy exits0;
standard locked release_preflight.sh exits0 with release preflight: passed and terminal exit0
markers, including workspace check/strict Clippy/tests/doctests and independent SDK default/
coding modes. Tools537/0failed/0ignored/0filtered includes all7new fixtures and enhanced wire
assertions. No new source/test failure or retry occurred. Only the existing authorized local
runtime socket filter is used; exact-head native CI must independently run that fixture and all
tests unfiltered. Preflight journal SHA256:
d5c5bace0cd864f099b2f5a66768eaf0e942e215f63bea222b6f46ff4fcb2f32.

Governance SQLite17/delivery12 and claims0warnings, format/whitespace and packaged source/full
MIT pass. Protected working-source/provenance review APPROVE names frozen transport1f3ed547,
compatibilityc4244c10, headers582ba9a7, executiontests63e9def7, transporttests5aa15966 and
headerwiretestsa8d8b5c0. The staged11-file diff contains only the bounded snapshot, tests and
owner/reference synchronization. Fresh main remains2d7f4b36; independent #735 draft77a8cf84
still has125non-overlapping files. #705 moveda5d3bddd772533506c212dc75a6747775e3896b0; all3
diagnostic changed paths were rechecked, with no Search/I293/C overlap or approval of that work.

This dormant library prerequisite enters stable-candidate Review. Exact-local/remote protected
review, terminal unfiltered native CI and fresh final CAS remain required before merge. Actual
PR/head/tree delivery will be indexed from #643. I293 remains Active/Claimed, Completion Pending;
full C/EP/parity/D/privacy/root/router/rollback/default activation and release remain gated.

## Merged Fixed Headers / Dormant Result Admission — 2026-10-20

#739 merged91c330ba557234070b996775289849f54267e603; sourcee6e762f7/tree5352859b.
CI3279/run39378524613 attempt1 all6jobs SUCCESS. Windows tools539/0failed/0ignored/0filtered,
macOS537/0/0/0 include all7new/enhanced fixtures; actual original socket, SDK default/coding,
standard preflight and all later gates PASS. LinuxDesktop112/0/0/0. Protected source6284613113,
terminal review/freshCAS6285079987 and merged delivery6285092167 establish this partial stage.
Actual merge tree80b2f6b0f23aa19cd59c7270f45dcf4e4966c085 matches clean computed integration,
preserving unrelated provider-doc advancement at1a469954. No prior failures are reclassified.

Fresh base/main91c330ba; effective#712/ADR090 actor and Work Slice remain unchanged before
branch creation. Full current iteration inventory: I293 Active continues C; I294 Active retains D;
I300 Active retains Git; I277 Review retains deferred human/device rows; I302 Review retains#717;
I249 Planned deferred; I164 Paused/superseded. Historical I146/I147/I150 notes do not reactivate
those tasks; I301/I303/I304/I305 remain Complete/Closed. Independent#735 drafthead260e92281d8190c4cbb1fa3ca93207aeb557031b
has125files across2pages; #705head5848f8992457c72e13a48b7c2791635b6326453e has4diagnosticfiles.
Fresh full changed-path checks find no search_backend/I293/SEARCH-C overlap. No other owner is
taken over or approved, #705 stays open/unmerged and#729 stays closed.

Runnable library-only deliverable: consuming fixed compatibility execution now returns an opaque,
redacted, URL-admissible display-result wrapper. It parses at most20 candidates, checks the whole
batch's count/field/backend bounds, filters URLs, then truncates to captured requested count1..20.
This prevents invalid leading candidates from concealing later usable results inside that window.
It does not scan beyond20, deduplicate further, confer fetch/rendering/SSRF authority or establish
full protocol/service parity. Existing family DuckDuckGo identity is retained for all4formats.

Protected design preview APPROVE requires complete trailing-bound/identity checks, guarded WHATWG
URL parsing, preserved fields/order, shared pre/post parse/admission and final deadline fences,
including errors. URL acceptance requires parsed absolute HTTP/HTTPS with host, empty parsed
username and no password; an empty userinfo marker may be discarded by WHATWG. Repairs and
original display text are explicit limitations. Bounds20input/20output,
8192URLbytes and16384bytes each title/snippet are internal candidate limits, not accepted default
activation policy, caller allocation caps or hard synchronous CPU/memory bounds. No new clock,
authority, HTTP retry/source fallback, dependency, public API/model schema or permission change.

Affected reference: SEARCH-001-C-COMPATIBILITY-ADMISSION.md and bound-execution reference. No
official root calls this dormant code. Exact implementation review, local locked validation,
stable publication, unfiltered native CI and final CAS remain required. Full C/EP/parity/D/privacy,
roots/router/health/cancellation/rollback/default activation and release remain Pending.

Fresh local convergence PASSED on pinned Rust1.99.0: focused locked network transport tests
88passed/0failed/0ignored/227filtered, plus existing separate child reexecution1passed/314filtered.
All8new fixtures and enhanced existing tests PASS. Strict network-test-target Clippy exits0;
standard locked release_preflight.sh exits0 with actual release preflight: passed and terminal
exit0 journal markers, workspace check/strict Clippy/tests/doctests and SDK default/coding PASS.
Tools545/0failed/0ignored/0filtered. No source/test failure or retry occurred. Only the pre-existing
authorized local runtime socket filter is used; remote exact-head native CI must run it unfiltered.
Preflight journal SHA256:
a9a4b4585b39b65e0b8677b697ec6444464a9b33a0c855050f7a4194ccbb0b5f.

Governance SQLite17/delivery12/0warnings and claims0warnings passed within standard preflight.
Format/whitespace, locked offline package inclusion of both admission modules and complete MIT
notice pass. Scale498sources/31packages/4workflows remains high-risk/release-managed/on-demand.
The admission code is Talos-authored; existing licensed parser/source provenance is unchanged.
Independent protected working-source review APPROVE has no blockers and names frozen blobs:
compatibilityff3a2545, executiontests642a0545, admissiondb8ff4b1 and admissiontestsc1989ffc.
Its nonblocking empty-userinfo wording clarification was applied to docs, without source changes.
Fresh main remains91c330ba. Final owner synchronization, staged review, exact-local/remote approval,
terminal native CI and fresh CAS follow. This is stable-candidate Review only; I293 remains
Active/Claimed, Completion Pending, #643/#624 OPEN and all remaining activation gates intact.

## Merged Result Admission / Dormant Fixed Source Chain — 2026-10-20

#741 merged4e20ee955963bf22a0b9bc8df15c4bfc3771229a, source61af856e/treec84dc081.
CI3285/run39387035530 attempt1 all6SUCCESS; Windows tools547/0failed/0ignored/0filtered,
mac545/0/0/0 and all8new/enhanced fixtures PASS. Actual original socket, SDK default/coding,
fullpreflight/laternative gates PASS; LinuxDesktop112/0/0/0. Protected source6285813092,
terminal/freshCAS6286612496 and merged delivery6286667796 establish this partial stage. Remote
parent91c330ba and exact candidate tree were verified. The local service disconnected after all
checks; on recovery the clean checkout was fast-forwarded to4e20ee95 before new branch creation.
No successful gate is retried or prior failure reclassified for that tooling interruption.

Fresh main4e20ee95 and effective#712/ADR090 actor/Work Slice remain unchanged. Inventory: I293
Active continues C, I294 Active retains D, I300 Active retains Git; I277 Review retains deferred
human/device work and I302 Review retains#717. I249 Planned deferred, I164 Paused/superseded;
I301/I303/I304/I305 Complete/Closed remain closed, historical I146/I147/I150 notes do not reactivate
them. Independent#735 draftheadaf87f7bef89d5f0ae1248e3f0b159f5abb4c1e66 has125files across2pages;
#705head5848f8992457c72e13a48b7c2791635b6326453e has4diagnosticfiles. All changed paths were
checked, with no search_backend/I293/SEARCH-C overlap. #705 stays open/unmerged, #729 closed;
no other task is taken over or approved.

Runnable dormant library deliverable: one owned fixed-source preparation retains Lite, Instant
Answer, HTML and Bing in that order. Count1..20 and all4 bounded payload/manifest memberships
validate before any UA selection, then4 fixed snapshots are captured without reroll. The same
objects survive future admission and consume one existing SearchTransport/shared budget. Only
existing admitted display results can win; no later source starts after success or terminal error.
Only explicit semantic Empty/Status/Challenge, Bing RateLimited and admission Empty/Invalid
permit the next source. Every transport error, parser Limit/Encoding/Parse and admission Limit/
Identity/Parse terminates. Parse includes recovered dependency panic, so it cannot safely be
treated as ordinary malformed-response fallback. Ordered exhaustion retains4 actual closed errors.

Protected design preview APPROVE requires exhaustive classification, validation-before-draw,
capture-failure immediate stop, live shared pre/post outcome/final completion fences and joined
actual socket evidence. This is the traditional sequential fixed4 subset, not full upstream
source/probe/retry/entropy parity, an independent native backend/router or an activation. Four
upfront draws include unused sources; transport failures remain terminal rather than upstream
fallback. No rate limiter, cached probe, retry/backoff, paid/SearXNG/Wikipedia route, new authority,
dependency, public API/model schema, permission relaxation or default/root change is introduced.

Fresh archive SHA25606b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee
and pinnedVCSb30180a0 searcher.rs bytes independently compared with freshly fetched immutable
source. SHA256768a3c5135434ed2e1fea766e42f92580bfafca193d24178d2adf3d3cde6effe. Existing full
MIT notice remains packaged; Talos owns the prepared chain, safety classification and tests.
Affected docs: SEARCH-001-C-COMPATIBILITY-CHAIN.md and execution/admission references. Caches and
pinned toolchain were removed by workspace maintenance; checksummed official1.99 restored.
Local standard validation, protected source/exact-head review, native CI and final CAS follow.
I293 Active/Claimed, Completion Pending; #643/#624 stay open and all remaining C/EP/parity/D/
privacy/roots/router/health/cancellation/rollback/default activation and release gates remain.

Initial focused validation:95pass/2fail. Both failures reached Instant Answer and correctly
terminated as Ddg(Parse): the new empty-result fixture used{}, omitting the pinned schema's
required fields. This was fixture classification error, not a production continuation defect.
The fixture now uses a complete empty response; a new actual controlled{} regression proves
schema failures remain terminal before HTML. No parser/default/allowlist/timeout is relaxed.
The original failed journal is retained as chain-focused-initial-failed.log; it is not a pass.
Fresh corrected focused validation exits0:98passed/0failed/0ignored/227filtered, plus existing
separate child1passed/324filtered. All10new tests PASS. Strict Clippy and full preflight follow.

Stable local convergence PASSED on pinned Rust1.99.0: strict network-test-target Clippy exits0;
standard locked release_preflight.sh exits0 with actual passed/terminal0 journal markers.
Workspace check/strict Clippy/tests/doctests and independent SDK default/coding both PASS.
Tools555passed/0failed/0ignored/0filtered, including all10new source-chain tests. The sole existing
authorized local runtime socket filter remains; remote native CI must execute it unfiltered.
The initial95/2 fixture failure above remains history and is not reclassified as a pass.
Preflight journal SHA256:
4ee51b140721d9de212052d1cf794581d910e322c86585443fc30d0dae92676a.
Corrected focused SHA256efe221262a61d722d387779936a24aa7c421408547b5437cfab4b54a5c1546d2;
strict network Clippy SHA25652acb8a6f3eb223b70b58f50a28b7962e07aaf0f8ee120fc2022b3afcd694578;
initial failed journal SHA2567a66e8fbade38f96b3720dd547678abf365d6055a93ce4fcc40b34c8cffd1f94.

Governance SQLite17/delivery12/0warnings and claims0warnings PASS. Locked offline package listing
includes chain.rs, chain/tests.rs and the complete existing MIT notice. Scale500sources/31packages/
4workflows remains high-risk/release-managed/on-demand. Protected working-source/delta review
APPROVE has no blockers, naming blobs compatibilityb350997d, chain7cfb9f48, chaintestsc9b2c56c,
executiontests6b7165eb and transporttests8dfdbad8. Shared identity is not natural-person approval.
Final owner/staged review, exact-local publication review, stable remote exact-source review,
unfiltered terminal native CI and immediate fresh CAS remain required. This is stage Review only;
I293 Active/Claimed, Completion Pending and all parent/activation gates above remain unchanged.

Publication refresh on2026-10-21: main remains4e20ee95; #735 advanced to
7036ddce0b2ca575fa28e29e798bc95a10ea5674. Its complete125-file scope was checked again with no
search_backend/I293/SEARCH-C overlap; #705 head remains5848f899. After successful preflight,
workspace maintenance again removed the temporary toolchain/cache. The final documentation-only
governance refresh initially could not run SQLite metadata because cargo was absent; this is an
environment failure, not a validation pass. Restore checksummed pinned1.99 and repeat the affected
governance/claims/format checks before publication; unchanged source validation above is retained.
Restored-toolchain refresh subsequently exits0: SQLite17/delivery12, governance0warnings,
claims0warnings, cargo fmt --all -- --check and diff whitespace PASS. Independent full staged-tree
review96ce799a APPROVE has no blockers; exact committed head and remote gates follow.

## Merged Fixed Chain / Dormant Direct Plan Binding — 2026-10-21

#742 merged9f9983928b17a3fa2039ce08efd41989ff31fc1a, source1fdb717f/treecd701c9e.
CI3290/run39457212458 attempt1 all6SUCCESS: Windows tools557/0failed/0ignored/0filtered,
mac555/0/0/0 and all10new source-chain tests PASS on each. Actual original socket, SDK default/
coding, standard preflight and every later native gate PASS; LinuxDesktop112/0/0/0. Protected
source6295766323, terminal/freshCAS6296299300 and merged delivery6296316872 record this partial
stage. Actual merge parent4e20ee95/treecd701c9e/main verified; local main fast-forwarded cleanly.

Selection refresh: main9f998392 and effective#712 actor@wjhuang88/Work Slice unchanged. Current
I293/I294/I300 Active retain C/D/Git; I277 Review retains deferred human/device validation and
I302 Review retains#717. I249 Planned deferred; I164 Paused/superseded. I301/I303/I304/I305 stay
Complete/Closed. Historical I146/I147/I150 notes do not reactivate those owners; TEMPLATE excluded.
Open#735head7036ddce0b2ca575fa28e29e798bc95a10ea5674 and#705head5848f8992457c72e13a48b7c2791635b6326453e
are unchanged from the complete125/4-file scope review with no search_backend/I293/SEARCH-C
overlap. #705 stays open/unmerged, #729 closed; no unrelated task is taken over or approved.

Runnable library-only deliverable: explicitly DirectCompatibilityConfiguration owns one immutable
fixed4 scope; preparation clones that configuration scope into a fresh invocation manifest and
retains it with the exact frozen chain and complete canonical Network/Domain profile. Borrowed
accessors and consuming execution prevent callers from supplying a replacement transport/manifest.
Externally supplied live authority and an existing Arc budget are reused unchanged; production
SystemSearchDns is fixed. Test relocation/DNS injection remain cfg(test)-only. No authority is
minted by the plan; no reprepare/header reroll/new clock/reset. Errors contain closed enums only.

Protected design preview APPROVE requires deduplicated exact host facets, full port/method/path
manifest retention, same configuration scope across queries with fresh invocation identity,
mismatch/denial before DNS/send and actual wrapper execution/socket cleanup. Direct-only means
no proxy/address exception/redirect or ambient configuration read, and cannot become an automatic
fallback for unsupported configured proxies. This is private preparation/execution, not official
PreparedNetworkToolInvocation/formatter/permission-lifecycle/root activation or full C acceptance.
Affected refs: SEARCH-001-C-COMPATIBILITY-PLAN.md, fixed-chain and carrier-migration references.
Toolchain/cache were again removed by environment maintenance; checksummed official1.99 restored.
Local focused/strict Clippy/standard preflight, source review, stable candidate, native CI and final
CAS follow. I293 remains Active/Claimed, Completion Pending; #643/#624 stay open. All full C/EP/
parity/D/privacy/router/health/rollback/default activation/release gates remain effective.

Fresh focused locked network transport validation exits0:107passed/0failed/0ignored/227filtered,
plus existing separate child1passed/333filtered. All9new wrapper tests PASS on their first run.
Strict network-test Clippy and full standard preflight remain in progress. Locked offline package
listing includes plan.rs/plan/tests.rs and existing full MIT notice; scale502sources/31packages/
4workflows retains high-risk/release-managed/on-demand. Independent protected working-source
review APPROVE has no blockers: compatibilityc90ef9b7, chainaf63481f, chaintests9859cc2e,
planf41ba010 and plantestseaa1073a. Shared identity is not distinct natural-person approval.

Stable local convergence PASSED on pinned Rust1.99.0. Strict network-test Clippy exits0 and
standard locked release_preflight.sh exits0 with actual passed/terminal0 markers. Workspace
check/strict Clippy/tests/doctests and independent SDK default/coding both PASS. Tools564passed/
0failed/0ignored/0filtered, all9new tests PASS. Only the existing authorized local runtime socket
filter remains; exact-head native CI must execute that original fixture unfiltered. This plan
stage had no source/test failure or retry; previous-stage fixture history above remains intact.
Preflight journal SHA256:
972fe82266dba86b2817784fffe7bd57a976c74527026f9d88733affe43cc1a8.
Focused SHA2564c7704bbdace695bfcd70424846f52e366cf9a2a968d63783a0e39e3897759e6;
strict network Clippy SHA256ac326313660a5088e35f94c3861cc4534ffccdc79b449356771e8b4f77283cc7.
Governance SQLite17/delivery12/0warnings and claims0warnings passed in standard preflight.
Frozen reviewed source blobs above remain unchanged. Final owner synchronization/staged review,
exact-local publication review, remote exact-source review, terminal native CI and immediate
fresh CAS follow. This is stage Review only; parent completion/default activation remain Pending.

## Merged Direct Plan / Dormant Core-Trait Bridge — 2026-10-21

#743 merged181d7584b1d571d2e898915741acb90e16721b0e, source280311a7/tree90cd6070.
CI3292/run39525808164 attempt1 all6SUCCESS: mac tools564/0failed/0ignored/0filtered,
Windows566/0/0/0 and all9new tests PASS on both. Original socket, SDKdefault/coding, fullpreflight
and all later gates PASS; LinuxDesktop112/0/0/0. Protected source6305512030, terminal/freshCAS
6306272569 and merged delivery6306282404 record this partial stage. Actual main/merge parent
9f998392 and exact tree90cd6070 verified; local main fast-forwarded cleanly.

Selection refresh: main181d7584, effective#712 actor@wjhuang88/Work Slice unchanged. I293/I294/I300
Active retain C/D/Git; I277 Review retains deferred human/device validation and I302 Review retains
#717. I249 Planned deferred, I164 Paused/superseded; I301/I303/I304/I305 Complete/Closed remain
closed. Historical I146/I147/I150 notes do not reactivate owners; TEMPLATE excluded. Open#735
head7036ddce0b2ca575fa28e29e798bc95a10ea5674 and#705head5848f8992457c72e13a48b7c2791635b6326453e
remain unchanged from complete125/4-file overlap review. #705 stays open/unmerged, #729 closed;
no unrelated owner is taken over or approved.

Runnable private library deliverable: PreparedCompatibilityInvocation implements existing core
PreparedNetworkToolInvocation, retaining the same owned direct plan, original bounded query and
snippet option plus immutable validated limits. It accepts only external live authority at dispatch
and creates one budget then, never during preparation/approval. TransportLimits extracts existing
bounds validation without clock reads; old TransportBudget::new validation/start behavior remains.
All sources share the same running budget. Final live fencing follows conversion/formatting.

The legacy WebSearch formatter body is shared as a pure function, preserving all existing source
labels/numbering/query/field/snippet/UTF-8 behavior. A crate-private compatibility conversion uses
fixed DuckDuckGo family identity after admitted results; formatting itself grants no authority.
Only successful established display contains query/results. Errors remain closed/content-free,
Debug redacted and continuation parts empty. Registered WebSearch/official root paths are not activated.

Protected design preview APPROVE requires unchanged limits, one dispatch clock, postformat fence,
exact formatter bytes, real permission-state proposal/Once/Session/admit_network with actual
controlled HTTP and Tokio clock resumed before socket setup. Test relocation/DNS remain cfg(test).
No public API/model schema/dependency/permission-policy/default change or implicit proxy substitute.
Affected refs: SEARCH-001-C-COMPATIBILITY-INVOCATION.md, direct-plan and carrier references.
Toolchain/cache were again removed by environment maintenance; checksummed official1.99 restored.
Focused formatter/transport, strict network-test Clippy, full standard preflight, exact-source
review, stable candidate/native CI and final CAS follow. I293 remains Active/Claimed, Completion
Pending; #643/#624 OPEN. Full C/EP/parity/D/privacy/roots/router/health/rollback/default/release gated.

Initial local validation: formatter driver observed1pass/343filtered/exit0; its cold-build journal
has an incomplete tail and is retained as partial, not terminal journal evidence. Focused transport
116passed/0failed/0ignored/228filtered plus separate child1passed/343filtered exits0; strict
network-test Clippy exits0. Initial standard preflight passed workspace check/Clippy/tests/doctests,
tools574/0failed/0ignored/0filtered and all10new tests, including the independent golden formatter.
It then FAILED in SDK compilation: cranelift-codegen0.135.3 archive creation saw a zero-length
object. This is not a full-preflight pass. Failed invocation-preflight.log SHA256:
415889d08e46eef17032667458e5057b87fdd758c623845f5bbdf066026829fa.

Observed cause at the archive boundary: checkout target had reverted to a real directory while
workspace builds used CARGO_TARGET_DIR outside checkout. SDK's explicit --target-dir used the
checkout directory, containing cranelift_codegen-9cf89bebde0b79ba...cgu.00.rcgu.o at0bytes; disk had
20GBfree. The underlying zeroing mechanism is unproven. Root omitted the post-restoration target
link check. The generated612MB failed directory was moved intact to /tmp/talos-i293-sdk-failure.z72U7b;
no source/lock/test was removed. Checkout target now links to /tmp/talos-i293-binding-target,
and the recovery driver asserts exact link resolution before the same standard preflight.
Existing EVOLUTION lesson60 was refined for explicit SDK target paths. Fresh preflight follows;
initial failure stays preserved, no new skip, dependency, timeout or CI relaxation.

After target isolation repair, fresh standard release_preflight.sh exits0 with actual passed and
TALOS_PREFLIGHT_TERMINAL_EXIT=0 markers. Workspace check/strict Clippy/tests/doctests, tools574/
0failed/0ignored/0filtered, all10new tests (9invocation+1formatter) and SDKdefault/coding PASS.
The complete resumed journal independently records golden formatter success; the initial partial
formatter journal is not substituted for it. Resumed preflight SHA256:
21872d4bf91a0cc7aa9c66d795321d88ec1a6ac44c7bf6690746e220233058bc.
Focused SHA25682fef15737901ef9b76ba165147e3705136b467cfd1a34bf6e520f6fdbd8ae0c;
strict network-test Clippy SHA2566b2a2b95e17dac46e64250a50148d1c37ce3b0d7cf656e18f8c8e55bedd0ba64.
Existing sole authorized local runtime socket filter remains; native CI must execute it unfiltered.
Governance SQLite17/delivery12/0warnings and claims0warnings PASS. Locked offline package listing
contains invocation modules and full MIT notice; scale504sources/31packages/4workflows retains
high-risk/release-managed/on-demand. Protected working-source review APPROVE has no blockers,
naming frozen blobs transportdd03d626, compatibilitybc0a4452, plan7ef723c0, plantestseba36c4a,
invocation1e568c32, invocationtestsa9492784 and websearchf39658fa. All7 remain unchanged through
the environment repair/revalidation. Shared identity is not natural-person approval. Final staged/
exact-local review, stable publication, remote source/native terminal review and fresh CAS follow.
This is stage Review only; I293 Active/Claimed, Completion Pending, all parent/activation gates intact.

Publication scope refresh rechecked all125files of#735 and4files of#705 at the unchanged heads.
The only common candidate path is EVOLUTION.md with#735: this stage edits validation lesson60,
while I300 adds separate Git lessons/index entries and does not change that lesson. No shared
implementation or lesson hunk exists. Under ADR-071 this disjoint documentation path is not a
claim conflict or reason to absorb/rebase/approve#735. #705 has no common candidate paths.
