# MEM-006: Memory System Pattern Research (Headroom)

**Status**: Research
**Priority**: P3
**Source**: User request 2026-06-26; analysis of [headroomlabs-ai/headroom](https://github.com/headroomlabs-ai/headroom)
**Iteration**: None yet

## Problem

Our memory system (I019/I050-I053) uses SQLite/FTS5 with ADD-only consolidation and bounded prompt injection. Headroom (Python + Rust core) implements a more sophisticated memory pipeline with vector search, temporal decay, multi-source retrieval, and implicit traffic learning. We need to assess which patterns are worth borrowing.

## Scope

Research and prototype-evaluate 5 borrowable patterns from headroom's memory system.

### Patterns to evaluate

1. **RecencyBoostRanker** — Narrowed by ADR-046. Current Talos retrieval already includes a
   recency component, so the earlier “lacks temporal decay” premise is stale. Recency may still be
   evaluated for explicit freshness/same-key version resolution, but it is rejected as general
   admission or retention importance. **NARROW EXPERIMENT ONLY.**

2. **Multi-Source MemoryQuery** — Construct retrieval query from user message + last N tool outputs + last K assistant turns, not just the user message. Tool outputs are the strongest retrieval signal in coding sessions. **HIGH value, trivial effort.**

3. **MemoryInjectionBudget struct** — Explicit `max_tokens` / `max_entries` / `min_similarity` budget applied at injection boundary. Our `MemoryPromptConfig` has `max_items`/`max_chars` but no similarity floor. **MEDIUM value, trivial effort.**

4. **Per-Project Storage Router** — Physical SQLite DB isolation per workspace. Fails closed (no memory) when project is unresolvable. Prevents cross-workspace memory bleed. **MEDIUM value, moderate effort.**

5. **TrafficLearner** — Zero-config, zero-latency rule-based pattern extraction from agent traffic (error→recovery, preferences, environment facts). Requires minimum evidence count (5) before saving. **LOW-MEDIUM value, significant effort (~1300 LoC port).**

6. **Tool Output Compression (SmartCrusher/CodeCompressor)** — Headroom's primary product compresses tool outputs, code blocks, and logs by 60-95% before they enter the model context. Our Compactor only handles retroactive history trimming (Layers 1-5); we have no pre-entry compression. This is evaluated separately in **MEM-007** (Active Context Compression). Headroom's deterministic strategies (field extraction, signature+key-line retention, dedup) are borrowable without the ML model dependency. **Connection to MEM-005/MEM-003**: effective pre-entry compression reduces compaction trigger frequency and may lower the urgency of MEM-003 LLM layers 4-5.

### Non-goals

- No vector search / embedding model dependency (our ADR-017 defers this to a Spike).
- No ML compression proxy (Headroom's kompress ONNX model is ~86MB; deferred to optional future).
- No cross-agent memory sharing (Talos is single-agent).
- No Qdrant/Neo4j backend.

## Acceptance

- [ ] RecencyBoostRanker benchmarked against current FTS5-only ranking with before/after retrieval quality comparison.
- [ ] Multi-Source MemoryQuery prototyped and tested with real session transcripts.
- [ ] Decision recorded: which patterns to adopt, which to defer, which to reject.

## Dependencies

- MEM-001 (Layered Memory Foundation) — complete via I050-I053.
- MEM-007 (Active Context Compression) — evaluates Headroom's compression approach for Talos.
- ADR-016 (memory architecture).
- ADR-046 (surprise-selected admission; narrows the recency hypothesis).

## Required Reads

- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/iterations/I051-bounded-memory-prompt-injection.md`
- `docs/decisions/046-surprise-selected-memory-admission.md`
- `crates/talos-memory/src/lib.rs`
- [headroom memory source](https://github.com/headroomlabs-ai/headroom/tree/main/headroom/memory)
