# Plugin MVP Security Review

**Status**: Review complete for I077/T110; T111 controls implemented
**Date**: 2026-07-01
**Scope**: `PLUGIN-001` first local explicit read-only WASM plugin MVP
**Related**: ADR-027, ADR-028, ADR-029, ADR-030, ADR-032, I077/T110-T111

## Decision

T111 may proceed only as a bounded local read-only plugin `AgentTool` slice. The slice is cleared
to integrate a fixture WASM module through the existing `ToolRegistry` and permission pipeline if
it satisfies the required controls below.

This review does not authorize:

- remote plugin installation;
- marketplace behavior;
- automatic plugin discovery;
- write-capable plugin tools;
- filesystem, network, process, environment, clock, or random host calls;
- plugin commands or hooks executed from WASM;
- permission grants from manifests;
- default presentation of plugin tools without an explicit policy gate.

## Current Evidence

- `talos-plugin` keeps `wasmtime` behind optional feature `wasm`.
- `cargo test -p talos-plugin --features wasm` passed on 2026-07-01: 24 unit tests, 8 integration
  tests, 0 doc tests.
- `cargo tree -p talos-plugin --features wasm` was recorded on 2026-07-01 and again during T111.
  T111 upgraded the actual dependency to `wasmtime v46.0.1` with `default-features = false` and
  features `cranelift`, `runtime`, `parallel-compilation`, and `wat`.
- Existing WASM tests cover success, invalid module, trap, fuel exhaustion, timeout, memory access
  trap, missing export, and denied imports.
- Existing manifest tests cover malformed TOML, missing plugin section, non-WASM and dylib carrier
  rejection, duplicate tool names, empty tool/skill fields, and "manifest permissions do not grant"
  behavior.

## Findings

| Finding | Severity | Assessment | Required T111 Action |
|---|---|---|---|
| Optional feature boundary is correct. | Low | `wasmtime` is not in the default feature set. | Preserve feature gate; do not add `wasmtime` to `talos-core`, CLI default paths, or always-on builds. |
| Runtime version differs from ADR discovery. | Resolved | ADR-032 recorded current discovery as `wasmtime = "46.0.1"`, while code used `wasmtime v29.0.1` at review time. | T111 upgraded the actual dependency to `wasmtime v46.0.1` and reran feature-gated plugin tests. |
| No host imports are available. | Low | Current adapter instantiates modules with an empty import list. This matches ADR-032 for the first slice. | Keep all host calls denied. If any host call appears, stop and create a follow-up ADR/test gate. |
| Timeout guard creates a sleeping thread per execution. | Medium | Successful execution does not wait for timeout, but the watchdog thread sleeps until timeout after each call. This is acceptable for a low-volume fixture, not for unbounded plugin execution. | T111 must cap plugin execution concurrency or replace the per-call sleeping thread before presenting plugin tools broadly. |
| Artifact paths are not yet confined. | High | Manifest parser validates strings but does not resolve or confine `plugin.artifact` or `tools[].handler` to a package root. Loading paths without confinement would allow path traversal or absolute-path surprises. | Reject absolute paths and `..` escapes before loading any artifact. Add tests for path escape, absolute paths, missing files, and package-root confinement. |
| Manifest permissions are ignored, not retained. | Medium | This is safe because manifests cannot grant permissions, but diagnostics will not show what the package requested. | T111 may continue to ignore permission declarations for authority, but should surface a bounded diagnostic if permission-like fields are present. |
| AgentTool integration does not exist yet. | High | Existing WASM adapter returns `i32`; it is not connected to `AgentTool`, `PermissionProfile`, tool presentation, or `ToolProvenance::Plugin`. | Implement a small adapter that sets plugin provenance, read-only nature, explicit permission profile, bounded output, and normal tool errors. |
| Tool naming and collisions are not enforced at registration. | High | Manifest only deduplicates tool names inside one plugin. It does not compare against native/MCP/tool registry names. | Reject collisions with existing tools and reserve built-in names. Prefer `plugin_name.tool_name` or an explicit collision error. |
| Output contract is too narrow. | Medium | `run() -> i32` is enough for a fixture but not a useful tool protocol. Returning arbitrary plugin output later will need bounds. | T111 may map `i32` to a bounded fixture result. Any string/JSON output path must add byte limits and malformed-output tests first. |

## Required Controls For T111

- Load only local explicit plugin package paths.
- Parse and validate the manifest before compiling or instantiating any WASM artifact.
- Confine all manifest artifact and handler paths to the package root.
- Register at most a read-only fixture tool through `AgentTool`/`ToolRegistry`.
- Set `ToolProvenance::Plugin { name, version, carrier: "wasm" }` on every plugin tool result path.
- Use the existing permission pipeline. Manifests may request capability metadata, but they do not
  grant permissions.
- Keep host imports empty.
- Bound runtime by fuel and wall-clock timeout.
- Convert compile, instantiate, missing export, trap, timeout, denied import, oversized output, and
  permission denial into recoverable tool/plugin errors.
- Do not default-present plugin tools until policy says they are visible.

## Required T111 Tests

- Valid local package registers one read-only plugin tool with plugin provenance.
- Malformed manifest fails before WASM compile.
- Absolute artifact path and `..` artifact path are rejected.
- Invalid module returns recoverable tool/plugin error.
- Trap returns recoverable tool/plugin error.
- Fuel exhaustion or timeout returns recoverable tool/plugin error.
- Missing `run` export returns recoverable tool/plugin error.
- Import request is denied because host calls are absent.
- Plugin tool permission denial prevents execution.
- Tool-name collision with an existing native tool is rejected.
- Output is bounded.

## Residuals

- Broad plugin protocol output/input shape remains unresolved beyond the fixture.
- Runtime version selection must be reconciled before T111 can close.
- Host-call design remains explicitly out of scope.
- Plugin commands, hooks, filters, package signing, remote install, and marketplace flows remain
  future work.
