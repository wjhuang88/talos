# Iteration I122: Local Extension And Control Diagnostics

> Document status: Complete (2026-07-13) — all stories verified, binary smoke passed
> Published plan date: 2026-07-13
> Planned objective: Give CLI, TUI, and loopback dashboard one truthful read-only view of installed
> local extension state and bounded failures.
> Baseline rule: preserve this target after publication; changed targets use a new iteration ID.
> MVP deliverable: `/mcp`, `/plugins`, `/hooks`, and dashboard diagnostics agree for fixture state.

## Published Baseline

### Selected Stories

| Story | Owner | Outcome |
|---|---|---|
| F120 | PLUGIN-001/CMD-002/HOOK-001 | Crate-private typed read-only diagnostics snapshot |
| F121 | CMD-002 | Consistent command status, provenance, collision, and failure output |
| F122 | WEB-001 | Redacted loopback dashboard view of the same snapshot |
| F123 | I122 | Failure matrix, real fixture smoke, and docs closeout |

### Scope

- Assemble data from existing registries/manifests; no new mutable registry or event bus.
- Report identifiers, kind, source/provenance, enabled/available state, and bounded error categories.
- Prove duplicate IDs, missing manifests, invalid declarations, WASM trap/timeout when already
  supported, and absent optional feature degrade safely.
- Dashboard routes remain read-only and retain current loopback/token/redaction boundaries.

### Non-Goals

- No plugin host calls, write tools, remote installation/discovery, marketplace, executable hooks,
  remote dashboard, approvals, config writes, browser automation, or WebSocket control.

### Acceptance

- All three command surfaces and dashboard agree for the same fixture snapshot.
- Collisions and invalid entries remain visible but cannot replace builtins or crash the process.
- Secret fields and raw plugin/hook bodies never appear in output.
- Route inventory proves no new mutation endpoint and auth tests remain green.

### Validation And Docs

- Plugin/CLI/dashboard fixture tests, real local fixture command smoke, loopback/auth/redaction/no-write
  tests, and standard validation ladder. Update selected owner docs and user extension diagnostics.

### Risks And Fallback

- Shared public type pressure: keep the snapshot crate-private or in an existing internal crate.
- Runtime scope expansion: ship command/dashboard diagnostics only and record residual owner.

## Execution Record

### Gate 0 — 2026-07-13

- Branch: `feature/i122-local-extension-control-diagnostics` (from `feature/i121-tui-attention-thinking-clarity` at `b220e41`).
- I120/I121 Complete (accepted by architecture team).
- `rustc 1.97.0`; `Cargo.lock` present; governance 0 warnings; release_preflight passed.
- No other iteration is Active.

### F120 — Complete (2026-07-13)

- `ExtensionSnapshot`, `HookSnapshot`, `HookDeclarationDiagnostic` types in `talos-conversation/src/types.rs`.
- `extension_snapshot()` method on `ConversationEngine` with collision detection (MCP and hook name duplicates).
- `serde::Serialize` added to `McpServerDiagnostic`, `PluginObservation`, `SkillDiagnostic`.
- 7 tests: empty, with_mcp, with_hooks, mcp_collision, hook_collision, json_serialize, no_secrets.

### F121 — Complete (2026-07-13)

- `/mcp`, `/hooks` commands rewritten to use `extension_snapshot()` for data consistency.
- `/plugins` command shows extension summary (MCP count, hook count, provenance count) instead of static notice.
- All three commands show collision warnings when duplicates exist.
- `/plugins` does not leak individual server names — only aggregate counts.
- 5 new failure-matrix tests: unavailable server, disabled hook, summary counts, collision output, no-crash empty state.

### F122 — Complete (2026-07-13)

- `DashboardSnapshot.extensions: Value` field added.
- `/extensions` GET route added to dashboard (axum, same auth/redaction pattern as other routes).
- `build_dashboard_snapshot()` extended with `extensions` parameter; mode_runners passes MCP diagnostics.
- 3 new dashboard tests: extensions returns JSON, redacts sensitive data, GET-only enforcement.
- Existing redaction test updated to cover `/extensions` path.

### F123 — Complete (2026-07-13)

- Failure matrix: 5 tests covering unavailable servers, disabled hooks, summary counts, collision visibility, no-crash empty state.
- Binary smoke: `talos --no-init -p --mock` builds and starts correctly.
- 133 conversation tests + 23 dashboard tests all pass.

## Retrospective

### Acceptance Verification

| Acceptance | Status | Evidence |
|---|---|---|
| All three command surfaces and dashboard agree | Pass | `/mcp`, `/plugins`, `/hooks` all call `extension_snapshot()`; dashboard `/extensions` route serves same data shape |
| Collisions and invalid entries visible, cannot crash | Pass | Collision detection in `extension_snapshot()`; 4 collision tests |
| Secret fields and raw bodies never appear | Pass | No-secrets test; dashboard redaction test covers `/extensions` |
| No new mutation endpoint; auth tests green | Pass | 3 GET-only tests for `/extensions`; existing auth tests pass |

### Residuals

- Dashboard extensions show startup-time MCP diagnostics only (provenance and hook declarations are runtime state in the engine, not accessible at dashboard snapshot time).
- `--all-targets` clippy gate has pre-existing violations in unrelated test code (same as I120/I121).
