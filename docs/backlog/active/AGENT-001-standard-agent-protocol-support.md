# AGENT-001: Standard Agent Protocol Support

**Status**: Planned
**Priority**: P2
**Source**: User request 2026-06-18
**Depends on**: Provider/config schema stability, permission pipeline stability, session protocol stability

## Problem

Talos currently uses its own configuration directory and internal runtime protocols. As the Agent
ecosystem matures, users will expect Talos to interoperate with common Agent protocol conventions
and shared configuration locations such as `~/.agent`. Without a compatibility layer, Talos risks
becoming harder to compose with other Agent tools, launchers, IDE integrations, and protocol-aware
orchestration surfaces.

## Scope

Design and implement support for comparatively standard, common Agent protocol conventions:

- Discover and read shared Agent configuration from `~/.agent` where a stable convention exists.
- Keep `~/.talos` as the Talos-owned configuration and state directory.
- Define deterministic precedence between Talos config, shared Agent config, environment variables,
  and workspace-local overrides.
- Add an adapter layer for external Agent protocol concepts instead of leaking them into core
  runtime types.
- Support migration/import commands before considering direct write-back to shared config.
- Document which protocol/config surfaces are supported, which are read-only, and which are
  intentionally unsupported.

## Out of Scope

- Replacing `~/.talos` as the source of Talos-specific state.
- Treating unstable third-party conventions as hard dependencies.
- Writing secrets into shared config without explicit user approval.
- Implementing remote multi-agent orchestration under this story; that remains adjacent to
  REMOTE-001.
- Implementing WASM runtime plugins; that remains PLUGIN-001.

## Acceptance Criteria

- [ ] A proposal or ADR identifies the target Agent protocol/config conventions and their stability.
- [ ] Talos can read a supported shared Agent config location, including `~/.agent` if the target
      convention requires it.
- [ ] Talos config precedence is documented and tested.
- [ ] Shared config import/migration does not overwrite user files without approval.
- [ ] Secrets remain env-var based or permission-gated; no secret is logged or echoed.
- [ ] External protocol/config DTOs are isolated from Talos core runtime types.
- [ ] User-facing documentation explains how Talos interoperates with shared Agent config.

## Verification Notes

Before implementation, perform a current ecosystem survey. Candidate surfaces may include shared
Agent config directories, editor/launcher Agent protocol conventions, and protocol schemas used by
other local coding agents. Treat findings as time-sensitive and cite source dates in the ADR.

## Required Reads

- `docs/proposals/standard-agent-protocol-support.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/021-tool-call-protocol-architecture.md`
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
