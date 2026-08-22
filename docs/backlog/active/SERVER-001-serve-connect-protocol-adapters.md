# SERVER-001: Serve/Connect Protocol Adapter Architecture

| Field | Value |
|---|---|
| Story ID | SERVER-001 |
| Source Issue | #142 |
| Status | Intake |
| Priority | P1 |
| Type | Architecture / Integration Epic |

## Disposition

Register the request for explicit refinement of `talos serve` and `talos connect`. The owner must
define one authoritative runtime/session/permission/persistence path and protocol adapter boundaries
before implementation is authorized. This record does not authorize a second runtime or protocol
implementation.

The 2026-08-22 server-direction intake is decomposed into separately unclaimed owners rather than
being implemented inside this parent:

- [SERVER-002](SERVER-002-remote-relational-persistence-profile.md) / Issue #360 owns remote relational
  persistence for server deployments;
- [SERVER-001-C](SERVER-001-C-standalone-server-host-composition.md) / Issue #361 owns the standalone
  server host/composition boundary using existing Talos runtime authorities;
- [TOOL-027](TOOL-027-s3-object-workspace-backend.md) / Issue #362 owns optional S3-compatible object
  workspace tools when no local filesystem is available.

These registrations do not authorize implementation and do not make either remote SQL or S3 a
mandatory property of every future server deployment.

## Required follow-up

- ADR and dependency analysis for session ownership, lifecycle, and adapter contracts.
- Explicit serve/connect process and readiness semantics.
- Reuse of existing ACP, MCP, AG-UI, task, permission, and shutdown authorities without duplication.
- Runnable iteration with protocol conformance and multi-client lifecycle tests.
- Coordinate SERVER-001-C with SERVER-002 and TOOL-027 through capability composition rather than
  duplicating storage/tool logic inside the host.

## Dependencies

Coordinate with #46 SESSION-009, #47 ACP-001, #49 runtime shutdown, #52 PERM-006, and DATA-002
(#141). Keep this scope separate from R02 CLI/TUI decomposition.

SERVER-002, SERVER-001-C and TOOL-027 remain Intake / Unclaimed and require their own refinement,
selected iterations and effective Collaboration Claims before production changes.
