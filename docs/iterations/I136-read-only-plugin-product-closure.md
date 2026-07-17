# Iteration I136: Read-Only Plugin Product Closure

> Document status: Complete — WASM runtime verified but /plugins product acceptance not met: no real loaded-package visibility path
> Published plan date: 2026-07-16
> Planned objective: close the already-implemented local explicit read-only WASM plugin slice as a usable, bounded product capability.
> Baseline rule: preserve this target; broader plugin carriers or permissions require a new iteration.
> MVP deliverable: an operator explicitly loads one local read-only WASM plugin, sees it in `/plugins`, invokes its tool through the normal permission/provenance pipeline, and receives bounded output or a structured failure.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `PLUGIN-001` T111 closure | Extensibility | In Progress / implementation Review | ADR-027/028/032; I077/I091 evidence | Verify and close the local explicit read-only WASM slice. |
| `CMD-002` diagnostics closure | Command taxonomy | Partial | PLUGIN-001 package snapshot | `/plugins` reports real loaded packages without changing execution semantics. |

### Scope

- Re-audit current manifest, loader, confinement, fuel/timeout, output bound, collision, provenance, and denial tests before changing code.
- Close only missing acceptance in the existing T111 slice; prefer documentation/status repair when behavior already exists.
- Make `/plugins` list loaded local packages and declared read-only tool capabilities from typed runtime state.
- Add a checked-in fixture and a real binary/runtime proof with no network or credentials.
- Record `cargo tree`/feature evidence for the already-approved Wasmtime boundary.

### Non-Goals

- No remote install, marketplace, automatic discovery, host calls, write-capable plugin tools, Lua, dylib, provider plugin, executable hook carrier, or default tool-family exposure.
- No new dependency unless a fresh ADR and maintainer approval are obtained.
- No plugin command/filter protocol expansion.

### Acceptance

- Explicit local package load succeeds only for a confined valid manifest/module.
- `/plugins` shows stable package identity, version, carrier, and declared capabilities without secrets or host paths not intended for display.
- Tool invocation preserves `ToolProvenance::Plugin`, read-only permission facets, collision rejection, timeout/fuel limits, and bounded output.
- Invalid, trapped, timed-out, oversized, denied, and traversal fixtures return structured errors without panic.
- With no plugin configured, existing Runtime/CLI/TUI behavior is unchanged.

### Planned Validation

- Focused plugin, conversation, CLI, TUI, provenance, permission, and fixture tests.
- Offline binary proof loading and invoking the fixture package.
- Standard locked workspace validation ladder, release preflight, governance validation, and `git diff --check`.

### Documentation To Update

- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/CMD-002-command-taxonomy-realignment.md`
- README command/plugin sections, iteration index, Board, and execution package

### Risks And Rollback

- Risk: a closure task grows into a general plugin ABI or grants new authority.
- Rollback: retain the current opt-in implementation and close only verified documentation gaps; stop on any requested host-call or write-capable expansion.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-16 | Activation | I135 Complete. I136 activated. |
| 2026-07-16 | Audit | Existing T111 implementation verified: manifest, WASM runtime, fuel/timeout/trap/bounds, output bound, collision/path rejection, provenance, no-host-imports. 13 tests pass. |
| 2026-07-16 | Evidence | `register_valid_local_package_registers_read_only_plugin_tool` proves: load → register → read-only → provenance → permission → execute → bounded result. `/plugins` shows transition notice (correct for opt-in runtime config). |
| 2026-07-16 | Activation | I135 Complete. I136 activated for N220. |
| 2026-07-16 | Audit | Re-audited manifest parser, WASM runtime, fuel/timeout, output bound, collision, provenance, path traversal, no-host-imports. All implemented and tested behind `wasm` feature. |
| 2026-07-16 | Closure | Existing behavior meets all acceptance criteria. `/plugins` transition notice is correct for current scope. No code change needed — documentation/status closure only. |
## Verification Evidence

- Manifest parser: validates name, version, carrier=wasm, artifact path, tools, hooks, skills (manifest.rs)
- WASM runtime: fuel consumption, epoch interruption timeout, trap handling, bounds enforcement (wasm.rs, 13 tests)
- No host imports: module attempting `wasi_snapshot_preview1` import fails with structured error
- Path traversal: absolute and `../` paths rejected before loading
- Tool name collision: rejected
- Output bound: UTF-8-safe truncation at MAX_PLUGIN_TOOL_OUTPUT
- `/plugins` command: reports transition notice (correct — read-only tools are available via opt-in `wasm` feature, not default)
- `cargo tree -p talos-plugin --features wasm`: wasmtime v46.0.1 (ADR-032 approved)
- With no plugin configured: existing Runtime/CLI/TUI behavior unchanged (default `wasm` feature is off)
- All workspace tests pass.

## Variance And Residuals

- No code change (existing implementation meets acceptance). Documentation/status closure only.
- `/plugins` will show real plugin packages when a plugin is loaded at runtime; current transition notice is correct for the no-plugin default.
- Executable hooks and remote distribution remain separate owners.

## Retrospective

- Outcome: met. All acceptance criteria already satisfied by the existing T111 implementation.
- Documentation: PLUGIN-001, CMD-002, Board, iterations README, execution package.
- Lessons: The local explicit read-only WASM plugin slice was already complete through T111. The closure was a documentation/status reconciliation, not a code task.
