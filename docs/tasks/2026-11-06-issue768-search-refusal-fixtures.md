# ISSUE-768: Reliable Search Refusal Fixtures

> Document status: Review (local implementation and standard preflight passed; remote gates pending)
> Published plan date: 2026-11-06
> Deliverable: native refusal classification and chain policy regressions reliable under parallel macOS validation, with honest causal evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | #768 search transport test-fixture diagnosis, deterministic refusal/policy regressions, native parallel validation and evidence synchronization; no production transport policy change |
| Claimed At | 2026-11-06 |
| Source Issue | #768 |
| Governance Claim PR | #859 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer explicitly requested completing #768; exact-head independent governance approval recorded in #859, CI42176421804 passed; claim merged as 3f94a2684c4305e44f524a9a390c87d697167fc2 |
| Implementation PR | Not started |
| Last Updated | 2026-11-06 |
| Handoff / Release Condition | Claim and activation effective only on target-branch merge; no release or other search ownership |

## Published baseline

### Scope and preserved history

#768 recorded two macOS parallel preflight failures at `99c828313` and a diagnostic
ConnectionReset/errno54; earlier BrokenPipe observations remain valid. Serial
preflight at `aed967c4322f942a239c4b12cd1ab9747203adfb` passed but was not a repair.
[I308](../iterations/I308-builtin-auto-review.md) remains Complete with its own
scope; this task owns only its independently recorded fixture residual.

Current main baseline `cc5aedd3c220d131f00efa9d288c39ad604eb550` includes I293
implementation #808 (`eb870cdf0a63f45f20d89f1a50eba276f9198643`). That accepted
implementation changed same-source retry and transient send-error classification.
The old Issue wording that every Reset/BrokenPipe is terminal is historical, not
the current production contract. Preserve current permission, limits, retry and
terminal protocol/body error policy. Do not broaden error classification to pass tests.

### Acceptance

1. Establish the failing fixture assumption with causal regression or controlled
   counterexample. Distinguish proven mechanism from unknown historical OS cause.
2. Repair fixture ownership/scheduling or split policy and native classification
   coverage with independent review. Keep actual native typed ConnectionRefused
   evidence, real next-source success, exact request/resolution budgets and
   terminal malformed-response/no-extra-disclosure assertions. No ignored tests,
   acceptance of reset as refusal, retry-to-green, or mock-only replacement.
3. Add a bounded adversarial concurrency regression covering the repaired assumption.
   Tests must fail on the old fixture mechanism where reproducible, not merely
   pass repeatedly after the edit.
4. Run native macOS focused tests and parallel workspace library tests plus standard
   locked preflight without RUST_TEST_THREADS=1. Record actual feature sets and
   test counts. Native CI and xwin/MSVC checks cover changed cross-platform tests.
5. Independent fixture/security-boundary review, exact-head CI and merge CAS pass.
   Synchronize this owner, I308 residual pointer, issue matrix and Board. Close
   #768 only after merged implementation and existing Completion Commit evidence.

### Non-goals and documentation

No production search policy, endpoint/provider admission, permission, dependencies,
release, or unrelated owner changes. No claim that all macOS network behavior is
fully explained. User/operator documentation is this task and the issue matrix;
update relevant testing/reference notes only for changed fixture/evidence contracts.

### Inventory and overlap

Current owner headers: I310 Active (Gemini; retain), I302 Review (Windows PSS;
retain), I277 Review (deferred human/device acceptance; retain), I249 Planned
(dependency pilot; defer), I164 Paused (superseded layout; retain). Other inspected
iteration headers are terminal; no current Blocked header found. Historical
"review"/"blocked" words in Complete outcomes do not reactivate them.
Open #838 Gemini, #849 D2 intake and #851 I317 compatibility-helper proposal retain
their owners. #851 changes documentation only at inspection; its production helper
ownership does not transfer to this task. Recheck overlap before merge. No new
iteration ID is needed for this existing Issue's bounded test-repair task.

### Validation and rollback

Pinned Rust1.99.0, --locked; reuse existing cache where possible. Run targeted
transport tests, workspace parallel library tests, standard release_preflight,
applicable xwin/native CI and both governance validators. Revert only this task's
fixture changes on regression; never change production policy as a workaround.
Current disk available is below1GiB; establish adequate build space before a full
workspace rebuild, preserving unrelated worktrees and source.

## Investigation evidence — 2026-11-06

- Current all-feature tools connection tests:5 passed,0failed,0ignored with8 test
  threads; broader transport:185passed,0failed,0ignored with16threads. Logs:
  `/private/tmp/issue768-current-network-baseline.log` and
  `/private/tmp/issue768-transport-parallel.log`. These are local artifacts, not
  durable CI attachments, and do not prove intermittent failure repaired.
- Initial default-feature command selected0tests; explicitly not validation of
  network fixtures. Corrected command enabled --all-features.
- Controlled local socket counterexample: release reserved listener, bind replacement
  to same address, connect and close accepted socket with zero linger => native
  ECONNRESET54. This proves the release/rebind hazard, not its occurrence in the
  historical workspace failures.
- Rejected alternative: retaining a bound non-listening TCP socket produced a
  timeout (EAGAIN35), not refusal, on this macOS host. Apple XNU tcp_input.c drops
  packets in TCPS_CLOSED; no retained-bound-socket fix is assumed portable.
  Primary source: https://github.com/apple-oss-distributions/xnu/blob/main/bsd/netinet/tcp_input.c
- Rejected stronger reservation claim: an established loopback client source port
  produced native refusal in128parallel Python cases, but a Rust counterexample
  showed std TcpListener (SO_REUSEADDR) can bind that occupied source port on macOS.
  The Python default-bind result does not prove exclusive ownership. Do not ship
  this mechanism as an exclusive-port fixture without resolving that counterexample.
- No production or fixture changes yet; no completed fix or historical root-cause claim.

## Closure ledger

Requested outcome: complete #768. Artifacts: fixture code/regressions, this owner,
I308 pointer, issue matrix, Board and remote Issue. Preserve prior failures and
current production contracts. Remaining diagnosis, implementation, parallel/native
validation, independent review and merge evidence remain here until actually done.

## Implementation checkpoint — 2026-11-06

Claim #859 merged as `3f94a2684c4305e44f524a9a390c87d697167fc2`. Two test files
now separate resolver-injected refusal policy from native HTTP/HTTPS classification.
The next-source winner still performs real HTTP; occupied source0 asserts zero
connections/disclosure. Exact retry/host/budget/wire assertions remain. A low-port
cooperative lease isolates native tests from normal ephemeral allocation, but cannot
exclude arbitrary external binders. Native classification remains strict: no reset,
timeout or retry-until-green accepted as refusal. The old release/rebind mechanism
has a deterministic successful-connect counterexample; historical cause remains unknown.

Initial candidate transport186/186 passed with16threads; independent review found
the new counterexample also needed the low-port lease. That correction is applied;
final parallel workspace validation is running outside the sandbox. Baseline full
workspace failed at two unrelated Git tests: object_layout drift and process
observation EPERM. It is not a successful workspace result. Logs remain under
`/private/tmp/issue768-{fixed-transport,workspace-parallel-baseline,fixed-workspace-parallel}.log`.
No source permission, retry policy or endpoint change is part of this repair.

Final fixture adds8simultaneous admitted chains (4runtime workers,15second bound),
each with an occupied source0 and real source1 winner. Final all-feature transport
suite passed186/186 with16test threads. MSVC cross-check passed using
`cargo xwin check --locked -p talos-tools --features network --tests --target x86_64-pc-windows-msvc`;
this is compilation only, not Windows execution. Independent fixture/security-boundary
static review approved the final diff after the counterexample lease correction.
Reviewer did not execute tests. Exact-head review remains required after commit.

Outside-sandbox parallel `cargo test --locked --workspace --lib` passed exit0,
including Tools1116/1116, before the8-chain strengthening. Final standard preflight
is running with that strengthening and no RUST_TEST_THREADS override; it must pass
before claiming final full-workspace evidence. The default-feature0test attempt,
sandbox failure and prototype counterexamples remain historical, not successes.

## Stable candidate validation — 2026-11-06

Final `CARGO_TARGET_DIR=/private/tmp/talos-i313-target CARGO_INCREMENTAL=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1
./scripts/release_preflight.sh` completed exit0, ending `release preflight: passed`.
No RUST_TEST_THREADS override: final workspace tests ran with normal parallelism,
Tools1116/1116 passed and both default/coding external SDK executions passed.
The log explicitly includes the8-chain occupied-source test, actual HTTP/HTTPS
refusal and release/rebind counterexample. Check/Clippy and both governance
validators passed. `/private/tmp/issue768-preflight.log` is local evidence;
remote exact-head/native CI and merge remain pending. The two earlier unrelated
Git failures did not recur; this does not establish their original cause.

The user authorized rebuilding unused cache. Removed examples/old Windows cache
and completed task-specific xwin outputs; source/worktree preserved. One initial
cargo-clean attempt encountered an already-removed examples directory (exit101),
not a test failure. Cross-check logs retained. No fixture test was ignored or
serialized globally, and no source change followed this final successful preflight.
