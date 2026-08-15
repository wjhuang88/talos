# I162 Publication Readiness Packet

Date: 2026-08-15
Baseline: `main@2891105d8a60e18cd5e0963432cea691355d2b63`
Candidate release: `v0.8.0`
Scope: readiness evidence only; no version bump, tag, GitHub Release, or real publication.

## External SDK Fixture

The independent Cargo root at `tests/fixtures/runtime-sdk-external/` exercises the documented
SDK boundary in local-path mode. It covers minimal runtime construction, a custom `AgentTool`, a
custom `LanguageModel`, approval handling, sandbox creation, permission rules, all three
`SandboxFallbackPolicy` values, durable sessions, and the explicit `coding` composition.

The following exact commands passed from a clean fixture target directory:

```text
cargo check --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml
cargo check --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml --features coding
cargo run --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml
cargo run --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml --features coding
```

Both runs printed `talos-runtime external fixture passed`. The fixture uses direct lower-level
dependencies for the types that are currently part of the supported contract; the stronger
single-dependency facade remains RUNTIME-006 / Issue #234 and is outside this release gate.

## Metadata-Derived Closure

`cargo metadata --locked --format-version 1` at the baseline produced 20 workspace packages in the
normal dependency closure of `talos-cli` and `talos-runtime`. `talos-models` is not in the closure.
The 16 registry-enabled members are:

```text
talos-agent talos-config talos-conversation talos-core talos-exploration talos-mcp
talos-memory talos-permission talos-plugin talos-provider talos-rpc talos-runtime
talos-sandbox talos-session talos-skill talos-tools
```

The four closure members still guarded by `publish = false` are `talos-cli`, `talos-dashboard`,
`talos-evolution`, and `talos-tui`. Their guards remain intentional and require separate release
authorization; they are not changed by I162. The dependency order for the final v0.8.0 publish
wave must be recomputed after version alignment.

## Package And Dry-Run Evidence

Package checks were attempted for every closure member with:

```text
cargo package --locked --offline --allow-dirty --no-verify -p <crate>
```

`talos-core`, `talos-exploration`, `talos-memory`, and `talos-skill` packaged successfully from
the local cache. The other registry-enabled crates failed because the offline index only contains
the historical `0.2.0` versions, while the workspace requires `0.7.0`; the failure is a registry
availability/version-closure blocker, not an implementation workaround. Guarded crates were
reported as `publish=false` and were not unguarded.

Dry-runs were attempted for every registry-enabled closure member with:

```text
cargo publish --locked --offline --allow-dirty --dry-run -p <crate>
```

All dry-runs stopped at the registry HTTP/index requirement (`attempting to make an HTTP request,
but --offline was specified`). A network-enabled exact-head run is still required before any GO
decision. Earlier registry API probes returned HTTP 403, so registry facts are not inferred.

## GO / NO-GO

**NO-GO for v0.8.0 publication.** The fixture is green, but publication cannot proceed because:

1. the current workspace is `0.7.0` and no version-alignment step is authorized in I162;
2. the 0.7.0 internal dependency closure is not visible in the available crates.io index;
3. network-enabled package and dry-run evidence is outstanding;
4. four product crates remain intentionally guarded by `publish = false` and need their separate
   release/install authorization;
5. GitHub Release must be completed before any real Cargo publication under I203 / REL-003.

No publish, tag, or GitHub Release was performed. I162 must remain in `Review` until the exact
implementation head, independent review, and closeout evidence are accepted. I203 remains blocked
and unclaimed until this packet is replaced or amended with a reviewed GO result.
