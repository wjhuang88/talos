# 048: MODEL-007 Variant Representation And Compatibility

> Status: Accepted (2026-07-17); amended (2026-07-18) under docs/sop/CHANGE-CONTROL.md
> Iteration: I141

## Amendment (2026-07-18)

Two corrections were issued by the maintainer during I141 planning, before any
MODEL-007 implementation code landed. Both are recorded here so future readers see the
current intent alongside the original rationale.

1. **Variant picker stage is conditional.** A model with no declared variants must
   **not** show a synthetic `Default` entry in a variant picker stage. The original
   wording below ("A model with no declared variants exposes one implicit `Default`
   variant that maps to `ModelConfig::default()`") is preserved for the persisted-config
   fallback semantics (an old config that happens to carry `variant = "default"` still
   resolves cleanly) but does **not** require the picker to surface a stage-3 screen
   with a single `Default` row. The picker skips the variant stage entirely when no
   variants are declared for the selected model.
2. **Picker UX mirrors `/connect`.** The MODEL-007 owner-doc requirement of stage-by-
   stage Esc/Back navigation (variant → model → provider → close) is replaced by the
   existing `/connect` UX pattern: a single primary picker screen with a conditional
   follow-up screen (Variant picker) that replaces the panel content when the selected
   model has declared variants. `Esc` closes the panel entirely, exactly as `/connect`
   does. This avoids inventing a new multi-stage navigation idiom.

The Compatibility Gate answers in the original Decision below remain valid; only the
picker UX contract and the synthetic `Default`-row behavior change.

## Context

MODEL-007 requires a three-stage Provider → Model → Variant picker. Before
implementation, the Architecture/Compatibility Gate requires an explicit
decision on variant identity, persistence, capability validation, and
dependency direction.

## Decision

### 1. Variants are catalog metadata derived from existing capabilities

A variant is a named projection of `ReasoningOptions` plus provider-specific
invocation presets already expressible in `ModelConfig`. Variants are NOT:

- User-created configurations (that remains `ModelConfig` in `[models.<key>]`)
- Arbitrary provider request JSON
- Credential-bearing entries

Variant definitions live in the built-in catalog (`models.toml`) as an
optional `[[variants]]` array per model entry. Example:

```toml
[[models]]
id = "o3"
provider = "openai"
# ... existing fields ...

[[models.variants]]
id = "default"
label = "Default"
reasoning_effort = "medium"

[[models.variants]]
id = "high-reasoning"
label = "High Reasoning"
reasoning_effort = "high"
```

A model with no declared variants exposes one implicit `Default` variant
that maps to `ModelConfig::default()` (no reasoning override). This is the
backwards-compatible path.

### 2. Stable persisted identity and migration

The persisted identity is `provider + model + variant_id` where `variant_id`
defaults to `"default"` when absent. Migration behavior:

- Existing configs with `provider` + `model` but no variant → resolves to
  `"default"` variant → no config rewrite needed
- Adding `variant` to `[talos]` config section is optional and additive
- Unknown/deleted variant → falls back to `"default"` with a diagnostic

No config migration is required. The `variant` field is additive and
optional in `Config`.

### 3. Capability validation and safe fallback

When a variant is selected but the provider/model does not support it
(e.g., reasoning effort on a non-reasoning model), the resolution order is:

1. If the variant's `reasoning_effort` is set and the model has
   `capabilities.reasoning = true` → apply it
2. If the model does not support reasoning → silently omit the reasoning
   field (the provider ignores it)
3. Unknown variant ID → resolve to `"default"` + diagnostic warning

### 4. No dependency cycle

Variants are defined in `talos-config::ModelMetadata` (already has
`capabilities`, `pricing`, etc.). The picker payload
(`ModelPickerVariantItem`) is a `talos-conversation::types` type that
references the variant by string ID — no `talos-config` dependency from
`talos-conversation`. The dependency direction remains:

```
talos-tui → talos-conversation → talos-core
talos-config → talos-core
talos-cli → talos-config + talos-conversation + talos-tui
```

No cycle is introduced.

## Consequences

- `Config` gains an optional `variant: Option<String>` field (additive, serde default)
- `ModelMetadata` gains an optional `variants: Vec<VariantDef>` field
- `ModelPickerData` gains `variants: Vec<ModelPickerVariantItem>` (additive)
- `ModelPickerData` gains `recent: Vec<ModelPickerItem>` (additive, I141 S6)
- `ModelPickerItem` gains `variant: Option<String>` (additive, I141 S6 redesign — used by Recent items to direct-switch with their recorded variant)
- `StatusSnapshot` gains `variant: Option<String>` (additive)
- All existing callers continue to work without changes (defaults to "default")
- No new top-level dependency other than `gix` in `talos-tui` (ADR-010 already approves it for read-only Git operations; used by TUI-031 only)

### Semver impact (I141 S2 reconciliation)

S2 lifted `ReasoningEffort` from `talos_config::types` to `talos_core::model` and
changed both `VariantDef.reasoning_effort` fields from `Option<String>` to
`Option<ReasoningEffort>`. This is a **source-incompatible** change for any
downstream Rust caller that constructs `VariantDef { reasoning_effort: Some("...".to_string()) }`.

Mitigations in place:

1. `talos_config::types::ReasoningEffort` is now a `pub use talos_core::model::ReasoningEffort;`
   re-export, so import paths still resolve.
2. `talos-config` and `talos-core` are **pre-1.0 workspace crates, not published to crates.io**.
   Their public APIs are internal to the Talos workspace and have no external consumers
   at this writing. The semver stability guarantee in AGENTS.md ("Crate public APIs are
   semver-bound") is therefore interpreted as workspace-internal stability until the
   crates are published externally (REL-002 gate).
3. This ADR is the decision record required by AGENTS.md for breaking changes; the
   migration plan is "update workspace callers", which S2 + S3+S4+S5 + S6 + Oracle
   review completed (every construction site verified via `cargo check --workspace
   --locked --tests` green).
4. The `Option<String>` → `Option<ReasoningEffort>` change does **not** affect the
   on-disk TOML/JSON wire format — `ReasoningEffort` uses `#[serde(rename_all = "lowercase")]`
   producing the same `"low"/"medium"/"high"` strings, so persisted configs and the
   `models.toml` catalog deserialize identically before and after the change.

When `talos-config` is eventually published externally (post-REL-002), the
`ReasoningEffort` lift will require either a `From<String> for ReasoningEffort`
adapter or a major-version bump at that time.

## Reversal Trigger

Revisit if a provider requires variant data that cannot be expressed as a
projection of existing `ModelConfig`/`ReasoningOptions` fields.
