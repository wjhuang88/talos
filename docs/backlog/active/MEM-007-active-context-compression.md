# MEM-007: Active Context Compression

**Status**: Research
**Priority**: P2
**Source**: User request 2026-06-26; Headroom compression analysis (MEM-006 Pattern 6)
**Iteration**: None yet

## Problem

Talos's context management is purely **reactive**: the 5-layer Compactor (budget → trim →
microcompact → collapse → autocompact) only fires when context exceeds a threshold, and only
operates on messages already in history. A `read` of a 500-line file enters history at full
size and stays there until Layer 2 (trim) discards it entirely.

Headroom's primary product is **proactive compression**: tool outputs, code blocks, and logs
are compressed 60-95% *before* they enter the model context. This delays compaction triggers,
preserves more useful history, and reduces per-turn token cost.

We need to evaluate whether to add active (pre-entry) compression and how to do it without
breaking prompt cache stability.

## Hard Constraint: Prompt Cache / KV Cache Preservation

**This is a gate, not a soft guideline. Any compression strategy that invalidates the prompt
cache prefix is rejected.**

### Why this matters

Providers (Anthropic, OpenAI) cache the prompt prefix — typically the system prompt plus early
conversation turns. A cache hit saves latency and cost (Anthropic charges ~10% for cached input).
Cache invalidation forces full re-processing of the entire prefix.

ARCH-006 (Prompt Cache Stability) already established the `build_stable_prefix()` /
`build_dynamic_suffix()` split: the stable prefix (identity + tools + skills) is computed once
and frozen across turns. The dynamic suffix (context + memory + recent turns) changes per turn.

### Cache-safe compression rules

1. **Stable prefix is immutable.** Compression never touches the stable prefix. The system
   prompt, tool definitions, and skill metadata remain byte-identical across all turns.

2. **Compression applies only to new tool results entering the dynamic suffix.** A tool result
   is compressed once, at the moment it enters history. It is never retroactively re-compressed
   in a way that changes its byte content — the compressed form is the canonical form from the
   moment of insertion.

3. **Compression is deterministic.** The same input tool output always produces the same
   compressed bytes. No timestamps, no random ordering, no session-dependent state in the
   compressed representation. Non-deterministic compression would change the cache key on
   every request even for identical logical content.

4. **Compaction layers must not rewrite messages inside the cached prefix boundary.** When
   Layers 1-5 compact old history, they must only modify messages in the dynamic suffix region.
   Messages that have fallen within the cached prefix (older turns that the provider already
   cached) are not eligible for retroactive compression. This is already MEM-005's design, but
   active compression makes it more important because compressed messages have different byte
   content than uncompressed ones.

5. **Cache-break detection.** The implementation must include a test that proves the stable
   prefix hash does not change when active compression is enabled vs disabled. A regression
   here is a P0 bug.

## Scope

Evaluate and prototype deterministic active compression for tool outputs.

### Compression strategies to evaluate

1. **Tool-specific output compression** (deterministic, per-tool rules):

   | Tool | Current behavior | Compression strategy |
   |---|---|---|
   | `read` | Returns full file content (up to limit) | If agent used offset/limit, preserve as-is. Otherwise, if triggered by `grep`/`find_symbol`, extract ±N lines around matched region. If standalone read of large file, return structure summary (function/class headers, line counts) + offer offset reads. |
   | `grep` | Returns all matches up to max | Top-N matches (configurable, default 20) + total count + "use offset for more" |
   | `git_diff` | Returns full unified diff | Hunk headers + statistics + top-K largest hunks; full diff stored in raw log |
   | `bash` | Returns full stdout/stderr | Last N lines + truncation marker; full output stored in raw log |
   | `list_symbols` | Returns all symbols in a file | Grouped by type (fn/struct/impl) with counts; detail on request |

2. **Cross-turn dedup** (deterministic):
   - Same file `read` multiple times → keep only latest full content; earlier reads become
     `"see turn N for {filename}"` reference markers.
   - Same `grep` query repeated → merge results, dedup by file:line.

3. **Structured format compaction** (deterministic):
   - JSON tool results → compact key-value format (remove whitespace, shorten keys)
   - Verbose status output → one-line summary

### Raw output preservation

Compressed tool results are the **model-facing** representation. The full uncompressed output
must be preserved for:
- `/export` transcript export
- Debug/diagnostics
- Potential re-expansion if the agent explicitly requests full content via a follow-up tool call

Storage: raw outputs are written to the session's JSONL log (already the durable source of
truth). The in-memory `Message::Tool` carries the compressed form.

### What compression is NOT

- Not an ML model (Headroom's kompress ONNX model is deferred — ~86MB, latency, feature gate).
- Not a proxy architecture (Talos is the agent, not a middleware between agent and provider).
- Not retroactive (messages already in history are not re-compressed mid-session).
- Not a replacement for compaction (Layers 1-5 still needed for long sessions; compression
  delays but does not eliminate the need).

## Relationship To Existing Requirements

| Requirement | Relationship |
|---|---|
| **MEM-005** (Compaction Policy) | Active compression reduces per-turn token pressure → compaction triggers fire later → less aggressive trimming needed. MEM-005's boundary-aware trigger still decides WHEN to compact; MEM-007 reduces HOW OFTEN. |
| **MEM-003** (LLM Layers 4-5) | If deterministic compression is effective enough, the urgency of LLM-based summarization (Layers 4-5) decreases. MEM-003 remains valid for extreme long sessions but is no longer the only defense. |
| **MEM-006** (Headroom Research) | Pattern 6 in MEM-006 evaluates Headroom's compression approach. MEM-007 is the actionable story that may result from that evaluation. |
| **ARCH-006** (Prompt Cache Stability) | Hard constraint source. MEM-007 must not break the stable prefix boundary. |
| **TOOL-002** (Tool Calling Architecture) | Compression hook lives in the tool execution pipeline (`tool_execution.rs`), after tool returns and before result enters history. |

## Architecture Sketch

```
Agent turn loop (run_inner):
  ...
  Tool executes → ToolResult (full output)
       │
       ├──→ Raw output → Session JSONL log (durable, uncompressed)
       │
       └──→ Active Compressor (deterministic, per-tool strategy)
                │
                ├──→ Compressed ToolResult → Message::Tool (model-facing)
                │
                └──→ Compression metadata (original_size, compressed_size, strategy)
                         → attached to message for observability

  [Stable prefix: NEVER touched by compression]
  [Dynamic suffix: compressed tool results live here]
  [Compaction Layers 1-5: operate on dynamic suffix only]
```

## Acceptance

- [ ] Tool output compression prototyped for `read`, `grep`, `git_diff`, `bash` with
      deterministic strategies and per-tool rules.
- [ ] Cross-turn dedup prototyped for repeated `read` and `grep` calls.
- [ ] Raw output preservation verified: `/export` produces full uncompressed transcript.
- [ ] **Cache stability test**: stable prefix hash is identical with compression enabled
      vs disabled. No regression.
- [ ] **Determinism test**: same tool output + same compression config → byte-identical
      compressed result across runs.
- [ ] Token savings measured: before/after comparison on representative agent sessions
      (coding task, exploration task, debugging task).
- [ ] Compaction delay measured: how many more turns before Layer 1 (budget) triggers
      with compression active.
- [ ] Decision recorded: which strategies to adopt, which to defer, interaction with
      MEM-005 trigger policy.

## Non-goals

- No ML-based compression model (ONNX/sentence-transformers).
- No proxy architecture (Talos is not a middleware).
- No compression of the stable prefix (system prompt, tools, skills).
- No retroactive compression of messages already in cached history.
- No new native dependencies.

## Dependencies

- MEM-005 (Compaction Policy) — compression interacts with trigger thresholds.
- ARCH-006 (Prompt Cache Stability) — hard constraint source.
- ADR-006 (event architecture boundary — no global bus for compression events).
- TOOL-002 (tool calling architecture — compression hook location).

## Required Reads

- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/MEM-003-llm-compaction.md`
- `docs/backlog/active/MEM-006-memory-pattern-research-headroom.md` (Pattern 6)
- `docs/backlog/active/ARCH-006-prompt-cache-stability.md`
- `crates/talos-agent/src/compaction.rs`
- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-agent/src/prompt.rs` (`build_stable_prefix` / `build_dynamic_suffix`)
- [headroom SmartCrusher source](https://github.com/headroomlabs-ai/headroom/tree/main/headroom/compression)
