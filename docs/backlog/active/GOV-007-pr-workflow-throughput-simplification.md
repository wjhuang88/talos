# GOV-007: PR Workflow Throughput Simplification

| Field | Value |
|---|---|
| Story ID | GOV-007 |
| Type | Governance / Delivery Throughput Spike |
| Priority | P0 |
| Status | Review - I205 evidence/decision packet complete; implementation PR #287 |
| Source | Maintainer direction on 2026-08-17 after the v0.8.0 delivery retrospective |
| Selected Iteration | I205 - Review / Claimed |
| Depends On | Current collaboration, Git, DOC-CHECK and change-aware CI contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-18 |
| Work Slice | Execute only GOV-007/I205's evidence-only audit and decision packet: measure recent claim/implementation/closeout/review churn, classify causes, map retained gates to demonstrated failures or Hard constraints, define the ordinary/protected/release/maintenance scenario matrix, migration and rollback, and identify the smallest separately claimable implementation slice. No SOP, validator, CI workflow, branch-protection, product/runtime, release-policy, security-gate or child-activation change. |
| Claimed At | 2026-08-18 |
| Source Issue | None |
| Governance Claim PR | #284 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer requested an evidence-based PR-flow simplification and unattended continuation of the long task. Claim head `5af455930a84871042b53b7bb1de12651edcc6a7` passed CI `32046397520`, both governance validators and merge-time CAS with no blocking feedback; PR #284 merged as `fd1eaad9076bed1b110e17bade3ff0dc48040fdf`. Any executable rule change requires its own bounded claim and review. |
| Implementation PR | #287 |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Audit packet is complete in PR #287. Establish a new bounded claim and iteration before changing SOPs, validators, workflows or branch protection. |

## Goal

Reduce repeated status-only PRs, duplicate owner/derived-view edits and unnecessary re-review while
preserving the controls that address real failure modes.

## Scope

- Measure claim, implementation, closeout, correction and review round trips for the recent
  I159-I205-era delivery chains selected by the audit scope.
- Classify each round trip as required safety evidence, mechanically preventable drift, stale-base
  reconciliation, derived-view duplication or avoidable process ceremony.
- Define a target workflow with explicit treatment for ordinary, bounded-maintenance, release and
  security-sensitive work.
- Evaluate generated derived views, post-merge closeout automation, exact-base validator defaults,
  and evidence reuse only when the reviewed content and exact head are unchanged.
- Publish migration, rollback and validator requirements before changing repository rules.

## Hard Gates To Preserve

- Effective claim before governed implementation.
- Independent review for sandbox, permission, process-hardening and explicitly protected scopes.
- Exact-head evidence after content changes and merge-time CAS.
- Owner-first truth and pre-existing Completion Commit evidence before Complete.
- Immutable release tags and GitHub-before-Cargo release ordering.

## Exclusions

- No immediate SOP, validator, workflow, branch-protection or product-code change.
- No self-approval, fabricated reviewer identity, security-review bypass or status self-certification.
- No claim that fewer PRs is automatically safer or faster without repository evidence.

## Acceptance

- A reproducible audit table reports PR count, review rounds, head changes and correction causes for
  the selected recent delivery chains.
- Every retained gate maps to a demonstrated failure mode or repository Hard constraint.
- Every proposed simplification states expected PR/review reduction, new risk, compensating
  automation, rollback and the exact SOP/validator owner that would change.
- The recommended target has a scenario matrix for ordinary, security-sensitive, release and
  bounded-maintenance work and identifies a smallest separately claimable implementation slice.

## Validation

- GitHub/API evidence can be regenerated from recorded commands without relying on prose claims.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`

## User-Facing Documentation

Infrastructure-only governance Spike. No runtime or user-facing product behavior is claimed.

## Execution Evidence - 2026-08-18

The evidence-only audit is complete in PR #287. The reproducible snapshot
`docs/reference/I205-PR-WORKFLOW-EVIDENCE.json` covers 42 explicitly selected PRs across ten
I159-I205-era chains: 40 merged, 2 closed without merge, 37 explicit review rounds, 11 REQUEST
CHANGES rounds, 26 approvals and 10 reviewed-head changes. The decision report selects atomic
claim activation as the smallest separately claimable follow-up and preserves all protected,
exact-head, CAS, owner-first, Completion Commit and release-order gates.

Validation evidence:

- `python3 -m py_compile scripts/audit_pr_workflow.py` passed.
- JSON summary assertions for the 42-PR population and zero unbound/unknown review rounds passed.
- `scripts/validate_project_governance.sh .` passed with 0 warnings.
- `COLLABORATION_VALIDATION_BASE=4635ef2b4cc9c894f03c0bcbce7e7802730e56ab bash scripts/validate_collaboration_claims.sh .` passed with 0 warnings.
- `git diff --check` passed.

This is evidence for `Review`, not completion. The selected atomic-activation workflow requires a
new bounded owner, iteration and effective claim before any SOP or validator changes.
