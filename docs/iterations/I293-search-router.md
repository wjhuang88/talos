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
| Implementation PR | #681 / #685 (merged partial evidence); #696 (Review; offline experiment) |
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
| Last Updated | 2026-10-10 |
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
