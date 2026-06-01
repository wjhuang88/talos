# ADR-009: Tool Provenance Tracking

- **Status**: Accepted
- **Date**: 2026-06-01
- **Iteration**: I009-S1

## Context

I009 extends the agent to invoke tools from multiple sources: built-in native
tools, MCP remote tools, and (future) other plugin-provided tools. Consumers
(TUI, RPC, evolution engine) need to know *where a tool came from* so they can
render provenance markers, filter UI displays, and apply per-source policies.

Prior to this change, `AgentTool` had no `provenance()` method and the
`AgentEvent::ToolCall` event carried only a `call: ToolCall` field. Callers
could not distinguish a native tool from a remote MCP tool.

## Decision

Add an additive `ToolProvenance` enum to `talos-core` and a `provenance()`
method to the `AgentTool` trait, then thread the value through `AgentEvent`:

1. **`ToolProvenance`** (in `talos-core::tool`):

   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
   #[serde(tag = "kind", rename_all = "snake_case")]
   #[non_exhaustive]
   pub enum ToolProvenance {
       #[default]
       Native,
       McpRemote { server: String },
   }
   ```

2. **`AgentTool::provenance()`** — default method returns
   `ToolProvenance::Native`. Implementations may override (e.g.
   `McpToolAdapter` returns `McpRemote { server }`).

3. **`AgentEvent::ToolCall { call, provenance }`** — add `provenance` field
   with `Default::default()` semantics on the receiver side. Mark the
   `AgentEvent` enum `#[non_exhaustive]` to allow future variants without
   breaking downstream matches.

4. **MCP adapter override** — `McpToolAdapter::provenance()` returns
   `ToolProvenance::McpRemote { server: self.remote.server.clone() }`.

## Backward Compatibility

- `ToolProvenance` is `#[non_exhaustive]`, so adding new variants is not a
  breaking change.
- `AgentEvent` is now `#[non_exhaustive]`, so all consumers need a wildcard
  arm (`_ => {}` or `Ok(_) => {}` on broadcast `Result`).
- The new `provenance` field on `AgentEvent::ToolCall` is additive; producers
  in this commit use `provenance: Default::default()` for native tools and
  the override path for MCP tools.
- `AgentTool::provenance()` is a default-method addition; existing
  implementations continue to compile.

## Out of Scope (deferred to a follow-up)

This ADR is structural only. The following TUI consumer features are
**not** part of I009-S1 and will land in a separate iteration:

- TUI rendering of provenance markers (e.g. `⌬ mcp:<server>` in the tool-call
  row)
- `/plugins` slash command to list available hooks and MCP servers
- Filtering / grouping of the tool palette by provenance

These were originally in the I009-S1 visual-engineering plan but were
deferred when the visual-engineering task was time-budgeted. Producers are
in place; consumers will be added without further API changes.

## Alternatives Considered

1. **Inline `provenance` on every `ToolCall`** — rejected: pollutes the
   protocol schema and duplicates data the `AgentTool` already knows.
2. **Stringly-typed `provenance: String`** — rejected: loses the type
   safety benefits of an enum.
3. **Global pub-sub for tool registration** — rejected: ADR-006 explicitly
   bans a global bus; tool provenance is per-tool metadata on the trait.
