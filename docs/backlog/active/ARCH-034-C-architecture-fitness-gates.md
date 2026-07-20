# ARCH-034-C: Architecture Fitness Gates And Drift Prevention

| Field | Value |
|---|---|
| Type | Governance/Technical Story |
| Parent Epic | ARCH-034 |
| Status | Refinement — depends on ARCH-034-A/B |
| Priority | P2 |

## Goal

Turn accepted audit invariants into low-noise checks so future development detects
architecture drift before it becomes expensive.

## Scope

- Add only evidence-backed checks for dependency direction/cycles, forbidden product
  dependencies, stale residual measurements, public API/docs drift, and newly created
  oversized/multi-responsibility roots.
- Prefer trend/baseline reports and explicit allowlists with owner/reason/expiry over
  brittle universal line limits.
- Document the review checklist for new crates, traits, registries, feature flags and
  cross-crate DTOs.
- Re-run the seven extension scenarios as an architecture maintainability scorecard.

## Acceptance

- A fixture or controlled mutation proves each gate fails for its intended violation.
- Existing justified exceptions are explicit and do not suppress unrelated regressions.
- Checks are deterministic, fast enough for normal validation, and have recovery text.
- Governance validation, locked workspace checks/tests and `git diff --check` pass.
- Final re-audit records residual debt owners and next-review trigger/date.

## Required Reads

- Parent ARCH-034
- ARCH-034-A/B outputs
- `scripts/validate_project_governance.sh`
- `scripts/assess_project_scale.sh`
- `.github/workflows/ci.yml`
