# Iteration I122: Local Extension And Control Diagnostics

> Document status: Active — Gate 0 passed 2026-07-13 (I121 Complete)
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

### F120 — In Progress
