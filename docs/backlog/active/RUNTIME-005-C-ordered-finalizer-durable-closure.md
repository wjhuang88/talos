# RUNTIME-005-C: Ordered Finalizer Registry And Durable Closure

**Status**: Active / Claimed (proposed by PR #347; ineffective until merge)

| Field | Value |
|---|---|
| Story ID | RUNTIME-005-C |
| Type | Runtime / Public SDK Story |
| Priority | P0 |
| Status | Active / Claimed (proposed by PR #347; ineffective until merge) |
| Parent Epic | RUNTIME-005 |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | I217 - Active / Claimed (proposed; ineffective until #347 merges) |
| Depends On | RUNTIME-005-B / I216 Complete; ADR-063 Accepted; SESSION-008-B Complete; RUNTIME-001 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline long-task session 2026-08-21 |
| Work Slice | I217/RUNTIME-005-C only: implement ADR-063's build-time frozen runtime-owned finalizer registry, fixed identifiers and unique order, one-global-deadline execution with per-finalizer caps, typed failure/panic/timeout/not-run outcomes, durable-reconciliation-before-finalizer ordering, immutable redacted report closure, legacy shutdown compatibility, deterministic runtime-owned test finalizers and directly affected SDK docs. Excludes public arbitrary third-party callback/plugin registration, TOOL-024, process, permission/sandbox, I189, product UI, dependency, persistence schema/writer, global bus, unsafe, version, tag, release and publication. |
| Claimed At | 2026-08-21 |
| Source Issue | #49 |
| Governance Claim PR | #347 |
| Authorization Mode | Independent review |
| Authorization Evidence | The maintainer directed continuation of the mainline long task after I216 closeout. PR #347 is the atomic claim+activation record and may merge only after exact-head CI, both validators, independent runtime architecture review and merge-time CAS. Shared-account review establishes Agent-role separation only. |
| Implementation PR | Not started |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Proposed claim and activation remain ineffective until finalized PR #347 passes exact-head CI, both validators, independent runtime architecture review and merge-time CAS, then reaches `main`; implementation starts only from that merge or later `main`. |

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

- [ ] Runtime construction freezes the reviewed runtime-owned finalizer set; duplicate fixed
      identifiers or order values fail construction, and no post-build registration path exists.
- [ ] Finalizers run exactly once in ascending order only after the Session actor reports active
      terminalization and durable reconciliation.
- [ ] A typed failure or caught panic is recorded without free text and later finalizers continue
      while the original global deadline has time remaining.
- [ ] A timed-out finalizer is cancelled and contained fail-closed; later entries run only with
      remaining global budget, and unstarted entries are `NotRunDeadline`.
- [ ] Per-finalizer caps shorten but never extend the one ADR-063 monotonic deadline; deterministic
      tests prove the clock is not reset between stages.
- [ ] Every caller receives the same immutable cached redacted report containing only fixed
      finalizer identifiers and closed outcome categories.
- [ ] Durable reconciliation failure remains authoritative and typed; cleanup can continue safely
      but no incomplete durable/finalizer outcome is reported as clean shutdown.
- [ ] Legacy `RuntimeHandle::shutdown(self)` remains source-compatible, preserves `ActorJoin`, and
      maps incomplete structured cleanup to bounded `ShutdownIncomplete` rather than `Ok(())`.
- [ ] Existing idle/active/concurrent/submit-start/Drop/Session durable regressions and an external
      SDK fixture pass together with finalizer order/failure/panic/timeout/deadline fixtures.
- [ ] Full locked preflight, Unix/Windows exact-head CI, redaction review and independent runtime
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

- Completion Commit: Pending
- A claim, activation, review or status-only commit cannot self-certify implementation.

## Residual Destination

Issue #49 remains open until this child is implemented, independently reviewed and closed. A later
third-party finalizer extension requires its own public API, panic, identifier, semver and
resource-containment decision. TOOL-024-B remains blocked until all RUNTIME-005 and PERM-006-C are
Complete.
