# Iteration I190: Change-Aware CI Routing

> Document status: Planned
> Published plan date: 2026-08-12
> Planned objective: introduce a deterministic fail-closed change classifier so narrowly allowlisted documentation-only pull requests keep documentation/governance gates without running the complete Unix and Windows Rust workspaces.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a real documentation-only PR proves stable required-check results with Rust workspace jobs skipped, while adversarial fixtures prove every ambiguous or control/executable change retains full validation.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-12 |
| Work Slice | Implement only I190/GOV-005: deterministic fail-closed changed-path classification, stable pull-request CI routing, adversarial fixtures and route documentation. Keep full validation for every code, control-plane, executable, schema, fixture, dependency, binary, ambiguous or mixed change. No product/runtime behavior, release authorization, branch-protection administration, unrelated CI optimization, closeout or I188/I189 activation. |
| Claimed At | 2026-08-12 |
| Source Issue | None |
| Governance Claim PR | #201 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer directed immediate CI-routing correction after pure-documentation PRs repeatedly ran full Unix/Windows workspaces and one unrelated Windows timing failure forced an 11-minute rerun. Exact-head CI, both governance validators and merge-time CAS remain required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | Pass exact-head CI and merge-time CAS, merge claim PR #201 to `main`, then implement only GOV-005 and prove the reduced path on a real PR. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Outcome |
|---|---|---|
| GOV-005 | New P0 maintainer correction; no existing owner or overlapping PR | Trusted fail-closed changed-path classification plus stable CI routing and adversarial fixtures |

### Scope, Non-Goals And Acceptance

The complete frozen contract is owned by
`docs/backlog/active/GOV-005-change-aware-ci-routing.md`. I190 implements exactly that contract.

### Planned Validation

- Classifier fixture matrix including docs-only, Rust/Cargo, workflow, SOP/AGENTS, script, schema,
  fixture, binary, rename/delete, missing-base, malformed and mixed changes.
- Both governance validators and `git diff --check`.
- Full `./scripts/release_preflight.sh` on the implementation head.
- Exact-head CI plus a real allowlisted documentation-only probe that shows stable skipped workspace
  checks and passing documentation/governance gates.

## Non-Terminal Coordination Record

- I185, I186 and I187 have merged implementation/decision evidence but remain Review until separate
  closeout commits cite `af978322`, `a5115f5c` and `e288afb5`; this story does not alter them.
- I188 and I189 remain Planned with effective claims; this P0 CI correction does not activate their
  decision or permission implementation.
- I159-I162 remain Blocked, I164 remains Paused, and recovery PRs #120/#121 remain archival.
- Open PR #198 is non-overlapping desktop documentation and supplies no authority or evidence here.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-12 | Selection | Maintainer promoted pure-documentation CI routing to immediate priority because the current unconditional matrix delays every governance synchronization and exposes unrelated flaky Rust tests. Claim remains ineffective before target-branch merge. |

## Verification Evidence

- Pending finalized claim, implementation and exact-head evidence.

## Completion Evidence

- No completion evidence. A later closeout must cite an already-existing implementation merge SHA.

## Variance And Residuals

- Windows config-lock test reliability and general CI performance work remain separately owned.

## Retrospective

- Pending execution.
