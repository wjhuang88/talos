# DATA-002: Storage Topology And Runtime Ownership

| Field | Value |
|---|---|
| Story ID | DATA-002 |
| Source Issue | #141 |
| Status | Intake |
| Priority | P0 |
| Type | Architecture / Reliability / Storage Safety |

## Disposition

Register the storage-safety architecture request for explicit refinement. The owner must define
supported filesystem topology, fail-closed behavior for unsupported/unknown mounts, and exclusive
runtime/Session ownership before implementation is authorized. This record does not authorize a
SQLite, shutdown, or sandbox implementation.

## Required follow-up

- ADR-backed storage-topology policy covering WAL-backed authoritative stores and rebuildable indexes.
- Cross-process/runtime ownership and handoff contract.
- Fail-safe orphan reconciliation and corruption evidence preservation.
- Runnable iteration with filesystem, ownership, shutdown, and recovery tests.

## Dependencies

Coordinate with #49 bounded shutdown, #136 cleanup diagnostics, existing DATA-001 storage hygiene,
and SESSION-005/SESSION-009. Keep this scope separate from R02 CLI/TUI decomposition.
