# MEM-001: Layered Memory Foundation

## Outcome

Talos memory is modeled as working, episodic, semantic, and procedural layers with explicit
consolidation and provenance. Retrieval uses multi-signal fusion; ingestion is ADD-only.

## Status

Planned. Selected into I019. **I024 (Working Memory + Episodic Memory wiring) is a prerequisite** —
Semantic Memory consolidation builds on top of persisted episode history.

## Priority

P2. I024 (MEM-002) is P0 and must land first.

## Required Reads

- `docs/iterations/I019-layered-memory-foundation.md`
- `docs/decisions/016-layered-memory-architecture.md` (including Comparative Analysis section)
- `docs/decisions/002-local-storage-architecture.md`
- `docs/decisions/008-sqlite-bundled-storage.md`
- `docs/backlog/active/OBS-001-observability-prompt-assets.md`

## Design Refinements (from mem0 V3 comparative analysis, 2026-06-23)

### ADD-only consolidation

New semantic/procedural memories are always appended. When the same `key` has conflicting entries,
retrieval ranks by `confidence × recency × evidence_count` and returns the best match. The existing
`content_hash` dedup prevents exact duplicates only; semantic duplicates with different evidence
are preserved. Rationale: mem0 V3 demonstrated +42% temporal reasoning gain by preserving
time-ordered facts instead of overwriting.

### Multi-signal retrieval

Even without vector search, retrieval fuses:

- **FTS5 relevance**: existing full-text search score
- **Recency**: `exp(-days_since_last_reinforced / 30)` — recently reinforced memories rank higher
- **Evidence strength**: `confidence × log(1 + evidence_count)` — well-supported memories rank higher
- **Entity overlap** (when entity linking is implemented): count of shared entities between query
  and memory

`final_score = fts × w1 + recency × w2 + evidence × w3 + entity × w4`

Default weights: w1=1.0, w2=0.3, w3=0.5, w4=0.5 (tunable via config).

### Memory decay

Pattern gains `last_accessed: DateTime` field. Search-time multiplier:
`decay = 1.0 + 0.5 × exp(-days_since_last_access / 7)`. Recently accessed memories get up to 1.5×
boost. No data deletion — ranking adjustment only.

### Entity linking (no external NLP)

Entity extraction uses existing Talos infrastructure — no spaCy, no Python:

- **Code entities**: tree-sitter (arborium, already integrated) extracts function/type/file names
  from tool call arguments (`find_symbol`, `read`, `edit` inputs)
- **Concept entities**: the LLM extraction prompt (already called for fact extraction during
  consolidation) extracts proper nouns, library names, and API names in the same pass — zero
  additional LLM cost
- **Pattern entities**: regex for file paths (`[\w/]+\.\w+`), URL hosts, capitalized terms

Entity store schema:

```sql
CREATE TABLE entities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,  -- 'code' | 'concept' | 'file' | 'url'
    created_at TEXT NOT NULL
);

CREATE TABLE memory_entities (
    memory_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    PRIMARY KEY (memory_id, entity_id)
);
```

Retrieval boost: `entity_score = |query_entities ∩ memory_entities| × 0.5`.

## Acceptance Criteria

- [ ] Memory layers are distinct in types/modules and documentation.
- [ ] Raw session JSONL remains the source of truth for episodes.
- [ ] Semantic/procedural memory records link back to evidence.
- [ ] ADD-only consolidation: same-key conflicts preserved, not overwritten.
- [ ] Retrieval uses multi-signal fusion (FTS5 + recency + evidence at minimum).
- [ ] Pattern has `last_accessed` field with search-time decay multiplier.
- [ ] Entity linking: code entities from tool calls + concept entities from LLM extraction.
- [ ] Retrieval is bounded and includes provenance, confidence, freshness, and contradiction metadata.
- [ ] No vector or graph DB dependency is added.

## Residual Work Destination

Vector/graph acceleration and exploration library integration stay in RES-001/I020.

