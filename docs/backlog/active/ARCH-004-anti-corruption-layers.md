# ARCH-004: Anti-Corruption Layers

**Status**: Complete (→ I029, 2026-06-18)
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

- [x] Public `talos-mcp` client facade no longer exposes `rmcp::model::Tool`.
- [x] Public `talos-mcp` errors no longer expose `rmcp::ErrorData`.
- [x] `talos-evolution` and `talos-session` public error types do not expose `rusqlite::Error` as
      their primary public variant names.
- [x] `talos-mcp` no longer imports `talos_config` in public client manager APIs.
- [x] Duplicate `ToolDefinition` semantics are either unified or explicitly renamed.
- [x] `cargo check --workspace` passes.

## Verification Notes

Run targeted `rg` checks for `rmcp::`, `rusqlite::`, `talos_config::`, and duplicate
`ToolDefinition` definitions after implementation.

- 2026-06-18: Completed in I029. `cargo check --workspace`, `cargo test --workspace`, and
  `cargo clippy --workspace -- -D warnings` passed.
