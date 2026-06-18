# I034: MCP Session Integration

**Status**: Planned
**Target Window**: After I031 or parallel only after Skill startup path is stable
**Depends On**: ARCH-003/004 complete, I031 preferred

## Outcome

Make MCP a first-class session capability. Configured MCP tools should be discovered at session
startup, registered beside native tools, exposed to the model before the first turn where possible,
routed through the same permission/display pipeline, and shown with provenance/status in the
conversation surfaces.

## Selected Stories

- [ ] #MCP-001-A: Inventory current MCP client/server wiring and session startup gaps
- [ ] #MCP-001-B: Load configured MCP clients/servers at CLI composition root
- [ ] #MCP-001-C: Discover MCP tools before first model turn and register them in the tool registry
- [ ] #MCP-001-D: Route MCP tool calls through permission and summary/display metadata
- [ ] #MCP-001-E: Surface MCP connection/tool status and provenance in TUI/CLI diagnostics
- [ ] #MCP-001-F: Define unavailable-server behavior and prompt cache semantics

## Acceptance Criteria

- [ ] A configured MCP tool is visible to the model before the first turn.
- [ ] MCP tool execution uses the same permission pipeline as native tools.
- [ ] MCP provenance is preserved in tool display/conversation events.
- [ ] MCP discovery failures are user-visible and non-fatal by default.
- [ ] Prompt cache behavior is documented for startup-discovered versus unavailable MCP tools.
- [ ] Tests cover discovery, permission routing, provenance, and unavailable server behavior.
- [ ] No `rmcp` DTOs leak outside the MCP boundary.

## Risks

- Mid-session dynamic MCP tool mutation can invalidate prompt cache assumptions. Prefer startup
  discovery first.
- MCP status must flow through the existing single-consumer event model; do not introduce a global
  event bus.
- Permission behavior must be equivalent to native write/execute-capable tools.

## Verification Log

(to be filled as stories land)
