# MODEL-003: Reasoning / Thinking Field Support

**Status**: In Progress (UX100 ADR-034 accepted v3 2026-07-03; UX101-UX106 implementation pending)
**Priority**: P1
**Source**: MODEL-001 split (2026-06-20); original proposal `docs/proposals/reasoning-thinking-field.md` (2026-06-02)
**Depends on**: MODEL-001 catalog (model metadata schema for reasoning capability flags); ADR gate

## Problem

Modern LLM providers expose reasoning/thinking fields that Talos currently ignores. This is now a
P1 UX reliability item because missing thinking compatibility makes thinking-capable models appear
stalled or degraded even though Talos already has a transient TUI preview boundary:

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

## Priority Update 2026-07-03

Maintainer feedback elevated this story from P2 to P1 and grouped it under
`UX-001: Experience Reliability Program`. The first implementation container is
`I084: Experience Reliability`, with UX100-UX102 covering ADR, stream normalization, and request-side
reasoning configuration. The previous ADR gate still stands; elevation changes sequencing, not the
requirement to decide provider request schema and persistence boundaries before code.

### ADR gate (RESOLVED — ADR-034 v3 accepted 2026-07-03)

Per the original proposal's explicit gate: *"This proposal is not sufficient as an
implementation authority. Create an ADR before code when the design changes
provider request schemas, session persistence, stream event types, TUI rendering,
JSON-RPC payloads, or evolution hook contracts."*

ADR-034 decisions (all 7 dimensions resolved):
- [x] Request format: per-model `reasoning: Option<ReasoningOptions>` on `ModelConfig`
- [x] Provider-specific mapping: Anthropic (`thinking` block + `temperature:1`), OpenAI (`reasoning_effort` + `max_completion_tokens`), OpenAI-compatible (`reasoning_content` stream/replay)
- [x] Stream event shape: keep `ThinkingDelta` for display + new `ReasoningComplete { blocks }` for durable payload
- [x] Persistence: structured `ReasoningBlock` / `AssistantReasoning` via `SessionMetadata.reasoning` (JSONL round-trip inside talos-session); display stays transient
- [x] TUI rendering: existing one-line `"thinking: {text}"` preview; no new widget for this slice
- [x] Cost model: `reasoning_tokens: u32` on `Usage` as informational subset of `output_tokens`, priced at output rate
- [x] RPC/JSON-RPC: both variants flow through existing `AgentEvent` stream; `#[non_exhaustive]` covers additive variant

### OpenCode Reference Implementation (2026-06-20 research)

OpenCode (`opencode-ai/opencode`, Go) provides the most mature open-source
reasoning/thinking implementation. Talos should follow these proven patterns:

**Config shape**: Talos differs from OpenCode here. OpenCode uses a flat
`reasoningEffort: "low|medium|high"` on the Agent struct. Talos should use
**per-model options** in `[providers.{name}.models.{id}]` blocks — this is
already Talos's config pattern and the opencode import module already maps
per-model `limit` fields. Adding `options.thinking` follows the same pattern.

**Capability gating** (`CanReason bool` on Model struct):
- MODEL-001 catalog tracks whether each model supports reasoning.
- Provider code checks this flag before sending reasoning fields.
- Config validation: if `reasoning` is configured but model lacks `CanReason`,
  warn and skip (OpenCode does this at config load).

**OpenAI path** (from OpenCode):
- `reasoning_effort: "low|medium|high"` → maps to `ReasoningEffortLow/Medium/High`.
- **Critical**: use `max_completion_tokens` instead of `max_tokens` when reasoning
  is enabled (o-series API requirement).
- OpenAI streaming SDK does NOT expose reasoning deltas — reasoning content is
  only in the final response, not as stream chunks.

**Anthropic path** (from OpenCode):
- `thinking: {type: "enabled", budget_tokens: N}` in request body.
- **Content-triggered** activation: OpenCode checks if the user prompt contains
  "think" via `DefaultShouldThinkFn`. The budget is auto-calculated as **80% of
  maxTokens**, and temperature is forced to 1 (Anthropic requirement).
- SSE stream: `thinking_delta` events → dedicated `Thinking` field.

**Stream events** (from OpenCode):
- `ProviderEvent { Type: EventThinkingDelta, Thinking: string }` — dedicated
  event type, NOT a variant of TextDelta. This keeps thinking and text content
  cleanly separated.
- Talos equivalent: add `AgentEvent::ReasoningDelta { delta: String }` variant.

**Message persistence** (from OpenCode):
- `ReasoningContent` as a `ContentPart` with `Thinking: string` field.
- Serialized as `{"type":"reasoning","data":{"thinking":"..."}}` in message parts.
- Stored alongside text content in the same message, not as a separate message.
- Talos equivalent: add `reasoning_content: Option<String>` to `ChatMessage`.

**TUI rendering** (from OpenCode):
- Shows thinking content only when no text content exists yet (thinking phase).
- Once text content arrives, switches to showing text (answer phase).
- Status bar: "Thinking..." → "Generating..." transition.

### Implementation (post-ADR)

- Request body construction per provider:
  - Anthropic `thinking: {type: "enabled", budget_tokens: N}`
  - OpenAI o-series `reasoning_effort` plus `max_completion_tokens` handling
  - OpenAI-compatible nested options such as `options.thinking`
- SSE stream parsing for reasoning chunks:
  - Anthropic `thinking_delta`
  - OpenAI-compatible `reasoning_content` or configured interleaved reasoning field
  - provider-marked hidden reasoning must not be exposed by default
- Normalize provider chunks into Talos's transient thinking preview boundary.
- Keep final assistant text separate from thinking preview text.
- Decide whether any reasoning is persisted before changing JSONL/session schema.
- Usage/cost display in status bar and exit summary where provider metadata supports reasoning token
  accounting.

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

## UX-001 Integration

MODEL-003 pairs with `PROVIDER-002`:

- MODEL-003 handles **what** different providers mean by thinking/reasoning and how Talos normalizes
  it.
- PROVIDER-002 handles **when** the provider call is considered stalled, retryable, timed out, or
  failed.
- TUI/conversation work in I084 must consume both so users see meaningful states instead of silent
  waits.

## Required Reads

- `docs/proposals/reasoning-thinking-field.md`
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `crates/talos-provider/src/anthropic.rs`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-core/src/message.rs` (AgentEvent variants)
- `crates/talos-config/src/opencode.rs` (opencode import precedent)
- `https://github.com/opencode-ai/opencode` (reference implementation)
