# Iteration I095: Runtime Validation Evidence

> Document status: Planned
> Published plan date: 2026-07-04
> Planned objective: add or specify durable validation execution evidence needed for Talos-primary
> development.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: permission-bounded validation evidence records or a precise design blocker.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `RUNTIME-001` residual | Embeddable runtime | Complete pre-1.0 facade | ADR-024 | Validation evidence gap narrowed. |
| `REL-002` prerequisite | v1 self-bootstrap gate | Planned — not ready | Runtime/governance maturity | Self-bootstrap blocker evidence updated. |

### Scope

- Define command, exit status, output summary, and permission decision evidence records.
- Keep execution allowlisted and explicit.
- Prefer a small local validation profile before broader workflow automation.

### Non-Goals

- No arbitrary shell execution policy expansion.
- No scheduled execution, Guardian auto-approval, exec DSL, or hidden pass/fail.
- No release claim.

### Acceptance

- Given validation is executed,
  When evidence is recorded,
  Then the user can see command, status, output summary, and permission decision.
- Given execution cannot be safely bounded,
  When the iteration closes,
  Then a design-only blocker is recorded instead of weakening permissions.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
