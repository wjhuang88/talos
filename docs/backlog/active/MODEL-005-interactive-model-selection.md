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

### `/model` And `/connect` Commands

Registered as CMD-001 BuiltinCommands with separate responsibilities:

```
/model    →  Open picker for directly usable models. Select a model → switch.
/connect  →  Open provider setup. Select provider → credential + optional endpoint → register.
```

### Interactive Model Picker

Renders as an extension of the existing TUI-010 slash menu popup layer.
Single-line per model, metadata shown in the tip area on selection change.

```
  /claude-sonnet-4-20250514        Anthropic · 200K · $3/$15
  /gpt-4o                          OpenAI · 128K · $2.50/$10
  /deepseek-v3                     DeepSeek · 128K · $0.27/$1.10
  /claude-haiku-4-20250514         Anthropic · 200K · $0.80/$4

  ↑↓ select   Enter switch   / filter   Esc cancel
```

When selection changes, the tip area shows the selected model's metadata:
```
Tip: claude-haiku-4 · Anthropic · 200K ctx · $0.80/$4 per 1M · reasoning ✗
```

### Behavior

| Action | Result |
|---|---|
| `/model` | Opens picker showing models whose provider has a credential present |
| `/connect` | Opens provider setup using the full catalog, including providers not yet configured |
| `↑` `↓` | Navigate; tip area updates with model metadata |
| `Enter` in `/model` | Switch to selected model. |
| `Enter` in `/connect` | Register or update provider config, including credential and optional `base_url`. |
| `/` | Filter by name or provider |
| `Esc` | Cancel, keep current model |

## Non-Goals

- Do not auto-switch models mid-turn (only between turns).
- Do not implement a full provider marketplace.
- Do not change MODEL-003 reasoning scope.
- Do not verify provider credentials or custom endpoints with network calls in the picker flow.

## Acceptance Criteria

- [ ] `/model` opens the model picker in the existing TUI-010 slash menu layer.
- [ ] `/connect` opens provider setup from the full catalog.
- [ ] Single-line per model: id + provider + context + pricing.
- [ ] Tip area shows selected model's full metadata on navigation.
- [ ] ↑↓ navigate, Enter switches immediately.
- [ ] `/model` only shows models for providers with a credential present.
- [ ] `/connect` can register or update a provider credential.
- [ ] `/connect` can set, preserve, clear, or update a provider `base_url` for custom endpoints.
- [ ] `/` filters by name or provider.
- [ ] Model switch takes effect on the next turn.
- [ ] `cargo test -p talos-tui -p talos-config` passes.

## Required Reads

- `crates/talos-config/src/model.rs` — ModelMetadata, builtin_models()
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `crates/talos-tui/src/app.rs` — TUI-010 popup layer
