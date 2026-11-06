# Issue #768: Reliable Search Refusal Fixtures

> Document status: Active (atomic proposal; ineffective until claim merge)
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
| Authorization Evidence | Maintainer explicitly requested completing #768; required independent governance and fixture-boundary review before delivery |
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
