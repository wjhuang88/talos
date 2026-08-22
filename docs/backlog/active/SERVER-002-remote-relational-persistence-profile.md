# SERVER-002: Remote Relational Persistence Profile

| Field | Value |
|---|---|
| Story ID | SERVER-002 |
| Source Issue | #360 |
| Status | Intake / Unclaimed |
| Priority | P1 |
| Type | Architecture / Server Persistence Story |
| Parent | SERVER-001 |
| Selected Iteration | None |

## Disposition

Own the remote relational persistence profile required by the standalone `talos-server` direction.
The target is to let server deployments separate service/process lifetime from durable Talos state by
using a remotely managed relational database instead of treating the embedded local SQLite layout as
server-authoritative storage.

This intake records the boundary only. It does **not** authorize a database implementation, schema
migration, SQL backend dependency, server runtime, or changes to existing local CLI/TUI persistence.

## Required Outcome

Before implementation selection, refine and accept a contract that:

- identifies which current durable state belongs behind a storage abstraction and which state stays
  process/session-local;
- reuses existing persistence/session/domain logic rather than cloning behavior into `talos-server`;
- defines supported remote relational backends and compatibility expectations without creating
  backend-specific runtime truth;
- defines migration/versioning, transaction/concurrency, connection-pool, retry/deadline and
  fail-closed startup semantics;
- preserves local SQLite behavior for existing local products unless a separately governed change
  says otherwise;
- coordinates with SERVER-001 host composition, DATA-002 storage topology, SESSION-009 ownership,
  RUNTIME-005 shutdown and permission/security owners;
- proves service restart/state continuity and multi-process safety using a separately claimed,
  runnable implementation slice.

## Explicit Non-Goals

- implementing `talos serve` / `talos connect` host composition;
- using S3/object storage as relational state or transaction authority;
- redefining Session/runtime authority;
- introducing remote/LAN authentication or transport policy;
- replacing local SQLite for CLI/TUI/Desktop by default;
- production dependency/schema changes before ADR-backed refinement and an effective claim.

## Dependencies / Relationships

- Parent architecture: [SERVER-001](SERVER-001-serve-connect-protocol-adapters.md) / Issue #142.
- Storage topology: [DATA-002](DATA-002-storage-topology-and-runtime-ownership.md) / Issue #141.
- Session authority: [SESSION-009](SESSION-009-multi-client-session-architecture.md) / Issue #46.
- Runtime shutdown/lifecycle: RUNTIME-005 is Complete and remains authoritative for bounded finalization.
- S3/object workspace support is separately owned by TOOL-027 / Issue #362.

## Governance Boundary

`Ready`, `Active`, database implementation, schema/dependency changes, or a server runtime may only be
established through normal requirement refinement, ADR/owner decisions where required, a selected
iteration, and an effective Collaboration Claim. This owner is registration/architecture intake only.
