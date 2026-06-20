# MODEL-001: Model Catalog And Reasoning Capability Foundation

| Field | Value |
|-------|-------|
| Story ID | MODEL-001 |
| Priority | P2 |
| Status | Planned |
| Depends On | AGENT-001; MEM-005; TUI-009 |
| Note | Reasoning/thinking support split to MODEL-003 (ADR gate). This story is catalog-only. |
| Origin | User feedback 2026-06-19 — models.dev should inform Talos model choices, defaults, reasoning/thinking support, pricing, and compaction timing |

## Problem

Talos currently treats model selection mostly as a provider/model string plus a
small amount of manually configured runtime limit data. This leaves three gaps:

1. Reasoning/thinking-capable models are not modeled end-to-end. Existing
   provider code may ignore provider-specific reasoning request fields and
   reasoning stream chunks.
2. Users do not have a built-in model dataset for onboarding, `/model`, default
   settings, or provider selection. This should be closer to OpenCode's model
   catalog approach instead of relying on users to hand-type every model.
3. Context-window metadata is required for correct compaction timing. A fixed
   fallback like `128_000` is not good enough once users switch between small,
   medium, and million-token-context models.

## Scope

Introduce a model catalog foundation for Talos.

Required capabilities:

- A built-in offline model dataset shipped with Talos for common providers and
  models.
- A local cache model for externally refreshed catalog data, with user config
  still taking precedence.
- A clear mapping from catalog metadata to Talos runtime decisions:
  - context window
  - output limit
  - input/output/cache pricing
  - tool-call support
  - structured-output support
  - reasoning/thinking support
  - modality support
  - release/update date
- A reasoning/thinking capability model that can represent provider-specific
  request shapes without forcing one global field into every provider.
- A source policy for `models.dev`: use it as a catalog input and reference
  source, not as a startup-time network dependency.

## Models.dev Input

`models.dev` provides useful catalog fields through JSON endpoints:

- `https://models.dev/models.json` — provider-agnostic model metadata.
- `https://models.dev/api.json` — provider serving metadata, including pricing
  and provider-specific limits.
- `https://models.dev/catalog.json` — combined model/provider catalog.

Talos should treat this data as an importable/updatable catalog source. The
first implementation should not require network access during normal startup.

## Built-In Dataset Policy

Talos should ship a curated built-in model dataset so users can choose from
known provider/model options without refreshing an external catalog first.

Rules:

- Built-in data is a default, not a source of truth above user config.
- User config overrides built-in model limits, pricing, and capability flags.
- Refreshed catalog cache overrides built-in data only when the user opts into
  refresh/update behavior.
- Built-in data must include source dates, so stale defaults are visible.
- Secrets and API keys never appear in the dataset.

## Reasoning / Thinking Scope

This story promotes the existing reasoning/thinking proposal into backlog scope.
Implementation must still respect its ADR gate when changing provider request
schemas, stream events, session persistence, TUI rendering, JSON-RPC payloads,
or evolution hook contracts.

The model catalog should represent at least:

- whether the model supports reasoning/thinking
- whether reasoning is visible, hidden, interleaved, or provider-specific
- request configuration shape where known, e.g. Anthropic `thinking`, OpenAI
  `reasoning_effort`, or OpenAI-compatible nested `options.thinking`
- budget/effort controls
- whether reasoning tokens are billed separately or surfaced in usage metadata

## Relationship To MEM-005

MEM-005 needs accurate model context and output limits to choose a safe
compaction trigger. MODEL-001 supplies the model metadata source for that
decision.

The intended precedence for compaction limits is:

1. Explicit user config for the active provider/model.
2. Refreshed local model catalog cache.
3. Built-in model dataset.
4. Conservative fallback.

The compaction policy must reserve output budget and reasoning budget where
applicable; it should not use the full context window as input budget.

## Non-Goals

- Do not add a runtime dependency on `models.dev` availability.
- Do not auto-update model data during startup.
- Do not implement a provider marketplace.
- Do not expose hidden chain-of-thought by default.
- Do not choose a new provider SDK or Node/AI SDK integration.

## Acceptance Criteria

- [ ] A Talos-owned model catalog schema is defined with serde + schemars.
- [ ] The schema can represent context/output limits, pricing, capabilities,
      modalities, release/update dates, and source provenance.
- [ ] Built-in model data exists as a reviewable embedded asset.
- [ ] User config precedence over catalog data is documented and tested.
- [ ] `models.dev` import/refresh behavior is designed as explicit user action,
      not startup-time network access.
- [ ] Reasoning/thinking support has an ADR-ready design covering request fields,
      stream event shape, persistence, TUI/RPC exposure, and usage/cost behavior.
- [ ] MEM-005 can consume model context/output/reasoning budget information for
      compaction trigger decisions.
- [ ] TUI/status/session summary can display model catalog source and estimated
      cost without pretending stale pricing is exact.

## Required Reads

- `docs/proposals/reasoning-thinking-field.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/AGENT-001-standard-agent-protocol-support.md`
- `docs/backlog/active/TUI-009-input-and-session-exit-polish.md`
- `docs/iterations/I036-research-consolidation.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `crates/talos-config/src/lib.rs`
- `crates/talos-agent/src/compaction.rs`
- `crates/talos-agent/src/token.rs`
- `crates/talos-tui/src/scrollback.rs`
