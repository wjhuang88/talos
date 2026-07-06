# MC-002: Remove Runtime catalog.db Residuals

| Field | Value |
|---|---|
| ID | MC-002 |
| Type | Cleanup / Architecture Hygiene |
| Priority | P1 |
| Status | Planned |
| Source | Maintainer correction 2026-07-06 — runtime `catalog.db` behavior was superseded and should not be kept alive by new work |
| Depends on | MC-001 runtime catalog decision |
| Blocks | Future model/provider metadata work that might accidentally reintroduce runtime DB behavior |

## Problem

The accepted product behavior after the 2026-07-05 maintainer decision is:

- Fresh installs do not create `~/.talos/catalog.db`.
- `/model` and `/connect` use packaged offline `models.toml` plus user config.
- Model/provider metadata refresh happens only at build time through `BUILD_MODELS=1`.
- `--import-models` is compatibility/no-op and must not write a runtime DB.

However, residual code and historical docs from the earlier MC-001 implementation still mention or
implement SQLite-backed `ModelCatalog` / `catalog.db`. Even when no runtime path calls it, leaving
that surface in place creates confusion and makes future fixes likely to update a dead path.

## Goal

Remove or quarantine runtime `catalog.db` residuals so future model catalog work cannot accidentally
revive the old DB-backed behavior.

## In Scope

- Audit all current references to `catalog.db`, `ModelCatalog`, `talos-models`, and
  `--import-models`.
- Confirm whether `talos-models` has any remaining non-runtime value:
  - If no production crate depends on it, remove it from the workspace or move it to an archived
    test/reference location.
  - If parser code is still useful, migrate parser/build-time pieces into the build-time model
    refresh path and delete the SQLite store.
- Remove runtime DB creation/open/seed code if any still exists.
- Keep `--import-models` as a no-op compatibility warning only if removing the flag would be a
  user-facing breaking change.
- Update owner docs so active docs do not describe runtime catalog DB behavior as planned or valid.
- Add a guard test or static check proving Talos CLI/TUI cannot create `~/.talos/catalog.db`.

## Out of Scope

- Do not remove packaged `crates/talos-config/src/models.toml`.
- Do not remove `BUILD_MODELS=1` build-time refresh.
- Do not change `/model` or `/connect` user workflows except to ensure they stay DB-free.
- Do not reintroduce runtime network refresh.
- Do not implement a replacement database.

## Acceptance Criteria

- [ ] No production runtime path opens, creates, seeds, or reads `~/.talos/catalog.db`.
- [ ] `rg "catalog.db|ModelCatalog|talos_models"` has only allowed references:
      historical docs, this requirement, or explicitly non-runtime archived code.
- [ ] If `talos-models` remains in the workspace, its purpose is documented as non-runtime and no
      CLI/TUI/runtime crate depends on it.
- [ ] If `talos-models` is removed, workspace manifests and references are cleaned up.
- [ ] `--import-models` remains no-op compatibility or is removed through a documented breaking
      change decision.
- [ ] Regression coverage proves `/connect`, `/model`, and `--available-models-browser` use
      packaged `models.toml` metadata and do not create a DB file.
- [ ] README / README.zh-CN / active backlog docs agree that model metadata is packaged, not
      runtime-DB-backed.

## Validation Plan

```sh
rg "catalog.db|ModelCatalog|talos_models" crates docs README.md README.zh-CN.md
cargo test -p talos-cli model
cargo test -p talos-cli connect
cargo check --workspace
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

## Required Reads

- `docs/backlog/active/MC-001-model-catalog-modernization.md`
- `docs/iterations/I085-model-catalog-modernization.md`
- `crates/talos-config/src/model.rs`
- `crates/talos-config/build.rs`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/models_browser.rs`
- `crates/talos-cli/src/main.rs`
- `crates/talos-models/` if still present
