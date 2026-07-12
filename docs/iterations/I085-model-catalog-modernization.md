# Iteration I085: Model Catalog Modernization — talos-models, /model, /connect

> Document status: Complete (2026-07-12)
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
| 2026-07-03 | Execution | Stage 2 partial landed in commit `d7e37df`. **MC104 partial `/connect`**: new slash command registered; `ConnectPickerData`/`ConnectPickerItem` types added to conversation crate; `UiOutput::ConnectPicker` + `UiOutput::ConnectProviderRequest` variants; `PanelKind::ConnectPicker` + `open_connect_picker` in TUI; `handle_connect` + `handle_connect_with_credential` in CLI; credential routing via `connect_mode` flag on `CredentialRequestData`/`CredentialResponseData`; config.toml merge with api_key + api_key_env. Remaining MC104 work: wire `ModelCatalog`/catalog.db into `/connect` and support optional `base_url`. **MC105 mostly complete `/model` refactor**: `open_model_picker` groups by provider using `BTreeMap`; current model in "Current" top group; unauthenticated providers show "Use /connect to set up" hint. **MC106 partial**: provider-level headers inserted as non-navigable `PanelItemAction::Header` items, but group-aware search filtering with empty-group hiding remains incomplete. 1578 workspace tests passed for the partial slice. |
| 2026-07-03 | Review | Second acceptance review identified 3 blocking gaps against the `d7e37df` partial: (1) `/connect` used only `builtin_models()`, never `catalog.db`; (2) no optional custom `base_url` input in the connect flow; (3) MC106 group-aware search filtering was unimplemented (pickers always showed the full unfiltered list; `selected_index` had inconsistent filtered-vs-raw semantics). |
| 2026-07-03 | Execution | All 3 gaps closed. **Gap 1 (catalog wiring)**: `model_lifecycle::CatalogSnapshot` + `open_catalog_snapshot()` open `~/.talos/catalog.db` once at TUI startup (via `talos-models`, new `talos-cli` dependency); corrupt/incompatible existing DBs degrade to `builtin_models()` and never block startup. 2026-07-04 follow-up clarified missing-file behavior: a fresh install implicitly creates and seeds `catalog.db` from packaged `models.toml` on first catalog access. `build_connect_picker_data` prefers catalog provider name/`api_base_url`/`doc_url`/model counts when present. **Gap 2 (base_url)**: `CredentialRequestData.default_base_url` and `CredentialResponseData.base_url` added; `BottomPanelState` gained a `base_url_buffer` + `CredentialField` (ApiKey/BaseUrl) two-phase flow — in `connect_mode`, Enter on the API key advances to an optional base URL field before submitting; `handle_connect` resolves the default endpoint as existing `providers.<name>.base_url` > catalog `api_base_url` > `None`, and `handle_connect_with_credential` writes `cred.base_url` only when present, leaving existing/absent values untouched otherwise (never overwrites unrelated provider fields). **Gap 3 (MC106)**: `BottomPanelState::filtered_indices` reimplements filtering as group-aware (splits on `PanelItemAction::Header`, hides a group's header when no sibling item matches, "Current" follows the same rule — hidden if it doesn't match); `select_next`/`select_prev`/rendering were rewritten so `selected_index` is always a raw index into `self.items` (never a filtered-list position), fixing a latent inconsistency in the original slash-menu implementation; picker panels (`ModelPicker`/`ConnectPicker`/`SessionPicker`) now respond to "type to filter" via a new `TuiState::panel_query()` (previously hardcoded to `""`). 1605 workspace tests pass (up from 1578); 3 pre-existing e2e failures unrelated to this work (confirmed via `git stash` diff against unmodified `main`: local `~/.talos/config.toml` has an empty `model` field on this dev machine). |
| 2026-07-03 | Incident + Fix | Root-caused the 3 "pre-existing" e2e failures noted above: they were not actually pre-existing drift — this session's new `handle_connect_with_credential_*` tests called `Config::save()` (via the production function under test) before HOME-isolation was added, writing test-fixture data over the developer's real `~/.talos/config.toml`. Confirmed via file mtime + literal fixture strings found in the corrupted file. A stale `~/.talos/config.toml.bak-20260606` backup exists but is a month old and pre-dates the current provider-nested config schema, so the exact state between 2026-06-06 and this incident cannot be recovered; user opted to hand-edit their config rather than have it reconstructed. Separately, investigating the resulting "invalid configuration: 'model' is required" crash surfaced a real, independent, pre-existing design bug: `Config::load()` called `self.validate()` internally, so *any* on-disk config with an empty `model` (from an interrupted wizard save, manual edit, or this incident) hard-fails `Config::load()` before the three mode runners' own `needs_model_setup`/`needs_api_key` first-run-wizard logic ever gets a chance to run — and before `talos config set` (which itself calls `Config::load()` first) can be used to repair it. Fixed: `Config::load()` no longer calls `validate()`; mode runners already re-check `config.model.is_empty()` post-load (unchanged), and `run_config_set` still calls `.validate()` explicitly after applying an edit, before saving. Added regression tests `test_load_existing_file_with_empty_model_succeeds` and `test_load_then_set_model_recovers_from_empty_model_on_disk` (`talos-config/src/tests.rs`). While fixing this, discovered and fixed a second, related bug: two independent test-local `HOME`-mutation mutexes (`init_wizard.rs`'s pre-existing `ENV_MUTEX` and this session's new `mode_runners.rs` mutex) do not serialize against each other despite both mutating the same process-wide `HOME` env var, since `cargo test` runs tests in parallel threads within one process — causing flaky cross-module corruption of each other's temp-dir redirection. Consolidated into one shared `crate::test_support::HOME_ENV_MUTEX` used by both modules. 1610 workspace tests pass (up from 1605), 0 failures — the 3 previously-"pre-existing" e2e failures are now fixed and passing. |
| 2026-07-04 | Post-review Fix | User acceptance found four UX/robustness gaps. Fixed: `/model` now omits unauthenticated provider rows entirely (provider setup belongs in `/connect`); slash-menu filtering prioritizes command-name prefix matches so typing `/mo` then Enter completes `/model` instead of executing `/help`; `/connect` fallback now uses packaged `models.toml` plus built-in provider endpoint metadata; group headers render with a higher-contrast warning accent instead of dim text. Then-current follow-up required implicit `catalog.db` seeding on first missing-file access; that runtime DB path was superseded by the 2026-07-05 maintainer decision recorded below. Added regression coverage for the 2026-07-04 behavior. |
| 2026-07-04 | Pause | MC107 README/onboarding residual closed and catalog lifecycle verified by automated tests. I085 is paused before I090 activation because the only remaining item is a real terminal `/connect` walkthrough, which cannot be honestly claimed from the unattended/headless validation run. |
| 2026-07-12 | LT002 retry | Built `talos 0.3.4` and launched the real binary with disposable HOME under two allocated PTYs (`TERM=xterm-256color` and `TERM=dumb`). Both reached the terminal cursor-position query (`ESC[6n`) but the execution PTY is not a terminal emulator and could not return a valid cursor report before timeout. No real/fake credential was entered and no user config was touched. MC107 remains Paused; resume in Alacritty/iTerm/Terminal using the LT002 steps in the developer long-task record. |
| 2026-07-12 | MC107/LT002 complete | Re-ran the same disposable-HOME binary inside a detached `screen` terminal emulator. `/connect` rendered Connected/Available groups and 151 providers; filtering `openai`, navigating to OpenAI, and entering the credential view worked. Escape cancelled without entering or saving a credential. `/model` then rendered the configured DeepSeek model group. Talos exited normally with zero turns. I085 acceptance is complete. |

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
- Stage 2 gap-fix closeout (2026-07-03). `cargo fmt --all -- --check`: clean.
  `cargo check --workspace`: clean. `cargo test -p talos-models`: 36 passed.
  `cargo test -p talos-config model`: 44 passed. `cargo test -p talos-conversation`:
  94 passed. `cargo test -p talos-tui`: 240 passed (up from 233; +7 base_url tests
  +12 MC106 group-filtering tests). `cargo test -p talos-cli`: 129 unit tests passed
  (up from 118; +11 catalog/connect tests); 3 e2e tests failed at this point —
  initially misattributed as pre-existing/environment-dependent, later root-caused
  and fixed (see Incident + Fix log entry above and the `Config::load()` fix below).
  `cargo clippy -p talos-models -p talos-config -p talos-conversation -p talos-tui
  -p talos-cli -- -D warnings`: clean.
- `Config::load()` first-run/repair fix (2026-07-03). Removed the internal
  `self.validate()?` call so an on-disk config with an empty `model` loads
  successfully instead of hard-failing before the mode runners' own
  `needs_model_setup`/`needs_api_key` wizard logic or `talos config set`'s
  repair path can run. `cargo test -p talos-config`: 109 passed (up from 107;
  +2 regression tests). Shared `crate::test_support::HOME_ENV_MUTEX` replaces
  two independent per-module `HOME`-mutation mutexes in `talos-cli`
  (`init_wizard.rs`, `mode_runners.rs`) that were racing against each other
  under parallel test execution. `cargo test -p talos-cli`: 129 passed,
  reproduced stable across 5 repeated runs and a 16-thread stress run with no
  further `~/.talos/config.toml` mtime changes. `cargo test --workspace`:
  **1610 passed, 0 failed** (all 3 previously-failing e2e tests now pass).
  `scripts/validate_project_governance.sh .`: 0 warnings. `git diff --check`: clean.
- Catalog-in-`/connect` evidence: `build_connect_picker_data_uses_catalog_provider_metadata`,
  `build_connect_picker_data_catalog_takes_precedence_over_builtin`,
  `open_catalog_snapshot_missing_file_creates_seeded_catalog`,
  `open_catalog_snapshot_corrupt_file_returns_none_not_panic` (all in
  `talos-cli/src/mode_runners.rs::connect_tests`).
- Catalog lifecycle follow-up evidence (2026-07-04):
  `cargo test -p talos-cli import_models_creates_and_seeds_catalog_db -- --nocapture`,
  `cargo test -p talos-cli open_catalog_snapshot_missing_file_creates_seeded_catalog -- --nocapture`,
  and `cargo test -p talos-cli` all passed. Missing `catalog.db` is now implicitly created and
  seeded from packaged `models.toml`; `--import-models` refreshes the SQLite catalog from
  models.dev JSON.
- README onboarding follow-up (2026-07-04): the Interactive Commands section now explains
  `/model` vs `/connect`, provider credential setup, optional `base_url`, implicit first-access
  `catalog.db` creation from packaged `models.toml`, and `--import-models <path>` refresh.
- Runtime catalog supersession (2026-07-05): maintainer decision removed the runtime `catalog.db`
  path from the accepted behavior. Current runtime uses packaged offline `models.toml` only; model
  metadata refresh is build-time via `BUILD_MODELS=1`; `--import-models` is a no-op compatibility
  notice. The earlier implicit-DB evidence above is historical and no longer the active product
  contract.
- base_url merge evidence: `handle_connect_with_credential_writes_new_provider_api_key_and_base_url`,
  `handle_connect_with_credential_preserves_unrelated_provider_fields`,
  `handle_connect_with_credential_updates_base_url_when_provided`,
  `handle_connect_default_base_url_prefers_existing_config_over_catalog`,
  `handle_connect_default_base_url_falls_back_to_catalog` (same test module);
  TUI-side two-phase flow: `connect_mode_first_submit_advances_to_base_url_field`,
  `connect_mode_second_submit_returns_typed_base_url`,
  `connect_mode_empty_base_url_falls_back_to_default`,
  `connect_mode_empty_base_url_with_no_default_is_none`,
  `connect_mode_empty_api_key_cancels_without_advancing`,
  `non_connect_mode_ignores_base_url_and_submits_single_phase` (`talos-tui/src/state.rs`).
- MC106 group-aware filtering evidence: `model_picker_search_matching_provider_hides_other_groups`,
  `model_picker_search_no_match_hides_all_groups`, `model_picker_navigation_skips_headers_and_filtered_out_items`,
  `model_picker_select_next_prev_never_select_header`,
  `model_picker_enter_selects_correct_original_item_after_filtering`,
  `connect_picker_search_matches_provider_group`,
  `reset_selection_for_query_lands_on_first_navigable_match` (`talos-tui/src/state.rs`).

## Variance And Residuals

- `BUILD_MODELS=1` refresh path implemented but not validated with live network
  fetch (requires models.dev reachability). Code compiles and the non-network
  path is verified. A future closeout pass should run `BUILD_MODELS=1 cargo build`
  and verify `git diff -- crates/talos-config/src/models.toml`.
- Stage 2 is unblocked and its current accepted runtime path is closed: `/connect` reads
  packaged `models.toml` provider data, supports an optional custom `base_url` without losing
  unrelated provider fields, and both `/model` and `/connect` support group-aware "type to filter"
  search. The earlier `catalog.db` runtime path is superseded.
- MC107 terminal residual closed 2026-07-12 with a disposable-HOME `screen` walkthrough of
  `/connect`, safe credential cancellation, and `/model`.
- MC107 README residual closed: README now documents `/connect` in its own onboarding paragraph.
- Runtime `catalog.db` creation is no longer the accepted behavior. The active contract is
  no runtime model-metadata DB, no startup network fetch, and no explicit user initialization step:
  packaged `models.toml` is the runtime source, while richer models.dev metadata is incorporated by
  rebuilding with `BUILD_MODELS=1`.

## Retrospective

- Unit coverage was not sufficient for the terminal picker acceptance; a terminal emulator was
  required because a bare PTY cannot answer cursor-position queries.
- The final walkthrough preserved the user's real config and exercised the cancel path without a
  credential.
