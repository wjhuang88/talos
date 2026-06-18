# ARCH-004: Anti-Corruption Layers

**Status**: Planned
**Priority**: P2
**Source**: ARCH-002 audit
**Depends on**: ARCH-003 recommended first

## Problem

External dependency types currently leak into Talos crate boundaries. That makes future dependency
upgrades more expensive and exposes implementation details as public API.

## Scope

- Define Talos-owned MCP descriptors/errors so `rmcp` types stay inside MCP adapter/server modules.
- Define crate-owned SQLite store errors for `talos-evolution` and `talos-session`.
- Replace `talos-mcp` direct config dependency with MCP-owned config DTOs converted at the CLI
  composition root.
- Resolve duplicate provider/tool cache `ToolDefinition` types.

## Acceptance Criteria

- [ ] Public `talos-mcp` client facade no longer exposes `rmcp::model::Tool`.
- [ ] Public `talos-mcp` errors no longer expose `rmcp::ErrorData`.
- [ ] `talos-evolution` and `talos-session` public error types do not expose `rusqlite::Error` as
      their primary public variant names.
- [ ] `talos-mcp` no longer imports `talos_config` in public client manager APIs.
- [ ] Duplicate `ToolDefinition` semantics are either unified or explicitly renamed.
- [ ] `cargo check --workspace` passes.

## Verification Notes

Run targeted `rg` checks for `rmcp::`, `rusqlite::`, `talos_config::`, and duplicate
`ToolDefinition` definitions after implementation.
