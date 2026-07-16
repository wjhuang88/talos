# I020: Exploration Library

**User can**: Ask Talos to research a question, preserve sources and conclusions locally, and later
reuse the result with provenance.

## Status: Complete (2026-06-29) — S1-S3 delivered via I054-I055; S4 explicitly deferred by ADR-017

## Scope

This iteration implements the first local research-library slice under ADR-017.

## Selected Stories

- [x] #I020-S1: define research-run, source-card, claim, edge, and synthesis schema. (I054)
- [x] #I020-S2: implement local library storage on bundled SQLite + FTS5. (I054)
- [x] #I020-S3: implement permission-aware research artifact writer. (I055)
- [ ] #I020-S4: add vector/graph storage Spike with benchmark and dependency report. (Deferred — requires Spike + ADR per ADR-017)

## Acceptance Criteria

- [x] Research runs store query, plan, sources, extracted claims, synthesis, caveats, and unresolved questions.
- [x] Stored conclusions cite source IDs and can be traced back to source chunks.
- [x] Library search works through SQLite FTS without an external service.
- [x] Vector/graph DB adoption is not implemented without a follow-up ADR.
- [x] Network/paper search tools remain permission-aware and can be disabled.
- [x] `cargo test --workspace` passes.

Evidence: I054 delivered ExplorationStore with full schema (research_runs, sources, source_chunks/FTS5, claims, claim_edges, syntheses) + citation integrity enforcement. I055 delivered ingestion pipeline (text/fetched), deterministic claim extraction, citation-preserving synthesis, and CLI explore commands. Runtime verified: ingested README.md (92 chunks), FTS search returns snippets.

## Out of Scope

- External database servers.
- Cloud sync.
- Fully autonomous unattended web crawling.
