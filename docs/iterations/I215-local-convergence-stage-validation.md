# Iteration I215: Local Convergence And Stage Validation

> Document status: Review / Claimed - atomic activation merged; locally converged stage candidate ready
> Published plan date: 2026-08-21
> Planned objective: replace remote micro-PR editing loops with local convergence and stable-stage
> validation while preserving Talos security, release, ownership and completion gates.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: repository SOPs and deterministic fixtures make one atomic claim+activation and
> one locally converged implementation candidate runnable and testable across four workflow classes.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-21 |
| Work Slice | Implement only the I205-selected delivery-workflow optimization: atomic claim+activation, mandatory local convergence before remote implementation submission, stable-stage exact-head CI/review semantics, reduced PR-number/status backfill churn, safe owner-first closeout combination, executable scenario fixtures, and the reusable process lesson. Preserve protected-scope independent review, release preflight/order, merge-time CAS, target-branch truth, published baselines and pre-existing Completion Commit evidence. No product/runtime/TUI/permission/sandbox/release/dependency behavior change. |
| Claimed At | 2026-08-21 |
| Source Issue | #339 |
| Governance Claim PR | #340 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer explicitly directed that Talos optimize the governance guidance before further development and specified local design/implementation/test convergence followed by stage-level remote validation. This governance-only PR proposes claim and activation atomically; neither is effective until merge. Exact-head CI, both governance validators, no blocking feedback and merge-time CAS remain required. |
| Implementation PR | Not started - stable candidate not yet pushed |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Submit the locally converged stable candidate for exact-head CI and review. Complete only after merge and owner-first closeout cite a pre-existing implementation/evidence commit. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| GOV-008 | Ready / Unclaimed | GOV-007/I205 Complete | Executable local-convergence and stage-validation workflow |

### Scope And Non-Goals

The complete scope, retained gates and exclusions are owned by
`docs/backlog/active/GOV-008-local-convergence-stage-validation.md`. This is an explicit
infrastructure-only governance iteration; its runnable output is the SOP/validator scenario
harness, and it claims no user-facing product behavior.

### Planned Validation

- Exercise ordinary, protected, release and bounded-maintenance scenario fixtures.
- Run both governance validators with an explicit trusted base.
- Parse the governance manifest, run the scale harness and run `git diff --check`.
- Obtain exact-head CI and review for the single stable implementation candidate.

### Documentation To Update

- `AGENTS.md` and affected collaboration, iteration, Git, document-check and long-task SOPs.
- GOV-008/I215 owners, current inventories, manifest, Issue #339 and EVOLUTION lesson.
- No runtime-user documentation is affected.

### Risks And Rollback

- Risk: fewer remote checkpoints could conceal scope drift until submission.
- Control: bounded owner scope, local stable-candidate checklist, scenario fixtures and fresh
  exact-head evidence after submitted content changes.
- Rollback: revert the workflow implementation and return to separate claim/activation PRs without
  changing published owner history or completed evidence.

## Non-Terminal Inventory And Disposition

Exact selection baseline: `main@14531bbc70db4e401b922cf68f8983d33e15ad46`.

| State | Iterations | Disposition |
|---|---|---|
| Active | I214 | Keep its decision PR #338 in independent review; pause new I214 edits while the maintainer-directed, non-overlapping governance repair runs. Do not change I214 scope or status. |
| Review | I197, I198, I201, I210 | Preserve their corrective destinations and do not reopen or absorb them. |
| Planned / Claimed | I189, I213 | Keep unactivated and independently owned; I213 remains Dashboard-only. |
| Planned / Unclaimed | I206, I207, I208 | Preserve published steering order and do not claim. |
| Blocked | None at iteration-document level | Preserve RUNTIME-005-B/C, PERM-006-B/C and TOOL-024-B/C/D owner blockers. |
| Paused | I164 | Preserve superseded state; do not resume. |

The maintainer explicitly inserted I215 before further development. This is a bounded governance
repair exception to the normal single-active preference, not authority for parallel product work.
Archival Draft PRs #120/#121 remain untouched.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-21 | Atomic claim+activation proposal | Governance PR #340 proposes both Claimed and Active from exact `main@14531bbc`. Neither state is effective until merge; no SOP, validator, product or runtime implementation exists on this branch. |
| 2026-08-21 | Atomic activation merge | PR #340 head `1e00249b` passed CI `32439457491`, both validators, manifest/scale checks and CAS, then merged as `e66d039c`. Claim and Active became effective together. |
| 2026-08-21 | Local convergence | AGENTS/SOP rules, POSIX/PowerShell validator integration, 12 workflow scenarios and the EVOLUTION lesson converged locally from the activation merge without intermediate implementation pushes. I215 moves to Review for one stable stage candidate. |

## Verification Evidence

- Atomic governance PR #340: CI `32439457491`, both validators, manifest parse, scale harness and
  merge-time CAS passed at exact head `1e00249b`; merge `e66d039c`.
- `python3 scripts/validate_delivery_workflow.py`: 12/12 scenario expectations passed locally.
- `COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .`:
  passed locally with 0 warnings.
- `pwsh -NoProfile -File scripts/validate_project_governance.ps1 .`: passed locally with both the
  SQLite 17-case self-test and delivery-workflow 12-case harness executed, 0 warnings.
- `./scripts/release_preflight.sh`: passed from the final local candidate tree, including locked
  workspace check, Clippy, tests and doctests.
- Manifest parsing, scale assessment, classifier/public-site/installer validation,
  `git diff --check`, EOF checks and the no-Rust/Cargo scope check passed locally.

## Completion Evidence

- Completion Commit: pending.
- A later status transition must cite pre-existing SOP/validator/evidence commits; this claim and
  activation record cannot self-certify completion.

## Variance And Residuals

- I214 remains separately in progress. Its #338 review found an admission check-to-start race and
  an invalid-options/Drop semantic conflict; correct both locally after I215 closes, then submit one
  new stable #338 head under the revised workflow.

## Retrospective

- The local loop caught and repaired a PowerShell validator wiring defect before any implementation
  push: the draft block had skipped the SQLite self-test and referenced the delivery validator too
  early. Cross-platform execution, not syntax-only parsing, is required evidence for validator
  changes.
