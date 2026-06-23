# ADR-016: Layered Agent Memory Architecture

- **Status**: Accepted for architecture; implementation phased
- **Date**: 2026-06-05
- **Iteration**: I019

## Context

Talos already has session JSONL, SQLite session indexes, and an evolution store. That is not yet a
complete agent memory architecture. The next stage needs to distinguish short-lived context,
episodes, durable facts, and learned procedures so Talos can improve across sessions without
turning every past event into prompt stuffing.

Research and cognitive architecture references support this separation. Neuroscience literature
describes consolidation as memories changing across brain networks over time, with hippocampal
indexing and cortical/generalized representations as useful inspiration. Cognitive architectures
such as Soar also separate working, semantic, episodic, and procedural memory.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Memory writes are persistent local state and must be auditable | Hard | AGENTS.md storage/permission constraints | No |
| Session JSONL remains source of truth for raw interaction history | Hard | ADR-002 | No |
| SQLite bundled is the approved structured local store | Hard | ADR-008 | Only by ADR |
| Memory retrieval must be bounded before prompt injection | Hard | Safety/context-budget constraint | No |
| Neuroscience analogies are design inspiration, not correctness proof | Hard | Engineering rigor | No |
| Vector or graph DB dependencies need separate dependency review | Hard | AGENTS.md / ADR-008 / self-contained-first | No |

## Reasoning

A single vector store is insufficient. It loses temporal context, makes provenance weak, and often
retrieves semantically similar but operationally wrong material. A single event log is also
insufficient because it forces every useful fact or habit to be re-derived repeatedly.

The architecture should mirror the useful parts of cognitive memory without overclaiming biology:

| Layer | Brain/cognitive analogy | Talos responsibility | First storage |
| --- | --- | --- | --- |
| Working memory | Active task context | Current turn state, active goals, selected retrieved context | In-memory + bounded prompt sections |
| Episodic memory | Context-bound events | Sessions, turns, tool calls, decisions, source snapshots | JSONL source + SQLite index |
| Semantic memory | Consolidated facts | Stable facts, entities, claims, preferences, project knowledge | SQLite tables + FTS5 |
| Procedural memory | Learned how-to behavior | Skills, patterns, playbooks, approval heuristics, remediation recipes | Versioned text assets + SQLite metadata |

The important mechanism is consolidation: selected episodes become semantic facts or procedural
patterns only after evidence, confidence, provenance, and contradiction checks.

## Decision

Talos will adopt a four-layer memory architecture:

1. **Working memory**: bounded, per-turn, never treated as durable truth by itself.
2. **Episodic memory**: append-only event history with timestamps, session/turn identity, tool
   provenance, and source references.
3. **Semantic memory**: consolidated claims/facts with provenance, confidence, freshness, and
   contradiction metadata.
4. **Procedural memory**: learned operational procedures stored as reviewable artifacts and
   activated through explicit retrieval/adaptation rules.

Core design rules:

- Raw episodes are not automatically injected into future prompts.
- Every semantic/procedural memory item must link back to evidence.
- Consolidation is an explicit background or end-of-session step, not an accidental side effect of
  retrieval.
- **ADD-only consolidation**: new semantic/procedural memories are always appended. When the same
  `key` has conflicting entries, retrieval ranks by recency + confidence + evidence_count and
  returns the best match. Old entries are preserved as fallback, not overwritten. (Refined
  2026-06-23 based on mem0 V3 comparative analysis.)
- Retrieval uses multi-signal fusion: FTS5 relevance + recency decay + evidence strength + entity
  overlap (when available). Single-signal ranking is insufficient.
- Contradictions must be first-class records, not overwritten facts.
- Memory decay is a search-time ranking signal (`last_accessed` boost), never data deletion.
- The first implementation uses existing SQLite/FTS5. Vector and graph indexes are optional
  accelerators behind interfaces and require separate Spike/ADR before dependency adoption.

## Research Notes

- Neuroscience systems-consolidation work supports the idea that memories change across circuits
  over time rather than remaining a single static trace.
- Cognitive algorithms literature continues to distinguish episodic and semantic memory, with
  episodic memory associated with context/time and semantic memory with generalized knowledge.
- Soar is a practical precedent for separating working, semantic, episodic, and procedural memory
  in an agent architecture.

## Comparative Analysis: mem0 (2026-06-23)

A comparative study of [mem0ai/mem0](https://github.com/mem0ai/mem0) V3 architecture identified
four design refinements for Talos. The analysis confirms ADR-016's four-layer direction while
suggesting concrete changes to ingestion, retrieval, and lifecycle policies.

### mem0 V3 Key Design Points

| Dimension | mem0 V3 | Talos (current) |
|---|---|---|
| Ingestion | ADD-only: single LLM extraction pass, no UPDATE/DELETE | Consolidation with content_hash dedup + contradicting_count |
| Conflict resolution | Deferred to retrieval time | During consolidation (Pattern contradicting_count) |
| Retrieval | Three-signal fusion: vector + BM25 + entity boost | FTS5 + confidence |
| Entity linking | spaCy NER → entity store → retrieval boost | None |
| Decay | Search-time recency re-ranking (recent × 1.5, stale dampened) | None |
| Audit trail | SQLite event log (ADD/UPDATE/DELETE with old/new text) | observations + patterns tables (no event log) |

### Design Refinements Adopted

1. **ADD-only ingestion**: Do not UPDATE or DELETE semantic/procedural memories during
   consolidation. New patterns are always ADDed. When the same `key` has multiple entries,
   retrieval ranks by `confidence × recency × evidence_count` and returns the best match. Old
   entries remain as fallback. Rationale: mem0 V3 demonstrated +42% temporal reasoning gain by
   preserving time-ordered facts instead of overwriting. Talos's `content_hash` dedup remains
   for exact-duplicate prevention, but semantic duplicates with different evidence are preserved.

2. **Multi-signal retrieval**: Even without vector search, retrieval should fuse multiple signals:
   - FTS5 relevance score (existing)
   - Recency: `exp(-days_since_last_reinforced / 30)`
   - Evidence strength: `confidence × log(1 + evidence_count)`
   - Entity overlap (if entity linking is implemented): `linked_entities ∩ query_entities`
   
   `final_score = fts × w1 + recency × w2 + evidence × w3 + entity × w4`

3. **Memory decay**: Add `last_accessed` timestamp to Pattern. Search-time multiplier:
   `decay = 1.0 + 0.5 × exp(-days_since_last_access / 7)`. Memories accessed within 7 days get
   up to 1.5× boost. No data deletion — only ranking adjustment.

4. **Entity linking without external NLP**: spaCy is Python-only and too heavy for Talos. Entity
   extraction uses existing infrastructure:
   - **Code entities**: tree-sitter (arborium) extracts function/type/file names from tool call
     arguments
   - **Concept entities**: the existing LLM extraction prompt (already called for fact extraction)
     extracts proper nouns, library names, and API names in the same pass — zero additional cost
   - **Pattern entities**: simple regex for file paths (`[\w/]+\.\w+`), URLs, and capitalized terms
   - Entity store: `entities(id, name, kind)` + `memory_entities(memory_id, entity_id)` in SQLite
   - Retrieval boost: `score += entity_overlap × 0.5`

### Where Talos is Already Stronger Than mem0

- **Contradiction handling**: Talos has first-class `Conflict` records. mem0 V3 has none (ADD-only
  means contradictions accumulate unmanaged).
- **Layer separation**: Working / Episodic / Semantic / Procedural vs mem0's flat model.
- **Self-contained**: Pure Rust + SQLite/FTS5, no external services. mem0 requires vector DB +
  LLM API for every operation.
- **Provenance**: Every semantic/procedural item links to evidence. mem0 has weaker provenance.

### What Talos Explicitly Rejects from mem0

- **Graph memory**: mem0 V2 had graph memory; V3 replaced it with entity linking. Talos also
  rejects graph DB (ADR-016 constraint). Entity linking in SQLite is the right level.
- **External vector store as hard dependency**: mem0 requires a vector DB. Talos keeps vector
  search optional behind an interface, gated by separate ADR.
- **LLM call for every add()**: mem0 calls the LLM on every `add()`. Talos's consolidation is
  batch (end-of-turn or end-of-session), not per-message, to control cost and latency.

## Reversal Trigger

Revisit if benchmarks show SQLite/FTS cannot support expected memory retrieval, or if a pure-Rust
embedded vector/graph store becomes mature enough to replace part of the first storage design with
less risk than maintaining our own hybrid schema.

## References

- Nature Reviews Neuroscience: <https://www.nature.com/articles/s41583-018-0031-2>
- Cognitive algorithms and systems of episodic memory, semantic memory and their learnings:
  <https://arxiv.org/abs/2602.07261>
- Soar semantic memory manual: <https://soar.eecs.umich.edu/soar_manual/06_SemanticMemory/>
- Soar episodic memory manual: <https://soar.eecs.umich.edu/soar_manual/07_EpisodicMemory/>
- mem0 V3 architecture (comparative analysis, 2026-06-23):
  <https://github.com/mem0ai/mem0> — ADD-only ingestion, three-signal retrieval, entity linking

