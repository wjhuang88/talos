# GOV-008: Local Convergence And Stage Validation

| Field | Value |
|---|---|
| Story ID | GOV-008 |
| Type | Governance / Delivery Workflow |
| Priority | P0 |
| Status | Review / Claimed - atomic activation merged; locally converged stage candidate ready |
| Source | Maintainer correction on 2026-08-21; GOV-007/I205 throughput audit; Issue #339 |
| Selected Iteration | I215 - atomic claim+activation pilot |
| Depends On | GOV-007/I205 Complete; current collaboration, Git, iteration, document-check and long-task contracts |

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

## Goal

Make local iteration the default development loop and reserve GitHub CI/review for stable delivery
stages, while retaining every gate that protects security, releases, target-branch ownership and
completion truth.

## Scope

- Allow one governance-only PR to establish both `Claimed` and `Active` when it reaches the target
  branch; the open PR has no ownership or activation effect.
- Require a local convergence checkpoint covering scope, implementation, tests, documentation,
  owner consistency and diff review before the first implementation PR push.
- Define remote CI/review as validation of a stable stage candidate. Local amend/fix cycles before
  push require no remote evidence; substantive changes after review require a new exact-head result.
- Remove routine activation-only PRs and avoid PR-number backfill commits where target-branch truth
  can remain deterministic without them.
- Permit implementation evidence and truthful owner-first Review state in the stage candidate, and
  permit safe closeout combination only when Completion Commit evidence already exists.
- Add executable fixtures for ordinary, protected, release and bounded-maintenance paths.
- Record migration, rollback and the repeated process lesson.

## Exclusions

- No weakening of security, sandbox, permission, process-hardening or release review.
- No weakening of exact-head evidence after substantive content changes or merge-time CAS.
- No self-certifying Completion Commit, target-branch claim bypass or published-baseline rewrite.
- No product, runtime, API, TUI, persistence, dependency, version, tag or publication change.
- No activation or authority transfer for I189, RUNTIME-005-B/C, PERM-006-B/C or TOOL-024-B/C/D.

## Acceptance

- An ordinary governed iteration can move from unclaimed/planned to claimed/active with one merged
  governance PR and no separate activation PR.
- The SOPs state an explicit local convergence loop and stable-candidate checklist before remote
  implementation submission.
- Exact-head invalidation distinguishes substantive reviewed-content changes from local edits that
  were never submitted.
- Protected and release scenarios retain their current independent-review and preflight gates.
- Validator fixtures exercise ordinary, protected, release and bounded-maintenance scenarios and
  fail on at least one invalid transition per applicable rule.
- Owner-first completion still requires a pre-existing implementation/evidence SHA.

## Validation

- Scenario fixtures for the revised collaboration validator.
- `scripts/validate_project_governance.sh .`
- `COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .`
- Parse `.agent-governance/manifest.yaml`.
- `scripts/assess_project_scale.sh .`
- `git diff --check`

## Documentation

Governance-only deliverable. Update `AGENTS.md`, collaboration/iteration/Git/document-check and
long-task SOPs as required; no end-user product documentation change is claimed.

## Rollback

Revert the executable workflow change and return to the pre-I215 separate claim/activation flow.
Do not roll back owner truth, completed evidence or published baselines.

## Execution Checkpoint - 2026-08-21

PR #340 head `1e00249b` passed exact-head CI `32439457491`, both governance validators, manifest
parsing, the scale harness and merge-time CAS, then merged as `e66d039c`. That single merge made
both claim and activation effective; no separate activation PR was used.

The implementation worktree starts at that merge. AGENTS/SOP updates, the cross-platform harness,
12 ordinary/protected/release/bounded-maintenance scenarios and the EVOLUTION lesson have converged
locally without an intermediate implementation push. GOV-008 is Review, not Complete; the stable
candidate still requires remote exact-head validation and merge evidence.

Final local convergence passed `./scripts/release_preflight.sh`, both governance validators,
PowerShell strict parsing and execution, the SQLite 17-case self-test, the delivery-workflow
12-case harness, manifest parsing, scale assessment, classifier/public-site/installer checks,
workspace check/Clippy/tests/doctests, `git diff --check`, and the no-Rust/Cargo scope check. During
that loop, the PowerShell validator was found to call the delivery harness before its variable was
defined and to omit the SQLite self-test; the defect was corrected locally and every affected gate
was rerun before the first implementation push.
