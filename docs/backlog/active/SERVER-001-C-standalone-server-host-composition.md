# SERVER-001-C: Standalone Server Host Composition

| Field | Value |
|---|---|
| Story ID | SERVER-001-C |
| Source Issue | #361 |
| Status | Intake / Unclaimed |
| Priority | P1 |
| Type | Architecture / Server Host Story |
| Parent | SERVER-001 |
| Selected Iteration | None |

## Disposition

Own the standalone `talos-server` host/composition boundary: reuse the existing Talos runtime,
Session, provider, tool, permission and persistence abstractions while exposing API interaction as a
peer delivery surface to TUI/Desktop rather than reimplementing Agent logic inside a server package.

This intake does **not** authorize a new runtime authority, API protocol implementation, remote bind,
permission bypass, database backend, S3 tooling, dependency addition, or production server binary.

## Required Outcome

Refinement must define a bounded server-host composition that:

- has one composition root for the existing Talos runtime/session/tool/provider authorities;
- treats API transport as an interaction adapter rather than a second Agent implementation;
- defines startup/readiness/shutdown and health semantics using existing bounded runtime lifecycle;
- selects only storage/workspace capabilities actually available in the host environment;
- keeps filesystem tools absent when no local filesystem workspace exists;
- composes remote relational persistence only through SERVER-002 after its contract is accepted;
- composes optional S3 workspace tools only through TOOL-027 rather than pretending object storage is
  a local filesystem;
- remains compatible with later SERVER-001 protocol-adapter decomposition without making one
  protocol the core runtime authority;
- preserves permission and security decisions through the canonical pipeline.

## Explicit Non-Goals

- duplicating conversation/Agent/tool execution logic;
- implementing remote relational persistence (SERVER-002 owns it);
- implementing S3/object workspace tools (TOOL-027 owns them);
- changing Session multi-client authority (SESSION-009);
- remote/LAN authentication, public exposure or tunnel policy without separate security governance;
- WebSocket/AG-UI/ACP semantics merely because a server host exists.

## Dependencies / Relationships

- Parent: [SERVER-001](SERVER-001-serve-connect-protocol-adapters.md) / Issue #142.
- Remote relational state: [SERVER-002](SERVER-002-remote-relational-persistence-profile.md) / Issue #360.
- Optional object workspace: [TOOL-027](TOOL-027-s3-object-workspace-backend.md) / Issue #362.
- Session/client architecture: SESSION-009 / Issue #46.
- Permission convergence: PERM-006; active protected work remains independently owned.
- Runtime lifecycle: RUNTIME-005 complete baseline.

## Governance Boundary

This owner is intake only. A production `talos-server` package/binary, protocol routes, dependencies,
storage wiring or runtime changes require normal refinement, a selected iteration, an effective
Collaboration Claim, exact-head validation and the applicable architecture/security reviews.
