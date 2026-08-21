# Iteration I216: Bounded Shutdown Coordinator And Admission Fence

> Document status: Active / Claimed
> Published plan date: 2026-08-21
> Planned objective: implement only ADR-063 RUNTIME-005-B as a bounded, independently runnable
> runtime SDK shutdown coordinator without finalizer-registry, permission, product or release work.
> MVP deliverable: an external `talos-runtime` fixture and deterministic workspace tests exercise
> one bounded cached shutdown report across idle, active, concurrent and submit/start races.

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
| Authorization Evidence | The maintainer directed continuation of the active mainline long task after I214 closeout. PR #344 exact head `e0f572a0` passed CI `32454558957`, both validators, independent runtime architecture review `5366165116` and merge-time CAS, then merged as `2016acce`. Shared-account review establishes Agent-role separation only. |
| Implementation PR | Not started |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Claim and activation are effective through PR #344 merge `2016acce`. Converge the complete published slice locally before one stable implementation PR; completion requires exact-head Unix/Windows CI, independent runtime architecture review, merge-time CAS and a later status closeout naming pre-existing implementation commit evidence. |

## Published Baseline

### Selected Story

- `RUNTIME-005-B` — `docs/backlog/active/RUNTIME-005-B-shutdown-coordinator-admission.md`

### Dependencies

- RUNTIME-005-A / I214 is Complete/Closed at Completion Commit `6719c876`.
- ADR-063 is Accepted through PR #338 merge `fc70e396`, CI `32449605985`, independent architecture
  review `5365529351` and merge-time CAS.
- SESSION-008-B and RUNTIME-001 are Complete.

### Runnable Deliverable

A rebuilt SDK fixture can start a runtime, clone a shutdown-only controller, select
`FinishCurrent` or `Interrupt`, race submit/start/shutdown deterministically, cancel a waiter, and
receive one bounded immutable redacted report while the legacy consuming shutdown API remains
available.

### Scope

- Runtime-local shutdown coordinator and validated options.
- One SDK/actor admission-start arbiter with explicit no-await/no-external-work discipline.
- Actor `StartCommitted` boundary with cancellation token installed before external work.
- First-valid-plan arbitration, two active policies, one monotonic deadline and cached report.
- Primary/controller Drop semantics and legacy wrapper behavior.
- Public SDK compatibility migration note, external fixture, tests and affected SDK documentation.

### Exclusions

- RUNTIME-005-C registry/finalizers and final durable reconciliation closure.
- TOOL-024, process, permission, sandbox, I189, product UI, dependency, persistence schema,
  `unsafe`, version, tag, release or publication changes.
- Serialized `SessionOp::Shutdown` changes or a second turn-finalization owner.

### Acceptance

- [ ] Invalid options cannot reach coordinator state or primary Drop.
- [ ] Concurrent valid callers share one plan/report and caller cancellation does not stop cleanup.
- [ ] Submit/start/shutdown races are linearized by one arbiter without a lock across await or
      external work.
- [ ] Both active policies respect one total deadline and preserve ADR-058 ownership.
- [ ] Reports and errors are typed, bounded and display-safe; incomplete cleanup is never `Ok`.
- [ ] Legacy/open-state behavior, direct Session compatibility and serialized protocol remain
      unchanged.
- [ ] Focused race tests, external SDK fixture, full locked validation, docs and Unix/Windows CI
      pass at one stable implementation head.

### Documentation Targets

- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- root `README.md` and `README.zh-CN.md` shutdown examples when the code exists
- migration/changelog note for the next-minor public error boundary
- RUNTIME-005/I216 owner evidence and derived views

### Validation

- focused `talos-agent`/`talos-runtime` locked tests plus deterministic race fixtures
- external public API/migration fixture
- `./scripts/release_preflight.sh`
- explicit-base governance and Collaboration Claim validators
- exact-head Unix/Windows CI and independent runtime architecture review

### Rollback

Reject or revert the B implementation, preserve the current consuming unbounded shutdown behavior,
leave RUNTIME-005-C and TOOL-024 blocked, and keep ADR-063 as the accepted target contract. Do not
partially ship only the public types without the shared arbiter and race evidence.

## Exact-Main Non-Terminal Inventory — 2026-08-21

Baseline: `main@3c98f3151c6b836a3a522e3a8bd7059943fc23cb`; local and `origin/main` match,
the primary worktree is clean, and only archival Draft PRs #120/#121 are open.

| State | Iterations | Disposition |
|---|---|---|
| Active | None | I216 may be proposed only; it remains unactivated until this finalized claim reaches `main`. |
| Review / Claimed | I197, I198, I201, I210 | Preserve their completed implementation evidence and separately owned corrective residuals; no TUI/provider/skill authority transfers. |
| Planned / Claimed | I189, I213 | Keep I189 deliberately unactivated; I213 remains in the independent Dashboard lane. |
| Planned / Unclaimed | I206, I207, I208, I216 | Preserve steering order; select only I216 for the non-overlapping runtime slice. |
| Blocked | None at iteration-document level | RUNTIME-005-C, PERM-006-B/C and TOOL-024-B/C/D retain backlog-owner blockers. |
| Paused | I164 | Preserve supersession; do not resume. |

Historical `stash@{0}`/`stash@{1}` remain untouched. No open PR or effective claim overlaps
RUNTIME-005-B. The I216 proposal changes only governance records and creates no implementation
branch or code authority before merge.

## Completion Evidence

- Completion Commit: Pending
- The claim/activation record and later status-only commits cannot self-certify implementation.

## 2026-08-21 Execution Checkpoint

PR #344 exact head `e0f572a0` passed CI `32454558957`, both governance validators, independent
runtime architecture review `5366165116` and merge-time CAS, then merged as `2016acce`. The atomic
claim and activation are effective, and implementation started from that exact merge.

The current locally converged stage implements and tests the B-only coordinator/admission contract,
including deterministic reserve/fence/start-commit order, both active policies, caller-cancellation
independence, Drop/legacy behavior, deadline containment, durable-failure projection, report
redaction and the external fallback-match migration fixture. It is not yet Review or Complete:
publication as one stable implementation PR, exact-head Windows/Unix CI and independent runtime
architecture review remain required. Full local preflight has passed, and the Published Baseline
above remains unchanged.
