# MODEL-005: Interactive Model Selection & Runtime Registration

**Status**: Planned
**Priority**: P2
**Source**: User request 2026-06-20
**Depends on**: MODEL-001 (catalog data), MODEL-004 (runtime integration), CMD-001 (BuiltinCommand), TUI-010 (popup layer)

## Problem

Users currently configure models by editing `~/.talos/config.toml` and restarting
Talos. There is no runtime path to:
1. Switch the active model during a session
2. Browse available models from the built-in catalog
3. Register a new model without editing config files

CMD-001's audit already flagged `/model` as a future BuiltinCommand — this
story implements it.

## Scope

### `/model` Command

Registered as a CMD-001 BuiltinCommand. Without arguments, opens the model
picker. With subcommands:

```
/model              → Open interactive model picker
/model switch       → Alias for picker
/model info <id>    → Show model metadata (context, pricing, capabilities)
/model add <provider> <id>  → Register a new model manually
/model import       → Refresh from models.dev (if previously imported)
/model current      → Show current active model + provider
```

### Interactive Model Picker

Reuses TUI-010's popup layer. Opens when `/model` is typed:

```
┌── Select Model ──────────────────────────────┐
│                                               │
│  ● claude-sonnet-4-20250514    (current)      │
│    Anthropic · 200K ctx · $3/$15 per 1M       │
│                                               │
│  ○ claude-opus-4-20250514                     │
│    Anthropic · 200K ctx · $15/$75 per 1M      │
│                                               │
│  ○ gpt-4o                                     │
│    OpenAI · 128K ctx · $2.50/$10 per 1M       │
│                                               │
│  ○ deepseek-v3                                │
│    DeepSeek · 128K ctx · $0.27/$1.10 per 1M   │
│                                               │
│  ── Catalog (not configured) ──               │
│  ○ claude-haiku-4-20250514                    │
│    Anthropic · 200K ctx · $0.80/$4 per 1M     │
│                                               │
│  ↑↓ navigate  Enter select  Esc cancel        │
│  / filter   Tab add to config                 │
└───────────────────────────────────────────────┘
```

### Model Sources

The picker shows three groups:

| Group | Source | Behavior |
|---|---|---|
| **Configured** | `[providers.{name}.models]` in config | Select → switch immediately |
| **Catalog** | Built-in dataset + models.dev cache | Select → prompt to add to config |
| **Manual** | `/model add` entries | Select → switch immediately |

### Selecting an Unconfigured Model

When the user selects a catalog model that isn't in their config:

1. Picker shows "This model is in the catalog but not configured."
2. Prompt: "Add claude-haiku-4 to config and switch? [y/N]"
3. If yes: writes the model to `~/.talos/config.toml` (or in-memory for the session)
4. If no: returns to picker

### `/model add` (Runtime Registration)

```
/model add anthropic claude-haiku-4-20250514
```

- Looks up metadata from built-in catalog
- If found: registers the model + provider config for the current session
- If not found: asks for manual context/output limits
- Option to persist to `~/.talos/config.toml`

### `/model import` (Catalog Refresh)

- Re-fetches from models.dev (if previously imported)
- Updates `~/.talos/cache/models/models.json`
- Shows count: "Imported N models, M new"

## Non-Goals

- Do not auto-switch models mid-turn (only between turns).
- Do not implement a full provider marketplace.
- Do not change MODEL-003 reasoning scope.

## Acceptance Criteria

- [ ] `/model` opens interactive picker with configured + catalog models.
- [ ] Picker shows model name, provider, context window, pricing per group.
- [ ] ↑↓ navigate, Enter select, Esc cancel, typing filters.
- [ ] Selecting a configured model switches the active model for the session.
- [ ] Selecting a catalog model prompts to add to config first.
- [ ] `/model add <provider> <id>` registers a model at runtime.
- [ ] `/model info <id>` shows full metadata from catalog.
- [ ] `/model import` refreshes models.dev cache.
- [ ] Model switch takes effect on the next turn (not mid-turn).
- [ ] `cargo test -p talos-tui -p talos-config` passes.

## Required Reads

- `crates/talos-config/src/model.rs` — ModelMetadata, builtin_models()
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `crates/talos-tui/src/app.rs` — TUI-010 popup layer
