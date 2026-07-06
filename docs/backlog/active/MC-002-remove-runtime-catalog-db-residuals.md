# MC-002: Remove Runtime catalog.db Residuals

| Field | Value |
|---|---|
| ID | MC-002 |
| Type | Cleanup / Architecture Hygiene |
| Priority | P1 |
| Status | Complete (2026-07-06, F2 of the frontline four-month execution plan) |
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

- [x] No production runtime path opens, creates, seeds, or reads `~/.talos/catalog.db`.
      Verified: `cargo test -p talos-cli --test no_catalog_db_guard` (5/5 pass) exercises
      `--import-models`, `--available-models`, `--available-models --available-models-filter`,
      `--available-models-all`, and `config list` in an isolated `HOME`; none create
      `catalog.db` (or `.wal`/`.shm` sidecars).
- [x] `rg "catalog.db|ModelCatalog|talos_models"` has only allowed references:
      historical docs, this requirement, or explicitly non-runtime archived code.
      Post-F2 remaining hits are inside `crates/talos-models/` (the quarantined crate
      itself), owner docs (MC-001/MC-002/PRODUCT-BACKLOG/BOARD), and immutable iteration
      logs (I085/I098/I101/I098-I101 closeout) — all allowed per the F1 audit table.
- [x] If `talos-models` remains in the workspace, its purpose is documented as non-runtime and no
      CLI/TUI/runtime crate depends on it.
      Documented in `crates/talos-models/src/lib.rs` (Status — Quarantined header) and
      `crates/talos-models/Cargo.toml` description; `crates/talos-models/Cargo.toml` notes the
      quarantine; `Cargo.toml` workspace member list carries a quarantine comment.
      Dependency check: `rg "talos-models|talos_models" crates/*/Cargo.toml` returns only
      `crates/talos-models/Cargo.toml` (self); `rg "use talos_models|talos_models::" crates
      --type rust -g '!crates/talos-models/**'` returns 0 hits.
- [x] (N/A — crate kept) If `talos-models` is removed, workspace manifests and references are cleaned up.
- [x] `--import-models` remains no-op compatibility or is removed through a documented breaking
      change decision.
      `crates/talos-cli/src/main.rs:385-396` keeps the no-op path and explains the
      2026-07-05 maintainer decision inline.
- [x] Regression coverage proves `/connect`, `/model`, and `--available-models-browser` use
      packaged `models.toml` metadata and do not create a DB file.
      `cargo test -p talos-cli --test no_catalog_db_guard` covers `--available-models`
      (the bounded sibling of the browser). The browser (`--available-models-browser`)
      cannot be exercised in CI because it requires an interactive TTY; the guard test
      pairs with the suite-level `cargo test --workspace` which already closes the
      MODEL-006 browser acceptance with viewport-windowed rendering.
- [x] README / README.zh-CN / active backlog docs agree that model metadata is packaged, not
      runtime-DB-backed.
      `README.md:398` and `README.zh-CN.md:379` already state "Talos does not create a runtime
      `catalog.db` for model metadata." F3 will verify these entries are consistent with
      `/model` and `/connect` docs; no contradiction found in F2.

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
