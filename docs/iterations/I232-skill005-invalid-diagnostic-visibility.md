# Iteration I232: Invalid Skill Diagnostic Visibility

> Document status: Planned
> Published plan date: 2026-08-28
> Planned objective: close SKILL-005/#333 by surfacing bounded field diagnostics for explicitly activated invalid local Skill documents.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: real CLI activation distinguishes invalid `triggers` documents from genuinely absent Skills while preserving fail-closed discovery.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #333 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Establish an effective claim before implementation. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SKILL-005 | I211 corrective owner / Issue #333 | Ready / Unclaimed | SKILL-004/I198 optional-trigger compatibility; parser diagnostics | Explicit activation surfaces a bounded `triggers` diagnostic for known invalid local documents. |

### Scope

- Preserve fail-closed exclusion of malformed Skill documents.
- Retain bounded discovery diagnostics keyed to invalid local document identity.
- Surface the matching `triggers` diagnostic during explicit activation.
- Keep truly absent Skills distinguishable and cover scalar/mapping malformed containers through the real CLI path.

### Non-Goals

- No automatic activation, discovery policy, routing, permission, persistence, parser redesign, dependency, release, Dashboard, Desktop or `/auto` change.

### Acceptance

- Invalid scalar/mapping `triggers` documents remain excluded from normal Skill listing and prompt context.
- `/skills activate <invalid-name>` returns an actionable bounded diagnostic naming `triggers`.
- `/skills activate <absent-name>` retains deterministic not-found behavior.
- Real CLI binary tests cover both invalid shapes and absent control.

### Planned Validation

- Focused Skill/CLI unit and real-binary tests, locked workspace tests, strict Clippy, release preflight, governance validators and diff check.
- Exact-head CI, independent review and merge-time CAS.

### Documentation To Update

- SKILL-005, SKILL-004/I198 corrective disposition, user Skill author docs, Board, Backlog, iterations README, manifest and Issue #333.

### Risks And Rollback

- Risk: retaining diagnostics accidentally activates invalid content or leaks paths/content.
- Rollback: retain fail-closed exclusion and restore generic not-found while leaving diagnostic owner Review.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-28 | Claim preparation | Prepared from `main@88261365`; I231 is Complete/Closed, I198 remains Review and no overlapping implementation PR exists. Claim and activation remain ineffective until governance merge. |

## Verification Evidence

- Pending.

## Completion Evidence

- Completion Commit: Pending.
