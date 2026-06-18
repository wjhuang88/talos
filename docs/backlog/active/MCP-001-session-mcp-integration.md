# MCP-001: MCP Session Integration

**Status**: Planned
**Priority**: P1
**Source**: User correction 2026-06-18
**Depends on**: ARCH-003/004 complete, prompt cache stability, permission pipeline stability

## Problem

Talos has MCP infrastructure and MCP server/client work from earlier iterations, but MCP capability
needs to be productized as a session-level integration. A normal session should be able to discover
configured MCP tools at startup, expose them through the same tool registry and prompt surface as
native tools, route execution through the same permission/display pipeline, and preserve provenance
in the conversation/event model.

Without this, MCP remains an extension subsystem rather than a first-class session capability.

## Scope

- Load configured MCP servers/clients at session startup through the CLI composition root.
- Discover MCP tools before the first model turn where possible, so the tool list and prompt cache
  prefix remain stable for the session.
- Register MCP tools into the same `AgentTool`/tool registry path used by native tools, preserving
  typed `ToolProvenance`.
- Route MCP tool calls through the existing permission pipeline and summary/display metadata.
- Surface MCP connection/tool status in CLI/TUI diagnostics without adding a global event bus.
- Define behavior when an MCP server becomes unavailable mid-session.
- Keep MCP DTOs isolated at crate boundaries per ARCH-004.

## Acceptance Criteria

- [ ] A configured MCP tool is discoverable and visible to the model before the first turn.
- [ ] MCP tool execution flows through the same permission pipeline as native tools.
- [ ] TUI/history display distinguishes MCP provenance where provenance is available.
- [ ] MCP discovery failures are visible but do not crash sessions unless strict mode is enabled.
- [ ] Prompt cache behavior is documented for startup-discovered versus mid-session MCP tools.
- [ ] Tests cover MCP tool discovery, permission routing, unavailable server behavior, and
      provenance display metadata.
- [ ] No `rmcp` external DTOs leak into public APIs outside the MCP boundary.

## Verification Notes

Use existing MCP e2e tests as a starting point, then add session-start integration coverage. Avoid
mid-session dynamic tool mutation unless the prompt/cache behavior is explicitly designed and
tested.

## Required Reads

- `docs/iterations/I009-extensible-agent.md`
- `docs/backlog/active/ARCH-003-crate-boundary-cleanup.md`
- `docs/backlog/active/ARCH-004-anti-corruption-layers.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/009-tool-provenance.md`
- `docs/decisions/021-tool-call-protocol-architecture.md`
