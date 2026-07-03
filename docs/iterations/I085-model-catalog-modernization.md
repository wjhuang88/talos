# Iteration I085: Model Catalog Modernization — talos-models, /model, /connect

> Document status: Active
> Published plan date: 2026-07-03
> Planned objective: Replace the ad-hoc model catalog pipeline (hand-maintained TOML + broken
> models.dev import + auth-gated picker) with a proper `talos-models` crate backed by SQLite,
> a catalog-aware resolver that preserves config precedence, a deterministic built-in data refresh
> path, and an explicit split between model selection (`/model`) and provider registration
> (`/connect`).
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: users see all available models from all connected providers in `/model`
> (grouped, searchable), and can connect new providers from the full models.dev catalog via
> `/connect`.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| MC100 | MODEL-001/MODEL-005 | Planned | ADR-013, ADR-008, ADR-023 | Shared catalog types + new `talos-models` crate with SQLite catalog store |
| MC101 | MODEL-001 | Planned | MC100 | models.dev layered fetch (models.json + api.json) with correct format parsing |
| MC102 | MODEL-001 | Planned | MC101 | deterministic built-in data refresh: `BUILD_MODELS=1` updates `models.toml` only when explicitly requested |
| MC103 | MODEL-001/MODEL-004 | Planned | MC100 | catalog-aware model resolver: catalog.db + builtin fallback + user config overlay |
| MC104 | MODEL-005 | Planned | MC103 | `/connect` command: full provider list from catalog.db, credential + optional endpoint setup, config.toml merge |
| MC105 | MODEL-005 | Planned | MC103 | `/model` refactor: only directly-usable models, grouped by provider, current model on top |
| MC106 | MODEL-005/TUI-010 | Planned | MC105 | Group-aware search filtering: hide empty provider groups during search |
| MC107 | MODEL-001 | Planned | MC100-MC106 | Docs, validation, and residual closeout |

### Scope

- Move shared catalog data types (`ModelMetadata`, pricing, capabilities, provider info) to a
  non-cyclic boundary, preferably `talos-core::model`.
- Create `talos-models` crate: SQLite-backed model catalog store (providers, models, pricing),
  depending on shared catalog types rather than `talos-config`.
- Fix `import_models_dev()` to handle the actual models.dev JSON format (object, not array).
- Implement layered fetch: `models.json` (172KB, capabilities) + `api.json` (3.4MB, pricing + provider routing).
- Add a deterministic built-in refresh path that regenerates `models.toml` when `BUILD_MODELS=1`
  is set; normal builds must never require network access.
- Introduce an explicit catalog-aware resolver used by CLI/TUI model workflows. The resolver reads
  `catalog.db` when available, falls back to compiled-in `models.toml`, then applies user
  `config.toml` overrides.
- Keep `/model` as the single model-selection command and add `/connect` for provider registration.
- `/model`: only directly-usable models (provider credential present), grouped by provider, current
  model in a special top group.
- `/connect`: full provider list from models.dev, credential entry, optional custom endpoint
  (`base_url`) entry, and merges `[providers.xxx]` into config.toml without overwriting unrelated
  existing provider fields.
- Both commands support fuzzy search filtering with group-aware hide-empty-groups behavior.

### Non-Goals

- No auto-fetch at startup (catalog.db is populated by explicit `--fetch-models` or `build.rs`).
- No provider marketplace or remote model discovery beyond models.dev.
- No change to MODEL-003 reasoning implementation.
- No change to ADR-013 provider config schema boundary (still schema/config only, no dynamic loading).
- No change to the `LanguageModel` trait or provider adapters.
- No per-provider compatibility overrides (omp.sh-style `requiresReasoningContentForToolCalls`).

### Acceptance

- Given a user has Anthropic and OpenAI API keys set, when they run `/model`, then they see only
  Anthropic and OpenAI models grouped by provider, with the current model in a top group.
- Given a user runs `/connect`, when the catalog.db is populated, then they see all 100+ providers
  from models.dev with model counts and base URLs.
- Given a user selects a provider in `/connect` and enters a credential and optional custom
  endpoint, when the settings are saved, then `[providers.xxx]` is merged into config.toml and the
  provider's models appear in `/model`.
- Given a user types in the search box in `/model` or `/connect`, when the filter matches some
  models/providers, then only matching items and their non-empty group headers are shown.
- Given `BUILD_MODELS=1 cargo build` is run, when network is available, then `models.toml` is
  deterministically regenerated from models.dev input and the build succeeds.
- Given a normal `cargo build` (no `BUILD_MODELS`), when network is unavailable, then the build
  succeeds using the existing committed `models.toml`.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-models`
- `cargo test -p talos-config`
- `cargo test -p talos-conversation`
- `cargo test -p talos-tui`
- `cargo test -p talos-cli`
- `cargo clippy -p talos-models -p talos-config -p talos-conversation -p talos-tui -- -D warnings`
- `cargo test --workspace` at closeout
- `scripts/validate_project_governance.sh .`
- Manual TUI verification: `/model` shows grouped models, `/connect` shows full provider list

### Documentation To Update

- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/MODEL-005-interactive-model-selection.md`
- `docs/backlog/active/CONF-002-model-onboarding.md`
- `docs/reference/REFERENCE-PROJECTS.md` (models.dev endpoint reference)
- `docs/reference/config.reference.toml` (`/connect` workflow)
- README (if user-visible model selection behavior changes)
- `docs/BOARD.md` after owner docs
- `docs/iterations/README.md` I085 entry

### Risks And Rollback

- Risk: SQLite schema migration complexity. Mitigation: store a schema version (`PRAGMA user_version`
  or `catalog_meta.schema_version`) and route all opens through an idempotent migration function.
  Corrupt or incompatible catalog DBs degrade to the built-in TOML fallback.
- Risk: models.dev API format changes. Rollback: `import_models_dev()` handles both object and
  array formats; unknown fields are ignored.
- Risk: `build.rs` fetch fails in CI. Rollback: `BUILD_MODELS` defaults to off; committed
  `models.toml` is the fallback.
- Risk: `/model` and `/connect` split still requires users to learn where provider setup lives.
  Mitigation: keep `/model` as the familiar model-selection entry point and provide a visible
  action/message that directs users to `/connect` when no provider credential is present.

### Staged Implementation

Implement data plumbing before changing interactive command behavior.

1. MC100-MC103 first: shared types, `talos-models`, models.dev parsing/fetch, schema migration,
   deterministic built-in refresh, and catalog-aware resolver with tests.
2. MC104-MC107 second: `/connect`, `/model`, group-aware filtering, docs, manual TUI validation.

The second stage should not begin until the resolver can return the precedence chain required by
MODEL-001: user config overrides catalog.db, catalog.db overrides built-in TOML, and failures fall
back without blocking startup.

## Design Notes

### talos-models Crate Architecture

```
crates/talos-models/
├── Cargo.toml          # rusqlite/bundled, talos-core::model types, serde
├── src/
│   ├── lib.rs          # public API: ModelCatalog, queries
│   ├── store.rs        # SQLite schema, CRUD, connection management
│   ├── fetch.rs        # models.dev HTTP fetch (models.json + api.json)
│   ├── import.rs       # JSON → shared ModelMetadata/ProviderInfo parsing
│   └── types.rs        # CatalogEntry, FetchStatus, crate-owned errors
└── tests/
    └── integration.rs  # round-trip, query, filter tests
```

**Public API:**
```rust
pub struct ModelCatalog {
    // wraps a rusqlite::Connection to ~/.talos/catalog.db
}

impl ModelCatalog {
    pub fn open(path: &Path) -> Result<Self, CatalogError>;
    pub fn open_or_create(path: &Path) -> Result<Self, CatalogError>;

    // Seed from embedded models.toml when explicitly requested.
    pub fn seed_from_builtin(&self, models: &[ModelMetadata]) -> Result<(), CatalogError>;

    // Fetch from models.dev
    pub fn fetch_models_json(&self) -> Result<usize, CatalogError>;  // 172KB
    pub fn fetch_api_json(&self) -> Result<usize, CatalogError>;     // 3.4MB

    // Queries never panic; database errors are returned to the caller so the
    // resolver can fall back to built-in data.
    pub fn all_models(&self) -> Result<Vec<ModelMetadata>, CatalogError>;
    pub fn models_by_provider(&self, provider: &str) -> Result<Vec<ModelMetadata>, CatalogError>;
    pub fn all_providers(&self) -> Result<Vec<ProviderInfo>, CatalogError>;
    pub fn find_model(&self, provider: &str, model_id: &str) -> Result<Option<ModelMetadata>, CatalogError>;
    pub fn last_refreshed(&self) -> Result<Option<DateTime<Utc>>, CatalogError>;

    // Search (SQLite LIKE or FTS if needed later)
    pub fn search_models(&self, query: &str) -> Result<Vec<ModelMetadata>, CatalogError>;
    pub fn search_providers(&self, query: &str) -> Result<Vec<ProviderInfo>, CatalogError>;
}
```

**SQLite Schema:**
```sql
CREATE TABLE IF NOT EXISTS providers (
    id          TEXT PRIMARY KEY,         -- "anthropic", "openai", etc.
    name        TEXT NOT NULL,            -- display name
    api_base_url TEXT,                    -- "https://api.anthropic.com/v1/messages"
    env_var     TEXT,                     -- "ANTHROPIC_API_KEY"
    npm_package TEXT,                     -- "@ai-sdk/anthropic"
    doc_url     TEXT,
    source      TEXT NOT NULL DEFAULT 'builtin'  -- builtin | models_dev
);

CREATE TABLE IF NOT EXISTS models (
    id              TEXT NOT NULL,        -- "claude-sonnet-4-5"
    provider        TEXT NOT NULL,        -- "anthropic"
    name            TEXT,
    context_limit   INTEGER,
    output_limit    INTEGER,
    reasoning       INTEGER DEFAULT 0,    -- bool
    tool_call       INTEGER DEFAULT 0,    -- bool
    structured_output INTEGER DEFAULT 0,  -- bool
    attachment      INTEGER DEFAULT 0,    -- bool
    release_date    TEXT,
    source          TEXT NOT NULL DEFAULT 'builtin',
    PRIMARY KEY (provider, id),
    FOREIGN KEY (provider) REFERENCES providers(id)
);

CREATE TABLE IF NOT EXISTS pricing (
    model_id        TEXT NOT NULL,
    provider        TEXT NOT NULL,
    input_per_1m    REAL,
    output_per_1m   REAL,
    cache_read_per_1m REAL,
    cache_write_per_1m REAL,
    PRIMARY KEY (provider, model_id),
    FOREIGN KEY (provider, model_id) REFERENCES models(provider, id)
);

CREATE TABLE IF NOT EXISTS catalog_meta (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);
-- Keys: 'models_json_refreshed_at', 'api_json_refreshed_at', 'schema_version'
```

Migration rules:
- Use one migration entry point on every open.
- Version 1 creates the tables above and records the schema version.
- Future schema changes must add ordered migrations and tests for old-version fixtures.
- If migration fails because the database is corrupt or from an unsupported future version, callers
  must be able to ignore the DB and use built-in TOML data.

Runtime path:
- `talos-config` does not depend on `talos-models`.
- CLI/TUI code constructs `ModelCatalog` and passes it to a catalog resolver.
- The resolver applies user config overrides after reading catalog data, preserving
  MODEL-001 precedence.
- A refresh performed during a running TUI session rebuilds picker data from the resolver; no
  restart is required for the picker to see refreshed catalog rows.

### build.rs Design (talos-config)

```rust
// crates/talos-config/build.rs
fn main() {
    if std::env::var("BUILD_MODELS").is_ok() {
        // Fetch models.dev inputs.
        // Parse into shared ModelMetadata entries.
        // Sort providers/models deterministically.
        // Preserve source dates/provenance.
        // Regenerate src/models.toml.
        // Tell cargo to rerun-if-changed=src/models.toml
    }
    // Normal build: models.toml is already committed, include_str! works as-is
}
```

Key properties:
- Default: no-op (just `println!("cargo:rerun-if-changed=src/models.toml")`)
- `BUILD_MODELS=1`: fetch, parse, regenerate TOML, write to `src/models.toml`
- No recompile loop: `build.rs` writes to `src/`, but only when explicitly triggered
- The regenerated `models.toml` is committed to git (it's a generated artifact, but committed)
- Output must be stable across machines for the same upstream payload: deterministic ordering,
  deterministic formatting, no local timestamps except explicit upstream/source refresh dates.
- If models.dev data lacks pricing, the generated TOML must leave pricing absent rather than invent
  values. Runtime catalog pricing can come from `api.json`/`catalog.json`.

### /model Command Design

```
BottomPanelState::open_model_picker(config, catalog)

Panel layout:
┌─────────────────────────────────────────────────────┐
│  Current                                             │  ← special group
│    ▸ claude-sonnet-4-5   Anthropic   200K   $3/$15  │  ← current model highlighted
│  Anthropic                                           │  ← provider group
│    claude-opus-4-1       Anthropic   200K   $15/$75 │
│    claude-haiku-4-5      Anthropic   200K   $1/$5   │
│  OpenAI                                              │  ← provider group
│    gpt-4o                OpenAI      128K   $2.5/$10│
│    o3                    OpenAI      200K            │
│                                                      │
│  ↑↓ navigate   Enter switch   type to filter   Esc   │
└─────────────────────────────────────────────────────┘
```

Data source: catalog-aware resolver output filtered by `config.provider_authenticated(provider)`.
Grouping: by `m.provider`, sorted alphabetically (current model's provider group could be first).
Current model: first item, in a "Current" pseudo-group for visual prominence.

`provider_authenticated()` only proves that a non-empty inline key or environment variable value is
present. UI copy must avoid implying the key has been remotely verified; use wording such as
"configured" or "credential present".

### /connect Command Design

```
BottomPanelState::open_connect_picker(catalog, config)

Panel layout:
┌─────────────────────────────────────────────────────┐
│  Connected                                           │  ← already registered
│    Anthropic         25 models    credential present│
│    OpenAI            51 models    credential present│
│  Available                                           │  ← not yet connected
│    DeepSeek           4 models    api.deepseek.com  │
│    Google            22 models    generative...      │
│    Groq              15 models    —                  │
│    Mistral           30 models    —                  │
│    ...                                               │
│                                                      │
│  ↑↓ navigate   Enter connect   type to filter   Esc  │
└─────────────────────────────────────────────────────┘
```

Data source: `catalog.all_providers()`.
Connected: providers where config has a provider entry or credential source. Credential status is
"credential present" when `config.provider_authenticated(provider)` returns true.
Selecting an available provider → provider setup flow → credential input + optional custom endpoint
input → merges `[providers.xxx]`.

Credential storage follows the project secret boundary: ADR-023 permits local inline `api_key`
round-trips in the user's own `~/.talos/config.toml`; shared/sample docs should prefer
`api_key_env`. `/connect` must preserve existing provider settings and only fill missing or
explicitly changed fields.

Custom endpoint rules:
- The setup flow offers the catalog/default endpoint as the initial value when known.
- Users may leave the endpoint unchanged, clear it to use the built-in adapter default, or enter a
  custom `base_url` for gateways and OpenAI-compatible deployments.
- Existing `providers.<name>.base_url` is preserved unless the user explicitly changes it.
- Endpoint validation is syntactic only in this iteration (non-empty HTTP(S) URL when supplied);
  provider reachability checks are out of scope.

### Group-Aware Search Filtering

Current behavior: each `PanelItem` is independently filtered by the search query.
New behavior:
1. Split `panel_items` into groups (delimited by `PanelItemAction::Header` entries).
2. For each group, check if any non-header item matches the query.
3. If yes: include the header + matching items.
4. If no: skip the entire group (header + items).
5. The "Current" pseudo-group always shows (or follows the same rule).

Implementation: keep `BottomPanelState.items` flat for this iteration. Add a helper such as
`filtered_indices(&self, query: &str) -> Vec<usize>` that understands header-delimited groups and
returns indices into the original `items` vector. Render and navigation code must use those indices
so `selected_index` continues to refer to the original item list. Do not introduce
`Vec<Group { header, items }>` until multiple panel kinds need nested state.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-03 | Planning | Created from maintainer feedback that model catalog pipeline is broken (import format mismatch, write-only cache, auth-gated picker hides most providers). |
| 2026-07-03 | Correction | Live api.json verified (2.9MB fetch): top level is an object keyed by provider (150 providers); each `provider.models` is an **object map keyed by model id** with `cost`/`limit`/`modalities` fields. `import_models_dev()`'s flat `"provider/model-id"` key assumption and `pricing` field name both mismatch the real shape — importing live data would yield ~150 bogus provider-named entries. A `github-copilot` provider exists (25 models) whose auth cannot be expressed by the static schema; split to PROVIDER-003. |
| 2026-07-03 | Execution | Interim manual catalog refresh landed (commit `071449b`): every `models.toml` entry verified against live api.json; fabricated dated OpenAI ids (`gpt-4.1-2025-04-14`, `o3-2025-04-16`, `o4-mini-2025-04-16`) replaced with canonical ids; MiniMax-M3/google/zhipu/openrouter field corrections; 18 current-model additions (60 models, 12 providers). MC101 pipeline scope unchanged — this is a stopgap, not the fix. |
| 2026-07-03 | Activation | I085 activated after `v0.2.2` release closeout and programmer handoff preparation. Non-terminal inventory disposition: no conflicting active iteration; I086-I089 remain planned/deferred behind I085; I081-I083 remain superseded historical shells; paused high-risk/provider-plugin gates remain paused. Active scope starts with Stage 1 H100-H101 / MC100-MC103: shared catalog types, `talos-models`, SQLite catalog store/migrations, models.dev import parsing, gated built-in refresh, and catalog-aware resolver. Stage 2 `/model` and `/connect` work remains gated until resolver precedence tests pass. |
| 2026-07-03 | Execution | Stage 1 (S1-A through S1-E) implemented in a single session. **S1-A**: `ModelMetadata`, `ModelSource`, `ModelPricing`, `ModelCapabilities`, `ProviderInfo`, `ProviderSource`, and lookup helpers (`find_model`, `find_model_by_provider`, `models_with_id`) moved to `talos-core::model`; `talos-config` re-exports for backward compatibility. New `talos-models` crate added to workspace. **S1-B**: `ModelCatalog` SQLite store with `open`/`open_memory`/`seed`/`upsert_provider`/`upsert_model`/`all_models`/`models_by_provider`/`all_providers`/`find_model`/`search_models`/`search_providers`/`set_meta`/`get_meta`; schema v1 with `schema_version` table; corrupt DB propagates errors (no panic); incompatible schema versions rejected with `IncompatibleSchema` error. **S1-C**: `import_models_dev_api` and `import_models_dev_models` parsers in `talos-models` handle the real models.dev object-keyed format (provider → models map); both return `ImportResult { providers, models }` with full provider metadata (name, env var, API base URL, docs URL). **S1-D**: `crates/talos-config/build.rs` with `BUILD_MODELS=1` gated refresh via curl; normal builds stay offline; parse failure returns error and preserves committed TOML. **S1-E**: `Config::all_models_with_catalog` and `Config::resolve_model_limits_with_catalog` accept `Option<&[ModelMetadata]>` catalog overlay; `talos-config` does NOT depend on `talos-models`; catalog failure degrades gracefully to builtin TOML. |
| 2026-07-03 | Review | Acceptance review identified 3 gaps: (1) schema version not validated against `SCHEMA_VERSION`; (2) `BUILD_MODELS=1` could write empty `models.toml` on parse failure; (3) provider metadata not imported. All 3 fixed: schema version check added with `IncompatibleSchema` error + test; `unwrap_or_default()` replaced with error propagation + empty-result guard; provider-level fields (`name`, `env`, `api`, `doc`) parsed from real api.json format and returned via `ImportResult`; `seed()` accepts `&[ProviderInfo]` alongside `&[ModelMetadata]`. |
| 2026-07-03 | Execution | Stage 2 (MC104-MC106) implemented. **MC104 `/connect`**: new slash command registered; `ConnectPickerData`/`ConnectPickerItem` types added to conversation crate; `UiOutput::ConnectPicker` + `UiOutput::ConnectProviderRequest` variants; `PanelKind::ConnectPicker` + `open_connect_picker` in TUI; `handle_connect` + `handle_connect_with_credential` in CLI; credential routing via `connect_mode` flag on `CredentialRequestData`/`CredentialResponseData`; config.toml merge with api_key + api_key_env. **MC105 `/model` refactor**: `open_model_picker` now groups by provider using `BTreeMap`; current model in "Current" top group; unauthenticated providers show "Use /connect to set up" hint. **MC106**: provider-level headers inserted as non-navigable `PanelItemAction::Header` items. 1578 workspace tests pass. |

## Verification Evidence

- Stage 1 complete (2026-07-03). `cargo fmt --all -- --check`: clean.
- `cargo check --workspace`: clean.
- `cargo clippy -p talos-models -p talos-config -p talos-core -- -D warnings`: clean.
- `cargo test --workspace`: 1578 passed, 0 failed.
  - `talos-core`: 44 tests (6 new model module tests).
  - `talos-config`: 107 tests (9 new catalog-aware resolver tests).
  - `talos-models`: 36 tests (10 import parser tests with provider metadata, 26 store CRUD/query/search/schema-version tests).
- Normal build verified offline: `BUILD_MODELS` not set → no network calls.
- `BUILD_MODELS=1 cargo build`: not run (network-dependent; requires models.dev
  reachability — recorded as validation not run per handoff instructions).
- S1-E resolver precedence tests prove: user config > catalog > builtin > fallback;
  catalog `None` does not block startup; empty catalog falls back to builtin.
- Acceptance review gaps fixed: (1) `IncompatibleSchema` error on version mismatch;
  (2) build.rs parse failure preserves committed TOML; (3) provider metadata
  (name, env, api, doc) parsed from api.json and stored via `seed(&[ProviderInfo], &[ModelMetadata], &str)`.

## Variance And Residuals

- `BUILD_MODELS=1` refresh path implemented but not validated with live network
  fetch (requires models.dev reachability). Code compiles and the non-network
  path is verified. Stage 2 validation should run `BUILD_MODELS=1 cargo build`
  and verify `git diff -- crates/talos-config/src/models.toml`.
- Stage 2 (`/model` and `/connect` interactive changes) remains gated. The
  catalog-aware resolver methods (`all_models_with_catalog`,
  `resolve_model_limits_with_catalog`) are available for Stage 2 wiring.

## Retrospective

- Pending.
