# RUNTIME-005-C: Ordered Finalizer Registry And Durable Closure

**Status**: Complete / Closed

| Field | Value |
|---|---|
| Story ID | RUNTIME-005-C |
| Type | Runtime / Public SDK Story |
| Priority | P0 |
| Status | Complete / Closed |
| Parent Epic | RUNTIME-005 |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | I217 - Complete / Closed |
| Depends On | RUNTIME-005-B / I216 Complete; ADR-063 Accepted; SESSION-008-B Complete; RUNTIME-001 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline long-task session 2026-08-21 |
| Work Slice | I217/RUNTIME-005-C only: implement ADR-063's build-time frozen runtime-owned finalizer registry, fixed identifiers and unique order, one-global-deadline execution with per-finalizer caps, typed failure/panic/timeout/not-run outcomes, durable-reconciliation-before-finalizer ordering, immutable redacted report closure, legacy shutdown compatibility, deterministic runtime-owned test finalizers and directly affected SDK docs. Excludes public arbitrary third-party callback/plugin registration, TOOL-024, process, permission/sandbox, I189, product UI, dependency, persistence schema/writer, global bus, unsafe, version, tag, release and publication. |
| Claimed At | 2026-08-21 |
| Source Issue | #49 |
| Governance Claim PR | #347 |
| Authorization Mode | Independent review |
| Authorization Evidence | The maintainer directed continuation of the mainline long task after I216 closeout. PR #347 exact head `3253abb5` passed CI `32471570214`, both validators, independent runtime architecture review `5368607605` and merge-time CAS, then merged as `bb6f5ed9`. Shared-account review establishes Agent-role separation only. |
| Implementation PR | #348 |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Closed through implementation commit `44e840d7` and PR #348 merge `6e5fa8c3` after exact-head CI `32475052535`, independent runtime architecture review `5369328072` and merge-time CAS. A future third-party finalizer extension requires a separate decision and claim. |

## Goal And User Value

Finish the bounded shutdown contract so Rust embedders receive a truthful, immutable report only
after durable Session custody is reconciled and every configured Talos-owned resource finalizer has
run once in deterministic order or received an explicit bounded non-success outcome.

## Runnable Deliverable

Workspace and external SDK fixtures build a runtime with the reviewed Talos-owned finalizer set,
shut it down through the structured and legacy entrypoints, and observe durable reconciliation
before a fixed ordered list of typed finalizer outcomes. Deterministic runtime-owned test
finalizers prove order, exactly-once execution, continuation after failure/panic, timeout
containment and exhaustion of the original global deadline.

This is a runtime SDK infrastructure slice with an observable `ShutdownReport`; it does not claim a
new end-user product surface or authorize arbitrary embedder callbacks.

## Scope

- Add one registry populated by reviewed runtime composition before `RuntimeBuilder::build` and
  frozen as part of successful build.
- Use code-owned fixed identifiers and unique order values; duplicate identifiers or orders fail
  build before the runtime starts.
- Run every registered finalizer at most once in ascending order after actor-owned active-turn
  terminalization and durable pending reconciliation.
- Give each finalizer only the lesser of its configured cap and the remaining ADR-063 global
  deadline; no stage, timeout, panic handler or report assembly resets that deadline.
- Catch and type finalizer failure and panic, contain timeout/cancellation fail-closed, continue
  later finalizers only while global time remains, and mark unstarted entries
  `NotRunDeadline`.
- Extend the immutable cached report with fixed identifiers and closed outcome categories only;
  keep prompts, reasoning, messages, tool data, paths, secrets and arbitrary error text absent.
- Close the durable-reconciliation projection and legacy `shutdown(self)` compatibility behavior
  without creating a second Session finalizer or durable writer.
- Add deterministic runtime-owned test finalizers, compatibility fixtures and directly affected
  SDK/README documentation.

## Exclusions

- No public API for arbitrary third-party callbacks, plugins, caller-supplied identifiers or
  caller-supplied display/error strings.
- No TOOL-024 supervisor registration, process spawning, permission/sandbox policy, I189/PERM-006
  activation, Desktop/Dashboard/TUI behavior or global event bus.
- No new persistence schema, durable writer, transcript owner, replay behavior or alternate
  ADR-058 turn-finalization path.
- No new dependency, `unsafe`, workspace version, tag, release or crates.io publication.

## Acceptance Criteria

- [x] Runtime construction freezes the reviewed runtime-owned finalizer set; duplicate fixed
      identifiers or order values fail construction, and no post-build registration path exists.
- [x] Finalizers run exactly once in ascending order only after the Session actor reports active
      terminalization and durable reconciliation.
- [x] A typed failure or caught panic is recorded without free text and later finalizers continue
      while the original global deadline has time remaining.
- [x] A timed-out finalizer is cancelled and contained fail-closed; later entries run only with
      remaining global budget, and unstarted entries are `NotRunDeadline`.
- [x] Per-finalizer caps shorten but never extend the one ADR-063 monotonic deadline; deterministic
      tests prove the clock is not reset between stages.
- [x] Every caller receives the same immutable cached redacted report containing only fixed
      finalizer identifiers and closed outcome categories.
- [x] Durable reconciliation failure remains authoritative and typed; cleanup can continue safely
      but no incomplete durable/finalizer outcome is reported as clean shutdown.
- [x] Legacy `RuntimeHandle::shutdown(self)` remains source-compatible, preserves `ActorJoin`, and
      maps incomplete structured cleanup to bounded `ShutdownIncomplete` rather than `Ok(())`.
- [x] Existing idle/active/concurrent/submit-start/Drop/Session durable regressions and an external
      SDK fixture pass together with finalizer order/failure/panic/timeout/deadline fixtures.
- [x] Full locked preflight, Unix/Windows exact-head CI, redaction review and independent runtime
      architecture review pass at one stable implementation head.

## Validation Plan

- `cargo fmt --all -- --check`
- focused locked `talos-runtime` and `talos-agent` tests, including deterministic runtime-owned
  finalizer fixtures and durable-order probes
- external supported-SDK and legacy migration fixtures
- `./scripts/release_preflight.sh`
- both governance validators with explicit `origin/main` base
- exact-head Unix and Windows workspace CI
- independent runtime architecture review of frozen ownership, order/exactly-once behavior,
  cancellation containment, one-deadline accounting, durable order, redaction and compatibility

## Completion Evidence

- Completion Commit: `44e840d73370c94fca1e5e7a8d1faa7fde924f0c`
- Implementation PR: #348, merged as `6e5fa8c3bd95f938a7adc14ea8b9aa90bc4d7258`
- Exact-head CI: `32475052535` at `0921eb0c2c978d658eb3c34fbcf801cb738f7518`
- Independent runtime architecture review: `5369328072`, APPROVE at the same exact head
- Merge-time CAS: passed with `origin/main@bb6f5ed9`, only #348 plus archival Drafts #120/#121
- This closeout status commit cannot self-certify the implementation.

## Residual Destination

Issue #49 closed after this evidence-bearing closeout merged as `1503ca21`. A later
third-party finalizer extension requires its own public API, panic, identifier, semver and
resource-containment decision. TOOL-024-B remains blocked until all RUNTIME-005 and PERM-006-C are
Complete.

## 2026-08-21 Claim Effective And Local Convergence

PR #347 exact head `3253abb5` passed CI `32471570214`, both governance validators, independent
runtime architecture review `5368607605` and merge-time CAS, then merged as `bb6f5ed9`. I217's
claim and activation are effective, and implementation started from that exact merge.

The locally converged stage freezes and validates the internal Talos-owned registry at build,
orders durable reconciliation before bounded finalizers and actor join, shares ADR-063's original
deadline, contains failure/panic/timeout into fixed redacted outcomes, and preserves the existing
legacy wrapper. Deterministic finalizer tests passed three consecutive runs; both external SDK
fixture modes and the full locked release preflight passed. The default production registry is
intentionally empty, and no public arbitrary callback registration surface was added. I217 remains
Active pending publication as one stable implementation PR, exact-head Unix/Windows CI,
independent runtime architecture review and merge-time CAS.

## 2026-08-21 Stable Review Submission

Implementation commit `44e840d7` was created directly from claim merge `bb6f5ed9` after local
convergence and full locked preflight passed. Draft PR #348 was opened only to obtain and backfill
its number; I217 now moves to Review / Claimed. The metadata backfill adds no implementation
behavior. Exact-head Unix/Windows CI, both validators, independent runtime architecture review and
merge-time CAS remain required, and this implementation PR cannot mark the iteration Complete.

## 2026-08-21 Completion

PR #348 exact head `0921eb0c` passed all five jobs in CI `32475052535`. Independent runtime
architecture review `5369328072` approved that exact head after source inspection and repeated
focused/external fixture execution. Merge-time CAS confirmed `origin/main@bb6f5ed9`, unchanged
owner/claim/dependencies, open Issue #49 and no overlapping non-archival PR; #348 then merged as
`6e5fa8c3`.

I217/RUNTIME-005-C is Complete/Closed at pre-existing implementation commit `44e840d7`; this
status-only closeout cannot cite itself. The initial production registry remains intentionally
empty and no public arbitrary finalizer registration API was introduced. RUNTIME-005's A/B/C chain
is now complete. Issue #49 may close only after this closeout reaches `main`; TOOL-024-B remains
blocked on PERM-006-C, and no I189, TOOL-024 implementation, release or publication authority
transfers.

## 2026-08-21 Remote Issue Reconciliation

Closeout merge `1503ca21` reached `main`, and remote Issue #49 was then closed as Completed.
TOOL-024-B remains blocked on PERM-006-C and separate effective authority; no implementation
authority transfers from this reconciliation.
