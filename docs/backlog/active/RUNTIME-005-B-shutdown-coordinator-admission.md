# RUNTIME-005-B: Bounded Shutdown Coordinator And Admission Fence

**Status**: Active / Claimed (proposed by PR #344; ineffective until merge)

| Field | Value |
|---|---|
| Story ID | RUNTIME-005-B |
| Type | Runtime / Public SDK Story |
| Priority | P0 |
| Status | Active / Claimed (proposed by PR #344; ineffective until merge) |
| Parent Epic | RUNTIME-005 |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | I216 - Active / Claimed (proposed; ineffective until #344 merges) |
| Depends On | RUNTIME-005-A / I214 Complete; ADR-063 Accepted; SESSION-008-B Complete; RUNTIME-001 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline long-task session 2026-08-21 |
| Work Slice | I216/RUNTIME-005-B only: implement ADR-063's runtime-local shutdown coordinator, validated options, cloneable shutdown-only controller, one SDK/actor admission-start arbiter and StartCommitted token boundary, two active-turn policies, one total monotonic deadline, immutable cached redacted report, primary/controller Drop semantics, legacy shutdown compatibility, deterministic race tests, external next-minor migration fixture and directly affected SDK docs. Excludes RUNTIME-005-C finalizer registry/durable closure, TOOL-024, process, permission/sandbox, I189, product UI, dependency, persistence schema, unsafe, version, tag, release and publication. |
| Claimed At | 2026-08-21 |
| Source Issue | #49 |
| Governance Claim PR | #344 |
| Authorization Mode | Independent review |
| Authorization Evidence | The maintainer directed continuation of the active mainline long task after I214 closeout. PR #344 is the atomic claim+activation record and may merge only after exact-head CI, both validators, independent runtime architecture review and merge-time CAS. Shared-account review establishes Agent-role separation only. |
| Implementation PR | Not started |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Proposed claim and activation remain ineffective until finalized PR #344 passes exact-head CI, both validators, independent runtime architecture review and merge-time CAS, then reaches `main`; implementation starts only from that merge or later `main`. |

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

- [ ] Invalid options fail before coordinator or primary-handle access and cannot trigger Drop.
- [ ] Two valid callers racing while Open receive the same accepted plan identifier and immutable
      terminal report; a cancelled waiter does not cancel the runtime-owned shutdown driver.
- [ ] Submit/start/shutdown races have one observable order: pre-fence admission is handled by the
      accepted plan, while post-fence submit returns typed `RuntimeClosing` without enqueueing.
- [ ] Actor preparation paused before start commit performs no model/tool work when shutdown wins;
      when start commit wins, the installed token makes the item active under the selected policy.
- [ ] `FinishCurrent` never starts pending work during grace and uses only the remaining total
      deadline before actor-owned interrupt; `Interrupt` immediately requests the existing
      cancellation/finalization path.
- [ ] Deadline expiry returns promptly with a redacted incomplete report; no prompt, reasoning,
      message, tool data, provider payload, path, credential or arbitrary error text is exposed.
- [ ] Primary Drop can initiate only the default plan while Open; controller Drop is inert; no Drop
      path replaces an already accepted explicit plan or blocks unwinding.
- [ ] Legacy `shutdown(self)` remains source-present, preserves `ActorJoin`, and never reports
      incomplete cleanup as `Ok(())`.
- [ ] Current runtime submission/preview/interrupt/event behavior remains unchanged while Open;
      direct lower-level Session construction remains default-open unless it installs the seam.
- [ ] Focused deterministic race tests, external SDK migration fixture, full locked workspace tests,
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

- Completion Commit: Pending
- A status or closeout commit cannot self-certify this behavior.

## Residual Destination

RUNTIME-005-C remains the sole owner of the frozen ordered finalizer registry, final durable
reconciliation projection and final compatibility closure. Issue #49 remains open through C.
TOOL-024-B remains blocked until all RUNTIME-005 and PERM-006-C complete.
