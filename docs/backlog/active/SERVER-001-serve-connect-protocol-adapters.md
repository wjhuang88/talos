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

## Required follow-up

- ADR and dependency analysis for session ownership, lifecycle, and adapter contracts.
- Explicit serve/connect process and readiness semantics.
- Reuse of existing ACP, MCP, AG-UI, task, permission, and shutdown authorities without duplication.
- Runnable iteration with protocol conformance and multi-client lifecycle tests.

## Dependencies

Coordinate with #46 SESSION-009, #47 ACP-001, #49 runtime shutdown, #52 PERM-006, and DATA-002
(#141). Keep this scope separate from R02 CLI/TUI decomposition.
