# Iteration I094: gix Upgrade And Git Boundary

> Document status: Complete
> Published plan date: 2026-07-04
> Planned objective: upgrade `gix` safely and audit Git host-fallback boundaries without expanding
> permission or publication authority.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `gix` upgrade attempt with tests and an operation-by-operation fallback decision
> matrix.
> Completed: 2026-07-04

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
| 2026-07-04 | Activation | Activated by the direct-owner high-risk execution set. Non-terminal inventory disposition: I085 remains Paused with MC107 real-terminal `/connect` walkthrough residual; I086-I089 remain planned product-hardening shells; I095-I097 remain planned and depend on I094/I095/I096 completion or explicit pause; I090-I093 remain Complete and are not reopened. I094 starts with dependency upgrade/fallback audit only: no permission-default change, destructive Git operation, publish, tag, or release action is authorized. |
| 2026-07-04 | Execution | Upgraded `talos-tools` to request `gix = "0.85"` and confirmed `Cargo.lock` resolves `gix 0.85.0`. The explicit feature set stayed unchanged: `basic`, `status`, `revision`, `blob-diff`, `index`, and `sha1`; no network or worktree-mutation features were enabled. Added an unavailable-host regression for retained host-`git` fallbacks. |
| 2026-07-04 | Fallback audit | GIT-001 now records the `gix 0.85.0` fallback matrix. Native `gix` remains accepted for read-only local status/diff/log/show/branches. Add/commit, push, pull, and checkout remain structured host-`git` fallbacks. Stash/reset/merge/rebase/tags/remotes remain deferred future scope. |

## Closeout Evidence

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p talos-tools`: passed.
- `cargo test -p talos-tools git`: passed.
- `cargo test -p talos-tools`: passed, 226 unit tests plus 18 integration tests and doctests.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `cargo tree --invert gix@0.85.0 -e features`: passed; feature tree stayed within the accepted
  Talos feature set.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

## Residuals

- No I094 residual blocks I095 activation.
- Host-`git` fallback replacement remains future GIT-001 work and requires a scoped iteration when
  `gix` exposes complete Talos-ready workflows for the retained fallback operations.
