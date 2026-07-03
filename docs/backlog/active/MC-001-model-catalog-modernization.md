# MC-001: Model Catalog Modernization — talos-models, /model, /connect

| Field | Value |
|-------|-------|
| Story ID | MC-001 (Epic) |
| Priority | P1 |
| Status | Planned |
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

## Acceptance Criteria

- [ ] Shared catalog types are moved to a non-cyclic boundary.
- [ ] `talos-models` crate exists with SQLite-backed `ModelCatalog` API.
- [ ] catalog.db has an explicit versioned migration path and corrupt/incompatible DB fallback.
- [ ] `import_models_dev()` correctly parses the actual models.dev JSON format.
- [ ] `build.rs` regenerates `models.toml` from models.dev when `BUILD_MODELS=1`.
- [ ] Catalog-aware resolver reads from catalog.db when available, falls back to `builtin_models()`,
      then applies user config overrides.
- [ ] `/model` shows only credential-present provider models, grouped by provider, current model on top.
- [ ] `/connect` shows full provider list with credential entry and optional custom endpoint
      (`base_url`) entry that merges provider config into config.toml.
- [ ] Both commands support group-aware search filtering (empty groups hidden).
- [ ] Tests cover: catalog CRUD, import parsing, query/filter, config.toml write, TUI rendering.
- [ ] `cargo test --workspace` passes.
- [ ] End-to-end: `/model` and `/connect` reachable from real TUI binary.

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
