# MODEL-004: Model Catalog Runtime Integration

**Status**: Partial (M1/M2 complete via I045; M3 residual selected into crate distribution hardening two-month plan)
**Priority**: P2
**Source**: I038 residual (2026-06-20)
**Depends on**: MODEL-001 (I038 — catalog data layer complete); MEM-005 (compaction policy)

## Problem

MODEL-001 / I038 built the model catalog data layer (`ModelMetadata`, `models.toml`,
models.dev import). I045 then wired the catalog into runtime limit resolution and compaction setup.
The remaining gap is user-visible metadata: the TUI status bar and exit summary still use
hardcoded cost/rate logic instead of catalog context/pricing metadata.

Without the remaining UI integration, part of the catalog is still dead data from the user's
perspective.

## Planning Link

Selected into
`docs/tasks/2026-06-29-crate-distribution-hardening-two-month-plan.md` as a reconciled M1-M3
feature track. M1/M2 are baseline evidence tasks because I045 already completed them; M3 remains
implementation work.

## Scope

Wire the model catalog into three runtime subsystems:

### 1. Config Layer: Model Limit Resolution

`Config::resolve_model_limits()` already looks up the active model in the catalog:

```
Precedence:
1. User-configured ProviderConfig.models.{id}.context_limit → use
2. Built-in catalog → look up by model id
3. models.dev cache → look up by model id
4. Conservative fallback (128_000)
```

The runtime call sites now pass resolved limits into `SessionConfig`; the conservative fallback
remains `128_000` when catalog data is absent.

**Files**: `crates/talos-config/src/lib.rs`, `crates/talos-agent/src/session.rs`

### 2. Compaction: Catalog-Aware Limit

`Compactor::new()` receives `SessionConfig.model_context_limit`, and CLI/session setup paths pass
the resolved catalog-aware value.

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

- [x] `Config::resolve_model_limits()` returns context/output limit from
      catalog when user config does not specify them.
- [x] Runtime session construction uses resolved catalog-aware limits instead of hardcoded
      `128_000` in active CLI paths.
- [x] Compactor receives the correct model limit from the catalog.
- [ ] Status bar and exit summary display metadata sourced from catalog.
- [x] Fallback to 128_000 works when model is not in catalog.
- [ ] `cargo test --workspace` passes.

## Execution Baseline

- I045 closed on 2026-06-24 and records MODEL-004-R as complete for catalog limit wiring.
- Current code evidence:
  - `crates/talos-config/src/config.rs` defines `Config::resolve_model_limits()`.
  - `crates/talos-config/src/tests.rs` covers catalog fallback, user precedence, and fallback.
  - `crates/talos-cli/src/mode_runners.rs`, `mode_inline.rs`, `mode_print.rs`, and
    `model_lifecycle.rs` pass resolved limits into `SessionConfig`.
  - `crates/talos-agent/src/session.rs` passes `SessionConfig.model_context_limit` to
    `Compactor::new()`.
- Residual implementation:
  - `crates/talos-tui/src/scrollback_status.rs` and `crates/talos-tui/src/app_summary.rs` still
    contain hardcoded cost logic and do not display catalog context/pricing metadata.

## Required Reads

- `crates/talos-config/src/model.rs` — ModelMetadata, builtin_models()
- `crates/talos-config/src/lib.rs` — Config, ProviderConfig, ModelConfig
- `crates/talos-agent/src/session.rs` — SessionConfig, model_context_limit
- `crates/talos-agent/src/compaction.rs` — Compactor::new
- `crates/talos-tui/src/app.rs` — status bar, exit summary
