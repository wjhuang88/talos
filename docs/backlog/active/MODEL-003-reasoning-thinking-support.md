# MODEL-003: Reasoning / Thinking Field Support

**Status**: ADR-needed
**Priority**: P2
**Source**: MODEL-001 split (2026-06-20); original proposal `docs/proposals/reasoning-thinking-field.md` (2026-06-02)
**Depends on**: MODEL-001 catalog (model metadata schema for reasoning capability flags); ADR gate

## Problem

Modern LLM providers expose reasoning/thinking fields that Talos currently ignores:

| Provider | Request field | Stream handling |
|---|---|---|
| Anthropic Claude | `thinking: {type: "enabled", budget_tokens: N}` | Thinking content in SSE stream chunks |
| OpenAI o-series | `reasoning_effort: "low" / "medium" / "high"` | `reasoning_content` delta |
| Bailian / OpenAI-compatible | `options.thinking: {type: "enabled", budgetTokens: N}` | Provider-specific SSE field |

Consequences:
- Users pointing at thinking-capable models get no thinking budget requested
- Reasoning stream chunks are silently dropped
- Thinking tokens may be billed but invisible to the user
- Pricing display cannot account for reasoning token cost

## Scope

Design and implement end-to-end reasoning/thinking support across the full Talos
pipeline.

### ADR gate (must be resolved before any implementation)

Per the original proposal's explicit gate: *"This proposal is not sufficient as an
implementation authority. Create an ADR before code when the design changes
provider request schemas, session persistence, stream event types, TUI rendering,
JSON-RPC payloads, or evolution hook contracts."*

ADR must decide:
- [ ] Request format: per-model `models.{name}.options` block vs global `ReasoningConfig`
- [ ] Provider-specific mapping for each known provider (Anthropic, OpenAI, OpenAI-compatible)
- [ ] Stream event shape: `ReasoningDelta` variant or same `TextDelta` with discriminator
- [ ] Persistence: store reasoning output in JSONL as separate field, or strip to save disk
- [ ] TUI rendering: collapsible section, hidden by default, or inline indicator
- [ ] Cost model: whether reasoning tokens are billed separately and surfaced in usage
- [ ] RPC/JSON-RPC exposure

### Implementation (post-ADR)

- Request body construction per provider
- SSE stream parsing for reasoning chunks
- `AgentEvent` extension (if needed)
- TUI rendering
- Session persistence
- Usage/cost display in status bar and exit summary

## Relationship To MODEL-001

MODEL-001 catalog tracks **capability flags** (whether a model supports reasoning,
whether reasoning is visible/hidden/interleaved). MODEL-003 implements the
**runtime behavior** — actually sending the thinking budget and handling the stream.

## Non-Goals

- Do not implement without the ADR gate (proposal requirement).
- Do not add a global `ReasoningConfig` that forces one shape on all providers.
- Do not expose hidden chain-of-thought by default in the TUI.

## Acceptance Criteria (post-ADR)

- [ ] ADR documented per ADR template with Constraint Decomposition
- [ ] Provider-specific reasoning request fields constructed correctly
- [ ] Reasoning stream chunks parsed and routed (not silently dropped)
- [ ] TUI renders reasoning output per ADR decision
- [ ] Session JSONL preserves reasoning per ADR decision
- [ ] Usage/cost display accounts for reasoning tokens where provider metadata supports it
- [ ] Existing provider tests pass; new tests cover each provider's reasoning path

## Required Reads

- `docs/proposals/reasoning-thinking-field.md`
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `crates/talos-provider/src/anthropic.rs`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-core/src/message.rs` (AgentEvent variants)
