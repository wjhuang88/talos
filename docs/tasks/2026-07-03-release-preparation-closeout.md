# Release Preparation Closeout — 2026-07-03

**Status**: Complete (v0.2.2 tag pushed 2026-07-03)

## Purpose

Close currently executing work before preparing the next release. The original closeout did not
authorize a tag, push, crate publish, GitHub release, or version claim. On 2026-07-03 the maintainer
explicitly authorized a release with tag push only and no build wait; the release version is
`v0.2.2`.

## Scope

- Close I084 Experience Reliability from Review to Complete.
- Sync owner docs and derived views for UX-001, MODEL-003, PROVIDER-002, BOARD, backlog, and
  iterations index.
- Keep I085 Model Catalog Modernization as Planned/post-release candidate, not part of this release
  unless explicitly activated.
- Preserve the simplified command plan: `/model` remains the model-selection command; `/connect`
  handles provider setup and optional custom endpoint (`base_url`).

## Closeout Decisions

- I084 is release-facing and complete.
- I085 is planned only and must not enter the release without a new activation/validation pass.
- Current release preparation is a work freeze on new feature activation.
- `v0.2.2` is a patch release scope: I084 reliability closeout, the thinking-label animation polish,
  the models.dev import compatibility fix, and documentation/planning sync.
- Push only the git tag for this release. Do not push `main`, wait for GitHub Actions, publish
  crates, or create a manual GitHub Release in this task.

## Validation Evidence

- 2026-07-03: `v0.2.2` tag created and pushed (tag commit dated 2026-07-03 16:25 +0800,
  verified via `git tag`/`git log`). The hardening plan's "start after `v0.2.2` tag push"
  precondition is satisfied.
- 2026-07-03 pre-tag validation for `v0.2.2`:
  - `cargo fmt --all -- --check` — pass.
  - `cargo check --workspace` — pass.
  - `cargo clippy --workspace -- -D warnings` — pass.
  - `cargo test --workspace` — pass.
  - `scripts/validate_project_governance.sh .` — pass, 0 warnings.
  - `scripts/check_publish_guard.sh .` — pass.
  - `scripts/validate_public_site.sh` — pass, 14 HTML files, 0 errors, 0 warnings.
  - `git diff --check` — pass.
- Earlier I084 closeout validation:
- `cargo fmt --all -- --check` — pass.
- `cargo check --workspace` — pass.
- `cargo clippy --workspace -- -D warnings` — pass.
- `cargo test -p talos-config` — pass.
- `cargo test -p talos-cli --test mcp_client_e2e` — pass after a transient full-workspace failure.
- `cargo test --workspace` — pass.
- `scripts/validate_project_governance.sh .` — pass, 0 warnings.

## Residuals

- The first full `cargo test --workspace` run had a transient
  `mcp_client_e2e_routes_tool_call_through_fixture_server` failure. The targeted test passed on
  rerun, and the final full workspace test passed.
- I085 remains the next product candidate for catalog modernization.
- Release tag push is allowed only for `v0.2.2`; crate publish and manual GitHub Release remain out
  of scope.
