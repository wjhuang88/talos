# ARCH-034-R11: Architecture Documentation Truth

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F26 |
| Status | Ready |
| Priority | P1 |
| Selected Iteration | Not selected |
| Preserved behavior | Documentation-only; historical evidence remains immutable |

## Problem And Boundary

Current-state architecture text omits the implemented tool contribution model and retains stale CLI
boundary wording. ADR-007/R0 status text also needs factual reconciliation, but security semantics
cannot be changed without R04 review.

## Scope

- Update current-state crate/composition/extension documentation from source evidence.
- Label historical snapshots as historical and link the August audit.
- Route ADR/process-hardening semantic edits through R04; make only non-semantic factual fixes here.

## Exclusions

- No decision reversal, history rewrite, production edit, behavior claim, or security policy change.

## Acceptance And Validation

- Architecture extension paths match current source and R01 exception verdicts.
- Historical audit/iteration baselines remain intact and current facts are not backdated.
- DOC-CHECK, governance/claim validators, scale assessment, link/search checks, and diff checks pass.

## Rollback / Residual

Revert inaccurate prose. Any decision change belongs to a new ADR or R04 security review.
