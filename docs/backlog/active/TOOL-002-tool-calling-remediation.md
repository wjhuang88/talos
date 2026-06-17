# TOOL-002: Tool Calling Architecture Remediation

| Field | Value |
|-------|-------|
| Story ID | TOOL-002 |
| Priority | P0 |
| Status | P0 Complete; P1-P2 Partial |
| Depends On | ADR-021 (Tool Protocol Architecture) |
| Blocks | Reliable tool execution across all providers |
| Origin | Architecture review 2026-06-15 |

## Problem

The tool calling pipeline evolved organically through incremental patches during the
initial tool calling implementation (CODE-002). Multiple issues exist across the
system prompt → provider parsing → agent execution → session storage chain.

## Issues

### P0 — Severe

#### 1. Tool parameter schema not sent to model

System prompt only includes tool name + description, not parameter schema.
The model has no way to know `write` expects `{"path": String, "content": String}`
or `bash` expects `{"command": String}`. It guesses parameter names from context.

**Fix**: Include `tool.parameters()` JSON Schema output in the system prompt
under each tool description, or use provider-native `tools` API field when
`ToolProtocol::Native` is active.

**Files**: `crates/talos-agent/src/prompt.rs` (ToolDescription struct + build()),
`crates/talos-core/src/tool.rs` (AgentTool::parameters())

#### 2. Dual parsing paths produce inconsistent results

Provider has `parse_text_tool_calls` (with UUID + 3-strategy JSON parser).
Session has `extract_tool_calls_from_text` from talos-core (simpler version).
Same text parsed twice, results may differ.

**Fix**: Unify to a single parser. Move `parse_text_tool_calls` to talos-core
or make session call the provider's version via a trait.

**Files**: `crates/talos-provider/src/lib.rs`, `crates/talos-core/src/message.rs`,
`crates/talos-agent/src/session.rs`

#### 3. Permission bypass for text-parsed tool calls

Text-parsed ToolCall events go through `execute_single_tool` which uses the raw
`PermissionEngine`. The `Ask` decision was patched to `Allow` as a workaround.
This means write tools (bash, write, edit) bypass TUI approval when invoked via
text-based tool calls.

**Fix**: Route text-parsed tool calls through the same permission pipeline as
native tool calls. Either:
- Send approval requests via the existing `event_tx` channel
- Or make the TUI's approval handler intercept ToolCall events

**Files**: `crates/talos-agent/src/lib.rs` (execute_single_tool),
`crates/talos-cli/src/main.rs` (TuiPermissionAwareTool)

#### 4. Agent local messages vs Session history inconsistency

Agent's `messages` vector in `run_inner` stores raw text (including tool syntax).
Session's `commit_finished_turn` stores cleaned text + tool_calls.
If compaction operates on the agent's messages, the model sees raw tool syntax
on subsequent turns.

**Fix**: Clean tool syntax from text BEFORE adding to the agent's messages
vector, not only in `commit_finished_turn`.

**Files**: `crates/talos-agent/src/lib.rs` (run_inner, after turn loop)

### P1 — Important

#### 5. Tool syntax visible in TUI during streaming

Raw `<tool_call>...</tool_call>` blocks appear in scrollback during streaming.
The `text_accumulator` in both providers forwards all text as TextDelta events
before tool call parsing happens at TurnEnd.

**Fix**: Implement streaming ToolSyntaxFilter — buffer text when `<tool_call>`
opening tag is detected, suppress until closing tag or emit as ToolCall event.

**Files**: `crates/talos-provider/src/lib.rs`, `crates/talos-provider/src/openai.rs`

#### 6. Provider parsing code duplication

Anthropic and OpenAI providers each have their own `text_accumulator` +
`parse_text_tool_calls` call logic. Same pattern, different code paths.

**Fix**: Extract to a shared `ToolCallPipeline` struct or trait in talos-provider.

**Files**: `crates/talos-provider/src/lib.rs`, `crates/talos-provider/src/openai.rs`

#### 7. No schema validation on tool inputs

`normalize_tool_input` only does basic string cleanup (trim, path sanitization).
Does not validate against the tool's parameter schema. Model can pass
`{"path": 123}` (number instead of string) and it reaches the tool.

**Fix**: Validate tool input against `AgentTool::parameters()` JSON Schema
before execution. Reject invalid inputs with a clear error message.

**Files**: `crates/talos-agent/src/lib.rs` (execute_single_tool)

#### 8. No tool call deduplication

Same tool call (same name + same args) can execute multiple times in a single
turn. The doom loop detector catches identical calls across turns, not within
a single turn batch.

**Fix**: Dedup by `(name, input.to_string())` within a single tool batch.

**Files**: `crates/talos-agent/src/lib.rs` (execute_tools)

### P2 — Nice to Have

#### 9. Redundant format instructions

`TOOL_CALLING_FORMAT` (fenced block) and `TOOL_CALLING_STRICT` (XML) both
exist. Parser still recognizes fenced block format as a fallback. With
TalosStrict as default, the fenced block parser is dead code for the primary
path.

**Fix**: Remove fenced block format when `ToolProtocol::TalosStrict` is active.
Keep as recovery only for `ToolProtocol::Compat`.

**Files**: `crates/talos-provider/src/lib.rs` (parse_text_tool_calls)

#### 10. Message::System/Context not fully wired

`workspace_context` on Agent is always `None`. The prompt builder still
concatenates context files into the system prompt string. The
`Message::System` / `Message::Context` split exists but is unused.

**Fix**: Set `workspace_context` from CLI when loading AGENTS.md.
Move context files from prompt builder to `Message::Context`.

**Files**: `crates/talos-agent/src/lib.rs`, `crates/talos-cli/src/main.rs`

## Recommended Implementation Order

```
Phase 1 (P0):
  #1 → Tool schema in system prompt
  #3 → Permission pipeline for text-parsed tools
  #4 → Clean agent messages before storage
  #2 → Unify parsing

Phase 2 (P1):
  #5 → Streaming syntax filter
  #6 → Provider code dedup
  #7 → Schema validation
  #8 → Dedup

Phase 3 (P2):
  #9 → Remove redundant formats
  #10 → Wire Message::System/Context
```

## Acceptance Criteria

### P0 — Severe (all complete)

- [x] System prompt includes parameter schema for each tool (#1)
- [x] Text-parsed tool calls go through permission pipeline (#3)
- [x] Agent messages are cleaned of tool syntax before storage (#4)
- [x] Single parsing implementation shared across providers (#2, via ToolSyntaxFilter)

### P1 — Important (partially complete)

- [x] Tool syntax is not visible in TUI during streaming (#5, ToolSyntaxFilter implemented)
- [x] Tool inputs are validated against schema before execution (#7, `registry.validate_input()` called before `tool.execute()`)
- [x] Duplicate tool calls within a turn are deduplicated (#8, `HashSet<(String, String)>` in `execute_tools`)
- [ ] Provider code dedup: shared ToolCallPipeline (#6)

### P2 — Nice to Have (partially complete)

- [x] Redundant format instructions resolved (#9, Native protocol default, format files cleaned up)
- [ ] `Message::System`/`Message::Context` fully wired (#10)

### Remaining

- [ ] `cargo test --workspace` passes (excluding pre-existing compilation errors)
- [x] `cargo clippy --workspace -- -D warnings` passes

## Required Reads

- `docs/decisions/021-tool-call-protocol-architecture.md`
- `crates/talos-core/src/tool.rs` (AgentTool trait, ToolProtocol)
- `crates/talos-provider/src/lib.rs` (parse_text_tool_calls)
- `crates/talos-agent/src/lib.rs` (run_inner, execute_single_tool)
- `crates/talos-agent/src/session.rs` (commit_finished_turn)
- `prompts/tool_calling_strict.txt`
