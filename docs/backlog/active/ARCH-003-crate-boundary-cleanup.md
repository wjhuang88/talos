# ARCH-003: Crate Boundary Cleanup

**Status**: Complete (→ I027, 2026-06-18)
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

- [x] `talos-mcp/Cargo.toml` no longer depends on `talos-agent`.
- [x] `talos-rpc` public context uses a Talos-owned trait/adapter rather than concrete `Agent`.
- [x] Message and execution tool result types have distinct names.
- [x] `cargo check --workspace` passes.
- [x] `cargo test -p talos-rpc -p talos-mcp -p talos-core` passes.

## Verification Notes

Use `docs/reference/ARCHITECTURE-AUDIT-2026-06-18.md` as the source audit.

## Closure Evidence (2026-06-18)

All three stories landed in I027:

- **ARCH-003-A**: Removed `talos-agent` line from `crates/talos-mcp/Cargo.toml`; zero source
  usage confirmed.
- **ARCH-003-B**: Renamed `talos_core::message::ToolResult` → `MessageToolResult` across 14
  files; `AgentEvent::ToolResult` variant preserved; execution-layer `tool::ToolResult`
  untouched.
- **ARCH-003-C**: Introduced `Runtime` trait + `RuntimeError` in
  `crates/talos-rpc/src/runtime.rs`; `MethodContext.agent` is now `Arc<dyn Runtime>`;
  `RpcServer::new` accepts `Arc<dyn Runtime>`; concrete `AgentRuntime(Agent)` adapter lives in
  `crates/talos-cli/src/runtime_adapter.rs`; `talos-rpc/Cargo.toml` no longer declares
  `talos-agent`.

Workspace verification:
- `cargo check --workspace` — passed.
- `cargo test --workspace` — all suites green (0 failures, 1 pre-existing ignored).
- `cargo clippy --workspace -- -D warnings` — passed.
- `cargo fmt --all` — no changes.
- `scripts/validate_project_governance.sh .` — 0 warnings.
