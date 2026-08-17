# GOV-007: PR Workflow Throughput Simplification

| Field | Value |
|---|---|
| Story ID | GOV-007 |
| Type | Governance / Delivery Throughput Spike |
| Priority | P0 |
| Status | Ready - I205 Planned / Unclaimed |
| Source | Maintainer direction on 2026-08-17 after the v0.8.0 delivery retrospective |
| Selected Iteration | I205 - Planned / Unclaimed |
| Depends On | Current collaboration, Git, DOC-CHECK and change-aware CI contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Establish a separate effective I205 claim before changing SOPs, validators, workflows or branch protection. |

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
