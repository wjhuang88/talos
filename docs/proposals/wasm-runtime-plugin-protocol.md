# WASM Runtime Plugin Protocol

> Backlog: PLUGIN-001
> Status: Research proposal

## Intent

Create a future Talos plugin runtime where third-party plugins run in WASM and register capabilities
through a stable protocol rather than linking directly against Talos internals.

The protocol is the deliverable before implementation. It must be stable enough that plugin authors
can build against it and strict enough that Talos can keep its permission, provenance, and event
architecture intact.

## Capability Model

Plugins may provide these capability kinds:

| Capability | Purpose | Existing Talos boundary |
| --- | --- | --- |
| Tool | Adds callable operations to the agent tool registry. | `AgentTool`, permission pipeline, `ToolProvenance`. |
| Command | Adds user-invoked operations to the session command registry. | CMD-001 `PluginCommand`, command provenance, typed execution adapters. |
| Hook | Observes or modifies lifecycle events. | `talos-plugin::HookHandler`. |
| Filter | Applies deterministic transformations or policy decisions over messages, context, tool
  inputs, or tool outputs. | Related to hooks, but may need stricter ordering and error policy. |

## Protocol Shape

The exact encoding is undecided, but v1 should define these message families:

- `plugin.manifest`: identity, version, API compatibility, declared capabilities.
- `plugin.initialize`: host version, workspace metadata, allowed host calls, resource limits.
- `plugin.register`: capability descriptors for tools, commands, hooks, and filters.
- `tool.execute`: tool input, call id, permission/provenance metadata.
- `command.execute`: namespaced command id, validated arguments, invocation context, and bounded result.
- `hook.invoke`: hook event payload and mutable/immutable fields.
- `filter.apply`: filter input and expected output contract.
- `plugin.shutdown`: graceful unload.
- `plugin.error`: structured trap, timeout, malformed response, or policy denial.

## Safety Requirements

- Plugin-provided tools must go through the same permission pipeline as built-in tools.
- Plugin commands must be namespaced, cannot override built-in commands, and cannot emit arbitrary
  Session/TUI events. Any host operation they request follows the owning capability and permission
  boundary.
- Every plugin capability must carry provenance visible to TUI/RPC consumers.
- Host calls are deny-by-default and explicitly allowlisted.
- Filesystem and network access are disabled unless explicitly granted by a future ADR-backed
  policy.
- Timeouts, memory limits, and output size limits are mandatory.
- A plugin trap must not crash the Talos process.
- Malformed plugin messages degrade to structured errors.

## Versioning

The protocol must include:

- host protocol version;
- plugin requested protocol range;
- capability schema version;
- optional feature flags;
- clear behavior for incompatible plugins.

## Candidate Runtime Questions

Before implementation, evaluate:

- WASI component model vs raw WASM ABI.
- `wasmtime` vs other Rust-native runtimes.
- Whether plugin schemas use JSON Schema, wit/interface types, or Talos-owned Rust-generated DTOs.
- Whether hooks and filters share one ordered chain or remain separate capability classes.

## Non-Goals

- No plugin marketplace in v1.
- No network package installation in the first implementation slice. Runtime-downloadable plugin
  packages require the shared optional asset distribution strategy from `DIST-001` and a follow-up
  ADR.
- No native dynamic library loading.
- No Node.js or Python plugin runtime.
- No provider plugin execution unless a later ADR expands the scope.

## First Implementation Slice After ADR

1. Load one local WASM plugin from an explicit path.
2. Read manifest and register one read-only tool.
3. Execute that tool through the existing permission and provenance pipeline.
4. Enforce timeout/output limits and convert traps to tool errors.
5. Add a TUI/RPC provenance marker for WASM plugin tools.
