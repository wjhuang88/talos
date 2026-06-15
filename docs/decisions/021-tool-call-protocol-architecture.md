# ADR-021: Tool Call Protocol Architecture

- **Status**: Accepted
- **Date**: 2026-06-15
- **Story**: CODE-002

## Context

Talos needs to support tool calling across diverse LLM providers: native tool_use
(Anthropic Claude, OpenAI GPT-4, Gemini), text-based models (GLM, Qwen, Ollama),
and OpenAI-compatible gateways. Different models produce wildly different tool call
formats, and a single-parser approach creates reliability issues.

Reference implementations:
- **Pi Agent**: Pure native tool_use only — clean but limits model coverage
- **OpenCode**: Multi-parser with `toolParser` config — flexible but models can
  oscillate between formats, creating unpredictable behavior

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| Tool calls must not crash the process | Hard | AGENTS.md HC #9 | No |
| Must support non-native models (GLM, Ollama) | Soft | Product scope | Yes |
| Users should not need to debug tool call format failures | Soft | UX requirement | Yes |
| Parser complexity must not grow unboundedly | Soft | Maintenance cost | Yes |

## Decision

Adopt a **tiered protocol** architecture with a unified internal IR:

### Tier 1: Native Tool Use (preferred)
- Anthropic tool_use via `content_block_start`/`content_block_stop`
- OpenAI tools via `tool_calls` delta
- Gemini function_call via native events
- No text parsing needed — structural events from provider

### Tier 2: Talos Strict (standard fallback)
A single, well-defined text format enforced by the system prompt:

```
<tool_call>
{"name":"tool_name","args":{"key":"value"}}
</tool_call>
```

Rules enforced in system prompt:
- Output exactly one `<tool_call>` block
- Do not use markdown fences
- Do not explain before the tool call
- Content must be valid JSON with `name` and `args` fields

### Tier 3: Compat Parsers (recovery only)
Available but not the primary path:
- `json-tool` fenced blocks
- Legacy XML variants
- Raw JSON extraction
- Single-tool text recovery

Recovery successes log a warning; recovery failures fall through gracefully.

### Unified Internal IR

All parsers produce the same `ToolCall` struct:
```rust
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}
```

Downstream consumers (agent, TUI, session) never see parser-specific formats.

### Execution Pipeline

```
SSE chunk
  → StreamNormalizer (CR/LF, UTF-8)
  → ToolSyntaxFilter (strip raw syntax from visible output)
  → PrimaryParser (native or Talos strict)
  → RecoveryParsers (if primary fails)
  → ToolCall IR
  → Schema Validation (against tool parameter schema)
  → Dedup (same name + args within turn)
  → Permission Check
  → Tool Executor
```

### Configuration

Per-model, not per-session:
```toml
[model.claude]
tool_protocol = "native"

[model.glm5]
tool_protocol = "talos-strict"
```

The `LanguageModel` trait is unchanged; tool call handling is a provider-internal concern.

## Reversal Trigger

Re-evaluate if:
- A new model family requires a fundamentally different protocol not covered by the three tiers
- Native tool_use becomes universally supported across all target models
- Text-based parsing proves fundamentally unreliable across multiple model families

## References

- OpenCode PR #16531 (custom tool parsers)
- OpenCode issue #2917 (tool parser middleware)
- Pi Agent Rust (native tool_use architecture)
- ADR-020 (tree-sitter dependency)
