# RUNTIME-005-B: Bounded Shutdown Coordinator And Admission Fence

**Status**: Complete / Closed

| Field | Value |
|---|---|
| Story ID | RUNTIME-005-B |
| Type | Runtime / Public SDK Story |
| Priority | P0 |
| Status | Complete / Closed |
| Parent Epic | RUNTIME-005 |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | I216 - Complete / Closed |
| Depends On | RUNTIME-005-A / I214 Complete; ADR-063 Accepted; SESSION-008-B Complete; RUNTIME-001 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline long-task session 2026-08-21 |
| Work Slice | I216/RUNTIME-005-B only: implement ADR-063's runtime-local shutdown coordinator, validated options, cloneable shutdown-only controller, one SDK/actor admission-start arbiter and StartCommitted token boundary, two active-turn policies, one total monotonic deadline, immutable cached redacted report, primary/controller Drop semantics, legacy shutdown compatibility, deterministic race tests, external next-minor migration fixture and directly affected SDK docs. Excludes RUNTIME-005-C finalizer registry/durable closure, TOOL-024, process, permission/sandbox, I189, product UI, dependency, persistence schema, unsafe, version, tag, release and publication. |
| Claimed At | 2026-08-21 |
| Source Issue | #49 |
| Governance Claim PR | #344 |
| Authorization Mode | Independent review |
| Authorization Evidence | The maintainer directed continuation of the active mainline long task after I214 closeout. PR #344 exact head `e0f572a0` passed CI `32454558957`, both validators, independent runtime architecture review `5366165116` and merge-time CAS, then merged as `2016acce`. Shared-account review establishes Agent-role separation only. |
| Implementation PR | #345 |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Closed through implementation commit `c123328d` and PR #345 merge `020de694`. RUNTIME-005-C is Ready/Unclaimed and requires a separate child owner, runnable iteration and effective claim; Issue #49 remains open. |

## Goal And User Value

Give Rust embedders one bounded and repeatable shutdown API that closes new admission, deterministically
orders actor start against shutdown, applies the selected active-turn policy, and returns the same
redacted terminal report to every caller instead of hanging indefinitely or racing a second plan.

## Runnable Deliverable

An external `talos-runtime` fixture and the workspace tests can build a runtime, obtain a cloneable
shutdown controller, exercise idle/active/concurrent/submit-start races under both policies, cancel
one waiting caller without cancelling shutdown, and observe one bounded cached report. Existing
`RuntimeHandle::shutdown(self)` remains available and maps an incomplete result to a typed error.

## Scope

- Add the runtime-local monotonic `Open -> Closing(plan) -> Closed(report)` coordinator.
- Add validated private-field shutdown options and the two ADR-063 active-turn policies.
- Add a cloneable shutdown-only controller plus borrowing structured shutdown entrypoints.
- Share one admission/start arbiter across runtime SDK submission, shutdown fencing and the Session
  actor pending-to-`StartCommitted` transition.
- Reserve bounded channel capacity before the arbiter; never await or perform provider/tool,
  compaction, persistence or other external work while holding it.
- Install the turn cancellation token at start commit and make post-commit asynchronous stages
  observe it.
- Enforce one monotonic total deadline, first-valid-plan arbitration, caller-wait cancellation
  independence and immutable cached redacted reports.
- Preserve the consuming legacy shutdown entrypoint and define primary versus controller Drop
  behavior exactly as ADR-063 requires.
- Add the next-minor migration note and an external fixture for the non-exhaustive public error
  transition; do not change workspace version, tag or publication state.
- Update the supported runtime SDK reference and directly affected README examples.

## Exclusions

- No RUNTIME-005-C finalizer registry, finalizer callback API, ordered finalizer execution or
  finalizer panic/timeout handling.
- No new durable writer, persistence format/schema migration, alternate ADR-058 finalization path,
  or replay of model/tool side effects.
- No TOOL-024 supervisor, process spawning, permission/sandbox policy, I189/PERM-006 activation,
  Desktop/Dashboard/TUI behavior, dependency addition, `unsafe`, release, tag or publication.
- No change to serialized `SessionOp::Shutdown`, normal pre-closing event order, retry policy,
  provider behavior or the meaning of successful `submit` beyond admission acceptance.

## Acceptance Criteria

- [x] Invalid options fail before coordinator or primary-handle access and cannot trigger Drop.
- [x] Two valid callers racing while Open receive the same accepted plan identifier and immutable
      terminal report; a cancelled waiter does not cancel the runtime-owned shutdown driver.
- [x] Submit/start/shutdown races have one observable order: pre-fence admission is handled by the
      accepted plan, while post-fence submit returns typed `RuntimeClosing` without enqueueing.
- [x] Actor preparation paused before start commit performs no model/tool work when shutdown wins;
      when start commit wins, the installed token makes the item active under the selected policy.
- [x] `FinishCurrent` never starts pending work during grace and uses only the remaining total
      deadline before actor-owned interrupt; `Interrupt` immediately requests the existing
      cancellation/finalization path.
- [x] Deadline expiry returns promptly with a redacted incomplete report; no prompt, reasoning,
      message, tool data, provider payload, path, credential or arbitrary error text is exposed.
- [x] Primary Drop can initiate only the default plan while Open; controller Drop is inert; no Drop
      path replaces an already accepted explicit plan or blocks unwinding.
- [x] Legacy `shutdown(self)` remains source-present, preserves `ActorJoin`, and never reports
      incomplete cleanup as `Ok(())`.
- [x] Current runtime submission/preview/interrupt/event behavior remains unchanged while Open;
      direct lower-level Session construction remains default-open unless it installs the seam.
- [x] Focused deterministic race tests, external SDK migration fixture, full locked workspace tests,
      strict Clippy, docs and Unix/Windows exact-head CI pass.

## Validation Plan

- `cargo fmt --all -- --check`
- focused locked tests for `talos-agent` and `talos-runtime`, including deterministic pause hooks at
  reserve/fence/start-commit/active completion/deadline boundaries
- external fixture proving the public structured API and required fallback match for
  `RuntimeError`
- `./scripts/release_preflight.sh`
- both governance validators with an explicit `origin/main` base
- exact-head Unix and Windows workspace CI
- independent runtime architecture review of arbiter/await discipline, cancellation, Drop,
  deadline, redaction and compatibility claims

## Completion Evidence

- Completion Commit: `c123328d8699b4bd4990603b639578930f29ba4e`
- Implementation PR: #345, merged as `020de694d056ad580d8d2f13fb65f2369bdd73db`
- Exact-head CI: `32459530911` at `abf8d0dae44be44d1bb8307699f7c0523019f61a`
- Independent runtime architecture review: `5367434951`, APPROVE at the same exact head
- Merge-time CAS: passed with `origin/main@2016acce`, one non-archival open PR and Issue #49 open
- A status or closeout commit cannot self-certify this behavior.

## Residual Destination

RUNTIME-005-C remains the sole owner of the frozen ordered finalizer registry, final durable
reconciliation projection and final compatibility closure. Issue #49 remains open through C.
TOOL-024-B remains blocked until all RUNTIME-005 and PERM-006-C complete.

## 2026-08-21 Claim Activation And Local Implementation

PR #344 exact head `e0f572a0` passed CI `32454558957`, both governance validators, independent
runtime architecture review `5366165116` and merge-time CAS, then merged as `2016acce`. The I216
claim and activation are therefore effective. Implementation started from that exact merge in the
isolated `feat/i216-runtime005b-shutdown` worktree and remains inside the published B slice.

Current local convergence includes the shared SDK/actor admission-start seam, validated options,
cloneable controller, bounded runtime-owned driver, typed redacted report, legacy/Drop behavior,
deterministic race tests, durable-failure coverage, the external migration fixture and SDK docs.
This is not completion evidence: no implementation PR or exact-head remote validation exists yet,
and RUNTIME-005-C, I189, TOOL-024, release and publication remain unauthorized.

## 2026-08-21 Stable Review Submission

Implementation commit `c123328d` was created directly from claim merge `2016acce` after one local
convergence cycle. Full release preflight, focused agent/runtime suites, default and `coding`
external SDK fixtures, explicit-base validators, YAML/diff/EOF checks and Published Baseline
preservation passed before the first push. Draft PR #345 was then opened only to obtain and
backfill its number; this owner now moves to Review. Exact-head CI, independent runtime architecture
review, merge-time CAS and a later evidence-only closeout remain pending. No C, TOOL-024, I189,
release or publication authority transfers.

## 2026-08-21 Completion

PR #345 exact head `abf8d0da` passed CI `32459530911` with all five jobs, including Unix full
preflight and Windows workspace validation. Independent runtime architecture review `5367434951`
approved that exact head after source inspection and focused reruns. Merge-time CAS found
`origin/main@2016acce`, no overlapping non-archival PR, unchanged ownership and open Issue #49;
the PR then merged as `020de694`.

RUNTIME-005-B and I216 are Complete/Closed at pre-existing implementation commit `c123328d`. The
reviewer-recorded Tokio `time` change enables a feature on the already present dependency for the
single deadline; it adds no package and leaves `Cargo.lock` unchanged. C becomes Ready/Unclaimed
only. No C, TOOL-024, I189, release or publication implementation is authorized by this closeout.
