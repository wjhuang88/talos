# I019: Layered Memory Foundation

**User can**: Trust that future memory work follows a four-layer architecture rather than an
ad-hoc vector store or prompt-stuffing mechanism.

## Status: Planned

## Scope

This iteration turns ADR-016 into the first executable storage and retrieval foundation.

## Selected Stories

- [ ] #I019-S1: define `talos-memory` crate boundaries and memory item types.
- [ ] #I019-S2: implement episodic-to-semantic consolidation schema in SQLite.
- [ ] #I019-S3: implement bounded memory retrieval for prompt assembly.
- [ ] #I019-S4: add contradiction/provenance metadata and tests.

## Acceptance Criteria

- [ ] Working, episodic, semantic, and procedural memory are represented as distinct concepts.
- [ ] Raw sessions remain source-of-truth; semantic/procedural memory links back to evidence.
- [ ] Retrieval output is bounded by count/tokens and includes provenance.
- [ ] Contradictory facts are recorded explicitly instead of overwriting each other.
- [ ] No vector or graph DB dependency is added in this iteration.
- [ ] `cargo test --workspace` passes.

## Out of Scope

- Neural/biological fidelity claims.
- Vector ANN index adoption.
- Full autonomous research library.

