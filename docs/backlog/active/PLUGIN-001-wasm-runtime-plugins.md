# PLUGIN-001: WASM Runtime Plugin Protocol

| Field | Value |
| --- | --- |
| Story ID | PLUGIN-001 |
| Status | **In Progress — I077/T111 read-only tool slice in Review**. Repositioned from "WASM-only protocol" to "plugin encapsulation system" covering skill/mcp/hook + tools. ADR-027/028/029/030 accepted; ADR-032 cleared the focused `wasmtime` dependency/security review for the first local explicit read-only WASM plugin MVP after manifest parsing. T110 cleared only a bounded local explicit read-only fixture plugin tool slice; T111 implemented that slice with confinement, provenance, permission facets, bounded output, collision rejection, no host calls, and `wasmtime v46.0.1`. |
| Priority | P2 (elevated from P4, 2026-06-20 — unblocks TOOL-008 Phase 3 + WEBFETCH Phase 2+ WASM consumers) |
| Source | User request, 2026-06-18; model expanded 2026-06-30 (four-entity architecture) |
| Relates To | CMD-001, CMD-002, HOOK-001, I009 extensibility, ADR-009, ADR-013, `talos-plugin`, `talos-mcp`, `talos-rpc`, TOOL-008, DIST-001 |

## Current Selection

I091 activated 2026-07-04 to audit the local explicit plugin diagnostics/runtime state after the
T111 read-only WASM slice. The I091 scope is diagnostics/provenance/confinement visibility only:
no remote install, marketplace, automatic discovery, write-capable plugin tools, Lua, dynamic
library support, or broader host-call surface.

I091 A7 delivered the diagnostics/schema part of that scope: plugin manifests now accept and
validate `[[hooks]]` declarations, `HookRegistry` exposes a read-only registration snapshot, and
the conversation engine exposes `/hooks` diagnostics. Hook declarations are descriptive only; this
does not load hook carriers or execute plugin hooks.

## Requirement

Design a protocol specification and runtime architecture for loading Talos plugins.

> **2026-06-30 repositioning.** The owner declared the target architecture as four entities:
> skill / mcp / hook are three independent config-introduced atomic component types; **plugin** is
> a packaging/distribution format that bundles an arbitrary subset of {skill, mcp, hook} plus
> additional tool definitions, carried by an external artifact. Carrier set settled 2026-06-30:
> **WASM first-class, Lua optional, dynamic library rejected.** The detailed draft is
> [`docs/proposals/plugin-encapsulation-format.md`](../../proposals/plugin-encapsulation-format.md).
> ADR-027/028/029/030 accepted on 2026-06-30. ADR-032 accepted on 2026-07-01. PLUGIN-001 is no
> longer blocked on missing architecture decisions or the initial `wasmtime` dependency/security
> review; implementation must still start with manifest parsing, then a bounded local read-only
> WASM MVP with resource/failure tests.

Plugins may provide:

- tools;
- commands;
- hooks;
- filters;
- bundled skills and MCP component declarations (added 2026-06-30);
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
- Define how WASM plugins expose tools, commands, hooks, and filters.
- Define `PluginCommand` registration, namespacing, collision handling, provenance, availability,
  execution, and unload behavior against the session-scoped command registry from CMD-001.
- Define permission boundaries for plugin-provided tools and host calls.
- Define lifecycle events: load, initialize, register, execute, shutdown, error.
- Define compatibility/version negotiation.
- Define whether bulky plugin packages are bundled, locally installed, or downloaded through the
  shared optional asset distribution flow.
- Define deterministic failure behavior when a plugin panics, times out, traps, or returns
  malformed protocol messages.
- Decide whether the first implementation should use WASI component model, raw WASM + host ABI,
  or another Rust-native WASM runtime.

### Out of Scope Until ADR

- Implementing a WASM runtime.
- Loading untrusted network packages.
- Plugin marketplace.
- **Native dynamic library loading — firmly rejected per owner decision 2026-06-30.** A `.so`/`.dll`
  cannot be sandboxed and conflicts with Hard Constraint #1. This is a permanent non-goal, not a
  deferred decision.
- Node/Python plugin runtimes.
- Provider plugin execution.
  Runtime-downloadable plugin packages require DIST-001 and a follow-up ADR before implementation.

## Acceptance Criteria

- [ ] A protocol specification is written under `docs/reference/` or `docs/proposals/`.
- [ ] A decision record is created before adding a WASM runtime dependency.
- [ ] The spec defines tools, commands, hooks, and filters as first-class plugin capabilities.
- [ ] Plugin commands cannot override built-in commands or bypass Tool, Session, permission, or UI
      ownership boundaries.
- [ ] Plugin-provided tools use the existing permission pipeline and provenance model.
- [ ] Hook/filter execution order and failure policy are specified.
- [ ] Sandbox/resource limits are specified, including timeout, memory, filesystem/network access,
      and host-call allowlist.
- [ ] Version negotiation and forward/backward compatibility rules are specified.
- [ ] Optional plugin package distribution is aligned with DIST-001 instead of adding a separate
      download path.
- [ ] No implementation starts until the spec and ADR are accepted.

## Next Implementation Slice

1. Add plugin manifest parser and validation only; no executable artifact instantiation during
   discovery.
2. ~~Add `ToolProvenance::Plugin { name, version, carrier }` and render/serialize it through existing
   tool-call paths.~~ **Complete (T40, 2026-07-01)**: variant added to `talos-core`; observation key,
   scrollback badge, and TUI bubble rendering updated; 8 tests across core/conversation/TUI.
3. Add local explicit plugin package loading behind config/CLI opt-in.
4. Add one fixture WASM read-only tool through the existing `AgentTool`/permission pipeline.
5. Cover success, malformed manifest, invalid module, trap, timeout, oversized output, and denied
   permission.

I091 A7 note:
- `[[hooks]]` manifest declarations are parsed and validated for known event names, non-empty
  handlers, and duplicate names.
- `/hooks` is a read-only diagnostics surface; it does not load packages or execute hooks.

T110 security review (2026-07-01):
- Review artifact: `docs/reference/PLUGIN-MVP-SECURITY-REVIEW-2026-07-01.md`.
- T111 may proceed only as a local explicit read-only fixture plugin tool registered through
  `AgentTool`/`ToolRegistry`.
- T111 blockers to address before closeout: package-root confinement for artifact/handler paths,
  tool-name collision rejection, plugin provenance, permission pipeline denial tests, bounded
  output, and `wasmtime` version rationale or update.

T111 implementation review (2026-07-02):
- T111 implemented `register_read_only_wasm_tools` for local explicit package manifests only.
- Plugin tools are namespaced as `{plugin}.{tool}`, reject registry collisions, carry
  `ToolProvenance::Plugin`, expose read-only permission facets, and stay out of runtime-default
  tool presentation through the `Plugin` tool family.
- Manifest artifact and tool handler paths are confined to the package root and reject absolute
  paths or parent-directory traversal before module loading.
- Model-facing plugin output is bounded; host calls, write tools, automatic discovery, remote
  install, broad plugin protocol handling, and default presentation remain out of scope.
- `wasmtime` is now `46.0.1`, matching the current crate discovery recorded by ADR-032.

ADR-032 implementation constraints:
- add `wasmtime` only in the focused plugin runtime slice;
- keep host calls denied by default;
- record `cargo tree`/feature evidence after the dependency lands;
- use deterministic fuel/resource limits where possible, plus timeout guard;
- no write-capable plugin tools in the first executable slice.

Do not add remote package installation, marketplace behavior, Lua, dylib, write-capable plugin
tools, or automatic plugin discovery in the first slice.

## Required Reads

- `docs/proposals/plugin-encapsulation-format.md` **(governing draft, 2026-06-30)**
- `docs/decisions/027-plugin-runtime-boundary.md`
- `docs/decisions/028-plugin-tool-provenance-extension.md`
- `docs/decisions/029-extensibility-atomic-component-model.md`
- `docs/decisions/030-extensibility-command-taxonomy.md`
- `docs/decisions/032-wasmtime-dependency-security-review.md`
- `docs/proposals/wasm-runtime-plugin-protocol.md` (subsumed as the WASM-carrier slice)
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/backlog/active/CMD-002-command-taxonomy-realignment.md`
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md`
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
6. Should v1 plugin commands execute dedicated plugin handlers, alias plugin tools, or support both
   through separate executor kinds?
