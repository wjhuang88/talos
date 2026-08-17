# GOV-007: PR Workflow Throughput Simplification

| Field | Value |
|---|---|
| Story ID | GOV-007 |
| Type | Governance / Delivery Throughput Spike |
| Priority | P0 |
| Status | In Progress - I205 Active / Claimed |
| Source | Maintainer direction on 2026-08-17 after the v0.8.0 delivery retrospective |
| Selected Iteration | I205 - Active / Claimed |
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
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | After this claim merges, activate I205 separately from the claim merge or later main and execute only the evidence/decision Spike. Establish a new bounded claim before changing SOPs, validators, workflows or branch protection. |

## Goal

Reduce repeated status-only PRs, duplicate owner/derived-view edits and unnecessary re-review while
preserving the controls that address real failure modes.

## Scope

- Measure claim, implementation, closeout, correction and review round trips for the recent
  I159-I203 delivery chains.
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
