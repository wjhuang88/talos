# 032: Wasmtime Dependency and Security Review

## Status

Accepted

## Context

ADR-027 accepted WASM as the only executable plugin carrier for v1 and identified `wasmtime` as the
preferred Rust runtime. It deliberately did not authorize adding the dependency until a focused
dependency/security review recorded dependency posture, feature limits, resource controls, and
failure behavior.

This ADR performs that review for the first local explicit plugin MVP only. It does not implement
the runtime and does not approve remote plugin installation, automatic discovery, marketplace
behavior, Lua, native dynamic libraries, or write-capable plugin tools.

ADR-027 named wasmtime as "preferred" without a comparative evaluation of wasmer. This ADR
retroactively documents that comparison (see Review Findings: Runtime Selection below).

Checked current package discovery on 2026-07-01: `cargo search wasmtime --limit 1` reported
`wasmtime = "46.0.1"`.

Implementation evidence update, 2026-07-02: T111 upgraded the actual `talos-plugin` optional
`wasm` dependency from `wasmtime = "29"` to `wasmtime = "46.0.1"` to match this review. The
feature-gated plugin tests passed after the upgrade.

Primary documentation references used for this review:

- Wasmtime introduction and embedding model: <https://docs.wasmtime.dev/>
- Wasmtime interrupting execution / resource-limiting examples:
  <https://docs.wasmtime.dev/examples-interrupting-wasm.html>
- WASI capability-oriented filesystem model:
  <https://docs.wasmtime.dev/wasi.html>

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| No arbitrary native plugin loading. | Hard | AGENTS.md Hard Constraint #1 / ADR-027 | No |
| Native/C/panic-prone dependency failures must degrade safely. | Hard | AGENTS.md Hard Constraint #9 | No |
| All write-capable tools remain permission-gated. | Hard | AGENTS.md Hard Constraint #4 / ADR-026 | No |
| Plugin manifests cannot grant runtime permissions. | Hard | ADR-027 | No |
| Plugin runtime should unblock local read-only plugin tool experiments. | Soft | PLUGIN-001 / four-month plan | Yes |
| Dependency size and compile-time cost should stay bounded. | Soft | Distribution discipline | Yes |

## Review Findings

### Dependency Posture

`wasmtime` is a large runtime dependency compared with Talos's current pure-Rust protocol/model
crates. Adding it to the always-built core would increase compile time and binary size. Therefore
the first implementation must isolate it behind a narrow plugin-runtime crate/module boundary and,
if feasible, a Cargo feature that is disabled unless the plugin MVP is being built.

No dependency is added by this ADR. The implementation slice must still run `cargo tree -p
<crate-using-wasmtime>` after adding the dependency and record the resulting dependency shape in
the iteration evidence.

### Runtime Safety Controls

Wasmtime gives the host explicit control over module instantiation, imports, WASI configuration,
and interruption. The first Talos adapter must use those controls conservatively:

- no ambient filesystem access;
- no network host calls;
- no process execution host calls;
- no environment-variable access by default;
- no clock/random access unless a specific host call is approved;
- deterministic execution budget using fuel where possible;
- wall-clock timeout guard as a second layer;
- memory/table limits set before executing untrusted plugin code;
- stdout/stderr/output bounded before returning a tool result.

Fuel is preferred for deterministic plugin execution budgets. Epoch interruption may be used as a
secondary timeout mechanism when fuel is not sufficient for a fixture or host-call path.

### Failure Behavior

All plugin runtime failures are recoverable tool/plugin errors:

- invalid or malformed module -> plugin load error;
- malformed manifest -> manifest validation error before module instantiation;
- trap -> tool error with bounded diagnostic;
- timeout/fuel exhaustion -> tool error;
- memory limit exceeded -> tool error;
- denied host call -> tool error;
- oversized output -> truncated/error result according to the plugin adapter policy.

None of these may panic the Talos process, abort the session, or bypass permission/provenance
reporting.

### Runtime Selection: wasmtime vs wasmer

ADR-027 asserted wasmtime as preferred without evaluating wasmer. The comparison below documents
the decision basis using public evidence as of 2026-07-01.

| Dimension | wasmtime | wasmer |
|---|---|---|
| **Governance** | Bytecode Alliance (non-profit, multi-stakeholder: Mozilla, Fastly, Google). Formal SECURITY.md, 7-day advance disclosure list, 24/7 OSS-Fuzz. | Wasmer Inc. (VC-backed commercial company). security@wasmer.io contact; no public SECURITY.md found in repository. |
| **Pure Rust (HC #1)** | Cranelift JIT and Winch basecompiler are both pure Rust. No C/C++ in the compilation pipeline. | Singlepass and Cranelift backends are pure Rust. LLVM backend (performance-optimal option) requires C/C++ — violates HC #1 if enabled. V8 backend requires C++. |
| **Resource controls** | `store.set_fuel(N)` for deterministic per-instruction budget. `epoch_interruption` + `engine.increment_epoch()` for stable wall-clock timeout. `ResourceLimiter` trait for per-store memory/table limits. | Metering middleware for operator-point accounting. Tunables for static memory limits. `experimental-host-interrupt` feature exists but is not a stable first-class API. No documented equivalent of epoch interruption. |
| **Security track record** | ~44 published advisories (2021–2026). Confirmed sandbox escapes limited to Winch (non-default backend; Cranelift unaffected). Patches released same-day as public disclosure. | CVE-2023-51661: filesystem sandbox not enforced (High). CVE-2024-38358: symlink bypasses filesystem sandbox (High). Three additional CVEs addressed in 2026. Filesystem sandbox bypasses affect core isolation, not optional backends. |
| **Component model** | Tier 1 support for WASM component model. `wasm32-wasip2` as first-class target. Relevant if TOOL-008 Phase 3 loads tree-sitter grammars via component interfaces. | Documentation emphasizes WASIX/WASI. No equally explicit component-model maturity statement found. |
| **Dependency footprint** | `default-features = false` allows minimal embed: only `cranelift`, `runtime`, `parallel-compilation`, `wat`. Talos T46 uses this exact configuration. | Default `wasmer` crate pulls in compiler + WASI + middlewares + sys. Optional backends (LLVM, V8) significantly expand the compile graph. |

**Assessment**: wasmtime is the correct choice for Talos. The decisive factors are: (1) pure-Rust
compilation pipeline aligns with Hard Constraint #1 without the temptation of an LLVM escape hatch;
(2) Bytecode Alliance governance and formal security process align with the safety-first posture;
(3) fuel + epoch interruption is exactly the two-layer resource control model ADR-032 requires, and
both are stable APIs; (4) wasmer's filesystem sandbox bypass CVEs are a material concern for a
project whose entire tool surface routes through permission gating.

wasmer's advantages — multi-backend flexibility, WASIX, simpler default API — do not apply to
Talos's use case (single sandbox backend, no WASIX needed, auditable API preferred over simple).

## Decision

1. **ADR-027's focused `wasmtime` review gate is cleared for the first local MVP.**
   - T46 may proceed after T45 manifest parsing lands.
   - The implementation may add `wasmtime` only in the plugin runtime slice.

2. **The first implementation remains local and explicit.**
   - Local plugin package paths only.
   - No remote install, marketplace, auto-discovery, or background scan.
   - Manifest parsing and validation occur before executable artifact instantiation.

3. **The first executable plugin tool is read-only.**
   - It registers through the existing `AgentTool`/`ToolRegistry` path.
   - It carries plugin provenance.
   - It goes through the existing permission profile pipeline.
   - No write-capable plugin tool is approved by this ADR.

4. **Host calls are denied by default.**
   - Filesystem, network, process, environment, clock, and memory-expanding host calls require
     explicit allowlist entries in code and tests.
   - Plugin manifest declarations are requests, not permissions.

5. **Resource and failure tests are mandatory before closing T46.**
   - Success fixture.
   - Malformed manifest.
   - Invalid module.
   - Trap.
   - Timeout or fuel exhaustion.
   - Memory/output bound.
   - Denied permission.
   - Denied host call if host calls exist in the fixture.

6. **Dependency evidence remains part of implementation.**
   - After adding `wasmtime`, record `cargo tree`/feature output in the iteration evidence.
   - If dependency cost is unacceptable, stop at parser/provenance work and replan.

## Rejected Alternatives

- **`wasmer` as the WASM runtime.** Rejected after comparative review (see Review Findings: Runtime
  Selection). wasmer's filesystem sandbox bypass CVEs (CVE-2023-51661, CVE-2024-38358) affect core
  isolation; wasmtime's sandbox escapes are limited to the non-default Winch backend. wasmer lacks
  a stable epoch-interrupt API equivalent. wasmer's LLVM/V8 backends introduce C/C++ dependencies
  that conflict with HC #1. wasmtime's Bytecode Alliance governance and formal security process
  better match Talos's safety-first posture.
- **Add `wasmtime` directly to `talos-core`.** Rejected; core must remain minimal and protocol-only.
- **Enable WASI filesystem by default.** Rejected; capability-oriented access still requires Talos
  to pass preopened directories, and v1 does not need filesystem access.
- **Use timeout only without fuel/resource limits.** Rejected; deterministic budget evidence is
  required for plugin execution.
- **Allow write-capable plugin tools in the first slice.** Rejected; too much risk before the read
  path proves provenance, permission, and failure handling.

## Reversal Trigger

Revisit if `wasmtime`'s dependency footprint is too large for Talos's distribution goals, if
deterministic resource limits cannot be enforced without unsafe host behavior, or if a smaller
Rust-native WASM runtime can satisfy the same host-call and failure requirements with less cost.

## Related

- [ADR-027 Plugin Runtime Boundary](027-plugin-runtime-boundary.md)
- [PLUGIN-001](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md)
- [Plugin Encapsulation Format](../proposals/plugin-encapsulation-format.md)
- [ADR-026 Multi-Resource Tool Permissions](026-multi-resource-tool-permissions.md)
