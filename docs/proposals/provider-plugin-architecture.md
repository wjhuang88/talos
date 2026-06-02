# Provider plugin architecture for `talos`

## Status

Proposal. Captured 2026-06-02 as the long-term direction behind
[#I011-S1](../iterations/) (base_url override) and [#I011-S2](../iterations/)
(schema import foundation).

## Motivation

Today `talos` supports exactly two LLM providers, both hard-coded in
`talos-config::Provider` and `talos-provider`:

```rust
pub enum Provider { Anthropic, OpenAI }
```

Each new provider (DashScope, Bailian, Z.ai, self-hosted vLLM, LocalAI, ...)
requires a Rust compile-time addition: a new enum variant, a new
`impl LanguageModel for FooProvider`, a new arm in `build_provider()`. This
doesn't scale to the long tail of OpenAI-compatible gateways.

The reference project **opencode** already solves this with a JSON config
schema (documented at `https://opencode.ai/config.json`) where users declare
providers at runtime — a single config block can carry multiple gateways with
their own `baseURL`, auth, and model lists, no code change required. `talos`
should adopt the same shape.

## Reference: opencode provider schema (simplified)

Opencode's `provider` block in its JSON config declares gateways at runtime. The shape
(trimmed; full schema is at `https://opencode.ai/config.json`):

```json
"provider": {
  "<name>": {
    "npm": "@ai-sdk/openai-compatible",   // protocol adapter
    "name": "Display name",
    "options": {
      "baseURL": "https://gateway/v1"     // transport
    },
    "models": {
      "<model-name>": {
        "name": "Display",
        "modalities": { "input": ["text"], "output": ["text"] },
        "options": { "thinking": { "type": "enabled", "budgetTokens": 8192 } },
        "limit": { "context": 1000000, "output": 65536 }
      }
    }
  }
}
```

## Target shape for `talos`

The minimum that would replace today's hard-coded list:

```toml
# ~/.talos/config.toml
provider = "bailian-token-plan"            # user-chosen provider name
model = "glm-5"                            # user-chosen model

# (new section, populated from a separate file or by an import step)
[providers.bailian-token-plan]
base_url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"
protocol = "openai-chat"                   # only "openai-chat" for now; future: "anthropic-messages"
api_key_env = "BAILIAN_TOKEN_PLAN_API_KEY" # optional; defaults to OPENAI_COMPAT_API_KEY

[providers.bailian-token-plan.models.glm-5]
context_limit = 202752
output_limit = 16384
# thinking = { type = "enabled", budget_tokens = 8192 }   # see reasoning-thinking-field.md
```

The CLI keeps working: `talos -p "hi" --model glm-5` resolves the model name
against the providers table, looks up `base_url` + `protocol`, picks the
right adapter, and ships the request.

## Slicing

**S1 (shipped 2026-06-02, #I011-S1)**: Single hard-coded OpenAI provider, single
`base_url` override, single new env var (`OPENAI_COMPAT_API_KEY`). This is the
"dumb pipe" — works for any OpenAI-compatible endpoint, no schema.

**S2 (#I011-S2)**: Capture the opencode provider schema as a Rust type
(`ProviderSpec`, `ModelSpec`). Read a `~/.talos/providers.toml` (or
`[[providers]]` table in the main config) and use it to populate the runtime
config. One-way migration: opencode `provider.json` → `~/.talos/providers.toml`
(no need to keep them in sync).

**S3 (out of scope here)**: A provider written in another language that
`talos` loads at startup. This needs a stable IPC / FFI contract; defer until
S2 is in production and we have at least one external contributor asking for it.

## Open questions

- **TOML vs JSON for the providers file?** TOML is consistent with
  `config.toml` and the project is TOML-friendly. opencode uses JSON. Pick
  TOML, provide a one-shot `talos import opencode-config <path>` to translate.
- **Where do per-provider `api_key_env` defaults come from?** Conventions like
  `BAILIAN_TOKEN_PLAN_API_KEY` are gateway-specific and not standardized.
  Probably: let the user declare it, with a sensible default
  (`OPENAI_COMPAT_API_KEY`).
- **Model name resolution**: today `--model` is a free string. With multiple
  providers, the same model name (`glm-5`) might exist in several. Options:
  (a) require `provider/model` syntax (`bailian-token-plan/glm-5`); (b) error
  on ambiguity; (c) prefer the explicitly-set `provider`. Recommend (a) +
  fallback to the configured `provider` for the bare form.
- **Streaming protocol differences**: Most OpenAI-compatible gateways stream
  via SSE the same way, but chunk field names and reasoning-content placement
  differ. The `LanguageModel` trait needs an extension point for non-standard
  fields, or a `Provider` trait that lets the gateway-specific adapter rewrite
  chunks. See `reasoning-thinking-field.md`.

## Why "plugin" and not just "config"

The plugin framing is forward-looking. S1 and S2 are still configuration. The
"plugin" word signals that:

1. New providers can be added without touching the `talos` repo.
2. S3 (dynamic loading) is on the roadmap.
3. The configuration surface itself should be stable and documented as a
   contract, not a free-form TOML block that evolves ad hoc.
