# MC-001: Model Catalog Modernization — talos-models, /model, /connect

| Field | Value |
|-------|-------|
| Story ID | MC-001 (Epic) |
| Priority | P1 |
| Status | In Progress |
| Origin | Maintainer feedback 2026-07-03 — model catalog pipeline broken: import format mismatch, write-only cache, auth-gated picker hides most providers, only 12 hardcoded providers visible |
| Owns | MODEL-001 (catalog data layer), MODEL-005 (interactive selection), CONF-002 (onboarding) |
| Parent Epic | None (this IS the epic) |

## Problem

Talos's model catalog pipeline has three broken links:

1. **Import format mismatch**: `import_models_dev()` expects a JSON array, but models.dev returns
   an object keyed by `"provider/model-id"`. Every import silently fails.
2. **Write-only cache**: `--import-models` writes to `~/.talos/cache/models/models.json` but
   `all_models()` never reads it back. The cached data is dead.
3. **Auth-gated picker**: `/model` only shows models from authenticated providers. Users with one
   API key see only 5 models instead of the full catalog. There is no way to discover or connect
   new providers from within the TUI.

Additionally, the built-in dataset is only 46 models across 12 providers, hand-maintained in
`models.toml`. models.dev has 232 models across 100+ providers with pricing data.

## Program Shape

| Slice | Owner | Outcome |
|---|---|---|
| MC-A | MODEL-001 | Shared catalog types + `talos-models` crate with SQLite catalog store replaces ad-hoc TOML + JSON cache |
| MC-B | MODEL-001 | models.dev layered fetch with correct format parsing and `build.rs` gated refresh |
| MC-C | MODEL-005 | `/model` and `/connect` commands split model selection from provider setup |

## Implementation Principles

- Shared catalog types live outside `talos-config` and `talos-models` to avoid dependency cycles
  (preferred location: `talos-core::model`).
- The catalog-aware resolver is the runtime query path; `models.toml` is the compile-time
  seed/fallback.
- `talos-config` does not implicitly open SQLite. CLI/TUI code passes an optional catalog handle to
  the resolver.
- User config precedence: user `config.toml` overrides catalog.db overrides compiled-in `models.toml`.
- `/model` shows only directly-usable models (credential present for provider).
- `/connect` shows the full provider list from catalog.db and merges provider config, including an
  optional custom `base_url`, without overwriting unrelated existing fields.
- Both commands support fuzzy search with group-aware filtering using filtered original indices.
- `build.rs` fetches models.dev only when `BUILD_MODELS=1`; normal builds use committed TOML.
- Built-in refresh output is deterministic and reviewable: stable ordering, stable formatting,
  source provenance, and no invented pricing.
- catalog.db uses an explicit schema version and migration entry point; DB failures degrade to the
  built-in TOML fallback.

## Staging

Implement in two stages:

1. MC100-MC103: shared types, catalog store, fetch/import, deterministic built-in refresh, resolver.
2. MC104-MC107: `/connect`, `/model`, group-aware filtering, docs, validation.

The command split must not begin until the resolver precedence path is tested.

## Execution Log

| Date | Status | Record |
|---|---|---|
| 2026-07-03 | In Progress | I085 activated after `v0.2.2` closeout. Stage 1 starts with MC100-MC103 only: shared catalog types, `talos-models`, SQLite catalog store/migrations, models.dev import parsing, gated built-in refresh, and catalog-aware resolver. Stage 2 `/model` and `/connect` work remains blocked until resolver precedence tests pass. Programmer handoff: `docs/tasks/2026-07-03-programmer-handoff-i085-model-catalog.md`. |
| 2026-07-03 | Stage 1 Complete | S1-A through S1-E implemented + acceptance review gaps fixed. Shared catalog types in `talos-core::model`; `talos-models` crate with SQLite `ModelCatalog` (schema v1, version validation, CRUD, query, search, corrupt DB fallback); models.dev api.json/models.json parsers returning `ImportResult { providers, models }` with full provider metadata; `build.rs` gated refresh (`BUILD_MODELS=1`, parse failure preserves committed TOML); catalog-aware resolver via `Config::all_models_with_catalog` / `resolve_model_limits_with_catalog` (talos-config does NOT depend on talos-models). 1578 workspace tests pass. Resolver precedence verified: user config > catalog > builtin > fallback; `None` catalog does not block. Stage 2 unblocked. |
| 2026-07-03 | Stage 2 Partial | Commit `d7e37df` added the `/connect` command skeleton, provider credential routing, config merge for api_key/api_key_env, and provider-grouped `/model` display. It is not accepted as full Stage 2 because `/connect` still uses built-in catalog data instead of catalog.db, optional base_url setup is missing, group-aware search filtering is incomplete, and MC107 docs/closeout remain pending. |
| 2026-07-03 | Stage 2 Gaps Closed | All 3 blocking gaps from the second review fixed. `/connect` now opens `catalog.db` once at TUI startup (`CatalogSnapshot`/`open_catalog_snapshot`) and prefers live provider name/`api_base_url`/`doc_url`/model-count data, falling back to `builtin_models()` when the catalog is missing/fresh/corrupt/schema-incompatible — never blocking startup. Connect credential flow gained an optional two-phase base URL field (`CredentialField::ApiKey` → `BaseUrl`); merge precedence is existing `providers.<name>.base_url` > catalog default > `None`, and saving never overwrites unrelated provider fields or clears an existing value when the user leaves the field blank. `BottomPanelState::filtered_indices` now implements group-aware search (hides a provider group's header when no sibling item matches; "Current" pseudo-group follows the same rule) with `selected_index` fixed to always be a raw `self.items` index — this also fixed a latent filtered-vs-raw index inconsistency in the pre-existing slash-command menu. 1605 workspace tests pass (up from 1578); governance validation clean; 3 pre-existing e2e failures confirmed unrelated via `git stash` A/B (local dev machine `~/.talos/config.toml` has `model = ""`). MC107 (README `/connect` doc, manual runtime TUI verification) remains open. |
| 2026-07-04 | Stage 2 Post-review Fix | Fixed user acceptance gaps: `/model` no longer shows unauthenticated provider/setup rows; `/connect` owns provider setup and falls back to packaged `models.toml` plus built-in endpoint metadata when `catalog.db` is absent; missing `catalog.db` is not created by startup probing; slash prefix filtering selects `/model` for `/mo`; group headers use higher-contrast styling. |

## Acceptance Criteria

- [x] Shared catalog types are moved to a non-cyclic boundary. (`talos-core::model`, S1-A)
- [x] `talos-models` crate exists with SQLite-backed `ModelCatalog` API. (S1-B)
- [x] catalog.db has an explicit versioned migration path and corrupt/incompatible DB fallback.
      (schema v1 + `IncompatibleSchema` error; `open_catalog_snapshot` degrades to `builtin_models()`)
- [x] `import_models_dev()` correctly parses the actual models.dev JSON format.
      (`talos-models::import_models_dev_api`/`import_models_dev_models` handle the real
      provider-keyed-object-with-nested-models shape verified against the live `anomalyco/models.dev`
      source, including provider-level `name`/`env`/`api`/`doc` fields)
- [x] `build.rs` regenerates `models.toml` from models.dev when `BUILD_MODELS=1`. (S1-D; live-network
      run itself remains a residual — see I085 Variance And Residuals)
- [x] Catalog-aware resolver reads from catalog.db when available, falls back to `builtin_models()`,
      then applies user config overrides. (`Config::all_models_with_catalog`/`resolve_model_limits_with_catalog`, S1-E)
- [x] `/model` shows only credential-present provider models, grouped by provider, current model on top.
      (`open_model_picker` provider grouping + "Current" top group)
- [x] `/connect` shows full provider list with credential entry and optional custom endpoint
      (`base_url`) entry that merges provider config into config.toml. (catalog-backed `/connect` +
      two-phase credential/base_url flow; falls back to `builtin_models()`-derived provider list when
      `catalog.db` is unseeded/unavailable)
- [x] Both commands support group-aware search filtering (empty groups hidden).
      (`BottomPanelState::filtered_indices`; `/model` and `/connect` both use `is_picker()` +
      `TuiState::panel_query()` "type to filter")
- [x] Tests cover: catalog CRUD, import parsing, query/filter, config.toml write, TUI rendering.
      (`talos-models` 36 tests; `talos-config` 9 resolver tests; `talos-cli` 11 catalog/connect tests;
      `talos-tui` 19 base_url + group-filtering tests)
- [x] `cargo test --workspace` passes. (1605 passed; 3 pre-existing e2e failures unrelated to MC-001 —
      confirmed identical on unmodified `main` via `git stash`, caused by this dev machine's local
      `~/.talos/config.toml` having an empty `model` field, not by any code in this epic)
- [ ] End-to-end: `/model` and `/connect` reachable from real TUI binary.
      (Not yet performed in this session — unit/integration tests cover the logic paths, but a live
      interactive terminal walkthrough of `/connect` has not been recorded. MC107 residual.)

## Required Reads

- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/MODEL-005-interactive-model-selection.md`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
- `docs/backlog/active/CONF-002-model-onboarding.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `crates/talos-config/src/model.rs` — current `builtin_models()`, `import_models_dev()`
- `crates/talos-config/src/config.rs` — `all_models()`, `provider_authenticated()`
- `crates/talos-config/src/builtin.rs` — `builtin_provider_config()`
- `crates/talos-cli/src/model_lifecycle.rs` — `build_model_picker_data()`
- `crates/talos-tui/src/state.rs` — `BottomPanelState`, `open_model_picker()`
- `crates/talos-conversation/src/types.rs` — `ModelPickerData`, `UiOutput`
- `crates/talos-conversation/src/command_registry.rs` — `CommandDefinition`
