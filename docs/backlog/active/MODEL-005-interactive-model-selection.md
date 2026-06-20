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

Registered as a CMD-001 BuiltinCommand. Single entry point — no subcommands.
Opens the interactive model picker immediately.

```
/model   →  Open picker. Select a model → switch. That's it.
```

### Interactive Model Picker

Reuses TUI-010's popup layer. Opens when `/model` is typed:

```
┌── Models ─────────────────────────────────────┐
│                                                │
│  ● claude-sonnet-4-20250514   (active)         │
│    Anthropic · 200K ctx · $3/$15               │
│                                                │
│  ○ gpt-4o                                      │
│    OpenAI · 128K ctx · $2.50/$10               │
│                                                │
│  ○ deepseek-v3                                 │
│    DeepSeek · 128K ctx · $0.27/$1.10           │
│                                                │
│  ── More from catalog ──                       │
│  ○ claude-haiku-4            Anthropic         │
│    200K · $0.80/$4                             │
│                                                │
│  ↑↓ select   Enter switch   i info   r refresh │
│  / filter    Esc cancel                        │
└────────────────────────────────────────────────┘
```

### Key Bindings

| Key | Action |
|---|---|
| `↑` `↓` | Navigate |
| `Enter` | Switch to selected model (add to config if from catalog) |
| `i` | Toggle info panel: show full metadata for selected model |
| `r` | Refresh catalog from models.dev (if previously imported) |
| `/` | Filter list by name/provider |
| `Esc` | Cancel, keep current model |

### Selecting a Catalog Model

When user presses Enter on an unconfigured catalog model:
- "Added claude-haiku-4 to config. Switching..."
- Registers + switches in one step. No confirmation prompt.
- Persists to `~/.talos/config.toml` for future sessions.

### Info Panel (i key)

Toggles below the list:
```
┌── Model Info ──────────────────────────────────┐
│ claude-haiku-4-20250514                        │
│ Provider: Anthropic                            │
│ Context: 200,000 tokens                        │
│ Output:  8,192 tokens                          │
│ Pricing: $0.80 / $4.00 per 1M (in/out)        │
│ Released: 2025-05                              │
│ Capabilities: tools ✓  reasoning ✗  images ✓   │
│ Source: built-in catalog                       │
└────────────────────────────────────────────────┘
```

## Non-Goals

- Do not auto-switch models mid-turn (only between turns).
- Do not implement a full provider marketplace.
- Do not change MODEL-003 reasoning scope.

## Acceptance Criteria

- [ ] `/model` opens interactive picker — no subcommands needed.
- [ ] Picker shows configured models first, catalog models below.
- [ ] ↑↓ navigate, Enter selects and switches immediately.
- [ ] Selecting an unconfigured catalog model auto-registers + switches.
- [ ] `i` toggles full metadata info panel.
- [ ] `r` refreshes models.dev cache.
- [ ] `/` filters list by name or provider.
- [ ] Model switch takes effect on the next turn.
- [ ] `cargo test -p talos-tui -p talos-config` passes.

## Required Reads

- `crates/talos-config/src/model.rs` — ModelMetadata, builtin_models()
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `crates/talos-tui/src/app.rs` — TUI-010 popup layer
