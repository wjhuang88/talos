# GOV-006: CI Documentation Path Case Normalization

| Field | Value |
|---|---|
| Story ID | GOV-006 |
| Type | Governance / CI Reliability Residual |
| Priority | P2 |
| Status | Refinement - unclaimed review residual |
| Source | PR #202 independent review finding F1, comment `5262374485` |
| Selected Iteration | None |
| Depends On | GOV-005 routing contract and trusted-base classifier at merge `a69ffa30` |

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
| Authorization Evidence | PR #202 review comment `5262374485` demonstrated that case variants such as `docs/SOP/AGENT-COLLABORATION.md` receive reduced validation because the lowercase `docs/sop/` exclusion is matched case-sensitively. |
| Implementation PR | Not started |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | Refine and claim a bounded classifier-maintenance slice before implementation. |

## Goal And Boundary

Make the documentation-path exclusion policy independent of path-letter casing so variants of
`docs/sop/` cannot receive reduced validation. Preserve the finished GOV-005 allowlist, trusted-base
execution, fail-closed behavior and push-to-main full-validation fallback.

## Scope And Acceptance

- Normalize path casing before applying the `docs/sop/` exclusion, or enforce an equivalently
  deterministic comparison that sends every case variant to full validation.
- Add positive and adversarial fixtures for lowercase, uppercase and mixed-case SOP directory names.
- Prove ordinary allowlisted reference Markdown remains reduced and every existing full-route fixture
  remains full.
- Keep classifier execution dependency-free and sourced from the trusted base commit.

## Exclusions

No broader allowlist expansion, test sharding, caching, runner selection, branch-protection change,
product/runtime behavior, or activation of another iteration.

## Minimum Validation

Classifier fixtures, both governance validators, `git diff --check`, exact-head CI and a real route
probe if the workflow behavior changes.
