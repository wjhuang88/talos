# MODEL-004: Model Catalog Runtime Integration

**Status**: Planned (selected into crate distribution hardening two-month plan)
**Priority**: P2
**Source**: I038 residual (2026-06-20)
**Depends on**: MODEL-001 (I038 — catalog data layer complete); MEM-005 (compaction policy)

## Problem

MODEL-001 / I038 built the model catalog data layer (`ModelMetadata`, `models.toml`,
models.dev import), but it is not yet wired into the runtime. The system still
uses hardcoded `128_000` as the context limit fallback, and no runtime code
consumes model pricing or capability metadata.

Without integration, the catalog is dead data — it exists but nothing uses it.

## Planning Link

Selected into
`docs/tasks/2026-06-29-crate-distribution-hardening-two-month-plan.md` as the M1-M3 feature track:
runtime design checkpoint, catalog-backed limits, and compaction/UI metadata integration.

## Scope

Wire the model catalog into three runtime subsystems:

### 1. Config Layer: Model Limit Resolution

Add `Config::resolve_model_limits()` that looks up the active model in the catalog:

```
Precedence:
1. User-configured ProviderConfig.models.{id}.context_limit → use
2. Built-in catalog → look up by model id
3. models.dev cache → look up by model id
4. Conservative fallback (128_000)
```

This replaces the current hardcoded `128_000` in `SessionConfig::default()`.

**Files**: `crates/talos-config/src/lib.rs`, `crates/talos-agent/src/session.rs`

### 2. Compaction: Catalog-Aware Limit

`Compactor::new()` currently takes `model_limit` as a hardcoded parameter.
Change `SessionConfig` to resolve the limit from the catalog, so compaction
uses the model's actual context window instead of the fallback.

**Files**: `crates/talos-agent/src/session.rs`, `crates/talos-agent/src/compaction.rs`

### 3. UI: Model Metadata Display

Status bar and exit summary (TUI-009, TUI-011) should display model metadata
from the catalog where available:

- Model name + provider (already shown)
- Context limit (from catalog, not hardcoded)
- Pricing estimate (from `ModelPricing`, shown in exit summary cost)

**Files**: `crates/talos-tui/src/app.rs` (status/exit summary)

## Non-Goals

- Do not add catalog auto-refresh at startup.
- Do not add `/model` slash command (future story).
- Do not change MODEL-003 reasoning scope.

## Acceptance Criteria

- [ ] `Config::resolve_model_limits()` returns context/output limit from
      catalog when user config does not specify them.
- [ ] `SessionConfig::default()` uses catalog lookup, not hardcoded 128_000.
- [ ] Compactor receives the correct model limit from the catalog.
- [ ] Status bar and exit summary display metadata sourced from catalog.
- [ ] Fallback to 128_000 works when model is not in catalog.
- [ ] `cargo test --workspace` passes.

## Required Reads

- `crates/talos-config/src/model.rs` — ModelMetadata, builtin_models()
- `crates/talos-config/src/lib.rs` — Config, ProviderConfig, ModelConfig
- `crates/talos-agent/src/session.rs` — SessionConfig, model_context_limit
- `crates/talos-agent/src/compaction.rs` — Compactor::new
- `crates/talos-tui/src/app.rs` — status bar, exit summary
