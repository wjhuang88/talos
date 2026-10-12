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
| Implementation PR | #681 / #685 / #696 / #706 / #707 / #708 (merged partial evidence); endpoint-authority architecture packet Review (PR not yet assigned) |
| Completion | Pending |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-C Talos-owned SearchRouter behavior and deterministic characterization only |
| Claimed At | 2026-10-06 |
| Source Issue | #643 |
| Governance Claim PR | #664 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR; exact-head CI, governance validators, remote Issue reconciliation and merge-time CAS required. |
| Last Updated | 2026-10-12 |
| Handoff / Release Condition | Claim effective after #664 merge `d801790454e252699684253166984662ea9dab6a`; implementation remains bounded by ADR-085 and this iteration. |

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
