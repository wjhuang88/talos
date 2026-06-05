# RES-001: Exploration Library

## Outcome

Talos can persist research runs, sources, claims, syntheses, caveats, and unresolved questions
locally with provenance.

## Status

Planned. Selected into I020.

## Priority

P2.

## Required Reads

- `docs/iterations/I020-exploration-library.md`
- `docs/decisions/017-exploration-library-storage.md`
- `docs/decisions/008-sqlite-bundled-storage.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`

## Acceptance Criteria

- [ ] SQLite schema stores research runs, sources, chunks, claims, claim edges, and syntheses.
- [ ] Source chunks are searchable through FTS5 without an external service.
- [ ] Conclusions cite source IDs and distinguish evidence from inference.
- [ ] Vector/graph stores are evaluated by Spike before any dependency lands.
- [ ] Network/paper search tools remain permission-aware and can be disabled.

## Residual Work Destination

Adopting LanceDB, SQLite vector extensions, Kuzu, or another store requires Spike evidence and a
follow-up ADR.

