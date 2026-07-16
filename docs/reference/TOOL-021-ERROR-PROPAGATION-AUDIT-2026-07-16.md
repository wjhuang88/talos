# TOOL-021: End-to-End Tool Error Propagation Audit — Report

**Date**: 2026-07-16
**Iteration**: I131 / P120
**Auditor**: glm-5.2 (unattended)
**Status**: Review — FINDING-2 confirmed as data loss via integration test; follow-up owner story created

## Methodology

Direct code trace of every path from tool execution result to provider API request, covering:
`talos-tools` → `talos-agent/tool_execution.rs` → `talos-agent/lib.rs` → `talos-core/message.rs` → `talos-agent/compaction/engine.rs` → `talos-provider/openai_request.rs` + `anthropic_request.rs`

## Trace Summary

### 1. Tool Execution → Result (`is_error` determination)

| Source | File:Line | Logic |
|---|---|---|
| Bash tool | `talos-tools/src/bash_tool.rs:238` | `is_error = if is_expected_exit_code(cmd, code) { false } else { !success }` |
| Other tools | `talos-agent/src/tool_execution.rs:502` | `is_error: result.exit_code != 0` |
| `is_expected_exit_code` | `talos-tools/src/bash_tool.rs:563` | Returns true for exit 0 always; exit 1 for grep/rg/diff/cargo-fmt-check; false otherwise |

**Key**: `grep` returning no matches (exit 1) is NOT an error. The model sees normal output.

### 2. Result → Message (dual-path: UI vs LLM)

**File**: `talos-agent/src/lib.rs:772-804`, `tool_execution.rs:222-246`

```
ToolResult { content, is_error }
  ├── ui_result = MessageToolResult { content: raw, is_error }     → AgentEvent::ToolResult (UI display)
  └── llm_result =
        ├── if is_error: MessageToolResult { content: raw + "\n\n[Analyze the error above...]", is_error: true }
        ├── if bash && compression: MessageToolResult { content: compressed, is_error }
        └── else: MessageToolResult { content: raw, is_error }
                                                                        → messages.push(Message::Tool { result: llm_result })
```

**Key**: Error results get a guidance suffix for the LLM. The `is_error` flag is always preserved.

### 3. Compaction (3 layers — all preserve `is_error`)

**File**: `talos-agent/src/compaction/engine.rs:184-288`

| Layer | Function | Effect on content | Effect on `is_error` |
|---|---|---|---|
| Budget | `apply_budget` (184) | Truncate if > MAX_TOOL_RESULT_CHARS + suffix | **Preserved** (`..result`) |
| Trim | `apply_trim` (212) | Empty string for turns > TRIM_TURN_THRESHOLD | **Preserved** (`..result`) |
| Microcompact | `apply_microcompact` (258) | Empty earlier duplicates by tool_use_id | **Preserved** (`..result`) |

### 4. Provider Serialization

#### OpenAI (`openai_request.rs:158-177`)

```json
{
  "role": "tool",
  "content": "Error: <content>"   // if is_error
  "content": "<content>"           // if not error
  "tool_call_id": "<id>"
}
```

- Error prefix: `"Error: "` prepended to content (line 166-167)
- Empty content: replaced with `EMPTY_TOOL_RESULT_MESSAGE` (line 169)
- **Orphan results**: explicitly dropped with `tracing::warn!` (line 159-164)

#### Anthropic (`anthropic_request.rs:117-129`)

```json
{
  "role": "user",
  "content": [{
    "type": "tool_result",
    "tool_use_id": "<id>",
    "content": "<content>",
    "is_error": true              // only if is_error
  }]
}
```

- Native `is_error` flag (line 123-124)
- Content sent as-is
- **Orphan results**: sent to API as-is — no filtering (line 51-133, only System filtered)

### 5. Retry/Resume

**File**: `talos-agent/src/lib.rs:375-828`

| Scenario | Messages persisted? | Tool results preserved? |
|---|---|---|
| Normal turn success | `messages[persist_start..]` returned (line 614) | ✅ |
| MaxTokens continuation | `messages[persist_start..]` returned (line 652) | ✅ |
| Provider error | `Err(AgentError)` returned (line 434) — messages NOT in return value | ⚠️ Depends on caller |
| Tool execution error | Tool error IS stored in messages before error (line 804) | ✅ within turn; persistence depends on caller |
| Doom loop detection | Messages returned (line 609-614) | ✅ |

## Fixture Matrix

| # | Scenario | `is_error` | Content modification | OpenAI serialization | Anthropic serialization | Finding |
|---|---|---|---|---|---|---|
| F1 | Expected non-zero (grep exit 1) | `false` | None | Normal tool message | Normal tool_result | ✅ Preserved — not an error |
| F2 | Execution error (exit 2) | `true` | + "[Analyze...]" suffix | `"Error: <content>"` | `is_error: true` | ✅ Preserved with guidance |
| F3 | Paired result (call + result) | any | As above | Normal serialization | Normal serialization | ✅ Preserved |
| F4 | Orphan result (no matching call) | any | As above | **Dropped** with warning | **Sent as-is** | ⚠️ FINDING-1: provider difference |
| F5 | Retry after provider error | any | Already in messages | Depends on caller persistence | Depends on caller persistence | ⚠️ FINDING-2: caller-dependent |
| F6 | Resume after budget compaction | preserved | Content truncated | Truncated content | Truncated content | ✅ Flag preserved, content may be partial |
| F7 | Resume after trim compaction | preserved | Content emptied | Empty → `EMPTY_TOOL_RESULT_MESSAGE` | Empty content | ✅ Flag preserved, content empty |
| F8 | Resume after microcompact | preserved | Content emptied for old dupes | Empty → `EMPTY_TOOL_RESULT_MESSAGE` | Empty content | ✅ Flag preserved, content empty |
| F9 | Large successful output | `false` | Compressed (bash) or raw | Normal serialization | Normal serialization | ✅ Preserved |
| F10 | Empty error output | `true` | "[Analyze...]" suffix on empty | `"Error: \n\n[Analyze...]"` | `is_error: true`, content has guidance | ✅ Preserved |

## Findings

### FINDING-1: Orphan tool result handling differs by provider (observation, not defect)

- **OpenAI**: explicitly drops orphan tool results with `tracing::warn!` log (`openai_request.rs:159-164`). The model never sees the result.
- **Anthropic**: sends orphan results as-is (`anthropic_request.rs:51-133`). The Anthropic API may accept or reject them.
- **Impact**: Same conversation can produce different model-visible content depending on provider. This is a provider API constraint difference, not a Talos bug.
- **Recommendation**: No fix needed unless Anthropic API errors on orphan results. If it does, add matching orphan filtering to `anthropic_request.rs`. Create a follow-up story if needed.

### FINDING-2: Provider error may lose unpersisted tool results (caller-dependent)

- When a provider error occurs mid-turn, `run_inner` returns `Err(error)` without returning the `messages` vector (`lib.rs:434`).
- Tool results already pushed to `messages` (line 804) are lost from the return value.
- The session layer (`session/turn.rs`) calls `persist_turn_messages` only on turn success, not on error.
- **Impact**: If a tool executed successfully but the subsequent provider call failed, the tool result is not persisted. On retry/resume, the model doesn't see the tool result.
- **Recommendation**: Consider persisting messages even on provider error (turn partial success). Create a follow-up story for session-layer error persistence.

### DATA-LOSS RISK CONFIRMED (FINDING-2)

FINDING-2 is confirmed as actual data loss, not a conditional risk:
`talos-agent/src/session/turn.rs:188-200` is the canonical session turn path. The `Ok(Err(e))`
branch sends an error event but never calls `persist_turn_messages`. Tool results already
executed and pushed to the message vector are dropped. Integration test
`fixture_provider_error_drops_tool_results` proves this.

Other paths preserve errors correctly:
- Preserved with content and `is_error` flag (normal case)
- Preserved with truncated/empty content but intact `is_error` flag (compaction)
- Explicitly dropped with warning logging (orphan in OpenAI)
- Modified with visible annotations ("[Analyze...]", "Error: ")

But FINDING-2 represents a real gap where tool results can be silently lost on provider error.
- Preserved with content and `is_error` flag (normal case)
- Preserved with truncated/empty content but intact `is_error` flag (compaction)
- Explicitly dropped with warning logging (orphan in OpenAI)
- Modified with visible annotations ("[Analyze...]", "Error: ")

No path silently discards an error result without a trace.

## Follow-Up Owner Stories

| ID | Description | Priority |
|---|---|---|
| SESSION-006 | Session-layer: persist turn messages on provider error to avoid losing tool results | P1 |
| (conditional) | Anthropic: add orphan tool result filtering if API rejects them | P3 |
