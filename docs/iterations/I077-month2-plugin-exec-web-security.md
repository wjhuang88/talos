# Iteration I077: Month 2 — Plugin, Exec, And Web Security

> Document status: Active (2026-07-01)
> Published plan date: 2026-07-01
> Planned objective: Execute weeks 5-8 of the 2026-07-01 replan: plugin MVP security review,
> read-only plugin tool integration if cleared, WEB-001/WEB-005 security review, and direct exec
> permission policy plus implementation if cleared.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: explicit security decisions for plugin/web/exec boundaries with tests for any
> cleared implementation.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| T110 | PLUGIN-001 | Complete | T46/ADR-032 | Plugin MVP security review |
| T111 | PLUGIN-001 | Review | T110 | Read-only plugin AgentTool if cleared |
| T112 | WEB-001/WEB-005 | Complete | T42/T47 | Web/browser security review |
| T113 | WEB-001/WEB-005 | Review | T112 | Hardening fixes |
| T114 | TOOL-016/PERM-001 | Planned | Issue #16 | Exec permission policy |
| T115 | TOOL-016 | Planned | T114 | Direct exec tool if cleared |
| T116 | Replan | Planned | T110-T115 | Month-2 closeout |

### Scope

- Plugin, web/browser, and exec security reviews.
- Implementation only after the relevant security gate clears.

### Non-Goals

- No write-capable plugin tools.
- No remote dashboard access.
- No default-allow process execution.
- No real publish.

### Acceptance

- Given plugin runtime is reviewed, when T111 starts, then permission/provenance gaps are closed or implementation is deferred.
- Given dashboard/browser review completes, when fixes land, then no secret leakage or auth bypass exists.
- Given exec policy is accepted, when `exec` runs, then command, cwd, env, and timeout are permission-gated.

### Planned Validation

- Targeted plugin/dashboard/tools/permission tests
- `cargo test --workspace` at T116
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- Plugin/web/exec owner docs
- ADR or owner-doc decision for exec policy
- Issue #16 status comments
- `docs/BOARD.md`

### Risks And Rollback

- Risk: plugin or exec broadens execution authority. Rollback: keep adapters non-presented and mark blocked.
- Risk: dashboard grows beyond loopback MVP. Rollback: keep API/root index only.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-01 | Planning | Created as Month 2 shell for the replan. |
| 2026-07-01 | Activation | Activated after I076/T109 closeout. First packet is T110 plugin MVP security review. |
| 2026-07-01 | Review | T110 completed `docs/reference/PLUGIN-MVP-SECURITY-REVIEW-2026-07-01.md`. T111 is cleared only for a local explicit read-only fixture plugin tool with package-root confinement, provenance, permission pipeline, and bounded output tests. |
| 2026-07-02 | Review | T111 implemented a feature-gated local explicit read-only WASM plugin `AgentTool` registration path with plugin provenance, package-root confinement, read permission facets, output bounds, collision rejection, and no host calls. |
| 2026-07-02 | Review | T112 produced `docs/reference/WEB-DASHBOARD-BROWSER-SECURITY-REVIEW-2026-07-02.md`; T113 fixed dashboard boundary redaction, authenticated fallback coverage, and browser-page selected-link URL sanitization. |

## Verification Evidence

- 2026-07-01: `cargo tree -p talos-plugin --features wasm` recorded the actual optional dependency tree. It uses `wasmtime v29.0.1`, which T111 must reconcile against ADR-032's current-version discovery.
- 2026-07-01: `cargo test -p talos-plugin --features wasm` passed: 24 unit tests, 8 integration tests, 0 doc tests.
- 2026-07-02: `cargo search wasmtime --limit 1` confirmed latest `wasmtime = "46.0.1"`.
- 2026-07-02: `cargo update -p wasmtime` upgraded `wasmtime v29.0.1 -> v46.0.1`.
- 2026-07-02: `cargo test -p talos-plugin --features wasm wasm::tests` passed: 15 WASM tests.
- 2026-07-02: `cargo test -p talos-plugin --features wasm` passed after T111: 30 unit tests, 8 integration tests, 0 doc tests.
- 2026-07-02: `cargo test -p talos-core` passed: 34 unit tests, 0 doc tests.
- 2026-07-02: `cargo clippy -p talos-plugin -p talos-core --features talos-plugin/wasm -- -D warnings` passed.
- 2026-07-02: `cargo tree -p talos-plugin --features wasm` recorded `wasmtime v46.0.1`.
- 2026-07-02: `cargo test -p talos-dashboard` passed after T113: 14 unit tests, 0 doc tests.
- 2026-07-02: `cargo test -p talos-tools browser_page` passed after T113: 9 selected unit tests.
- 2026-07-02: `cargo test -p talos-tools fetch_url` passed: 11 selected unit tests.
- 2026-07-02: `cargo clippy -p talos-dashboard -p talos-tools -- -D warnings` passed.

## Variance And Residuals

- T111 resolved the `wasmtime` version mismatch by upgrading to `46.0.1`.
- T111 implemented artifact/handler package-root confinement for the local fixture path.
- The current per-call timeout watchdog still leaves a sleeping thread until timeout after successful execution. This remains acceptable only for bounded fixture use; broader plugin presentation must replace or cap it.
- T113 fixed review findings without authorizing remote dashboard access, web writes/actions, browser connectors, browser automation, standalone browser tools, or permission-default changes.
- T114 remains planned next for direct exec permission policy.

## Retrospective

- Pending.
