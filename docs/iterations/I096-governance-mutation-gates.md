# Iteration I096: Governance Mutation Gates

> Document status: Planned
> Published plan date: 2026-07-04
> Planned objective: design and implement the smallest safe governance preview/write gate needed
> before Talos can self-bootstrap owner-doc updates.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: typed governance plan/preview/write flow or a recorded blocker that preserves
> read-only governance.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `GOV-003` residual | Built-in project governance | Phase 1 complete | Governance docs | Mutation/gate gap narrowed. |
| `REL-002` prerequisite | v1 self-bootstrap gate | Planned — not ready | Runtime/governance maturity | Owner-doc sync blocker updated. |

### Scope

- Add a typed plan/preview/write boundary for owner-doc changes if safe.
- Require governance validation after mutation.
- Keep user-visible reasons for any rejected mutation.

### Non-Goals

- No silent owner-doc edits.
- No broad project-manager automation.
- No web write routes or remote dashboard mutation.

### Acceptance

- Given a governance mutation is proposed,
  When preview runs,
  Then affected owner docs and validations are visible before write.
- Given a write occurs,
  When governance validation runs,
  Then drift is caught or the write is rejected.

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
