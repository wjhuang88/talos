# Iteration I136: Read-Only Plugin Product Closure

> Document status: Planned
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

## Verification Evidence

- Pending I135 completion and activation gate.

## Variance And Residuals

- Executable hooks and remote distribution remain separate owners.
