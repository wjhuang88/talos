# ADR-017: Exploration and Library Storage Architecture

- **Status**: Accepted for direction; storage dependency choices remain gated by Spike
- **Date**: 2026-06-05
- **Iteration**: I020

## Context

Talos should eventually include a built-in exploration capability: search the web, search papers,
gather evidence, synthesize conclusions, and persist research results locally for reuse. This is
broader than a one-off search tool. It needs a local "library" that stores sources, claims,
summaries, decisions, unresolved questions, and provenance.

The user also asked whether embedded vector databases or graph databases should be considered.
Current options include SQLite vector extensions, LanceDB, and embedded graph databases such as
Kuzu. These are promising, but several introduce native C/C++ dependencies or larger storage
surfaces that conflict with Talos' self-contained-first and ADR-gated dependency model.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Research artifacts must persist locally with provenance | Hard | User decision point | No |
| Network and paper search are tools and must respect permission/policy boundaries | Hard | AGENTS.md tool constraints | No |
| Library writes are persistent storage behavior | Hard | ADR-002 / ADR-008 | Only by ADR |
| No external database server for first slice | Hard | Self-contained-first | No |
| Vector/graph dependencies require dependency and build review | Hard | AGENTS.md hard constraint #1 | No |
| Exploration conclusions must remain traceable to sources | Hard | Research quality | No |

## Reasoning

The library needs both document retrieval and relationship traversal:

- Source/document search benefits from FTS and eventually vectors.
- Claim graphs need relationships: source supports claim, claim contradicts claim, synthesis uses
  source, question remains unresolved.

However, adopting a graph or vector database first would prematurely lock Talos into a dependency
surface before the data model is proven. Existing bundled SQLite can store the first version:
documents, chunks, claims, edges, synthesis artifacts, and FTS indexes. This preserves
self-contained behavior and keeps the first storage migration small.

Vector and graph stores should be treated as accelerators:

| Candidate | Useful for | Risk for Talos first slice | Direction |
| --- | --- | --- | --- |
| SQLite + FTS5 + relation tables | Source cards, claims, edges, keyword search | May need custom ranking; no ANN vector search | **First slice** |
| SQLite `vec1` / vector extension | Single-file ANN next to SQLite data | C extension, SIMD/build portability, extension loading policy | Spike only |
| LanceDB | Embedded vector search, metadata, full-text/SQL support, Rust SDK | Larger dependency/storage model; not same SQLite source of truth | Spike as optional vector index |
| Kuzu | Embedded property graph and Cypher | Rust crate binds/compiles C++ Kuzu library | Reject for first slice; future ADR only |
| Qdrant/Milvus/etc. server | Scalable vector search | External service, not self-contained | Reject for local first slice |

## Decision

Talos exploration will be designed as a first-class runtime capability with a local library store.

First-slice library schema concepts:

- `sources`: URL, title, authors, publication date, fetched_at, license/access notes, content hash.
- `source_chunks`: source_id, chunk ordinal, text, token estimate, FTS body.
- `claims`: normalized claim text, confidence, status, freshness, owning synthesis.
- `claim_edges`: supports, contradicts, refines, depends_on, derived_from.
- `research_runs`: query, plan, tools used, timestamps, model/provider, status.
- `syntheses`: conclusion, caveats, cited source IDs, unresolved questions.

Exploration pipeline:

1. Plan research question and source strategy.
2. Search web/papers through permission-aware tools.
3. Fetch and normalize source cards.
4. Extract claims with citations.
5. Synthesize conclusions with caveats and unresolved questions.
6. Store run artifacts in the local library.
7. Consolidate high-confidence findings into semantic memory only after review/confidence checks.

Storage rules:

- First implementation uses bundled SQLite plus FTS5 and typed relation tables.
- Vector search is an optional index behind a trait and may be added only after a Spike compares
  SQLite vector extension and LanceDB under Talos' build/self-contained constraints.
- Graph database adoption is deferred. Model graph relationships in SQLite first; use a graph DB
  only if query complexity proves the need.
- No external database server is allowed in the first local library slice.

## Reversal Trigger

Revisit if library graph traversal becomes unmaintainable in SQLite, if vector retrieval quality
cannot meet acceptance targets with FTS/hybrid ranking, or if a mature pure-Rust embedded
graph-vector store satisfies Talos' dependency constraints.

## References

- SQLite Vec1: <https://sqlite.org/vec1>
- LanceDB docs: <https://docs.lancedb.com/>
- LanceDB Rust crate: <https://docs.rs/lancedb>
- Kuzu Rust crate: <https://docs.rs/kuzu>

