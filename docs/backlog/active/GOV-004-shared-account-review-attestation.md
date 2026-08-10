# GOV-004: Shared-Account Independent Review Attestation

| Field | Value |
|---|---|
| Story ID | GOV-004 |
| Type | Governance / Auditability Story |
| Priority | P1 |
| Status | Refinement — attestation fields and mechanical limits require review |
| Source | PR #177 independent review comment `5230395611` |
| Selected Iteration | None |
| Depends On | Existing `docs/sop/AGENT-COLLABORATION.md` independent-review path |

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
| Authorization Evidence | Review comment `5230395611` explicitly disclosed that a distinct natural-person reviewer used the shared `@wjhuang88` account and identified the missing machine-verifiable identity contract. |
| Implementation PR | Not started |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | Select an auditable attestation contract without claiming that repository automation can prove a human identity. |

## Goal And Boundary

Clarify how Talos records independent review when multiple natural people operate
through one GitHub account. Preserve the existing requirement for an independent
maintainer/reviewer and distinguish human attestation from GitHub-native account
identity or review-state evidence.

PR #186 review `5236234197` additionally proved that the current claim validator accepts a nonempty
Authorization Evidence field without cross-checking whether its named SHA or PR review event exists.
That finding is mechanical review-state linkage, not proof of natural-person identity.

## Scope And Acceptance

- Define the minimum explicit disclosure: distinct natural-person role,
  implementer/session separation, reviewed exact SHA, content-based method,
  verdict, blocking findings and shared-account caveat.
- State which fields belong in `Authorization Evidence`, the PR comment and any
  closure record.
- Extend the collaboration validator only for mechanically checkable presence and
  consistency; it must not claim to authenticate a natural person.
- Decide whether and how repository validation cross-checks an Authorization Evidence PR/SHA against
  observable review comments or review state, and document any offline/network limitation without
  treating absence of remote evidence as proof of identity.
- Add positive/negative fixtures for shared-account attestation and preserve
  normal distinct-account and single-maintainer authorization paths.
- Update the SOP, validator guidance and evolution lesson together.

## Exclusions

No retroactive invalidation of PR #174/#176/#177 evidence, no biometric/identity
provider, no mandatory second GitHub account, no weakening of protected-scope
review, and no representation of a top-level comment as GitHub `APPROVED` state.

## Minimum Validation

Both governance validators and their fixtures, `git diff --check`, scenario-based
review of the protected-scope case, and exact wording review by the maintainer.
