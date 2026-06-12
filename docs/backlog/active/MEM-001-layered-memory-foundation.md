# MEM-001: Layered Memory Foundation

## Outcome

Talos memory is modeled as working, episodic, semantic, and procedural layers with explicit
consolidation and provenance.

## Status

Planned. Selected into I019. **I024 (Working Memory + Episodic Memory wiring) is a prerequisite** —
Semantic Memory consolidation builds on top of persisted episode history.

## Priority

P2. I024 (MEM-002) is P0 and must land first.

## Required Reads

- `docs/iterations/I019-layered-memory-foundation.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `docs/decisions/002-local-storage-architecture.md`
- `docs/decisions/008-sqlite-bundled-storage.md`
- `docs/backlog/active/OBS-001-observability-prompt-assets.md`

## Acceptance Criteria

- [ ] Memory layers are distinct in types/modules and documentation.
- [ ] Raw session JSONL remains the source of truth for episodes.
- [ ] Semantic/procedural memory records link back to evidence.
- [ ] Retrieval is bounded and includes provenance, confidence, freshness, and contradiction metadata.
- [ ] No vector or graph DB dependency is added in I019.

## Residual Work Destination

Vector/graph acceleration and exploration library integration stay in RES-001/I020.

