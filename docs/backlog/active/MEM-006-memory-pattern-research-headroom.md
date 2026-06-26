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

1. **RecencyBoostRanker** — Re-rank retrieval candidates by `cosine × exp(-age_days / decay_days)`. Pure math, ~30 lines Rust. Our FTS5 ranking lacks temporal decay; stale high-FTS-score memories always win over fresh ones. **HIGH value, trivial effort.**

2. **Multi-Source MemoryQuery** — Construct retrieval query from user message + last N tool outputs + last K assistant turns, not just the user message. Tool outputs are the strongest retrieval signal in coding sessions. **HIGH value, trivial effort.**

3. **MemoryInjectionBudget struct** — Explicit `max_tokens` / `max_entries` / `min_similarity` budget applied at injection boundary. Our `MemoryPromptConfig` has `max_items`/`max_chars` but no similarity floor. **MEDIUM value, trivial effort.**

4. **Per-Project Storage Router** — Physical SQLite DB isolation per workspace. Fails closed (no memory) when project is unresolvable. Prevents cross-workspace memory bleed. **MEDIUM value, moderate effort.**

5. **TrafficLearner** — Zero-config, zero-latency rule-based pattern extraction from agent traffic (error→recovery, preferences, environment facts). Requires minimum evidence count (5) before saving. **LOW-MEDIUM value, significant effort (~1300 LoC port).**

### Non-goals

- No vector search / embedding model dependency (our ADR-017 defers this to a Spike).
- No compression proxy (headroom's primary product is context compression, not memory).
- No cross-agent memory sharing (Talos is single-agent).
- No Qdrant/Neo4j backend.

## Acceptance

- [ ] RecencyBoostRanker benchmarked against current FTS5-only ranking with before/after retrieval quality comparison.
- [ ] Multi-Source MemoryQuery prototyped and tested with real session transcripts.
- [ ] Decision recorded: which patterns to adopt, which to defer, which to reject.

## Dependencies

- MEM-001 (Layered Memory Foundation) — complete via I050-I053.
- ADR-016 (memory architecture).

## Required Reads

- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/iterations/I051-bounded-memory-prompt-injection.md`
- `crates/talos-memory/src/lib.rs`
- [headroom memory source](https://github.com/headroomlabs-ai/headroom/tree/main/headroom/memory)
