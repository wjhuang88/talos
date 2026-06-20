# I038: Model Catalog Foundation

> Document status: Active
> Published plan date: 2026-06-20
> Planned objective: Talos ships a built-in model dataset and can import model metadata
>   from models.dev. The agent knows each model's context window, output limit, pricing,
>   and capabilities without hardcoded fallbacks.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `ModelMetadata` struct with serde + schemars; built-in TOML dataset (~20
>   models); `talos config import models` command for models.dev JSON.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| MODEL-001 | — | Planned (split from reasoning) | AGENT-001 ✅, TUI-009 ✅ | Built-in model catalog with import |

### Scope

- Define `ModelMetadata` struct in `talos-config` (serde + schemars):
  `id`, `provider`, `context_limit`, `output_limit`, `pricing` (input/output/cache
  per 1M tokens), `capabilities` (tools, structured_output, reasoning, image_input),
  `release_date`, `source` (builtin/manual/models.dev)
- Ship a built-in TOML dataset covering ~20 mainstream models:
  Claude Sonnet/Opus/Haiku, GPT-4o/mini, DeepSeek V3/R1, Gemini Pro/Flash
- `talos config import models` command: fetch models.dev JSON, cache to
  `~/.talos/cache/models/`, merge into provider config
- Precedence: user config > cache > built-in > conservative fallback
- Built-in data includes source dates so stale defaults are visible
- No startup-time network access; import is explicit user action

### Non-Goals

- Do not implement reasoning/thinking request fields (MODEL-003).
- Do not add runtime network dependency on models.dev.
- Do not auto-refresh model cache.
- Do not implement `/model` slash command (future story).
- Do not change provider request construction.

### Acceptance

- Given a fresh Talos install
  When the user runs `talos config import models`
  Then models.dev data is cached and merged into provider config

- Given a model is in the built-in dataset
  When the user configures that model
  Then context_limit, output_limit, and pricing are populated from built-in data

- Given user explicitly configures context_limit for a model
  When the model is selected
  Then user config takes precedence over built-in and cached data

- Given stale built-in data
  When the user inspects model metadata
  Then the source date is visible so staleness is apparent

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo test -p talos-config` (new model metadata tests + existing 43 tests)
- Manual: `talos config import models`, verify cache created, verify precedence

### Documentation To Update

- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md` — AC status
- `docs/BOARD.md` — move MODEL-001 from Next to Now
- `README.md` — document `talos config import models` command
- `docs/iterations/I038-model-catalog-foundation.md` — this file

### Risks And Rollback

- Risk: models.dev JSON schema changes, breaking import.
  Rollback: version the import format; cache import failures gracefully.
- Risk: built-in dataset becomes stale quickly.
  Rollback: source dates are visible; users can always override via config.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-20 | Activation | Non-terminal inventory clean; MODEL-001 dependencies met; activated as I038 |

## Verification Evidence

(to be filled as stories land)

## Variance And Residuals

(to be filled)

## Retrospective

(to be filled)
