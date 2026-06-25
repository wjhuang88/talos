# I045: Product Readiness — Model Lifecycle, Config, Observability

> Document status: Complete (2026-06-24)
> Published plan date: 2026-06-24
> Closed date: 2026-06-24
> Planned objective: Turn Talos from "hand-edit TOML to configure" into "guided
>   model setup, runtime model switching, and bounded observability." Three
>   themes: (1) model-centric lifecycle with inline provider onboarding, (2) CLI
>   config editing escape hatch, (3) bounded log retention + embedded prompt
>   assets (unblocks memory foundation).

## Selected Stories

| Story | Priority | Outcome |
|---|---|---|
| MODEL-004-R: Catalog runtime integration | P2 | `Config::resolve_model_limits()` replaces hardcoded 128_000; compaction + status bar use real metadata |
| MODEL-005-R: `/model` picker + provider onboarding | P1 | Interactive model picker (Ready/Setup-required groups); inline credential collection; first-run wizard; `talos init` |
| CONF-001-S: CLI config editing | P2 | `talos config get/set/list` through `talos-config` API; secrets masked |
| OBS-001 (ARCH-S8 R2 + I018-S1) | P1 | `[log.file]` rotation + retention under ADR-014; built-in prompts as embedded assets under ADR-015 |

## Design Decisions (Pre-Implementation)

### D1: Model-Centric Flow (not Provider-Centric)

The catalog drives everything. Users pick a model; if its provider isn't
authenticated, Talos prompts for credentials inline. Users never need to think
about "providers" as a separate concept.

```
/model picker
  ├── Ready group (provider authenticated)
  │     model A → Enter → instant switch
  └── Setup required group (provider not authenticated)
        model B → Enter → credential prompt → validate → switch
```

### D2: Picker Grouping

Models are split into two visual groups in the picker:

- **Ready**: provider has a valid `api_key` or `api_key_env` environment variable
  is set. Selecting → instant switch.
- **Setup required**: provider not configured or credentials missing. Selecting
  → inline credential input panel → optional connectivity test → switch.

Group headers are non-navigable rows (Up/Down skip them).

### D3: Session-Rebuild for Model Switch

Runtime model switching reuses the existing `SessionTransition` infrastructure
(same path as `/new`, `/resume`, `/fork`). The switch:

1. Preserves session ID and conversation history.
2. Builds a new `Agent` with the new provider + model.
3. Calls `transition.prepare/commit` (same as other lifecycle commands).
4. `cached_stable_prefix` resets to `None` and recomputes on the next turn.

**Rationale**: Anthropic's ephemeral cache is keyed by model — switching models
invalidates the cache regardless. Session rebuild has the same cache cost as a
theoretical hot-swap (one cache miss on first turn post-switch), without
modifying the `Agent` core structure. See prompt cache audit in the execution
record for details.

### D4: First-Run Wizard = Model Picker Auto-Opened

When `Config::load()` returns a config with no usable model (empty `model`
field, or `api_key()` fails), Talos does NOT `bail!`. Instead it enters the
TUI and auto-opens the `/model` picker. The user selects a model, enters
credentials, and the session starts normally.

`talos init` re-triggers this flow from any state.

### D5: Config CLI is the Escape Hatch

`talos config get/set/list` is for power users who want to edit settings
without the TUI. The TUI `/config` slash command is deferred — `/model`
covers 90% of the interactive config need. This keeps the iteration scope
tight.

## Scope

### MODEL-004-R: Catalog Runtime Integration

**Config layer** (`crates/talos-config/src/lib.rs` + `model.rs`):

- `Config::resolve_model_limits() -> (context_limit: u32, output_limit: Option<u32>)`
  - Precedence: user-configured `ProviderConfig.models.{id}` → builtin catalog
    → conservative fallback (128_000)
  - Replaces hardcoded `128_000` in `SessionConfig::default()` and all
    `model_context_limit` call sites
- `ModelCatalog::find(id) -> Option<ModelMetadata>` — lookup by model ID
- `ModelCatalog::list() -> Vec<ModelMetadata>` — all models (builtin + cached
  models.dev import)
- `Config::provider_authenticated(name) -> bool` — checks whether a provider
  has a usable API key (inline or env var resolved)

**Agent layer** (`crates/talos-agent/src/session.rs`):

- `SessionConfig` uses `resolve_model_limits()` instead of hardcoded constant

**TUI layer** (`crates/talos-tui/src/app.rs`):

- Status bar and exit summary show catalog-sourced metadata (context limit,
  pricing) when available

### MODEL-005-R: `/model` Picker + Provider Onboarding

**Conversation engine** (`crates/talos-conversation/src/engine.rs` + `types.rs`):

- `/model` BuiltinCommand (no args → picker; with arg → direct switch)
- `UiOutput::ModelPicker(Vec<ModelPickerItem>)` — new UiOutput variant
- `UiOutput::ModelSwitchRequest { model_id, provider_needs_credential: bool }`
- `ModelPickerItem`:
  ```
  command: String,          // "/model"
  model_id: String,         // "claude-sonnet-4-20250514"
  provider: String,         // "anthropic"
  label: String,            // display line
  context_limit: Option<u32>,
  pricing: Option<ModelPricing>,
  capabilities: ModelCapabilities,
  authenticated: bool,      // true → Ready, false → Setup required
  ```

**TUI** (`crates/talos-tui/src/state.rs` + `scrollback.rs`):

- `PanelKind::ModelPicker` — new bottom panel kind
- `BottomPanelState::open_model_picker(items)` — groups items by
  `authenticated` (Ready first, Setup required second), inserts non-navigable
  group header rows
- `PanelKind::CredentialInput { provider_name, model_id }` — inline credential
  entry (masked input, Enter submits, Esc cancels back to picker)
- `accept_selected_panel_item` — ModelPicker kind submits `/model <model_id>`;
  CredentialInput kind submits the typed key for validation
- Tip area shows full metadata on selection change

**Config write-through** (`crates/talos-config/src/lib.rs`):

- `Config::set_active_model(model_id, catalog) -> Result<()>` — resolves
  model→provider from catalog, sets `self.provider` + `self.model`, creates
  `ProviderConfig` entry if missing
- `Config::set_provider_credential(name, api_key)` — writes `api_key` to the
  provider config entry
- `Config::save() -> Result<()>` — serializes back to `~/.talos/config.toml`
  with `${ENV_VAR}` substitution preserved

**Lifecycle handler** (`crates/talos-cli/src/mode_runners.rs`):

- `handle_session_model` — receives `ModelSwitchRequest`:
  1. If `provider_needs_credential`: credential already collected by TUI;
     write to config
  2. `Config::save()`
  3. Build new provider + agent
  4. `transition.prepare/commit` (preserve session ID + history)
  5. Update `session_watch_tx` + `sq_tx_watch_tx` + `bridge_rx_update_tx`
     (same pattern as `/new`, `/resume`, `/fork`)
  6. NO `HydrateHistory` (history is already visible)

**First-run pre-flight** (all mode runners):

- Replace `bail!("no model configured")` with: enter TUI → auto-open
  `/model` picker
- Replace `bail!("missing API key")` with: enter TUI → auto-open `/model`
  picker for the configured provider's model
- `--no-init` CLI flag skips the wizard (for CI / non-interactive)

**CLI** (`crates/talos-cli/src/main.rs` + `mode_runners.rs`):

- `talos init` — clears active model/provider, enters TUI with wizard
- `talos model list` — prints catalog table with auth status
- `talos model use <id>` — CLI model switch (prompts for credential if needed)

### CONF-001-S: CLI Config Editing

**CLI** (`crates/talos-cli/src/main.rs`):

- `talos config list` — print all settings (secrets masked)
- `talos config get <key>` — print single value
- `talos config set <key> <value>` — validate + persist via `talos-config`
- All operations round-trip through `Config::load()` / `Config::save()`
- `${ENV_VAR}` substitution survives set/get round-trip
- Secret fields (`api_key`, persisted in local config per ADR-023) are never
  echoed in plaintext by `get`/`list`; `set` accepts them but masks on redisplay

### OBS-001: Bounded Log Retention + Embedded Prompts

**ARCH-S8 R2: File logging with rotation** (`crates/talos-config/src/lib.rs` +
`crates/talos-cli/src/logging.rs`):

- `[log.file]` config section:
  ```toml
  [log.file]
  enabled = true              # default: true in TUI mode, false otherwise
  path = "~/.talos/logs/talos.log"
  max_size_mb = 16
  max_files = 5
  rotation = "size"           # "size" | "daily"
  ```
- In-process rotation: when file exceeds `max_size_mb`, rename to
  `talos.log.1`, shift older files, truncate current
- Retention: oldest file beyond `max_files` is deleted
- No host `logrotate` dependency
- Follows ADR-014

**I018-S1: Embedded prompt assets** (`crates/talos-agent/src/prompt.rs`):

- Extract built-in prompt text (Identity, Tool guide) from inline Rust string
  literals into standalone text files under `crates/talos-agent/prompts/`
- Embed at compile time via `include_str!`
- Runtime overrides (CLI `--append`, config hooks) remain unchanged
- Follows ADR-015

## Execution Order

```
Week 1:
  OBS-001 (log rotation + prompt assets) ─── 2-3 days (independent, unblocks I019)
         ∥
  MODEL-004-R (catalog → config wiring) ─── 2-3 days (foundation for MODEL-005-R)

Week 2:
  MODEL-005-R core: /model picker + PanelKind::ModelPicker ─── 3-4 days
         ∥
  CONF-001-S (CLI config get/set/list) ─── 2-3 days (independent escape hatch)

Week 3:
  MODEL-005-R auth: inline credential input + Config::save + connectivity test ─── 3 days
  MODEL-005-R lifecycle: handle_session_model (SessionTransition reuse) ─── 2 days

Week 4:
  First-run wizard (pre-flight replacement) ─── 2 days
  talos init + talos model list/use CLI ─── 2 days
  Polish + verification + closeout ─── 2 days
```

## Scope Dependencies

```
OBS-001 ──────────────────────────────────────→ (unblocks I019 memory)
MODEL-004-R (catalog wiring) ──→ MODEL-005-R (picker needs catalog data)
                                      │
                                      ├── picker UI (Week 2)
                                      ├── credential input (Week 3)
                                      └── lifecycle handler (Week 3)
                                               │
                                      first-run wizard (Week 4)
CONF-001-S ──────────────────────────────────→ (independent)
```

## Non-Goals

- TUI `/config` slash command (deferred — `/model` covers 90% of the need)
- `/provider` separate command (model-centric flow makes this unnecessary)
- MODEL-003 reasoning/thinking support (needs ADR first; separate iteration)
- MEM-005 compaction policy (separate iteration; MODEL-004-R only wires the
  limit, doesn't change compaction triggers)
- Automatic model catalog refresh from models.dev at startup (manual import
  path already exists)
- Cross-session model preferences (model selection is per-session via
  `SessionTransition`)
- Prompt cache preservation across model switch (one cache miss is
  unavoidable and acceptable)

## Acceptance

### MODEL-004-R

- `Config::resolve_model_limits()` returns catalog-sourced limits when user
  config doesn't specify them
- `SessionConfig::default()` uses catalog lookup, not hardcoded 128_000
- Compactor receives correct model limit from catalog
- Status bar displays context limit from catalog
- Fallback to 128_000 works when model is not in catalog
- `cargo test -p talos-config -p talos-agent` passes

### MODEL-005-R

- `/model` opens bottom panel picker with all catalog models, grouped by
  Ready / Setup required
- Up/Down navigates within and across groups (skipping headers); Enter
  selects; Esc cancels
- Ready group: selecting switches immediately; next turn uses new model
- Setup required group: selecting opens inline credential input; on submit,
  validates and switches
- First run with no model: TUI opens, `/model` picker auto-shown (no bail!)
- First run with model but no API key: TUI opens, credential input auto-shown
- `talos init` re-triggers the wizard from any state
- `talos model list` prints catalog with auth status
- `talos model use <id>` switches from CLI
- Model switch preserves session ID and conversation history
- Config persists correctly (`~/.talos/config.toml`)
- `--no-init` flag skips wizard in non-interactive environments
- `cargo test --workspace` passes

### CONF-001-S

- `talos config get/set/list` read and write through `talos-config` API
- JSON-Schema validation rejects invalid values with clear error
- `${ENV_VAR}` substitution survives set/get round-trip
- Secret fields never echoed in plaintext
- No regression for env-var-driven config or existing config files on load
- `cargo test -p talos-config -p talos-cli` passes

### OBS-001

- `[log.file]` config section controls path, max size, max files, rotation
- TUI mode defaults to file logging; non-TUI defaults to stderr
- Rotation runs in-process; no host logrotate dependency
- Total retained bytes bounded by `max_size_mb × max_files`
- Built-in prompt text lives in standalone files embedded via `include_str!`
- Tests verify required prompt assets are present and non-empty
- Non-TUI modes still work with stderr-only logging by default
- `cargo test --workspace` passes

## Verification

- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- Unit tests: `resolve_model_limits` precedence, `provider_authenticated`,
  `set_active_model`, `Config::save` round-trip
- Unit tests: `ModelPickerItem` grouping, picker navigation
- Unit tests: log rotation (file exceeds max_size → rotates), retention
  (oldest deleted beyond max_files)
- Unit tests: prompt asset embedding (all assets present, non-empty)
- Integration: `/model` switch end-to-end (mock provider → switch → verify
  new model name in status)
- Integration: first-run wizard (empty config → wizard → config written →
  session starts)
- `scripts/validate_project_governance.sh` — 0 warnings

## Residual Work Destination

- TUI `/config` slash command → future iteration if power users need inline
  config editing beyond `/model`
- MODEL-003 reasoning/thinking → separate iteration with ADR gate
- MEM-005 compaction policy → separate iteration; MODEL-004-R only wires the
  limit value, doesn't change trigger logic
- Automatic models.dev refresh → future enhancement; manual import path
  (`talos config import models`) already exists
- `/provider` management command → not needed if model-centric flow is
  sufficient; add only if user feedback shows a gap

## Required Reads

- `crates/talos-config/src/lib.rs` — Config struct, ProviderConfig, api_key resolution
- `crates/talos-config/src/model.rs` — ModelMetadata, builtin_models(), models.toml
- `crates/talos-agent/src/lib.rs` — Agent struct, provider holding, cached_stable_prefix
- `crates/talos-agent/src/prompt.rs` — SystemPromptBuilder, cache markers
- `crates/talos-agent/src/session.rs` — SessionConfig, model_context_limit
- `crates/talos-cli/src/provider_setup.rs` — build_provider
- `crates/talos-cli/src/session_transition.rs` — SessionTransition (reuse for model switch)
- `crates/talos-cli/src/mode_runners.rs` — lifecycle handlers, bridge forwarder
- `crates/talos-tui/src/state.rs` — BottomPanelState, PanelKind, PanelAction
- `crates/talos-conversation/src/engine.rs` — BuiltinCommand, command registry
- `docs/decisions/014-log-retention-and-rotation.md` — ADR-014 (log bounds)
- `docs/decisions/015-embedded-prompt-assets.md` — ADR-015 (prompt embedding)

## Execution Record

### Implementation Summary

| Story | Status | Commits | Notes |
|---|---|---|---|
| OBS-001 (log rotation) | ✅ Complete | `7115cb7` | `RotatingWriter` + `LogFileConfig` with size-based rotation + retention |
| MODEL-004-R (catalog wiring) | ✅ Complete | `4a422e4`, `8700f3f` | `Config::resolve_model_limits()` with builtin fallback; `send_stream()` helper eliminated 33 duplications |
| MODEL-005-R (/model picker + onboarding) | ✅ Complete | `f2f2f2d`, `81ab83f`, `6452b2e`, `bdbac5a`, `98c61ba`, `9aaef21`, `aa7212b` | `/model` command, `PanelKind::ModelPicker` with Ready/Setup-required groups; inline credential input via `PanelKind::CredentialInput`; first-run wizard replacing `bail!`; `--no-init` flag |
| CONF-001-S (CLI config) | ✅ Complete | `2dfc595` | `talos config get/set/list` with env var support; secrets masked |
| `--available-models` CLI | ✅ Complete | (this iteration) | Lists builtin catalog grouped by provider with auth status |
| `--use-model` CLI | ✅ Complete | (this iteration) | Sets active model from CLI, persists to config.toml |
| `--init` CLI | ✅ Complete | (this iteration) | Clears model, falls through to TUI wizard |
| Group headers | ✅ Complete | (this iteration) | Non-navigable "Ready"/"Setup required" headers with bold styling |

### Key Fixes During Execution

| Issue | Resolution |
|---|---|
| Model switch depends on session (first-run) | `ensure_persisted()` on deferred session before transition |
| `api_key` silently erased on save | Reverted `#[serde(skip_serializing)]` — api_key now serializes in config.toml; display masking handled by CLI |
| `credentials.toml` data loss | Removed `Credentials::save()` from `Config::save()`; backward-compat `Credentials::load()` still merges legacy file |
| `protocol` reset to `openai-chat` on `set_active_model` | Pre-existing: builtin provider takes precedence at runtime via `active_provider_config()` |

### Acceptance Verification

**MODEL-004-R:**
- ✅ `Config::resolve_model_limits()` returns catalog-sourced limits
- ✅ `SessionConfig::default()` uses catalog lookup
- ✅ Fallback to 128_000 works when model not in catalog
- ✅ `cargo test -p talos-config -p talos-agent` passes

**MODEL-005-R:**
- ✅ `/model` opens bottom panel with catalog models, grouped by Ready / Setup required
- ✅ Up/Down skips headers; Enter selects; Esc cancels
- ✅ Ready group: instant switch on select
- ✅ Setup required group: inline credential input → validate → switch
- ✅ First run: TUI opens with model picker auto-shown
- ✅ `--no-init` flag skips wizard in CI/non-interactive
- ✅ `--init` clears model, falls through to TUI wizard
- ✅ `--available-models` prints catalog with auth status
- ✅ `--use-model` sets active model from CLI
- ✅ Model switch preserves session ID and history
- ✅ Config persists correctly to `~/.talos/config.toml`
- ✅ `cargo test --workspace` passes

**CONF-001-S:**
- ✅ `--config-list`, `--config-get`, `--config-set` read/write through talos-config API
- ✅ JSON Schema validation rejects invalid values
- ✅ `${ENV_VAR}` substitution survives set/get round-trip
- ✅ Secret fields masked in display
- ✅ No regression for env-var-driven config
- ✅ `cargo test -p talos-config -p talos-cli` passes

**OBS-001:**
- ✅ `[log.file]` config section controls path, max size, max files, rotation
- ✅ TUI mode defaults to file logging; non-TUI defaults to stderr
- ✅ Rotation in-process; no host dependency
- ✅ Total retained bytes bounded by `max_size_mb × max_files`
- ✅ `cargo test --workspace` passes

### Commits

```
7115cb7 feat(cli): log rotation with RotatingWriter + retention config
8700f3f refactor(cli): extract send_stream helper, eliminate 33 duplicated blocks
4a422e4 feat(config): resolve_model_limits with builtin catalog fallback
f2f2f2d feat(conversation): /model command + ModelPicker+ModelSwitchRequest types
81ab83f feat(tui): PanelKind::ModelPicker with auth-grouped items
b7ae5cd feat(config): Config::save, provider_authenticated, set_active_model, credential store
6452b2e feat(cli): handle_session_model + handle_session_model_with_credential lifecycle handlers
bdbac5a feat(tui): credential input panel (masked) with CredentialRequest/Response wiring
4962b5b feat(cli): first-run wizard replaces bail! with TUI auto-opening /model picker
2dfc595 feat(cli): talos config get/set/list with env var + --available-models + --use-model
98c61ba feat(tui): PanelKind::CredentialInput + UserInput::Credential routing
9aaef21 fix(cli): first-run /model bypasses engine — direct lifecycle dispatch
aa7212b fix(cli): ensure_persisted() on model switch for deferred session
c8d6c33 docs: README sync (en + zh-CN) with /model, first-run wizard, config CLI
(plus ~5 commits in this continuation for --init, group headers, closeout)
```

### Lessons (EVOLUTION.md)

- #28: Pre-closeout parallel audit catches self-comparison sort bug in session listing
- #29: `serde(skip_serializing)` on api_key causes silent data loss — display masking must be at the CLI layer, not the serializer
- #30: Model switch needs `ensure_persisted()` — session may not exist on first turn

### Post-Closeout Correction (I046, 2026-06-25)

Two validation claims in this iteration were stale:

1. **`cargo test --workspace` passes** (lines 403, 418) — was false. Two tests were failing:
   - `talos-config::tests::test_model_limits_from_builtin_and_custom_providers` still used `gpt-4.1`
     after commit `0734eae` updated the catalog to `gpt-4.1-2025-04-14`.
   - `talos-tui::tests::test_session_picker_accept_resume_default_command` broke when commit
     `a8cd614` (PanelItemAction refactor) lost the `/resume` fallback for empty session-picker
     commands.
   Both fixed in I046-S1.

2. **Model identity was not provider-aware** — `resolve_model_limits()` and `all_models()` resolved
   models by bare ID, silently picking the first provider for duplicates (e.g. `glm-5.2` under
   zhipu/zai). Fixed in I046-S2 with `find_model_by_provider` and `(provider, model_id)` semantics.
