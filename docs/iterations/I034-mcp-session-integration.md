# I034: MCP Session Integration

> Document status: Complete
> Published plan date: 2026-06-19
> Planned objective: make startup-configured MCP tools a first-class, permission-routed session capability.
> Baseline rule: preserve startup-stable discovery; mid-session mutation and MCP command publication are out of scope.
> MVP deliverable: a local stdio MCP tool is model-visible before the first turn, executes through
> normal permissions, and reports provenance/status with non-fatal startup failure handling.

**Status**: Complete (2026-06-19)
**Target Window**: After I031 or parallel only after Skill startup path is stable
**Depends On**: ARCH-003/004 complete, I031 preferred

## Outcome

Make MCP a first-class session capability. Configured MCP tools should be discovered at session
startup, registered beside native tools, exposed to the model before the first turn where possible,
routed through the same permission/display pipeline, and shown with provenance/status in the
conversation surfaces.

## Selected Stories

- [x] #MCP-001-A: Inventory current MCP client/server wiring and session startup gaps
- [x] #MCP-001-B: Load configured MCP clients/servers at CLI composition root
- [x] #MCP-001-C: Discover MCP tools before first model turn and register them in the tool registry
- [x] #MCP-001-D: Route MCP tool calls through permission and summary/display metadata
- [x] #MCP-001-E: Surface MCP connection/tool status and provenance in TUI/CLI diagnostics
- [x] #MCP-001-F: Define unavailable-server behavior and prompt cache semantics
- [x] #MCP-001-G: Feed MCP status to the CMD-001 `/plugins` BuiltinCommand without treating MCP
      prompts as an additional command-definition origin

## Acceptance Criteria

- [x] A configured MCP tool is visible to the model before the first turn.
- [x] MCP tool execution uses the same permission pipeline as native tools.
- [x] MCP provenance is preserved in tool display/conversation events.
- [x] MCP discovery failures are user-visible and non-fatal by default.
- [x] Prompt cache behavior is documented for startup-discovered versus unavailable MCP tools.
- [x] Tests cover discovery, permission routing, provenance, and unavailable server behavior.
- [x] No `rmcp` DTOs leak outside the MCP boundary.
- [x] I034 does not create a parallel command registry or auto-publish MCP prompts as slash commands.

## Risks

- Mid-session dynamic MCP tool mutation can invalidate prompt cache assumptions. Prefer startup
  discovery first.
- MCP status must flow through the existing single-consumer event model; do not introduce a global
  event bus.
- Permission behavior must be equivalent to native write/execute-capable tools.

## Verification Log

2026-06-19 activation:

- I033 and SKILL-001 closed with real-binary runtime evidence; the startup composition prerequisite
  is satisfied.
- GOV-002 is complete and both governance validators pass, so no unresolved status conflict blocks
  activation.
- Required documentation owners for closure: this iteration, MCP-001, `README.md`, architecture
  reference if the composition boundary changes, Product Backlog, iterations index, Board, and the
  confirmed long-task record.
- Implementation begins with a current-wiring inventory and one bounded startup composition path;
  no new dependency, public API break, or command registry is authorized by this activation.

2026-06-19 implementation checkpoint:

- Added a session-scoped CLI MCP runtime used by RPC, print, TUI, inline, and legacy interactive
  startup. It owns the manager lifetime, cached tool adapters, and Talos-owned diagnostics.
- MCP manager discovery now lists tools once during startup and reuses the cached descriptors.
  Tool definitions therefore enter the Agent registry before the first turn and remain stable for
  the session prompt/cache prefix.
- Permission wrappers now evaluate declared `ToolNature` and preserve provenance. Read-only MCP
  tools can run under normal read policy; unknown/write tools require approval and are denied in
  headless modes where approval is unavailable.
- `/plugins` shows connected/unavailable MCP servers and startup tool counts before any tool call,
  while retaining observed provenance counts.
- Per-server startup failures remain non-fatal and visible. MCP requests time out after 30 seconds,
  pending requests are removed on timeout, child processes use kill-on-drop, and disconnected or
  timed-out calls return visible tool errors.
- The real CLI fixture test proves both execution/provenance (`fixture:ping`) and provider-visible
  pre-turn tool definition (`mcp:fixture:echo`, description, and schema).
- Targeted `cargo check`, tests, clippy with `-D warnings`, fmt check, and diff check pass. Full
  workspace closure remains T9.

2026-06-19 closure:

- `cargo fmt --all -- --check`, `cargo check --workspace`, and
  `cargo clippy --workspace -- -D warnings` passed.
- `cargo test --workspace` passed outside the restricted sandbox, including the real MCP fixture,
  provider listener tests, subprocess failure isolation, and all doctests. The existing
  timing-sensitive agent interrupt test remains ignored; I034 added no ignored tests.
- Both governance validators and `git diff --check` passed with 0 warnings/errors.
- Implementation commit: `ab9f77e`.
- Accepted residuals: HTTP transport, strict startup mode, and mid-session dynamic tool mutation are
  outside the published I034 baseline and require separate requirements before implementation.
