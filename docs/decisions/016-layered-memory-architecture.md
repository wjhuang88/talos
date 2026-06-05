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
- Retrieval must rank by relevance, recency/freshness, confidence, provenance quality, and task
  fit.
- Contradictions must be first-class records, not overwritten facts.
- The first implementation uses existing SQLite/FTS5. Vector and graph indexes are optional
  accelerators behind interfaces and require separate Spike/ADR before dependency adoption.

## Research Notes

- Neuroscience systems-consolidation work supports the idea that memories change across circuits
  over time rather than remaining a single static trace.
- Cognitive algorithms literature continues to distinguish episodic and semantic memory, with
  episodic memory associated with context/time and semantic memory with generalized knowledge.
- Soar is a practical precedent for separating working, semantic, episodic, and procedural memory
  in an agent architecture.

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

