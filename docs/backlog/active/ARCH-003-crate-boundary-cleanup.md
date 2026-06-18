# ARCH-003: Crate Boundary Cleanup

**Status**: Planned
**Priority**: P1
**Source**: ARCH-002 audit
**Depends on**: ARCH-002 complete

## Problem

Several crate boundaries are tighter than needed. These issues are small enough to fix before more
RPC/MCP/API surface grows on top of them.

## Scope

- Remove dead `talos-mcp -> talos-agent` dependency.
- Introduce an RPC-facing runtime trait or adapter so `talos-rpc` no longer exposes
  `Arc<talos_agent::Agent>` in `MethodContext`.
- Rename message-layer `talos_core::message::ToolResult` to `MessageToolResult` so it no longer
  collides conceptually with execution-layer `talos_core::tool::ToolResult`.

## Acceptance Criteria

- [ ] `talos-mcp/Cargo.toml` no longer depends on `talos-agent`.
- [ ] `talos-rpc` public context uses a Talos-owned trait/adapter rather than concrete `Agent`.
- [ ] Message and execution tool result types have distinct names.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test -p talos-rpc -p talos-mcp -p talos-core` passes.

## Verification Notes

Use `docs/reference/ARCHITECTURE-AUDIT-2026-06-18.md` as the source audit.
