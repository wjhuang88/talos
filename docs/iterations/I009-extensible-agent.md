# Iteration I009: Extensible Agent

**User can**: Extend Talos through hooks, MCP servers, and stdio JSON-RPC while existing permission
and sandbox boundaries remain enforced.

## Status: ACTIVE (2026-06-01)

R0 closed 2026-06-01. I009 starts in execution-plan order: S2 (hook system) → S3 (MCP client) →
S4 (MCP server) → S5 (JSON-RPC) → S1 (TUI surface).

## Selected Stories

- [ ] #I009-S1: TUI MCP tool markers + plugin status
- [ ] #I009-S2: Hook system (20+ extension points)
- [ ] #I009-S3: MCP client
- [ ] #I009-S4: MCP server
- [ ] #I009-S5: JSON-RPC server (stdio)

## Execution Plan

1. Define extension boundaries and hook event types without introducing a global pub/sub bus.
2. Implement the hook system as the primary local extension mechanism.
3. Add MCP client support for external tool providers.
4. Add MCP server support for exposing Talos capabilities under the permission pipeline.
5. Add stdio JSON-RPC as the automation/control surface.
6. Surface MCP/plugin state in the TUI after the backend paths are verified.

## Acceptance Criteria

- [ ] A local hook/plugin path can be loaded and observed firing on a real agent run.
- [ ] Talos can call at least one MCP-provided tool.
- [ ] Talos can expose at least one permission-gated capability through MCP server mode.
- [ ] `talos --mode rpc` accepts a stdio JSON-RPC request and returns a machine-readable response.
- [ ] TUI marks MCP-provided tools distinctly from built-in tools.
- [ ] Extension paths do not bypass permission checks, sandboxing, or command approval.
- [ ] `cargo test --workspace` exits 0.

## Out of Scope

- MCP OAuth.
- WebSocket RPC.
- Plugin marketplace.
- WASM plugin hosting unless a concrete I009 story is changed through change control.

## Verification Notes

Append end-to-end commands, mock MCP fixtures, and JSON-RPC examples here during execution.
