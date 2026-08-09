# ARCH-034-R04-AG2: Parent Hardening API Safety Fence

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-2 / unused parent-process mutation API |
| Status | Refinement — public API compatibility and call-order proof required |
| Priority | P1 |
| Selected Iteration | None |
| Preserved behavior | `dangerous_env_var_names()`, child hardening, sandbox errors, and all production callers |

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
| Authorization Mode | Independent security/API review required |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Select an additive fence or accept a migration ADR before changing the semver-bound public API. |

## Confirmed Baseline

`ProcessHardening::apply(&self)` and public
`sanitize_env_vars_internal(&self)` can mutate the process environment through
edition-2024 `unsafe`. They have no production caller, cannot enforce the
documented before-threads invariant, and their safety comment incorrectly says
that the caller holds a mutable reference. Four repeated `#[allow(warnings)]`
attributes also obscure the test boundary but are not production authorization.

## Scope And Acceptance

- Inventory downstream/public use before selecting an API fence.
- Make unsafe parent mutation impossible to call accidentally after threads
  start, or explicitly deprecate it with an accepted compatibility/migration plan.
- Retain the side-effect-free dangerous-name accessor used by child commands.
- Correct safety documentation so it states an enforceable invariant rather than
  claiming nonexistent mutable access.
- Add compile/API and controlled call-order fixtures; do not use a process-global
  test mutex as proof that arbitrary production callers are safe.
- Remove only redundant warning suppressions orphaned by this bounded change.

## Exclusions And Residuals

No parent rlimit application, Bash `pre_exec` changes, permission/sandbox-policy
broadening or unreviewed breaking removal. AG-1 owns the shipped child boundary.

## Minimum Validation

`cargo test --locked -p talos-sandbox`, public API/semver review, locked release
preflight, both governance validators and independent sandbox/security review.
