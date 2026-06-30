# Plugin Encapsulation Format

> Status: **DRAFT — awaiting architecture decision. Not committed work.**
> Supersedes/extends: [wasm-runtime-plugin-protocol.md](wasm-runtime-plugin-protocol.md)
> Owner declaration: user, 2026-06-30
> Blocks: PLUGIN-001, CMD-002, HOOK-001, TOOL-008 Phase 3

## Problem

Talos has three independent extensibility axes today, but they are not modeled as peers and one of
them (`plugin`) does not exist as a shipped concept despite already owning a slash-command name.

Concrete drift:

- **Skill** exists, is config-discoverable, and has `/skills`. ✅
- **MCP** exists and is config-introduced, but its status is reported under `/plugins`, which is a
  naming collision with a concept that has no implementation. ❌ naming
- **Hook** exists only as a code-registered `HookHandler` (`talos-plugin` crate); users cannot
  introduce hooks through configuration. ⚠️ partial
- **Plugin** has no implementation. The existing `wasm-runtime-plugin-protocol.md` proposal scopes
  it to WASM only. The owner vision is broader: plugin is an encapsulation format that can wrap any
  subset of {skill, mcp, hook} plus additional tool definitions, and may be carried by WASM or Lua
  script artifacts. ❌ absent + scoped too narrowly
- `ToolProvenance` (ADR-009) has only `Native | McpRemote`; there is no `Plugin` variant.

The result is a fragmented extensibility surface: three concepts that should be peers are modeled
in three different ways, and the command/UI vocabulary suggests a plugin system that does not
exist.

## Target Architecture

Four entities, two layers.

### Atomic Components (config-introduced)

Each is a first-class, independently configurable capability type.

| Component | Purpose | Config introduction today | Visible via |
|---|---|---|---|
| **Skill** | Prompt-level capability bundle (`SKILL.md`) with progressive disclosure (Level 0/1/2). | `.talos/skills/`, `~/.talos/skills/`, parent dirs, opt-in `~/.agents/skills/` | `/skills` |
| **MCP** | External stdio process providing tools (`mcp:<server>:<tool>`). | `[[mcp.servers]]` | should be `/mcp` |
| **Hook** | Lifecycle observer/modifier (`BeforeProviderCall`, `OnToolCall`, `TurnComplete`, etc.). | **None — code-registered only.** Must become config-introduced. | should be `/hooks` |

### Encapsulation Layer

**Plugin** is a *packaging and distribution format*, not a fourth atomic capability. A plugin
package bundles an arbitrary subset of {skill, mcp, hook} **plus additional tool definitions** that
are not expressible as MCP or built-in tools. The package is carried by an external artifact.

| Carrier | What it is | Sandbox profile | Fit with Talos posture |
|---|---|---|---|
| **WASM** | Compiled module run by a Rust-native runtime (e.g. `wasmtime`) | Native sandbox; capabilities explicitly granted; filesystem/network off by default | ✅ Aligned with safety-first; already studied in PLUGIN-001 |
| **Lua** | Script run by an embedded interpreter (`mlua`/`rustlua`) | Interpreter-level sandbox via restricted stdlib + capability allowlist; pure-Rust embed | ⚠ Medium — viable as a lightweight scripting carrier |
| **Dynamic library** (`.so`/`.dll`/`.dylib`) | Native code loaded via `libloading` | **No sandbox possible** — loading is equivalent to granting full process privilege | ⚠ Highest — conflicts with Hard Constraint #1 unless treated as a fully-trusted escape hatch |

A plugin declares which carrier it uses in its manifest. Talos loads it through the matching loader
and resolves the declared atomic components + tools into the existing registries.

**Dynamic library loading (`.so`/`.dll`/`.dylib`) is explicitly rejected** per owner decision
2026-06-30. A native library cannot be sandboxed — loading it is equivalent to granting the plugin
full process privilege, which conflicts irreconcilably with Hard Constraint #1 (Rust first / no
arbitrary C/C++ bindings) and the safety-first posture. Plugin authors who need host-level trust
should ship a builtin Rust hook/tool or an MCP server, not a dylib. This is a firm non-goal.

### Relationships

```
            ┌─────────── plugin package (manifest + artifact) ───────────┐
            │  declares: carrier, permissions, and a subset of:          │
            │    skill[*]   mcp[*]   hook[*]   tool[*]                    │
            └────────────────────────┬───────────────────────────────────┘
                                     │ loaded by carrier-specific loader
                                     ▼
   ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────────────┐
   │ SkillIndex │  │ MCP client │  │ HookChain  │  │ ToolRegistry       │
   │            │  │ manager    │  │            │  │ (ToolProvenance::  │
   │            │  │            │  │            │  │  Plugin{..})       │
   └────────────┘  └────────────┘  └────────────┘  └────────────────────┘
```

Every plugin-originated capability carries provenance so TUI/RPC/evolution consumers can distinguish
plugin-provided tools/skills/hooks from native and MCP ones.

## Current State Gap

| Entity | Implementation | Config-introduced | Provenance type | Command | Gap |
|---|---|---|---|---|---|
| Skill | ✅ `talos-skill` | ✅ | n/a (prompt-level) | `/skills` | none |
| MCP | ✅ `talos-mcp` + `McpSessionRuntime` | ✅ `[[mcp.servers]]` | `McpRemote` | `/plugins` (misnamed) | command rename |
| Hook | ⚠ `talos-plugin` `HookHandler` | ❌ code-only | n/a | none | config-ization |
| Plugin | ❌ absent | ❌ | ❌ no `Plugin` variant | `/plugins` stolen by MCP | entire system |

## Carrier Strategy

The carrier set is settled (owner decision 2026-06-30): **WASM is the first-class carrier and the
first implementation target; Lua is an optional lightweight scripting carrier; dynamic library
loading is rejected.**

**Hard Constraint #1** (Rust first — no arbitrary C/C++ bindings) and **Hard Constraint #5**
(sandbox review) govern the carrier design:

- **WASM** preserves the safety-first, auditable posture. It is the default and the only carrier in
  the first implementation slice. The runtime ADR records the runtime choice (e.g. `wasmtime`),
  capability-grant model, and sandbox limits.
- **Lua** is defensible as an optional follow-up carrier: a pure-Rust embed (`mlua`/`rustlua`) with
  an interpreter-level sandbox (restricted stdlib + host-call allowlist). It introduces a new runtime
  dependency, so it needs its own ADR before activation, but it does not conflict with Hard Constraint
  #1 the way native code loading would.
- **Dynamic library loading is rejected** (see the firm non-goal above). The owner considered and
  rejected it on 2026-06-30 because a `.so`/`.dll`/`.dylib` cannot be sandboxed and would silently
  grant plugin code full process privilege — irreconcilable with the safety posture. This decision
  is closed and not a carrier-strategy ADR input.

What remains open for the runtime ADR is the WASM-specific detail: runtime engine selection, WASI
component model vs raw ABI, host-call surface, resource limits, and failure/trap handling.

## Plugin Manifest (Draft Sketch)

The exact schema is an open question, but a v1 manifest should at least carry:

```toml
# plugin manifest sketch — NOT final
[plugin]
name = "my-plugin"
version = "0.1.0"
talos_protocol = "0.1"           # host protocol range
carrier = "wasm"                 # wasm | lua
artifact = "artifacts/my-plugin.wasm"
description = "..."
permissions = { ... }            # declared host capabilities (fs/network/host-calls)

# optional atomic components shipped by this package
[[skills]]
name = "..."
path = "skills/..."

[[mcp]]
# inline MCP server definition, or pointer to an existing config block

[[hooks]]
event = "BeforeProviderCall"
handler = "hooks/pre_call.wasm"  # or script entry

[[tools]]
name = "do_thing"
schema = { ... }                 # JSON Schema for tool input
handler = "tools/do_thing.wasm"
permission = { ... }             # permission facet profile
```

Open: whether MCP inside a plugin reuses the existing `[[mcp.servers]]` transport or a new in-process
transport; whether skills inside a plugin are just bundled `SKILL.md` files or carry richer metadata.

## Required Decisions / ADRs

Before any implementation, these must be accepted:

1. **Plugin runtime boundary ADR** — carrier strategy (WASM first-class, Lua optional, dylib
   rejected per owner 2026-06-30), WASM runtime/sandbox model, permission integration, lifecycle,
   failure behavior, and host-call surface.
2. **ADR-009 extension** — add a `Plugin { name, version, carrier }` variant to `ToolProvenance`
   (additive, `#[non_exhaustive]` already in place).
3. **Atomic component model ADR** — formally establish skill/mcp/hook as the three config-introduced
   peer component types and define how plugins declare/override them.
4. **Command taxonomy ADR** — `/skills` `/mcp` `/plugins` as three peer commands; `/plugins` realigns
   to list actual plugin packages; existing MCP status moves to `/mcp`.

## Open Questions

1. WASM runtime engine selection (`wasmtime` vs other Rust-native runtimes) and WASI component model
   vs raw ABI — to be decided in the runtime boundary ADR. (Carrier *set* is settled; dylib is
   rejected.)
2. Does a plugin's MCP component reuse the existing stdio MCP transport, or does it get a new
   in-process transport optimized for bundled servers?
3. Are plugin-bundled skills just `SKILL.md` files, or do they need a richer manifest to express
   activation policy, dependency ordering, and provenance?
4. Hook config-ization: what does the user-facing hook config schema look like, and how are builtin
   hooks distinguished from config-introduced ones?
5. Should plugin-provided tools go through the existing `AgentTool` registry directly, or through an
   adapter layer that enforces carrier-specific timeout/memory/trap handling?
6. Distribution: are plugin packages locally installed only (v1), or do they go through `DIST-001`
   optional asset distribution from the start?
7. Filter as a capability type: keep as a peer of hook, or fold into hook with stronger ordering
   semantics (already open in PLUGIN-001)?
8. How does plugin provenance render in TUI/RPC alongside native and MCP tools?

## Dependencies (Blocked Items)

The following are blocked pending acceptance of this proposal and its ADRs:

- **PLUGIN-001** — repositioned from "WASM protocol" to "plugin encapsulation system". Absorbs this
  proposal. Blocked on ADR #1 and #3.
- **CMD-002** (new) — command taxonomy realignment (`/plugins` → `/mcp`, `/plugins` repurposed for
  plugin packages, `/hooks` added). Blocked on ADR #4 and on the plugin entity existing.
- **HOOK-001** (new) — user-config-introduced hooks. Blocked on ADR #3.
- **TOOL-008 Phase 3** — runtime parser loading via WASM. Blocked on the plugin runtime (Phase 3
  explicitly reuses PLUGIN-001 infrastructure).

Items noted as related but not fully blocked:

- **DIST-001** — plugin package *distribution* depends on the plugin format, but DIST-001's broader
  research scope (model weights, optional assets) can proceed independently. The plugin-package
  slice of DIST-001 is blocked on this proposal.
- **EXT-001** — provenance marker TUI work is already complete via I014; only the command *naming*
  is affected, which is captured by CMD-002.

## Relationship To Existing Artifacts

| Artifact | Relationship |
|---|---|
| `docs/proposals/wasm-runtime-plugin-protocol.md` | Subsumed as the WASM-carrier slice of this proposal. Retain for the protocol-message detail; the capability model and non-goals there are narrowed by this proposal, with dynamic library loading explicitly rejected. |
| [ADR-009](../decisions/009-tool-provenance.md) | Extended by ADR #2 above to add a `Plugin` provenance variant. |
| `crates/talos-plugin/` | Current hook/observation system. Becomes the hook atomic component substrate once HOOK-001 lands. |
| `crates/talos-mcp/` | Current MCP client/adapter. Stays as the MCP atomic component implementation. |
| `crates/talos-skill/` | Current skill discovery/index. Stays as the skill atomic component implementation. |
| [CMD-001](../backlog/active/CMD-001-interactive-command-runtime-contract.md) | Provides `CommandDefinition`/`PluginCommand` infrastructure the command taxonomy ADR builds on. |

## First Implementation Slice (After ADRs Accepted)

To be detailed in a follow-up iteration plan. Rough shape, mirroring the existing WASM proposal's
first slice but generalized:

1. Load one local plugin package from an explicit path with carrier = wasm.
2. Read manifest; register one read-only tool and (optionally) one bundled skill.
3. Execute that tool through the existing permission and provenance pipeline with the new
   `ToolProvenance::Plugin` variant.
4. Enforce timeout/output/memory limits; convert traps to tool errors without crashing the process.
5. Surface the plugin and its declared capabilities under the realigned `/plugins` command.

No marketplace, no network package installation, no dylib, no Lua in the first slice.
