# 034: Provider Reasoning / Thinking Boundary

## Status

Accepted (v4 revised 2026-07-10 for bounded visible-history projection)

Revision history:

- **v1 (2026-07-03)**: Initial acceptance. Persistence decision: "thinking is transient-only,
  never persisted."
- **v2 (2026-07-03)**: Cross-project research (REFERENCE-PROJECTS.md §20) showed transient-only
  is incorrect for Anthropic tool conversations, local-server KV caches, and some gateways.
  Persistence revised to two layers: display transient, request-history durable via
  `reasoning: Option<String>` on `Message`.
- **v3 (2026-07-03, this revision)**: Architecture review (Oracle consultation + codebase fact
  check + provider ground-truth verification; recorded in I084) rejected v2's persistence data
  model. `Option<String>` cannot carry Anthropic's mandatory `signature` field or
  `redacted_thinking` payloads, making v2 internally contradictory (its own guardrails required
  "thinking blocks with signatures"). The review also corrected two factual premises: session
  JSONL does not serialize `Message` structs (so `#[serde(default)]` on `Message` alone cannot
  deliver JSONL compatibility), and official OpenAI Chat Completions never streams
  `reasoning_content`. This revision redesigns persistence around structured `ReasoningBlock`s
  with origin-gated replay.
- **v4 (2026-07-10)**: Maintainer change control for TUI-029 supersedes v3's transient-only
  display clauses. Reasoning text already exposed through `ThinkingDelta` may be archived as a
  separate static TUI history block and reconstructed on resume from displayable text fields.
  Signatures and `ReasoningBlock::Redacted` remain opaque and non-displayable; provider replay,
  origin gating, and durable storage remain unchanged.

## Context

Modern LLM providers expose reasoning/thinking fields that let a model emit intermediate
chain-of-thought tokens before (or interleaved with) the visible answer. Talos currently ignores
all of these:

- **Anthropic** (`claude-sonnet-4-5`, `claude-opus-4-1`, etc.): request `thinking: { type:
  "enabled", budget_tokens: N }`; SSE `content_block_delta` with `type: "thinking_delta"` is
  silently dropped by the parser.
- **OpenAI o-series** (`o3`, `o4-mini`): request `reasoning_effort: "low" | "medium" | "high"`;
  reasoning happens server-side and is **not** streamed through Chat Completions (see Provider
  Ground Truth below).
- **OpenAI-compatible gateways and local servers** (Bailian, GLM, DeepSeek, llama.cpp, vLLM,
  Ollama): SSE `delta.reasoning_content` is silently dropped because `OpenAIDelta` lacks the
  field.

Users pointing at thinking-capable models get no thinking budget requested, reasoning stream
chunks are lost, and thinking tokens may be billed without being visible. This makes
thinking-capable models appear stalled or degraded.

MODEL-003 (P1) requires an ADR before any provider request schema, stream event, persistence,
TUI rendering, JSON-RPC payload, or evolution hook changes. This is that ADR.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Provider-specific format knowledge stays in `talos-provider` | Hard | UX-001 implementation principle | No |
| Public config schema is semver-bound | Hard | AGENTS.md hard constraint #6 | Only with migration plan |
| No hidden chain-of-thought exposure by default | Hard | MODEL-003 non-goal / UX-001 plan | No |
| `AgentEvent` is `#[non_exhaustive]` and serializable | Hard | talos-core protocol contract | No |
| Session JSONL is durable conversation history | Hard | talos-session persistence contract | No |
| Anthropic thinking blocks replayed byte-for-byte with `signature` in tool conversations | Hard | Anthropic API contract (SDK marks `signature` Required; 400 on violation) | No |
| Reasoning blocks are origin-bound to the (provider, model) that produced them | Hard | Anthropic docs: blocks "tied to the model that produced them" | No |
| Session JSONL encodes messages as role/content strings plus `SessionMetadata`, not serialized `Message` JSON | Hard (current format) | `talos-session/src/jsonl.rs` `message_parts()` / `read_messages()` | Only with migration plan |
| Reasoning should be opt-in per model, not global | Soft | Cost-control, user safety | Yes |
| Capability gating should warn, not hard-block | Soft | Catalog may be incomplete for user providers | Yes |
| Config follows existing per-model `ModelConfig` pattern | Soft | ADR-013, existing codebase convention | Yes |

## Current State (Codebase Facts)

Verified 2026-07-03 during the architecture review. The downstream display pipeline is wired;
the durable-payload pipeline does not exist yet.

| Layer | File | Current State |
| --- | --- | --- |
| `AgentEvent::ThinkingDelta { delta }` | `talos-core/src/message.rs:128` | Variant defined, `#[non_exhaustive]` enum, serializable |
| `Message::Assistant` | `talos-core/src/message.rs:49` | `{ content: String, tool_calls: Vec<ToolCall> }` — no reasoning carrier |
| `Usage` | `talos-core/src/message.rs:98` | 4 fields (`input_tokens`, `output_tokens`, `cache_read_tokens`, `cache_write_tokens`); no `reasoning_tokens` |
| Conversation engine | `talos-conversation/src/engine.rs:292` | Handles `ThinkingDelta` → `UiOutput::ThinkingPreview`; clears on `TurnStart`/`TurnEnd`/`Error`/cancel |
| Session persistence | `talos-session/src/jsonl.rs:25` | Excludes `ThinkingDelta` from event persistence. **Messages are NOT serialized as `Message` JSON**: `message_parts()` (line 286) flattens to role/content strings; `read_messages()` (line 105) reconstructs manually. `SessionMetadata` (`types.rs:14`) already carries per-entry `provider` + `model` |
| Assistant persistence path (main) | `talos-cli/src/mode_runners.rs:729-747` | `SessionEvent::TurnCompleted { Success { new_messages } }` persists the **agent's own `Message` values** via `append_with_metadata` with provider/model metadata |
| Assistant persistence path (legacy `--repl`) | `talos-cli/src/event_loop.rs:283-299` | Rebuilds `Message::Assistant` from accumulated text deltas with `tool_calls: vec![]` (already loses tool calls) |
| `/model` switch | `talos-cli/src/model_lifecycle.rs:197-211` | Rebuilds the agent from `read_messages()` history passed verbatim as `SessionConfig.initial_history`. **Every model switch round-trips through JSONL** |
| `ChatMessage` (in-memory display) | `talos-conversation/src/types.rs:7` | No reasoning field; thinking stays in `current_thinking_text` (cleared per turn) |
| `/export`, `/copy` | `talos-conversation` transcript builders | Built from visible `ChatMessage` content and tool displays, not from durable `Message` metadata |
| `ModelCapabilities.reasoning` | `talos-config/src/model.rs:57` | Flag exists; populated for 20+ models in `models.toml` |
| `ModelConfig` | `talos-config/src/types.rs:21` | `context_limit`, `output_limit` only |
| Anthropic SSE parser | `talos-provider/src/lib.rs:458` | Handles `text_delta`; drops `thinking_delta`, `signature_delta`, `redacted_thinking` |
| OpenAI SSE parser | `talos-provider/src/openai.rs:224` | `OpenAIDelta { content, tool_calls }` — no `reasoning_content` field |
| Anthropic request body | `talos-provider/src/lib.rs:194` | Assistant history → `text` + `tool_use` blocks only; `max_tokens: 4096` hardcoded at line 264 |
| OpenAI request body | `talos-provider/src/openai_request.rs:41` | `OpenAIMessage { role, content, tool_calls, tool_call_id }`; no `reasoning_effort`, no `max_completion_tokens` |

## Provider Ground Truth (verified 2026-07-03)

Authoritative sources checked during the review. These facts are load-bearing for the decision.

| # | Fact | Source |
| --- | --- | --- |
| 1 | Anthropic `ThinkingBlockParam` marks **both `thinking` and `signature` as Required**. Thinking blocks must be passed back "complete and unmodified" in tool conversations. | [Extended thinking docs](https://platform.claude.com/docs/en/build-with-claude/extended-thinking); [TS SDK](https://github.com/anthropics/anthropic-sdk-typescript/blob/main/src/resources/messages/messages.ts) `ThinkingBlockParam`; Python SDK `thinking_block_param.py` |
| 2 | Anthropic `redacted_thinking` blocks carry an encrypted `data` field (Required) and must be replayed verbatim. Thinking blocks "cannot be edited, reordered, filtered, or reconstructed" — violations return `400 invalid_request_error`. | [API errors](https://platform.claude.com/docs/en/api/errors); [SDK](https://github.com/anthropics/anthropic-sdk-typescript/blob/main/src/resources/messages/messages.ts) `RedactedThinkingBlockParam` |
| 3 | With thinking enabled, a tool-continuation request whose trailing assistant `tool_use` turn lacks its thinking block fails: "Expected `thinking` or `redacted_thinking`, but found `tool_use`". Replay is **mandatory with tools**, optional without. | [Extended thinking with tool use cookbook](https://platform.claude.com/docs/en/cookbook/extended-thinking-extended-thinking-with-tool-use) |
| 4 | Signatures are **model-bound, not API-key-bound**: "Signature values are compatible across platforms (Claude APIs, Amazon Bedrock, and Google Cloud)"; "Thinking blocks are tied to the model that produced them." Claude Code strips signature blocks on model fallback for cross-model continuity, not key binding. | [Thinking encryption](https://platform.claude.com/docs/en/build-with-claude/extended-thinking#thinking-encryption); [Adaptive thinking](https://platform.claude.com/docs/en/build-with-claude/adaptive-thinking) |
| 5 | DeepSeek native API is **conditional**: without tool calls, replayed `reasoning_content` is ignored; **with tool calls, it is required** — missing it returns 400. | [DeepSeek thinking mode](https://api-docs.deepseek.com/guides/thinking_mode); [reasoning model guide](https://api-docs.deepseek.com/guides/reasoning_model) |
| 6 | Google Gemini 3 requires `thoughtSignature` replay for multi-turn function calling; `thoughtsTokenCount` counts internal thinking tokens. | [Thought signatures](https://ai.google.dev/gemini-api/docs/thinking) |
| 7 | Official OpenAI Chat Completions has **no `delta.reasoning_content`**: the SDK `ChatCompletionChunk` delta carries only `content`/`function_call`/`refusal`/`role`/`tool_calls`. Reasoning state lives in the Responses API. `reasoning_content` is an OpenAI-compatible gateway/local-server convention (DeepSeek, GLM, Qwen templates). | [OpenAI reasoning guide](https://developers.openai.com/api/docs/guides/reasoning); [openai-python `chat_completion_chunk.py`](https://github.com/openai/openai-python/blob/main/src/openai/types/chat/chat_completion_chunk.py) |
| 8 | Local llama.cpp-style servers re-tokenize the full chat-template prompt; Qwen3/DeepSeek-R1/GLM templates reconstruct `<think>` from `reasoning_content`. Dropping it diverges from the slot's KV cache and forces full prompt re-processing. | omp.sh `packages/catalog/src/types.ts:214-226` (issue #3528) |
| 9 | Some gateways 400 with "Extra inputs are not permitted" when thinking is **off** but `reasoning_content` is supplied; others 400 when thinking is **on** and a tool-call turn lacks it. | omp.sh `packages/catalog/src/compat/openai.ts:133+` (issues #1071, #1484) |

## Reasoning

### Stream events: keep `ThinkingDelta` for display; add `ReasoningComplete` for the durable payload

`AgentEvent::ThinkingDelta { delta: String }` stays as the display stream. It is the right shape
for live preview: text-only, transient, cleared per turn.

It is, however, **insufficient as the durable carrier**. Anthropic signatures arrive through
`signature_delta` frames and block boundaries (`content_block_start`/`content_block_stop`), and
`redacted_thinking` blocks contain no displayable text at all. Threading opaque signature bytes
through the display event would leak non-display data into every display consumer — the exact
boundary violation the dedicated-variant design exists to prevent.

**Decision: add one new variant to `AgentEvent`:**

```rust
/// Emitted once per provider response, before `TurnEnd`, when the response
/// carried reasoning blocks. Durable replay payload; never render this event
/// directly. TUI-029 display projection uses the filtered text helper only.
ReasoningComplete { blocks: Vec<ReasoningBlock> },
```

- `AgentEvent` is `#[non_exhaustive]`, so adding a variant is semver-compatible for embedders and
  wire-compatible for RPC consumers (serde-tagged enum; unknown variants are a consumer concern
  they already signed up for via `#[non_exhaustive]`).
- Emitted **per provider response** (per API round), not per Talos turn — a tool loop makes
  multiple API rounds inside one turn, and each round's assistant message needs its own blocks
  attached before the tool-continuation request is built (Ground Truth #3).
- The event carries blocks only, no provider/model identity: the provider adapter does not
  reliably know its config provider key. Identity stamping is the agent's job (see Replay
  Policy).

### Data model: structured reasoning blocks, not a plain string

The v2 `reasoning: Option<String>` is rejected (review verdict Q1/Q6): it cannot carry
`signature` (Ground Truth #1), cannot represent `redacted_thinking` (#2), and cannot represent
omitted-display thinking where the text is empty but the signature carries encrypted content.

New types in `talos-core/src/message.rs`:

```rust
/// One provider-native reasoning block attached to an assistant message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReasoningBlock {
    /// Signed thinking (Anthropic `thinking` block). `text` may be empty when
    /// the provider omits display text; `signature` is opaque and must be
    /// replayed byte-for-byte, never inspected or trimmed.
    Thinking {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Encrypted redacted thinking (Anthropic `redacted_thinking`). Replayed
    /// byte-for-byte; never rendered anywhere.
    Redacted { data: String },
    /// Plain reasoning text (OpenAI-compatible `reasoning_content`).
    Plain { text: String },
}

/// Reasoning payload for one assistant message, stamped with the identity
/// that produced it. Request-history metadata; display consumers may access
/// only the filtered text projection defined by ADR-034 v4 / TUI-029.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssistantReasoning {
    /// Config provider key that produced the blocks (e.g. `anthropic`, `my-gateway`).
    pub provider: String,
    /// Model id that produced the blocks (e.g. `claude-sonnet-4-5`).
    pub model: String,
    /// Provider-native blocks in stream order.
    pub blocks: Vec<ReasoningBlock>,
}
```

`Message::Assistant` gains one optional field:

```rust
Assistant {
    content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tool_calls: Vec<ToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reasoning: Option<AssistantReasoning>,
},
```

Why identity lives on the payload and not on each block: the replay gate needs one O(1)
comparison per message, and `SessionMetadata` already stores per-entry provider/model in the
same shape (precedent). Blocks from a single response always share one origin.

### Request format: per-model options block

The existing `ModelConfig { context_limit, output_limit }` is the natural home for per-model
reasoning options, following `ProviderConfig.models: HashMap<String, ModelConfig>` and ADR-013.
A global `ReasoningConfig` stays rejected: reasoning is a model-level capability, and different
providers need different fields.

**Decision: add `reasoning: Option<ReasoningOptions>` to `ModelConfig`:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ReasoningOptions {
    /// Reasoning effort for OpenAI-style providers ("low", "medium", "high").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Token budget for Anthropic thinking blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_tokens: Option<u32>,
    /// Replay captured reasoning in request history. Default: true.
    /// Disabling trades provider correctness for token savings — see Replay Policy.
    #[serde(default = "default_true")]
    pub replay: bool,
}

pub enum ReasoningEffort { Low, Medium, High }
```

TOML shape:

```toml
[providers.anthropic.models.claude-sonnet-4-5]
context_limit = 200000
output_limit = 16000
reasoning = { budget_tokens = 10000 }

[providers.openai.models.o3]
reasoning = { effort = "high" }

[providers.my-gateway.models.glm-5]
reasoning = { budget_tokens = 8192, replay = false }
```

If `reasoning` is set but both `effort` and `budget_tokens` are `None`, the provider adapter
enables reasoning with its default (Anthropic: 80% of `output_limit`; OpenAI: `"medium"`).

### Provider-specific mapping

The `talos-provider` adapter owns vendor-specific JSON construction and stream parsing.

**Anthropic** (`ProviderProtocol::AnthropicMessages`):

- Request: `"thinking": { "type": "enabled", "budget_tokens": N }`. Force `temperature: 1`
  (Anthropic requirement when thinking is enabled). `budget_tokens` must be less than
  `max_tokens` — which requires fixing the pre-existing hardcoded `max_tokens: 4096`
  (`lib.rs:264`) to use `ModelConfig.output_limit` with a conservative fallback.
- Stream parsing: track `content_block_start`/`content_block_stop` boundaries.
  - `content_block_delta` + `thinking_delta` → emit `ThinkingDelta { delta }` (display) AND
    accumulate text into the current block.
  - `content_block_delta` + `signature_delta` → accumulate signature (never emitted as display).
  - `content_block_start` with `redacted_thinking` → capture `data` verbatim.
  - At stream end, if any blocks were captured → emit `ReasoningComplete { blocks }` before
    `TurnEnd`.
- Request replay: for assistant messages whose `reasoning` survived the gate, emit
  `thinking` / `redacted_thinking` content blocks **first, in captured order, before `text` and
  `tool_use` blocks** (API contract: thinking precedes other content in assistant turns).
  Signatures byte-for-byte.
- **Degradation guardrail**: if thinking is enabled AND the trailing assistant message carries
  `tool_calls` AND no reasoning blocks (legacy history, cross-version resume, or `replay =
  false`), **omit the `thinking` parameter for that request** and log a warning. Sending it
  guarantees the Ground Truth #3 400; omitting it degrades gracefully to a non-thinking
  continuation.
- Interleaved thinking (beta header) is out of scope for this ADR.

**OpenAI official** (`ProviderProtocol::OpenAIChat`, built-in OpenAI provider):

- Request: `"reasoning_effort"` top-level; use `max_completion_tokens` instead of `max_tokens`
  when reasoning is enabled (o-series requirement). Non-reasoning models keep the existing body.
- Stream: official Chat Completions **never carries reasoning content** (Ground Truth #7). No
  `ThinkingDelta` will be emitted, no blocks captured, no replay happens. This is correct
  behavior, not a gap. The Responses API is a separate future decision.

**OpenAI-compatible gateways and local servers** (`ProviderProtocol::OpenAIChat`,
user-configured providers):

- Request: `"reasoning_effort"` top-level when configured (most common gateway shape). Gateways
  needing a nested `options.thinking` shape are a follow-up slice if evidence demands it.
- Stream: add `reasoning_content: Option<String>` to `OpenAIDelta` (`#[serde(default)]`). When
  present → emit `ThinkingDelta { delta }` and accumulate; at stream end emit
  `ReasoningComplete { blocks: vec![ReasoningBlock::Plain { text }] }`.
- Request replay: for assistant messages whose `reasoning` survived the gate, set
  `reasoning_content` on the assistant message (concatenated `Plain` texts). This covers the
  DeepSeek tool-call requirement (Ground Truth #5) and local-server KV-cache stability (#8).

**Google Gemini native protocol**: not supported by Talos today. If a Gemini adapter is ever
added, `thoughtSignature` replay (Ground Truth #6) needs its own mapping under this ADR's
structured-block model (`Thinking { signature }` fits). Recorded as a reversal trigger.

### Replay policy: origin-gated, resolved once, enforced in one place

Reasoning replay is **conditional**, not universal (review verdict Q2/Q7). The gate:

| # | Condition | Action |
| --- | --- | --- |
| 1 | `ReasoningOptions.replay == false`, or no `reasoning` configured for the current model | Strip all reasoning from outgoing request copies. Warn once at config load when reasoning-capable history may exist (see below) |
| 2 | `reasoning.provider != current provider key` OR `reasoning.model != current model id` | Strip from outgoing request copies (foreign blocks: signatures are model-bound per Ground Truth #4; foreign text wastes tokens and can trigger gateway 400s per #9) |
| 3 | Origin matches AND replay enabled | Keep; provider adapter serializes (Anthropic: blocks; OpenAI-compatible: `reasoning_content`) |

Division of responsibility (each layer does one thing):

- **talos-config / CLI**: resolves policy. Computes `replay_enabled: bool` from
  `ReasoningOptions` and hands the agent its identity `(provider_key, model_id)` plus the flag
  through the existing `talos-agent/src/configuration.rs` setter surface (semver-additive).
- **talos-agent**: enforces the gate. When assembling the request history for each provider
  call, it filters `reasoning` on a **copy** of each assistant message per the table above.
  The stored history and JSONL keep their blocks untouched — switching back to the origin model
  restores replay. The agent also stamps `AssistantReasoning { provider, model, blocks }` when
  attaching `ReasoningComplete` payloads to assembled assistant messages.
- **talos-provider**: pure formatter. Serializes whatever reasoning survived; applies the
  Anthropic trailing-tool_use degradation guardrail (the only adapter-side check, because only
  the adapter decides the `thinking` request parameter).

Why the gate lives in the agent and not the adapters: two adapters would duplicate the logic and
drift; the agent owns history assembly and receives identity/config through an existing surface.

Known interaction to surface to users: `replay = false` with thinking enabled on Anthropic
effectively limits thinking to the first API round of each turn (the degradation guardrail
omits `thinking` on tool continuations). The config-load warning must say so:
"reasoning replay disabled for <model>: Anthropic tool continuations will run without thinking;
local-server KV caches may be invalidated."

Known gateway variance that one static rule cannot fully resolve (Ground Truth #9 cuts both
ways): per-provider compatibility overrides (à la omp.sh `requiresReasoningContentForToolCalls`)
are deferred until evidence demands them; conditions 1-2 already prevent the known
"Extra inputs" class by never sending reasoning fields when thinking is off.

### Capability gating

- The built-in model catalog (`models.toml`) carries `capabilities.reasoning: bool` for known
  models.
- If `reasoning` is configured for a model and the catalog says `reasoning = false`, emit a
  warning at config load and skip sending reasoning fields (warn-and-skip, matching OpenCode
  behavior).
- If the model is user-configured (not in the built-in catalog), trust the config — the
  provider will return an error if the model does not support reasoning.
- Reasoning is never auto-enabled. The user must explicitly set `reasoning = { ... }` per model.

### Persistence: three layers, one durable payload

Thinking content serves three purposes with different persistence requirements:

1. **Live display** (preview): transient `ThinkingDelta` text while the provider is reasoning.
2. **Visible-history projection** (TUI-029): finalized displayable text may enter static terminal
   scrollback and may be reconstructed on resume. This projection is not provider context and is
   not duplicated into session message content.
3. **Request-history persistence** (session store → resume → replay): durable, structured,
   origin-stamped `AssistantReasoning` metadata.

The critical corrected fact (review verdict Q4): session JSONL does **not** serialize `Message`
values. `message_parts()` flattens to role/content strings; `read_messages()` reconstructs
manually. Therefore `#[serde(default)]` on `Message` does nothing for JSONL. The durable path
must be built explicitly — and the right place is `SessionMetadata`, which already carries
per-entry `provider`/`model` and rides every `SessionEntry`:

```rust
// talos-session/src/types.rs — SessionMetadata gains:
#[serde(default, skip_serializing_if = "Option::is_none")]
pub reasoning: Option<AssistantReasoning>,
```

Round-trip is implemented **once, inside talos-session**, so every call site is covered without
modification:

- **Encode**: `append_with_metadata()` lifts `Message::Assistant.reasoning` into
  `SessionMetadata.reasoning` before building the entry. The `content` string stays untouched —
  reasoning never enters the content/`json-tool` fence encoding.
- **Decode**: `read_messages()`'s assistant arm sets `reasoning: entry.metadata.reasoning`.
- **Trap (blocking)**: `SessionMetadata::is_empty()` MUST add `&& self.reasoning.is_none()`.
  Because `SessionEntry.metadata` uses `skip_serializing_if = "SessionMetadata::is_empty"`, a
  metadata whose only populated field is `reasoning` would otherwise be **silently dropped from
  disk**.

End-to-end data flow (no step is optional):

```
provider stream ──ThinkingDelta──────────────► conversation preview (display, transient)
       │                                              │
       │                                              └─► bounded static history projection
       │                                                   at answer/tool transition
       │
       └─ReasoningComplete{blocks}─► agent stamps AssistantReasoning{provider,model,blocks}
                                     and attaches to the assembled assistant Message
                                          │
                    ┌─────────────────────┼──────────────────────────┐
                    ▼                     ▼                          ▼
        in-turn tool continuation   TurnCompleted{new_messages}   next-turn history
        (replay gate → provider)    → CLI append_with_metadata    (replay gate → provider)
                                    → SessionMetadata.reasoning
                                    → JSONL
                                          │
                          resume / model switch: read_messages()
                          re-attaches reasoning from metadata
```

Compatibility matrix:

| Direction | Behavior | Verdict |
| --- | --- | --- |
| Old JSONL → new Talos | `metadata.reasoning` absent → `None`. First tool-use continuation after resume triggers the Anthropic degradation guardrail (thinking omitted for that request, warned) | Acceptable; matches Anthropic guidance that new conversations need no prior thinking |
| New JSONL → old Talos | Unknown `reasoning` key inside `metadata` is ignored (serde default; no `deny_unknown_fields` anywhere in the workspace) | Safe |

The legacy `--repl` path (`event_loop.rs:287`) rebuilds assistant messages from text deltas and
already drops `tool_calls`; it will equally not carry `reasoning`. Accepted limitation,
consistent with that path's existing behavior.

The existing JSONL exclusion of `AgentEvent::ThinkingDelta` stays, and
`AgentEvent::ReasoningComplete` MUST be added to the same exclusion (`jsonl.rs:25` and the
bridge filter at `mode_runners.rs:724`): its payload persists via `SessionMetadata`, and
persisting it as an event row would duplicate signatures into event history.

### Security boundary: only displayable reasoning text reaches visible history

Provider-native reasoning remains request-history metadata, not ordinary assistant content. TUI-029
adds a deliberately narrower display projection:

- Live archival is sourced from `ThinkingDelta`, which is already displayable preview text. It is
  finalized before the first answer/tool event for that provider response.
- Resume archival may project only `ReasoningBlock::Thinking.text` and
  `ReasoningBlock::Plain.text` through one centralized helper. Empty text is skipped.
- `ReasoningBlock::Thinking.signature` and `ReasoningBlock::Redacted.data` are never inspected,
  rendered, copied, exported, logged, truncated, or reformatted.
- The archive is a separate reasoning history role. It must never be concatenated into
  `Message::Assistant.content`, a tool result, a system message, or provider request context.
- `/copy` and `/export` exclude reasoning by default. `/export <path> --include-thinking` may
  include only the filtered text projection with an explicit `Thinking` heading. Raw metadata is
  never exported through this flag.
- Any future raw-session export feature must redact `SessionMetadata.reasoning` by default or
  require explicit opt-in.
- JSONL on disk gains encrypted (`signature`, `redacted_thinking.data`) and plaintext reasoning
  content. This stays within the existing local `~/.talos` session-storage trust boundary; no
  new remote surface is created.

### Compaction: minimal boundary now, full policy in MEM-007

Anthropic guidance: only the thinking attached to the **current tool-use continuation** is
strictly necessary (Ground Truth #3). Unbounded replay of all historical reasoning is therefore
cost without correctness benefit for old turns.

This ADR fixes only the correctness-critical boundary:

- Reasoning attached to messages inside the active turn's tool loop is always replayed (gate
  permitting).
- When existing context-compaction layers drop or summarize old turns, their reasoning is
  dropped with them — reasoning never survives its message.
- Age-based reasoning trimming (e.g., strip reasoning from turns older than N) is **deferred to
  MEM-007** (active context compression), where token budgeting already lives.

### TUI rendering: live preview plus static finalized archive

The live one-line `PreviewComponent` remains unchanged while processing. When displayable reasoning
transitions to answer text or tool use, TUI-029 finalizes the accumulated text into a separate,
static reasoning history block before the answer/tool entry. The block uses a `Thinking` label and
indented `| ` body lines with a subdued style. It is a one-shot terminal scrollback print and does
not add an interactive widget, retained render buffer, or alternate history surface, so ADR-035
remains intact.

Cancellation and provider failure clear an unfinished preview without archiving it in the first
implementation slice. `MaxTokens` may archive text already delivered as displayable reasoning, but
must not fabricate missing content.

### Cost model: surface reasoning tokens as informational subset of output

Cross-project research (REFERENCE-PROJECTS.md §20) shows two patterns: separate tracking
(Cline, omp.sh, Pi, Codex) vs folded-in (OpenCode, Claude Code, Aider). Talos follows the
separate-tracking pattern:

**Decision: add `reasoning_tokens: u32` (`#[serde(default)]`) to `Usage`. Informational subset
of `output_tokens`, not additive. Priced at the normal output rate. Surfaced in the status bar
and exit summary as a breakdown (e.g., "1234 out / 800 thinking").**

Provider extraction (best-effort; absent fields default to 0):

| Provider | JSON path |
| --- | --- |
| Anthropic | `usage.output_tokens_details.thinking_tokens` |
| OpenAI | `usage.completion_tokens_details.reasoning_tokens` |
| OpenAI-compatible | `usage.completion_tokens_details.reasoning_tokens` or `usage.reasoning_tokens` |

### RPC / JSON-RPC exposure

`ThinkingDelta` and the new `ReasoningComplete` flow through the existing serializable
`AgentEvent` stream automatically. `#[non_exhaustive]` covers the additive variant for
embedders. No new RPC surface.

### Hidden chain-of-thought

If a provider marks reasoning as hidden (no stream content, only token counts), there is nothing
to display or persist — the natural fallback. Talos does not attempt to extract, unmask, or
reconstruct hidden chain-of-thought.

## Decision

1. **Stream events**: keep `AgentEvent::ThinkingDelta { delta }` for display; add
   `AgentEvent::ReasoningComplete { blocks: Vec<ReasoningBlock> }` emitted per provider response
   before `TurnEnd` when reasoning was captured. Semver-safe on the `#[non_exhaustive]` enum.

2. **Data model**: `ReasoningBlock { Thinking { text, signature }, Redacted { data },
   Plain { text } }` and `AssistantReasoning { provider, model, blocks }` in talos-core.
   `Message::Assistant` gains `reasoning: Option<AssistantReasoning>` (`#[serde(default)]`).
   The v2 `reasoning: Option<String>` is rejected.

3. **Config schema**: `reasoning: Option<ReasoningOptions>` on `ModelConfig` with
   `effort`, `budget_tokens`, and `replay: bool` (default `true`). Semver-compatible addition.

4. **Provider mapping**: Anthropic — `thinking` block request param, `temperature: 1`,
   `max_tokens` hardcode fix, full block-boundary stream parsing (`thinking_delta`,
   `signature_delta`, `redacted_thinking`), block replay (thinking first, byte-identical
   signatures), trailing-tool_use degradation guardrail. OpenAI official — request-side
   `reasoning_effort` + `max_completion_tokens` only; no stream reasoning exists. OpenAI-
   compatible — `reasoning_content` stream capture and replay.

5. **Replay policy**: origin-gated (exact provider+model match), config-gated
   (`replay`, reasoning configured), enforced in talos-agent on request copies; stored history
   and JSONL always keep blocks. Foreign blocks are never forwarded.

6. **Capability gating**: warn-and-skip if catalog says `reasoning = false`; trust config for
   user-configured models. Never auto-enable.

7. **Persistence**: three layers. Live preview is transient; visible history is a filtered
   projection; durable provider replay rides `SessionMetadata.reasoning` with encode/decode
   implemented symmetrically inside talos-session (`append_with_metadata` lifts;
   `read_messages` re-attaches). The visible projection creates no duplicate session payload.
   `ReasoningComplete` joins `ThinkingDelta` in the event-persistence exclusion.

8. **Security boundary**: only displayable `Thinking.text` / `Plain.text` may reach reasoning
   scrollback. Signatures and redacted payloads remain non-displayable. `/copy` and `/export`
   exclude reasoning by default; explicit `--include-thinking` exports filtered text only; future
   raw exports redact metadata by default.

9. **Compaction**: reasoning never survives its message through compaction; age-based trimming
   deferred to MEM-007.

10. **TUI rendering**: keep the one-line live preview and add a distinct static reasoning history
    block at answer/tool transition. No interactive widget or managed history surface.

11. **Cost model**: `reasoning_tokens: u32` on `Usage` as informational subset of
    `output_tokens`, priced at output rate, shown in status bar and exit summary.

12. **RPC**: no new surface; both variants flow through the existing `AgentEvent` stream.

## Rejected Alternatives

- **`reasoning: Option<String>` on `Message` (v2 design).** Rejected by the 2026-07-03
  architecture review: cannot carry Anthropic `signature` or `redacted_thinking` data, making
  the ADR's own Anthropic replay requirement unimplementable; internally contradictory.

- **Raw `serde_json::Value` as the reasoning carrier.** Rejected: pushes provider-format
  knowledge into every consumer, defeats exhaustive matching, and invites silent schema drift.
  The typed `ReasoningBlock` enum keeps provider mapping at the adapter boundary.

- **Carrying signatures through `ThinkingDelta`.** Rejected: leaks opaque non-display bytes into
  every display consumer; breaks the display/durable type-level boundary.

- **Origin-free replay (send all stored reasoning to whatever model is current).** Rejected:
  signatures are model-bound (Ground Truth #4); foreign replay risks provider 400s and wastes
  tokens. Claude Code, omp.sh, and Pi all gate or strip on model change.

- **Replay gate inside each provider adapter.** Rejected: duplicated drift-prone logic; the
  agent owns history assembly and already has a configuration surface for identity.

- **Global `ReasoningConfig` on top-level `Config`.** Rejected: reasoning is model-level.

- **Discriminator on `TextDelta` (e.g., `is_reasoning: bool`).** Rejected: every consumer must
  check a flag; leaks reasoning into assistant text if one forgets.

- **Persisting reasoning into the JSONL `content` string (fence/prefix encoding).** Rejected:
  pollutes the display-content channel that `read_messages`/`strip_tool_syntax` parse; metadata
  is the structured channel that already exists for exactly this kind of data.

- **Auto-enable reasoning for thinking-capable models.** Rejected: unexpected billing.

- **Hard-block reasoning config when catalog says `reasoning = false`.** Rejected: catalog may
  be incomplete. Warn-and-skip.

- **Collapsible / scrollable thinking panel in TUI.** Rejected for this slice.

- **Per-gateway compatibility override flags in the first slice.** Deferred until evidence
  demands them; the origin+config gate prevents the known failure classes.

## Implementation Guardrails

Exact touchpoints, in dependency order. Each carries its own test obligation (see matrix).

1. **talos-core** (`message.rs`): add `ReasoningBlock`, `AssistantReasoning`, the
   `Message::Assistant.reasoning` field, `AgentEvent::ReasoningComplete`, and
   `Usage.reasoning_tokens`. All additions `#[serde(default)]`/`skip_serializing_if` per the
   shapes above. Every in-workspace `Message::Assistant { .. }` constructor (including tests)
   gains `reasoning: None` — mechanical, compiler-driven.

2. **talos-config**: `ReasoningOptions` + `ReasoningEffort` types; `ModelConfig.reasoning`
   field; config-load validation emitting (a) the capability warn-and-skip, (b) the
   `replay = false` consequence warning. Update `config.reference.toml`.

3. **talos-session**: `SessionMetadata.reasoning` field; **update `is_empty()`** (blocking trap
   — see Persistence section); lift in `append_with_metadata()`; re-attach in
   `read_messages()`; extend the `append_event` exclusion to `ReasoningComplete`.

4. **talos-provider**: Anthropic — request `thinking` param, `temperature: 1`, replace the
   `max_tokens: 4096` hardcode (`lib.rs:264`) with `ModelConfig.output_limit` (fallback 4096),
   block-boundary stream parsing, block replay ordering (thinking → text → tool_use),
   trailing-tool_use degradation guardrail. OpenAI — `reasoning_effort` +
   `max_completion_tokens` swap (only when reasoning enabled), `OpenAIDelta.reasoning_content`
   capture, `reasoning_content` replay on assistant messages. Both — `reasoning_tokens` usage
   extraction with graceful 0 default. Adapters must not panic when reasoning is configured but
   the stream carries none.

5. **talos-agent**: identity + replay-flag configuration setters
   (`configuration.rs`, semver-additive); stamp-and-attach `ReasoningComplete` payloads onto the
   assembled per-response assistant message **before** the tool-continuation request is built;
   replay gate on request copies (never mutate stored history).

6. **talos-conversation / talos-tui (v3 implementation)**: handle `ReasoningComplete` as a no-op
   in the engine match (explicitly, not via wildcard); status bar / exit summary gain the
   reasoning-token breakdown. ADR-034 v4 supersedes only the no-display conclusion through the
   separate filtered projection contract below; `ReasoningComplete` still must not be rendered
   directly.

7. **talos-cli**: no changes on the main persistence path (`TurnCompleted.new_messages` flows
   automatically); `event_loop.rs:287` constructor gains `reasoning: None` (legacy path,
   accepted limitation); `mode_runners.rs:724` bridge filter excludes `ReasoningComplete`.

Test matrix (minimum; all must exist before the slice closes):

| Area | Test |
| --- | --- |
| Anthropic parse | `thinking_delta` + `signature_delta` + `content_block_stop` → `ThinkingDelta`s + one `ReasoningComplete` with byte-identical signature |
| Anthropic parse | `redacted_thinking` block → `Redacted { data }` captured verbatim |
| Anthropic replay | Same-origin history → request JSON carries thinking blocks first with byte-identical signature |
| Anthropic degradation | Thinking on + trailing assistant `tool_use` without blocks → `thinking` param omitted + warning |
| Replay gate | Foreign (provider or model mismatch) → reasoning absent from request; stored history unchanged |
| Replay gate | `replay = false` → reasoning absent from request + config-load warning emitted |
| Gateway safety | Reasoning not configured + blocks in history → no reasoning fields sent |
| OpenAI-compatible | `delta.reasoning_content` → `ThinkingDelta` + `ReasoningComplete { Plain }`; replay sets `reasoning_content` |
| OpenAI official | Reasoning enabled → `reasoning_effort` + `max_completion_tokens` in body; non-reasoning models keep existing body |
| JSONL round-trip | Assistant with reasoning → append → read_messages → identical `AssistantReasoning` |
| JSONL compat | Old entry (no metadata.reasoning) → `reasoning: None`; entry JSON with unknown metadata key → loads |
| JSONL trap | Metadata with only `reasoning` set → serialized (is_empty updated); `ReasoningComplete` excluded from event rows |
| Usage | `reasoning_tokens` extraction per provider path; absent → 0 |
| Capability gating | Catalog `reasoning = false` + configured → warn-and-skip |

## Implementation Phasing (I084 first slice)

In scope for UX101-UX102 (order matters — data model first):

1. Data model + persistence round-trip (guardrails 1-3). **Structured storage is not deferrable**
   (review verdict Q6): shipping Anthropic replay without signatures is worse than not shipping
   it — mutated blocks violate the API contract.
2. Anthropic path complete (guardrail 4 Anthropic + 5): parse, attach, gate, replay, degrade.
3. OpenAI / OpenAI-compatible path (guardrail 4 OpenAI + 6-7).

Explicitly deferred:

- Age-based reasoning compaction → MEM-007.
- Interactive/collapsible thinking TUI → separate future decision. TUI-029 v4 now owns the bounded
  static-history projection only.
- Per-gateway compatibility overrides / nested `options.thinking` request shapes → follow-up
  slice on evidence.
- Gemini native adapter (`thoughtSignature`) → only with a future Gemini protocol decision.
- OpenAI Responses API (encrypted reasoning items) → separate ADR when Responses support lands.

If the slice must shrink further, cut the **entire Anthropic replay path** (ship stream preview
UX101 + request mapping UX102 without persistence) rather than shipping unsigned persistence.
Do not resurrect the v2 hybrid.

## v4 Implementation Contract (TUI-029)

The v4 display amendment is implementable as a presentation projection over existing structured
reasoning. It does not change provider adapters or the session wire format.

1. **Conversation state**: accumulate the existing `ThinkingDelta` text. Finalize it exactly once
   before the first `TextDelta`, `ToolCallStarted`, or `ToolCall` that follows that reasoning
   segment, or at a successful/`MaxTokens` `TurnEnd`. A tool loop may therefore produce more than
   one reasoning archive block. Error and cancellation discard unfinished text in the first slice.
2. **Typed history**: add `Reasoning` to the conversation-layer `MessageRole` and
   `MessageSource`; never encode the distinction in a string prefix or reuse `System`/`Assistant`.
   These public enums are semver-bound. The implementation change must be called out in release
   notes; downstream exhaustive matches must add the new variant. No `talos-core::Message` variant
   is added and provider protocol compatibility is unchanged.
3. **Static format**: render a one-shot scrollback block with a `Thinking` label followed by
   indented `| ` body lines in a subdued semantic color. Full displayable text is retained; no
   collapsible widget, alternate screen, or viewport history is introduced.
4. **Resume**: `talos-tui::hydrate_history` reconstructs the reasoning block immediately before
   its assistant answer/tool call by using a centralized projection helper over
   `AssistantReasoning`. The helper returns only non-empty `Thinking.text` and `Plain.text` in block
   order and ignores signatures and `Redacted` blocks.
5. **Copy/export**: reasoning-role messages are excluded from existing transcript and `/copy`
   output by default. `/export <path> --include-thinking` includes the same filtered projection in
   Markdown/plain text; no raw signature or redacted payload can enter the export path.
6. **No duplicate persistence**: do not add another session field or copy reasoning into message
   content. Existing `SessionMetadata.reasoning` is the sole durable source.

Minimum implementation evidence:

| Area | Required evidence |
| --- | --- |
| Conversation transition | Thinking -> answer and thinking -> tool each emit one reasoning archive before the next entry; repeated terminal events do not duplicate it |
| Failure boundary | Cancellation/error clear preview without archiving partial text |
| Security projection | Thinking/Plain text is returned in order; signature and Redacted payload sentinel values never appear |
| TUI rendering | Static `Thinking` heading and indented body render in terminal scrollback with no viewport widget |
| Resume | Persisted assistant reasoning rehydrates before the associated answer/tool entry |
| Export | Default export excludes reasoning; `--include-thinking` includes filtered text only |
| Runtime | Real TUI/provider fixture demonstrates live preview -> static history -> answer/tool ordering and resume behavior |
| Regression | Provider replay fixtures remain byte-identical; workspace test/clippy/governance gates pass |

## Reversal Trigger

Revisit if:

- A Gemini-native or OpenAI Responses adapter is added (both need new block variants/mappings).
- Evidence shows gateways need per-provider replay compatibility flags
  (`requires_reasoning_replay` / `forbids_reasoning_replay`).
- A committed requirement needs interactive/collapsible reasoning history beyond TUI-029's static
  projection (which would trigger ADR-035 redesign).
- Provider APIs change their reasoning field shapes in ways that break the current mapping.
- MEM-007 lands and reasoning-aware compaction changes replay behavior.

## Related

- [MODEL-003: Reasoning / Thinking Field Support](../backlog/active/MODEL-003-reasoning-thinking-support.md)
- [UX-001: Experience Reliability Program](../backlog/active/UX-001-experience-reliability-program.md)
- [Reasoning / Thinking Field Proposal](../proposals/reasoning-thinking-field.md)
- [ADR-013: Provider Config Schema Boundary](013-provider-config-schema-boundary.md)
- [ADR-009: Tool Provenance Tracking](009-tool-provenance.md)
- [Reference Projects §20: Reasoning / Thinking Token Usage](../reference/REFERENCE-PROJECTS.md#20-reasoning--thinking-token-usage-adr-034-research)
- [I084: Experience Reliability](../iterations/I084-experience-reliability.md) — UX100 review and
  revision record
