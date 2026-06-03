# I012: Portable Tools

**User can**: Use Talos on minimal or locked-down machines with fewer assumptions about
host POSIX utilities, while still exposing native tools through the same plugin/MCP/RPC
surfaces as external tools.

## Status: PLANNED

This iteration captures the requirement to reduce external environment dependency by
shipping a small Rust-native POSIX-style tool subset. It is linked to tool
pluginization: the built-in POSIX subset should be packaged like a native tool pack so
future plugin-provided tool packs can use the same registration path.

## Decision Gate

Create an ADR before implementation if this iteration changes any public or long-lived boundary:
`ToolPack`, `ToolProvenance`, `AgentTool`, tool listing schemas, config toggles, or MCP/RPC
exposure. The ADR should decide the initial native tool-pack shape, provenance names, enable/disable
config, and explicit non-goals.

## Selected Stories

- [ ] #I012-S1: Built-in POSIX basic tools subset
- [ ] #I012-S2: Embeddable tool pack interface

## Scope

- Implement a conservative set of POSIX/coreutils-like tools as structured `AgentTool`
  implementations.
- Initial read-only tools: `pwd`, `ls`, `cat`, `head`, `tail`, `wc`, `grep`.
- Initial write-capable tools: `mkdir`, `cp`, `mv`, `rm`.
- Register the set as a native tool pack that can later share the same path as
  plugin-provided local tools.
- Preserve all existing permission and sandbox boundaries.

## Non-Goals

- No general shell parser.
- No pipelines, redirects, glob expansion, or environment-variable expansion.
- No replacement for the existing `bash` tool.
- No arbitrary C/C++ bindings, Python FFI, Node.js runtime, or dynamic language runtime.
- No expansion beyond the initial tool subset without backlog change control.

## Acceptance Criteria

- [ ] ADR recorded before code if public API, provenance, config, or listing schema changes.
- [ ] Native POSIX tools are available without relying on host `ls`, `cat`, `grep`, etc.
- [ ] Every write-capable native tool is permission-gated by the existing pipeline.
- [ ] Read-only tools are marked read-only and can run concurrently with other read-only
      tools.
- [ ] Unsupported options fail with clear errors instead of falling back to host commands.
- [ ] Tool listing and provenance distinguish native built-in tools, native tool-pack
      tools, and MCP-remote tools.
- [ ] POSIX tool pack can be enabled by default and disabled by config.
- [ ] `cargo test --workspace` exits 0.

## Verification Notes

Append command outputs and test evidence here during execution. This iteration should
not move to Review until the tools are exercised on a deliberately minimal `PATH` to
prove the host utility dependency has actually been reduced.
