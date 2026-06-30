# 027: Plugin Runtime Boundary

## Status

Accepted

## Context

`PLUGIN-001` was blocked because the previous plugin proposal scoped plugins as a WASM-only idea
without deciding the runtime boundary, carrier set, sandbox posture, or native-code non-goals. The
2026-06-30 plugin encapsulation proposal reframed plugin as a package format that can bundle
skills, MCP declarations, hooks, and plugin-provided tools. A decision is needed before any plugin
runtime implementation starts.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| No arbitrary C/C++ bindings or native plugin loading. | Hard | AGENTS.md Hard Constraint #1 | No |
| All write-capable tools remain permission-gated. | Hard | AGENTS.md Hard Constraint #4 | No |
| Sandbox/process-hardening changes need security review. | Hard | AGENTS.md Hard Constraint #5 | No |
| Native/C/panic-prone dependency failures must degrade safely. | Hard | AGENTS.md Hard Constraint #9 | No |
| Plugin packages should eventually distribute optional capabilities. | Soft | PLUGIN-001 / DIST-001 | Yes |
| First slice should unblock TOOL-008 and plugin-tool experimentation. | Soft | TOOL-008 / WEBFETCH future handlers | Yes |

## Reasoning

Dynamic libraries are not a sandbox boundary. Loading `.so`, `.dll`, or `.dylib` code gives the
loaded code process privileges, which conflicts with Talos's Rust-first and safety-first posture.
Treating dylib as a "trusted escape hatch" would make plugin provenance look safer than it is.

WASM is the only acceptable first-class carrier for untrusted or semi-trusted plugin code because
it can be run with explicit host calls, bounded resources, and no ambient filesystem/network access.
Lua may be useful later as a lightweight scripting carrier, but it requires a separate dependency
and sandbox ADR before activation.

`wasmtime` is the preferred first implementation runtime because it is the mature Rust-native WASM
runtime with WASI support and explicit host-call integration. The implementation still needs a
focused dependency review before adding it to `Cargo.toml`; this ADR authorizes the direction, not
a blind dependency landing.

## Decision

1. **Plugin v1 uses WASM as the only executable carrier.**
   - The first implementation slice loads local plugin packages from explicit paths only.
   - No marketplace, remote package install, or automatic discovery in v1.

2. **`wasmtime` is the preferred runtime for the first WASM implementation slice.**
   - The implementation must record dependency weight, feature flags, and failure behavior before
     adding it.
   - Traps, timeouts, memory exhaustion, invalid modules, and host-call errors become tool/plugin
     errors, never process exits.

3. **Dynamic library loading is rejected.**
   - No `.so`, `.dll`, or `.dylib` plugin carrier.
   - Fully trusted host extensions should be built into Talos or exposed through MCP, not loaded
     into-process as native libraries.

4. **Lua is deferred.**
   - Lua requires a separate ADR covering interpreter choice, restricted stdlib, host-call
     allowlist, and failure behavior.

5. **Plugin host calls are explicit capabilities.**
   - Filesystem, network, process, memory, clock, and environment access are denied by default.
   - Tool execution still goes through `talos-permission`; plugin manifests cannot grant runtime
     permissions by themselves.

6. **Plugin runtime is an adapter layer, not a second tool pipeline.**
   - Plugin tools register through the existing `AgentTool`/`ToolRegistry` path.
   - The adapter owns carrier-specific timeout, trap, memory, and output bounding behavior.

## Rejected Alternatives

- **Dynamic library carrier.** Rejected because it cannot be sandboxed and violates the safety
  boundary.
- **MCP-only plugins.** Too narrow; useful for process-isolated tools but does not solve bundled
  skills/hooks or in-process parser/runtime modules.
- **Lua first.** Lighter weight but weaker isolation story and still needs sandbox design.
- **Remote plugin packages in v1.** Deferred to `DIST-001`; local explicit packages are enough to
  validate the runtime boundary.

## Implementation Guardrails

- Add the runtime dependency only in a focused implementation slice with `cargo tree`/feature
  review and security notes.
- Plugin execution must be testable with fixtures that cover success, trap, timeout, malformed
  module, oversized output, and denied permission.
- No plugin-originated capability may bypass provenance or permission reporting.
- No plugin package may auto-run code at discovery time; manifest parsing and validation happen
  before any executable artifact is instantiated.

## Reversal Trigger

Revisit if a Rust-native WASM runtime cannot satisfy timeout/memory/host-call constraints, or if a
future OS-level sandbox gives native plugins a real isolation boundary without violating Hard
Constraint #1.

## Related

- [PLUGIN-001](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md)
- [Plugin Encapsulation Format](../proposals/plugin-encapsulation-format.md)
- [ADR-009](009-tool-provenance.md)
- [ADR-026](026-multi-resource-tool-permissions.md)
