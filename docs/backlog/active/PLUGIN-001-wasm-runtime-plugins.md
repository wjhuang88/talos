# PLUGIN-001: WASM Runtime Plugin Protocol

| Field | Value |
| --- | --- |
| Story ID | PLUGIN-001 |
| Status | Research |
| Priority | P4 |
| Source | User request, 2026-06-18 |
| Relates To | I009 extensibility, ADR-009, ADR-013, `talos-plugin`, `talos-mcp`, `talos-rpc` |

## Requirement

Design a protocol specification and runtime architecture for loading WASM-based Talos plugins.
Plugins may provide:

- tools;
- hooks;
- filters;
- future extension capabilities registered through the same protocol boundary.

## Problem

Talos currently has built-in Rust hook infrastructure and MCP-based external tool integration, but
no stable runtime plugin protocol for third-party local extensions. Before implementing WASM
loading, Talos needs a protocol spec that defines capability registration, permissions, lifecycle,
host calls, sandbox limits, compatibility, and failure behavior.

## Scope

### Research / Specification First

- Define the plugin manifest format.
- Define host/plugin protocol messages for capability discovery and registration.
- Define how WASM plugins expose tools, hooks, and filters.
- Define permission boundaries for plugin-provided tools and host calls.
- Define lifecycle events: load, initialize, register, execute, shutdown, error.
- Define compatibility/version negotiation.
- Define deterministic failure behavior when a plugin panics, times out, traps, or returns
  malformed protocol messages.
- Decide whether the first implementation should use WASI component model, raw WASM + host ABI,
  or another Rust-native WASM runtime.

### Out of Scope Until ADR

- Implementing a WASM runtime.
- Loading untrusted network packages.
- Plugin marketplace.
- Native dynamic library loading.
- Node/Python plugin runtimes.
- Provider plugin execution.

## Acceptance Criteria

- [ ] A protocol specification is written under `docs/reference/` or `docs/proposals/`.
- [ ] A decision record is created before adding a WASM runtime dependency.
- [ ] The spec defines tools, hooks, and filters as first-class plugin capabilities.
- [ ] Plugin-provided tools use the existing permission pipeline and provenance model.
- [ ] Hook/filter execution order and failure policy are specified.
- [ ] Sandbox/resource limits are specified, including timeout, memory, filesystem/network access,
      and host-call allowlist.
- [ ] Version negotiation and forward/backward compatibility rules are specified.
- [ ] No implementation starts until the spec and ADR are accepted.

## Required Reads

- `docs/proposals/wasm-runtime-plugin-protocol.md`
- `docs/iterations/I009-extensible-agent.md`
- `docs/decisions/009-tool-provenance.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/reference/ARCHITECTURE.md`
- `crates/talos-plugin/src/`

## Open Questions

1. Should plugins register filters as hook handlers with stronger ordering semantics, or as a
   separate capability type?
2. Should plugin tools be invoked through the same `AgentTool` registry or a dedicated
   plugin-tool adapter layer?
3. Should the first protocol target the WASI component model, or a smaller JSON ABI over stdin-like
   host calls?
4. How should plugin provenance appear in TUI/RPC outputs alongside native and MCP tools?
5. What is the minimum useful host-call surface for v1?
