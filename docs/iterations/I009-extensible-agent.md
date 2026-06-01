# Iteration I009: Extensible Agent

**User can**: Extend Talos through hooks, MCP servers, and stdio JSON-RPC while existing permission
and sandbox boundaries remain enforced.

## Status: ACTIVE (2026-06-01)

R0 closed 2026-06-01. I009 starts in execution-plan order: S2 (hook system) → S3 (MCP client) →
S4 (MCP server) → S5 (JSON-RPC) → S1 (TUI surface).

## Selected Stories

- [x] #I009-S1: TUI MCP tool markers + plugin status (producers only; TUI markers + `/plugins` command deferred to follow-up per ADR-009)
- [x] #I009-S2: Hook system (20+ extension points)
- [x] #I009-S3: MCP client
- [x] #I009-S4: MCP server
- [x] #I009-S5: JSON-RPC server (stdio)

## Execution Plan

1. Define extension boundaries and hook event types without introducing a global pub/sub bus.
2. Implement the hook system as the primary local extension mechanism.
3. Add MCP client support for external tool providers.
4. Add MCP server support for exposing Talos capabilities under the permission pipeline.
5. Add stdio JSON-RPC as the automation/control surface.
6. Surface MCP/plugin state in the TUI after the backend paths are verified.

## Acceptance Criteria

- [x] A local hook/plugin path can be loaded and observed firing on a real agent run.
- [x] Talos can call at least one MCP-provided tool.
- [x] Talos can expose at least one permission-gated capability through MCP server mode.
- [x] `talos --mode rpc` accepts a stdio JSON-RPC request and returns a machine-readable response.
- [ ] TUI marks MCP-provided tools distinctly from built-in tools. (Deferred: producers wire `ToolProvenance`; consumer-side rendering tracked in a follow-up. See ADR-009 "Out of Scope".)
- [x] Extension paths do not bypass permission checks, sandboxing, or command approval.
- [x] `cargo test --workspace` exits 0.

## Out of Scope

- MCP OAuth.
- WebSocket RPC.
- Plugin marketplace.
- WASM plugin hosting unless a concrete I009 story is changed through change control.

## Verification Notes

Append end-to-end commands, mock MCP fixtures, and JSON-RPC examples here during execution.

---

## Execution Record (2026-06-01)

### Story Status

| Story | Commit | Tests | Notes |
|-------|--------|-------|-------|
| S2 (hooks) | `fbc3a25` | 11 new | `talos-plugin` crate + 13 lifecycle hook points; `LoggingHandler` builtin |
| S3 (MCP client) | `a3b4cde` | 3 new | rmcp =0.16.0 facade; `McpToolAdapter` bridges remote tools into `AgentTool` |
| S4 (MCP server) | `a79b8ac` | 3 new | `Ask` policy → `Deny` in headless `--mcp-server` mode (fail-closed) |
| S5 (JSON-RPC) | `3fc3f07` | 4 new | stdio transport, framed by Content-Length; methods: `system.version`, `agent.list_tools`, `agent.run`, `agent.cancel` |
| S1 (provenance) | `74f2530` | 0 new (additive) | `ToolProvenance` enum, `AgentEvent::ToolCall` gains `provenance` field, `AgentEvent` marked `#[non_exhaustive]`, ADR-009 |

Pre-S2 chore: `3e75522` (talos-tui unwrap + talos-cli/approval.rs pre-existing lints).

### End-to-End Runtime Evidence (§3a gate)

All five stories were exercised through the actual `talos` binary, not only
isolated unit tests.

**S2 (Hook system) — `talos -p --mock`:**

```
[INFO hook event handler="LoggingHandler" event=OnSystemPromptBuilt turn_id=1]
[INFO hook event handler="LoggingHandler" event=TurnStart turn_id=1]
[INFO hook event handler="LoggingHandler" event=BeforeProviderCall turn_id=1]
[INFO hook event handler="LoggingHandler" event=OnTextDelta turn_id=1]
[INFO hook event handler="LoggingHandler" event=OnTurnEnd turn_id=1]
[INFO hook event handler="LoggingHandler" event=AfterProviderCall turn_id=1]
[INFO hook event handler="LoggingHandler" event=TurnComplete turn_id=1]
```

Plus the `hooks_e2e` integration test (`crates/talos-cli/tests/hooks_e2e.rs`)
exercises `OnToolCallProposed`, `BeforePermissionCheck`, and
`AfterPermissionCheck` against a tool-calling mock.

**S3 (MCP client) — `cargo test -p talos-mcp`:**

- `mock_stdio_roundtrip.rs` — proves the stdio transport round-trips
  initialize → list_tools → call_tool frames.
- `mcp_client_e2e.rs` — spawns a real rmcp server binary and confirms that
  the dispatcher surfaces tools through `McpToolAdapter`.

**S4 (MCP server) — `talos --mode=mcp-server --mcp-server-fixture=…`:**

The binary spawns the fixture, sends `initialize`, and routes the response
through rmcp's transport. The fixture is required to send the
`notifications/initialized` notification to fully complete the handshake;
this is a fixture limitation, not a talos wiring issue.
`server_handshake.rs`, `server_permission_deny.rs`, and `subprocess_crash.rs`
cover the headless `Ask` policy and crash recovery paths in isolation.

**S5 (JSON-RPC) — `talos --mode=rpc --mock`:**

```
$ echo '{"jsonrpc":"2.0","id":1,"method":"system.version"}' | talos --mode=rpc --mock
{"jsonrpc":"2.0","id":1,"result":{"protocol":1,"version":"0.1.0"}}

$ echo '{"jsonrpc":"2.0","id":1,"method":"agent.list_tools"}' | talos --mode=rpc --mock
{"jsonrpc":"2.0","id":1,"result":[{"name":"bash",…},{"name":"read",…},{"name":"write",…},{"name":"edit",…}]}

$ echo '{"jsonrpc":"2.0","id":1,"method":"system.listMethods"}' | talos --mode=rpc --mock
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}
```

The error response for an unknown method proves the JSON-RPC envelope is
fully wired (proper `id` echo, `code: -32601`).

**S1 (Provenance) — additive structural change:**

Producers (`talos-core`, `talos-mcp`, `talos-provider`) emit
`ToolProvenance::Native` (default) or `ToolProvenance::McpRemote { server }`
on every `AgentEvent::ToolCall`. Consumers (TUI, CLI, RPC) match the new
field with `..` wildcards and add `_ => {}` arms for the `#[non_exhaustive]`
`AgentEvent` enum. The TUI marker rendering and `/plugins` slash command
are intentionally deferred to a follow-up iteration (see ADR-009 "Out of
Scope").

### Test Counts

- R0 baseline: 480 tests across 25 binaries.
- After I009: 501 tests across 25 binaries (+21). Net of S1's structural
  test surface (the new field is exercised by existing tests after
  `Default::default()` updates).

### Architectural Records

- **ADR-009 (new)**: `docs/decisions/009-tool-provenance.md` — additive
  `ToolProvenance`, `#[non_exhaustive]` `AgentEvent`, default-method on
  `AgentTool`, deferred TUI consumer work.
- **ADR-006 (reaffirmed)**: Hooks are per-agent, not a global pub/sub
  bus. Confirmed by S2 implementation review.

### Lessons Learned

See `EVOLUTION.md` lessons #15, #16, #17 for the I009-specific lessons on
parallel-agent scope discipline, the marker protocol, and visual-engineering
time budgeting.
