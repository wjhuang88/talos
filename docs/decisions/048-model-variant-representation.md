# 048: MODEL-007 Variant Representation And Compatibility

> Status: Accepted (2026-07-17)
> Iteration: I141

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
- `StatusSnapshot` gains `variant: Option<String>` (additive)
- All existing callers continue to work without changes (defaults to "default")
- No new dependency

## Reversal Trigger

Revisit if a provider requires variant data that cannot be expressed as a
projection of existing `ModelConfig`/`ReasoningOptions` fields.
