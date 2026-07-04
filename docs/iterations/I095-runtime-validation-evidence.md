# Iteration I095: Runtime Validation Evidence

> Document status: Complete
> Published plan date: 2026-07-04
> Planned objective: add or specify durable validation execution evidence needed for Talos-primary
> development.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: permission-bounded validation evidence records or a precise design blocker.
> Activated: 2026-07-04
> Completed: 2026-07-04

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
| 2026-07-04 | Activation | Activated after I094 closed with `gix 0.85.0`, unchanged feature scope, fallback audit, workspace validation, clippy, governance validation, and `git diff --check` passing. Scope remains runtime validation evidence only: no arbitrary shell policy expansion, scheduled execution, Guardian auto-approval, exec DSL, hidden pass/fail, release claim, tag, publish, or permission-default change. |
| 2026-07-04 | Execution | Added `talos validate run` alongside the existing read-only `talos validate plan`. `run` accepts only built-in validation profiles and records command, exit status, stdout/stderr summaries, and the allowlisted-profile permission decision. It does not accept arbitrary commands. |

## Closeout Evidence

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo test -p talos-cli validation`: passed, 8 validation/governance tests.
- `cargo check -p talos-cli`: passed.
- `cargo clippy -p talos-cli -- -D warnings`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `cargo run -p talos-cli -- validate run --profile governance --json`: passed; emitted a
  `governance` record with command `scripts/validate_project_governance.sh .`, `exit_status: 0`,
  `status: passed`, `permission_decision: allowlisted validation profile: governance`,
  `stderr_summary: <empty>`, and stdout summary `Governance validation passed: 0 warning(s).`
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

## Residuals

- No I095 residual blocks I096 activation.
- This is not a general command runner, scheduled execution system, Guardian approval path, exec
  DSL, release claim, or REL-002 qualifying Talos-primary session by itself.
