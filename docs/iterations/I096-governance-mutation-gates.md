# Iteration I096: Governance Mutation Gates

> Document status: Complete
> Published plan date: 2026-07-04
> Planned objective: design and implement the smallest safe governance preview/write gate needed
> before Talos can self-bootstrap owner-doc updates.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: typed governance plan/preview/write flow or a recorded blocker that preserves
> read-only governance.
> Activated: 2026-07-04
> Completed: 2026-07-04

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
| 2026-07-04 | Validation | Governance mutation gate write smoke: this row was written through talos governance iteration-record write with --confirm-preview; post-write governance validation passed. |
| 2026-07-04 | Activation | Activated after I095 closed with allowlisted validation evidence, README sync, REL-002 non-qualification posture, workspace validation, clippy, governance validation, and `git diff --check` passing. Scope is the smallest safe governance preview/write gate only: no silent owner-doc edits, broad project-manager automation, web write routes, remote dashboard mutation, release claim, publish, tag, or permission-default change. |
| 2026-07-04 | Execution | Added `talos governance iteration-record preview/write`, a narrow owner-doc mutation gate that can append a row to a selected iteration execution table. Preview prints the owner doc, validation command, and exact row. Write requires `--confirm-preview`, writes only after resolving a single `docs/iterations/I###-*.md` owner doc, runs `scripts/validate_project_governance.sh .`, and rolls back the file if validation fails. |

## Closeout Evidence

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo test -p talos-cli governance_mutation`: passed, 5 governance mutation tests.
- `cargo check -p talos-cli`: passed.
- `cargo clippy -p talos-cli -- -D warnings`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `cargo run -p talos-cli -- governance iteration-record preview --iteration I096 --date 2026-07-04 --record-type validation --record ...`: passed and printed the owner doc, post-write validation command, and exact row without writing.
- `cargo run -p talos-cli -- governance iteration-record write --iteration I096 --date 2026-07-04 --record-type validation --record ... --confirm-preview`: passed, wrote the validation smoke row above, and reported `Validation: passed`.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

## Residuals

- No I096 residual blocks I097 activation.
- The shipped write path is intentionally narrow. It is not broad project-manager automation, web
  mutation, remote dashboard mutation, arbitrary file editing, release authority, or a substitute
  for future typed governance actions beyond iteration execution records.
