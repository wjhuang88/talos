# I019: Layered Memory Foundation

**User can**: Trust that future memory work follows a four-layer architecture rather than an
ad-hoc vector store or prompt-stuffing mechanism.

## Status: Complete (2026-06-29) — all acceptance criteria closed via I050-I053

All known prerequisites are now satisfied as of I047:

- I024/MEM-002 working + episodic memory wiring is complete.
- I018/OBS-001 bounded logs and embedded prompt assets completed in I047 (log rotation by I045,
  prompt assets by I047).
- `MEM-001-A` in I047 starts the executable memory foundation but does not claim full I019
  completion. Full I019 activation remains a later decision (T10 in the long-running task).

## Scope

This iteration turns ADR-016 into the first executable storage and retrieval foundation.

## Selected Stories

- [x] #I019-S1: define `talos-memory` crate boundaries and memory item types.
- [x] #I019-S2: implement episodic-to-semantic consolidation schema in SQLite.
- [x] #I019-S3: implement bounded memory retrieval for prompt assembly.
- [x] #I019-S4: add contradiction/provenance metadata and tests.

## Acceptance Criteria

- [x] Working, episodic, semantic, and procedural memory are represented as distinct concepts.
- [x] Raw sessions remain source-of-truth; semantic/procedural memory links back to evidence.
- [x] Retrieval output is bounded by count/tokens and includes provenance.
- [x] Contradictory facts are recorded explicitly instead of overwriting each other.
- [x] No vector or graph DB dependency is added in this iteration.
- [x] `cargo test --workspace` passes.

Evidence: I047 MEM-001-A starter (schema + retrieval), I050 consolidation pipeline (episodic→semantic with evidence), I051 bounded prompt injection (provenance + budgets + hidden-output guard), I052 entity linking + procedural memory (entity overlap boost + permission boundary regression), I053 quality hardening (status + retention dry-run + corruption tolerance). All delivered 2026-06-26.

Post-completion refinement: MEM-009/ADR-046 replaces the future admission-policy direction without
reopening or rewriting this completed iteration baseline.

## Out of Scope

- Neural/biological fidelity claims.
- Vector ANN index adoption.
- Full autonomous research library.
