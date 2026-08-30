# Iteration I238: Completion Claim And Evaluation State Model

> Document status: Active / Claimed — proposed claim and activation are ineffective until PR #444 merges
> Published plan date: 2026-08-31
> Planned objective: implement WORK-001-C/P2 as a shared, exact-revision Completion Claim and
> Evaluation state contract without starting the evaluator runtime or Mission delivery gate.
> MVP deliverable: an executable state-machine fixture drives claim -> evaluation pending ->
> evaluating -> criterion verdict -> aggregate verdict -> stale/rework against canonical Work
> identities, while proving the executor cannot directly complete a Goal.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline WORK-001 session |
| Work Slice | WORK-001-C / I238 P2 only: storage-neutral Completion Claim, Acceptance Criterion, Evaluation subject/report/finding/verdict and deterministic revision-bound transition/staleness rules in the shared Work Domain. No evaluator runtime, persistence, Mission final gate, UI, Desktop, Dashboard, permission, `/auto`, release or publication. |
| Claimed At | 2026-08-31 |
| Source Issue | #29 |
| Governance Claim PR | #444 |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending exact-head governance CI and independent architecture/API review for PR #444. |
| Implementation PR | Not started |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Proposed Active/Claimed state, claim and activation are ineffective until finalized PR #444 reaches `main`; implementation starts from that merge or later current `main`. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WORK-001-C | WORK-001 | Active / Claimed (proposed; ineffective until PR #444 merges) | WORK-001-B/I237 Complete; ADR-061; canonical `talos_core::work` domain | One storage-neutral, executable Completion Claim/Evaluation state contract with exact-revision staleness and no evaluator runtime. |

### Non-Terminal Inventory And Disposition

| State | Iterations | Disposition |
|---|---|---|
| Active | None | I238 is only a proposal on this branch and has no effect before merge. |
| Review | None | I237 and WORK-001-B are Complete/Closed on `main@6ed8a24f`; no review work is bypassed. |
| Planned | I207, I208 | Preserve as Unclaimed TUI steering children; do not activate, merge or transfer their scope. |
| Blocked | None with a current iteration document status | WORK-001-D/P3 and E/P4 remain parent-owned blocked boundaries without selected iterations. |
| Paused | I164 | Superseded startup target; do not restore. |

Open PRs #120/#121 remain archival Draft recovery snapshots and are excluded from current work.
There is no other open main-base claim or implementation PR. I238 is the next unused iteration ID
after I237; no in-flight proposal reserves it.

### Scope

- Add shared typed Acceptance Criteria, Completion Claim and Evaluation subject/report/finding/
  verdict contracts.
- Bind all claims and reports to exact Mission, Goal and workspace-content revisions.
- Derive aggregate verdicts from criterion-level results and reject inconsistent reports.
- Define legal evaluation transitions and deterministic staleness after relevant subject mutation.
- Keep locale/presentation state out of evaluation identity.
- Add public API documentation and executable transition/staleness fixtures.

### Non-Goals

- No evaluator Agent/runtime, provider/model call, prompt/context design or independent inspection.
- No Validation execution, persistence/schema migration, Runtime/Session wiring, Mission final gate,
  coordinator, UI projection or end-to-end product flow.
- No Todo behavior change, Desktop/Dashboard/GPUI, permission/sandbox, `/auto`, release, version,
  tag or publication work.

### Planned Acceptance

- Executor submission can create only a revision-bound claim and evaluation-pending state; it
  cannot create a PASS verdict or Completed Goal.
- Required criterion verdicts deterministically derive PASS/FAIL/INCONCLUSIVE and inconsistent or
  duplicate reports fail closed.
- Relevant subject revision change makes prior PASS stale; locale-only changes do not.
- Validation evidence stays referential and cannot become completion authority.
- Existing WorkGraph and Todo compatibility behavior remains unchanged.

### Planned Validation

- Focused `talos-core` unit/property/state-machine tests for identity, aggregation, transitions and
  staleness.
- Existing locked WorkGraph and Todo compatibility tests.
- `cargo fmt --all -- --check`, locked workspace check/Clippy/tests and release preflight.
- Both governance validators, manifest/YAML validation and `git diff --check`.
- Exact-head CI, independent architecture/API review and merge-time CAS.

### Risks And Rollback

- Public types may prematurely encode P3 runtime policy. Keep P2 storage-neutral and reject any
  provider/context/runtime field that is not required to express state integrity.
- Aggregate verdict rules may hide optional/required ambiguity. Constructors must derive rather
  than accept contradictory aggregate state and fixtures must cover mixed outcomes.
- Revision binding may be too weak if locale or unrelated state is included/excluded incorrectly.
  Use explicit subject components and staleness matrices, not opaque equality alone.
- Rollback is source-only: remove the additive P2 types before adoption. No persistence or runtime
  migration is authorized, so no durable rollback is needed.

### Documentation Targets

- Add or update one shared Work/Evaluation API reference.
- Keep P1 Todo compatibility documentation accurate.
- Do not describe evaluator runtime, Mission Delivery or Desktop binding as shipped.

## Activation Rule

This governance-only branch proposes Claimed/Active atomically in PR #444. Both states remain
ineffective until the finalized exact head merges to `main`. No
implementation branch, Rust/Cargo edit, dependency change or experiment represented as progress is
authorized before that merge.

## Completion Rule

Implementation must first merge to `main`. A later owner-first closeout records an already-existing
implementation or merge SHA as `Completion Commit`; the status commit cannot self-certify. P3
remains blocked until that closeout reaches `main`.
