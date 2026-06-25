# I019: Layered Memory Foundation

**User can**: Trust that future memory work follows a four-layer architecture rather than an
ad-hoc vector store or prompt-stuffing mechanism.

## Status: Planned (prerequisites cleared 2026-06-25)

All known prerequisites are now satisfied as of I047:

- I024/MEM-002 working + episodic memory wiring is complete.
- I018/OBS-001 bounded logs and embedded prompt assets completed in I047 (log rotation by I045,
  prompt assets by I047).
- `MEM-001-A` in I047 starts the executable memory foundation but does not claim full I019
  completion. Full I019 activation remains a later decision (T10 in the long-running task).

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
