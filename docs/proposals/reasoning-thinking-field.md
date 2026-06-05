# Reasoning / thinking field support for LLM providers

## Status

Proposal. Captured 2026-06-02 from a gap surfaced while wiring
[#I011-S1](../iterations/) OpenAI-compatible `base_url` support.

This proposal is not sufficient as an implementation authority. If it becomes a
backlog story, create an ADR before code when the design changes provider
request schemas, session persistence, stream event types, TUI rendering,
JSON-RPC payloads, or evolution hook contracts.

## Problem

Many modern LLM providers expose a "reasoning" / "thinking" field that lets the
model emit intermediate chain-of-thought tokens (often hidden from the user, but
billed) before producing the visible answer. Three flavors observed in the wild:

| Provider / model | Field name | Where it lives |
|---|---|---|
| Anthropic Claude (extended thinking) | `thinking: {type: "enabled", budget_tokens: N}` | Request body, top-level |
| OpenAI o-series | `reasoning_effort: "low" \| "medium" \| "high"` | Request body, top-level |
| Alibaba Bailian `qwen3.6-plus` / `glm-5` (per published OpenAI-compatible schema) | `options.thinking: {type: "enabled", budgetTokens: N}` | Per-model nested in `models.{name}.options` |
| OpenCode abstract | `interleaved: {field: "reasoning_content"}` | Tells the client where to find the reasoning stream chunk in SSE |

`talos` currently ignores all of these. A user pointing at Bailian `glm-5` (or any
thinking-capable model) via the new `base_url` override will not get the
`<thinking>` budget requested, and the SSE stream chunks carrying reasoning content
will be lost (the model will still produce the final answer, but reasoning will be
omitted from any prompt-cache-aware context and may bill more than the user expects).

## Why we are NOT doing this in I011-S1

- Scope discipline: #I011-S1 ships a runtime base_url override so that any
  OpenAI-compatible gateway is reachable. Reasoning is a model-level behavior, not
  a transport-level one — it needs provider-specific request-body construction.
- Cross-provider: each vendor has its own field name and shape. Doing it well
  requires either a normalized `ReasoningConfig` or per-provider implementations.
- Cost surface: enabling reasoning without a budget is expensive. The config
  surface (system prompt budget vs. user override vs. CLI flag) needs design
  discussion before code.

## When this becomes a story

Trigger conditions (any one is enough):

- A user asks for "use the thinking model" and is surprised it does not think.
- I011-S1 ships and a follow-up user actually runs `talos` against Bailian
  `glm-5` and notices missing reasoning.
- Provider-plugin-architecture (#I011-S2) lands and at least one external provider
  exposes a thinking field; we need to forward it.
- A story proposes adding `ReasoningDelta`, persisted reasoning fields, or
  provider-specific request options. That story must reference the ADR gate
  above before implementation.

## Sketch (when we DO pick it up)

Two reasonable shapes:

**A) Per-model in the opencode-style config block:**
```toml
# ~/.talos/providers.toml  (proposed; not implemented)
[[providers.models]]
name = "glm-5"
options = { thinking = { type = "enabled", budget_tokens = 8192 } }
```

**B) First-class field on `Config` or on a future `ProviderConfig`:**
```rust
pub struct ReasoningConfig {
    pub enabled: bool,
    pub budget_tokens: Option<u32>,
}
```

(A) follows the opencode precedent and is the natural target for #I011-S2's
schema-import work. (B) is more explicit but only works for one provider at a
time. Recommendation when the time comes: do (A) as part of #I011-S2 so we don't
paint ourselves into a per-provider corner.

## Open questions

- Should reasoning output be persisted in the session (JSONL) as a separate field,
  or stripped to save disk?
- Should the user see reasoning in the TUI as a collapsible section, or hide it
  entirely? (UX decision; the opencode config has an `interleaved.field` knob for
  this.)
- For SSE streams that carry reasoning in a separate field (Bailian, others), does
  the existing `AgentEvent::TextDelta` need a sibling `ReasoningDelta` variant?
  This ripples into the TUI, the JSON-RPC protocol, and the I008 evolution hooks
  (which currently only observe `OnTextDelta`).
