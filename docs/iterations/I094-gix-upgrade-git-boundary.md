# Iteration I094: gix Upgrade And Git Boundary

> Document status: Planned
> Published plan date: 2026-07-04
> Planned objective: upgrade `gix` safely and audit Git host-fallback boundaries without expanding
> permission or publication authority.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `gix` upgrade attempt with tests and an operation-by-operation fallback decision
> matrix.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `GIT-001` | Embedded Git tools | P0-P2 complete; gix tracking active | ADR-010 | `gix 0.85.0` upgrade attempt and fallback matrix. |
| `ARCH-030` Git root | Architecture residuals | Tracking | Current source audit | Git root risk classified before further Git expansion. |

### Scope

- Attempt `gix 0.84.0 -> 0.85.0`.
- Keep current feature set unless an explicit test-backed reason exists.
- Audit `git_push`, `git_pull`, `git_checkout`, `git_add`, `git_commit`, and future P3 operations.
- Preserve structured host-`git` fallbacks where `gix` does not provide a complete safe workflow.

### Non-Goals

- No new native Git dependency.
- No credential workflow, issue sync, release publish, tag creation, force push, reset, clean, rebase,
  or remote management.
- No permission-default changes.

### Acceptance

- Given `gix` is upgraded or rejected,
  When validation completes,
  Then GIT-001 records the exact version, feature flags, and reason.
- Given host fallbacks remain,
  When the fallback matrix is updated,
  Then each fallback has keep/replace/defer rationale and tests.
- Given Git tools are permission-sensitive,
  When tests run,
  Then write/execute paths remain permission-gated.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test -p talos-tools git`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/GIT-001-embedded-git-tools.md`
- `docs/backlog/active/ARCH-030-remaining-production-root-residual-register.md` if the Git root
  risk changes
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`

### Risks And Rollback

- Risk: `gix` API or transitive features change Git tool behavior.
- Rollback: revert dependency update, keep current lockfile, and record blocker in GIT-001.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
