# Iteration I262: OBeiBuddy Runtime And Browser Contract Clarification

> Document status: Active (proposed; ineffective until #544 merges)
> Source: INTEGRATION-001 / #520
> Published plan date: 2026-09-12
> Deliverable: source/test-backed answers to Request A and a complete, non-lossy implementation boundary for Request B.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex single-developer mainline |
| Work Slice | Evidence-backed runtime shutdown/recovery clarification and frame-aware browser ownership/API/security proposal; no production changes. |
| Source Issue | #520 |
| Claimed At | 2026-09-12 |
| Governance Claim PR | #544 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested #520 closure in the continuing single-maintainer workflow; no independent natural-person reviewer available. Claim/activation require #544 exact-head CI, validators, no blocking feedback and merge-time CAS. No API or security decision accepted here. |
| Implementation PR | Not started |
| Last Updated | 2026-09-12 |
| Handoff / Release Condition | No production implementation under this Spike; new API/security decisions require explicit acceptance and separately effective domain claims. |

## Published Baseline

### Selected Story And Dependencies

INTEGRATION-001 is Intake / Unclaimed at selection. ADR-063 and ADR-072 are Accepted;
WEB-007 is Intake / Unclaimed, BROWSER-001 is Refinement / Unclaimed, WEB-005 remains
research/mock-only and TOOL-014 remains Partial. These are investigation inputs, not
browser implementation authorization. Verify #45/#49 source and delivery evidence.

### Scope

- Compare consumer pin `fad6e24e4b716b90b776b358d92ff69015688adf` with the investigated main SHA.
- Answer all five Request A questions using actual API definitions, call paths and named tests;
  distinguish admission closure, work containment, durable finalization and unknown side effects.
- Answer all five Request B decisions, preserve the existing read-only versus interactive boundary,
  and identify a proposed API/crate boundary, migration and security decision gates.
- Preserve all requested frame identities, seven frame-fixture classes, stale-reference executor
  exclusion, child-origin isolation, closed schema, executor revalidation, redacted bounds and no replay.
- Return a separate disposition for A and B; report unresolved evidence or decisions honestly.

### Exclusions

No Rust/Cargo/API/schema/permission/persistence/UI/default-registration changes; no browser
binary distribution, credentials/cookie extraction, arbitrary JavaScript fallback or automatic
replay. No activation of WEB-007, BROWSER-001, Desktop or unrelated capability owners.

### Acceptance And Validation

- [ ] Each Request A question has commit-pinned source/test evidence or a specifically owned gap.
- [ ] Each Request B question has an ownership proposal, security/compatibility gates and no lost requirement.
- [ ] Runtime examples/test commands are reproducible; never claim an unexecuted test passed.
- [ ] Downstream migration, documentation targets and residual domain owners are explicit.
- [ ] Independent Agent-role API/security review checks the report against source; shared-account identity limits are disclosed.
- [ ] Both governance validators and `git diff --check` pass; Issue #520 receives exact merged evidence.

This is a contract-investigation exception to binary-facing acceptance: its testable output is
an audited source/test matrix, not newly implemented user behavior. Closing this Spike alone
does not mean frame-aware browser behavior is delivered or that Issue #520 may be closed.

### Documentation And Rollback

Update INTEGRATION-001, this iteration, a downstream-facing contract report under
`docs/reference/`, applicable SDK documentation links, Board/backlog/index/manifest and #520.
Preserve published and dated history. Correct an erroneous report with an additive correction;
no persistent/runtime rollback is needed because this slice changes neither.

## Preflight Inventory — 2026-09-12

Verified main `e99638e7c506d758f14763f2ab1bc0c4abbb3c98` after fetch; clean worktree,
no stash, one main worktree and no open PRs before creating this governance branch.
Enumerated iteration document headers across all numeric IDs:

| Existing iteration/state | Disposition |
|---|---|
| Active / Review / Blocked iterations | None declared in current iteration headers; terminal text mentioning past review is not a Review state. |
| I164 Paused | Superseded by I165; do not resume. |
| I249 Planned / Unclaimed | Leave unselected; dependency pilot is unrelated to #520 and I250 already owns the completed full upgrade. |
| I261 Complete / Closed | Preserve implementation #543 and closure evidence; no transferred authority. |
| I001-I056 legacy records and I081-I089 obsolete records | Historical, not activation authority; unchanged. |

Existing INTEGRATION-001 is the sole #520 owner. Do not create Issues for local investigation,
test runs, corrections or synchronization. Browser implementation must reconcile WEB-007/#452
and BROWSER-001/#508 instead of creating competing ownership.

## Execution And Completion

- Governance preparation only; no investigation deliverable or implementation claimed complete.
- Completion Commit: pending

### Proposed Activation — 2026-09-12

PR #544 proposes Active / Claimed for the investigation only. Both remain ineffective until
merge. The Published Baseline remains unchanged. Independent Agent-role review of the eventual
API/security report is mandatory and does not claim natural-person identity separation.

Local validation: `COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .`
and `bash scripts/validate_project_governance.sh .` passed with 0 warnings; `git diff --check`
passed. The first project-validator attempt failed because the sandbox denied Cargo registry
metadata extraction; the authorized retry passed (17 SQLite fixtures, 12 delivery fixtures).
No Rust compilation/test result is claimed for this governance-only candidate.

Candidate `794ed1f6` remote reconciliation failed because the #520 matrix owner cell contained
two links while `validate_remote_issue_owners.py::ROW_RE` accepts a single owner link. The row
now retains INTEGRATION-001 as its sole owner and mentions I262 in its explanatory cell.
This is a local correction in #544, not a new Issue or a parser-policy change. New head-bound
validation and review are required before merge; earlier CI is not carried forward.
