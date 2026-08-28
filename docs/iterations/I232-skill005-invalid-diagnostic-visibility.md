# Iteration I232: Invalid Skill Diagnostic Visibility

> Document status: Complete / Closed
> Published plan date: 2026-08-28
> Planned objective: close SKILL-005/#333 by surfacing bounded field diagnostics for explicitly activated invalid local Skill documents.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: real CLI activation distinguishes invalid `triggers` documents from genuinely absent Skills while preserving fail-closed discovery.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline SKILL-005 session |
| Work Slice | Implement only SKILL-005: retain bounded invalid local Skill diagnostics and surface the matching `triggers` error during explicit activation, preserving fail-closed exclusion and absent-skill behavior. Exclude parser policy, discovery/routing, permission, persistence, dependencies, release, Dashboard, Desktop and `/auto`. |
| Claimed At | 2026-08-28 |
| Source Issue | #333 |
| Governance Claim PR | #423 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized claim requires exact-head CI, validators and review before merge. |
| Implementation PR | #424 |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Implementation PR #424 merged as `fedd6fac`; exact-head CI/review and merge-time CAS are recorded below. |

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
| 2026-08-28 | Atomic claim+activation | PR #423 exact head `c93705ce` passed CI and governance gates, then merged as `f2c98ff1`; I232/SKILL-005 claim and Active state became effective. |
| 2026-08-28 | Implementation and closeout | PR #424 exact implementation head `fb47b0c2` passed exact-head CI `33141878176` (5/5, including Windows workspace) and independent Skill/CLI review `5448628671`; merged to `main` as `fedd6fac`. |

## Verification Evidence

- Exact implementation head `fb47b0c2` preserves fail-closed discovery while surfacing bounded `triggers` diagnostics for explicit invalid scalar/mapping activation. `talos-skill` (81), `talos-cli` (358), real-binary (3), format, Clippy and locked preflight passed; CI `33141878176` and independent review `5448628671` are bound to that head.

## Completion Evidence

- Completion Commit: `fedd6fac94708628478836b94b6fd01954de53e0` (PR #424 merge; implementation source `fb47b0c2`).
