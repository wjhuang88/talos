# Iteration I217: Ordered Finalizer Registry And Durable Closure

> Document status: Review / Claimed
> Published plan date: 2026-08-21
> Planned objective: implement only ADR-063 RUNTIME-005-C as a bounded, independently runnable
> runtime SDK finalization slice without third-party callbacks, TOOL-024, permission, product or
> release work.
> MVP deliverable: structured and legacy shutdown fixtures observe durable reconciliation followed
> by one fixed ordered list of typed finalizer outcomes under the original global deadline.

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
| Authorization Evidence | The maintainer directed continuation of the mainline long task after I216 closeout. PR #347 exact head `3253abb5` passed CI `32471570214`, both validators, independent runtime architecture review `5368607605` and merge-time CAS, then merged as `bb6f5ed9`. Shared-account review establishes Agent-role separation only. |
| Implementation PR | #348 |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Claim and activation are effective through PR #347 merge `bb6f5ed9`. Implementation PR #348 is in Review at implementation commit `44e840d7`; require exact-head CI, both validators, independent runtime architecture review and merge-time CAS. Completion requires a later closeout naming this pre-existing implementation commit. |

## Published Baseline

### Selected Story

- `RUNTIME-005-C` — `docs/backlog/active/RUNTIME-005-C-ordered-finalizer-durable-closure.md`

### Dependencies

- RUNTIME-005-B / I216 is Complete/Closed at Completion Commit `c123328d`.
- PR #345 merged as `020de694` after exact-head CI `32459530911`, independent runtime review
  `5367434951` and merge-time CAS.
- ADR-063 is Accepted; SESSION-008-B and RUNTIME-001 are Complete.

### Runnable Deliverable

Workspace and external SDK fixtures shut down a runtime through structured and legacy entrypoints,
observe actor-owned durable reconciliation before finalizers, and receive one immutable report with
fixed ordered typed outcomes. Runtime-owned deterministic finalizers prove freeze, duplicate
rejection, exact-once order, failure/panic continuation, timeout containment and the non-resetting
global deadline.

### Scope

- Frozen build-time registry for reviewed runtime-owned finalizers only.
- Fixed identifiers, unique order, per-finalizer cap and cancellation-safe fail-closed containment.
- Durable reconciliation before exactly-once ordered finalizer execution.
- Typed redacted cached report closure and legacy wrapper compatibility.
- Deterministic finalizer/durable/deadline regressions, external fixture and directly affected SDK
  documentation.

### Exclusions

- Public arbitrary third-party callback/plugin registration or caller-defined identifiers/text.
- TOOL-024, process, permission, sandbox, I189, product UI, dependency, persistence schema/writer,
  global bus, `unsafe`, version, tag, release or publication.
- A second turn finalizer, transcript owner or side-effect replay path.

### Acceptance

- [ ] Build freezes a valid runtime-owned registry and rejects duplicate identifier/order values.
- [ ] Durable reconciliation completes or records a typed failure before finalizers start.
- [ ] Finalizers run at most once in ascending order and failures/panics do not stop later entries
      while budget remains.
- [ ] Timeout/cancellation is contained fail-closed; per-entry caps do not reset or extend the
      original monotonic deadline; unstarted entries are `NotRunDeadline`.
- [ ] The shared cached report is immutable and exposes no caller text, arbitrary error string,
      secret or other sensitive payload channel.
- [ ] Structured and legacy shutdown behavior, actor-owned ADR-058 outcomes and Session durable
      compatibility remain intact.
- [ ] Focused fixtures, full locked preflight, docs, Unix/Windows exact-head CI and independent
      runtime architecture review pass at one stable implementation head.

### Documentation Targets

- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- `README.md` and `README.zh-CN.md` shutdown contract sections when behavior exists
- next-minor migration/changelog note if the public report projection changes
- RUNTIME-005/I217 owner evidence, Issue #49 reconciliation and derived views

### Validation

- focused locked `talos-runtime`/`talos-agent` tests and deterministic finalizer fixtures
- external supported-SDK and legacy compatibility fixtures
- `./scripts/release_preflight.sh`
- explicit-base governance and Collaboration Claim validators
- exact-head Unix/Windows CI and independent runtime architecture review

### Rollback

Reject or revert the C implementation as one slice, retain the completed B coordinator/report
baseline, keep Issue #49 and TOOL-024-B blocked, and do not ship public finalizer registration or a
partial registry without order, containment and deadline evidence.

## Exact-Main Non-Terminal Inventory — 2026-08-21

Baseline: `main@e88a23477d8d4b57ca7b9070d886cfe389326db2`; local and `origin/main` match,
the primary worktree is clean, and only archival Draft PRs #120/#121 are open.

| State | Iterations | Disposition |
|---|---|---|
| Active | None | I217 may be proposed only; it remains unactivated until the finalized claim reaches `main`. |
| Review / Claimed | I197, I198, I201, I210 | Preserve their implementation evidence and separately owned human/corrective residuals; no TUI/provider/skill authority transfers. |
| Planned / Claimed | I189, I213 | Keep I189 deliberately unactivated; I213 remains in the independent Dashboard lane. |
| Planned / Unclaimed | I206, I207, I208, I217 | Preserve steering order; select only I217 for this non-overlapping runtime slice. |
| Blocked | None at iteration-document level | PERM-006-B/C and TOOL-024-B/C/D retain backlog-owner blockers. |
| Paused | I164 | Preserve supersession; do not resume. |

I162's title-level `Complete / Review outcome recorded` is a terminal historical record, not an
open Review iteration. Historical `stash@{0}`/`stash@{1}` remain untouched. No open PR or effective
claim overlaps RUNTIME-005-C. This proposal changes governance records only and creates no
implementation authority before merge.

## Completion Evidence

- Completion Commit: Pending
- The claim/activation record and later status-only commits cannot self-certify implementation.

## 2026-08-21 Claim Effective And Local Convergence

PR #347 exact head `3253abb5` passed CI `32471570214`, both governance validators, independent
runtime architecture review `5368607605` and merge-time CAS, then merged as `bb6f5ed9`. The claim
and activation are effective, and implementation started from that exact merge.

The single local stage now implements the frozen internal registry, fixed identity/order
validation, durable-before-finalizer barrier, exactly-once cached execution, one-deadline caps,
failure/panic/timeout containment, typed redacted report projection, legacy compatibility and the
external SDK fixture. Full locked release preflight and three consecutive focused finalizer runs
passed. The Published Baseline above remains byte-preserved. I217 stays Active until this locally
converged stage is published once and receives fresh exact-head CI, independent runtime
architecture review and merge-time CAS; Completion Commit remains Pending.

## 2026-08-21 Stable Review Submission

Implementation commit `44e840d7` was created directly from claim merge `bb6f5ed9` after local
convergence and full locked preflight passed. Draft PR #348 was opened only to obtain and backfill
its number; I217 now moves to Review / Claimed. This metadata-only follow-up does not change the
Published Baseline or implementation behavior. Exact-head Unix/Windows CI, both validators,
independent runtime architecture review and merge-time CAS remain required, and Completion Commit
stays Pending until a later closeout.
