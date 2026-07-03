# Iteration I091: Plugin, Hook, And Distribution Boundary

> Document status: Active
> Published plan date: 2026-07-04
> Planned objective: harden local plugin/hook diagnostics and optional distribution policy without
> enabling remote install, write-capable plugin tools, or unsafe package behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: local diagnostic/runtime evidence and a distribution policy that preserve
> confinement, provenance, and explicit opt-in boundaries.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `PLUGIN-001` next safe slice | Plugin encapsulation | In Progress / review residual | ADR-027/028/029/030/032 | Local explicit plugin state is inspectable and remains confined. |
| `HOOK-001` diagnostics | Config-introduced hooks | Planned | ADR-029 | Hook config can be listed/diagnosed without executable hook expansion. |
| `DIST-001` policy | Optional asset distribution | Research | PLUGIN/DIST gates | Optional assets/packages have checksum/cache/offline/consent policy. |

### Scope

- Diagnostics and state visibility before more runtime expansion.
- Package-root confinement and provenance checks.
- Distribution policy for optional assets and plugin packages.

### Non-Goals

- No remote package install.
- No marketplace.
- No write-capable plugin tools.
- No Lua or dynamic library support.
- No automatic plugin discovery.

### Acceptance

- Given local explicit plugin/hook config,
  When diagnostics are requested,
  Then Talos reports state/provenance without executing hidden behavior.
- Given optional package distribution is discussed,
  When policy is recorded,
  Then checksum, cache, consent, offline/mirror, and failure behavior are explicit.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Plugin/hook focused tests if code changes.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: diagnostics drift into execution or remote install.
- Rollback: keep runtime changes disabled and ship policy/diagnostics only.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-04 | Activation | Activated after I090 completed bounded extraction and ripgrep-backed search stabilization with full workspace/governance closeout. Non-terminal inventory disposition: I085 remains Paused with MC107 manual `/connect` walkthrough residual; I086-I089 remain planned product-hardening shells; I092-I093 remain planned direct-owner shells. I091 starts with diagnostics/policy only: audit existing plugin/hook/distribution docs and code before adding runtime behavior. |
| 2026-07-04 | A7 execution | Closed the first local diagnostics slice without runtime expansion. Added `/hooks` as a read-only slash command that reports hook diagnostics, disabled config-introduced hooks, disabled executable hook carriers, and the builtin hook event catalog. Added `HookRegistry::registrations()` as a read-only registration snapshot. Extended plugin manifest parsing with validated `[[hooks]]` declarations so plugin packages can describe hook capabilities without executing or loading hook carriers. |

## Verification Evidence

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy -p talos-conversation -p talos-plugin -- -D warnings`: passed.
- `cargo test -p talos-conversation -p talos-plugin`: 95 conversation tests and 19 plugin unit
  tests plus plugin integration/doc tests passed.
- `cargo test --workspace`: passed.

## Variance And Residuals

- No runtime expansion has started. Remote install, marketplace behavior, write-capable plugin
  tools, automatic discovery, Lua, dynamic library loading, and executable standalone hook carriers
  remain out of scope.
- A8 remains open for DIST-001 distribution policy. Config-introduced hook execution remains out
  of scope; this slice only validates declarations and exposes diagnostics.

## Retrospective

- The immediate product gap was diagnostic clarity, not execution. A read-only `/hooks` command and
  manifest declaration validation make the boundary inspectable while preserving ADR-027/029
  runtime gates.
