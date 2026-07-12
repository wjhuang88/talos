# PERF-001: Compile-Time Embedded TOML Materialization

| Field | Value |
|---|---|
| ID | PERF-001 |
| Type | Product/Performance Story |
| Priority | P1 |
| Status | Complete — Phase 1 (models.toml via `generate_compiled_models()` in build.rs) and Phase 2 (bash_permission_policy.toml via build.rs) both delivered; reconciled 2026-07-12 (I116/LT010) |
| Source | Maintainer request 2026-07-06 — embedded TOML should be parsed at build time instead of consuming runtime startup/workflow cost |
| Depends on | MC-001, PERM-003 |
| Blocks | Large model catalog startup/interaction polish; lower-noise permission runtime follow-up |

## Problem

Talos currently embeds some repository-owned TOML files with `include_str!()` and parses them with
`toml::from_str()` at runtime. That is acceptable for small developer-time assets, but it is now
too expensive for the large packaged model catalog.

Known runtime embedded TOML parse sites:

- `crates/talos-config/src/model.rs`
  - `builtin_models()` parses `crates/talos-config/src/models.toml`.
  - `builtin_providers()` parses the same `models.toml` again.
  - Current catalog size is thousands of model rows, so repeated runtime parsing is avoidable
    overhead.
- `crates/talos-tools/src/bash_tool.rs`
  - `parse_bash_permission_policy()` parses
    `crates/talos-tools/src/bash_permission_policy.toml`.
  - The file is small, but the policy is repository-owned and stable at build time, so it should
    follow the same embedded-resource rule.

## Goal

Move repository-owned embedded TOML parsing to build time while preserving the public runtime APIs
and all current safety boundaries.

## Non-Goals

- Do not move user config parsing to build time. `~/.talos/config.toml`,
  `.agents/talos/config.toml`, workspace-local config, plugin manifests, and any user/project
  runtime TOML remain runtime inputs.
- Do not reintroduce runtime `catalog.db`.
- Do not require network access in normal builds.
- Do not change model/provider semantics, permission policy semantics, or public config/tool APIs.
- Do not add a new native dependency or unsafe code.

## Proposed Implementation

### Phase 1: `models.toml`

- Extend `crates/talos-config/build.rs` so normal builds parse `src/models.toml` locally and
  generate a Rust source artifact under `OUT_DIR`.
- Keep `BUILD_MODELS=1` behavior as the explicit network refresh path for updating
  `src/models.toml`; after refresh, generate the same build artifact from the refreshed local file.
- Replace runtime `toml::from_str(include_str!("models.toml"))` in `builtin_models()` and
  `builtin_providers()` with generated static data.
- Preserve existing function signatures unless a narrower internal helper is added:
  - `builtin_models() -> Vec<ModelMetadata>`
  - `builtin_providers() -> Vec<BuiltinProvider>`
- Prefer generated borrowed/static entries and conversion into existing owned DTOs over embedding
  large `String` allocations in static initializers.

### Phase 2: `bash_permission_policy.toml`

- Add a build-time generation path for `crates/talos-tools/src/bash_permission_policy.toml`.
- Replace runtime parsing in `parse_bash_permission_policy()` for the embedded policy path with
  generated static data.
- Keep test-only/custom-string parsing available if tests need to validate TOML syntax or
  malformed policy behavior.

## Acceptance Criteria

- [x] `bash_permission_policy.toml` no longer has to be parsed at runtime for the production
      embedded policy path.
- [ ] `builtin_models()` no longer parses embedded `models.toml` at runtime. (Phase 1 — future)
- [ ] `builtin_providers()` no longer reparses embedded `models.toml` at runtime. (Phase 1 — future)
- [x] User config TOML and plugin manifest TOML still parse at runtime.
- [x] Normal builds remain offline and deterministic.
- [ ] `BUILD_MODELS=1` still refreshes `src/models.toml` and fails safely without writing an empty
      catalog. (Phase 1 — future)
- [x] Generated code is confined to `OUT_DIR`; committed source remains readable and reviewable.
- [ ] Tests prove model/provider counts and representative entries match the committed TOML. (Phase 1 — future)
- [x] Tests prove bash permission classification is unchanged for representative read-only,
      validation, network/package-manager, write/mutating, and complex-shell commands. (33 bash tests pass)
- [x] Workspace validation passes.

## Validation Plan

Required targeted checks:

```sh
cargo test -p talos-config model
cargo test -p talos-cli models_browser
cargo test -p talos-cli connect
cargo test -p talos-tools bash_tool
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Recommended implementation audit:

```sh
rg -n "include_str!\\(\"models\\.toml\"\\)|toml::from_str\\(.*models|bash_permission_policy\\.toml|parse_bash_permission_policy" crates/talos-config crates/talos-tools
```

The audit should show no production runtime parse path for the embedded `models.toml` or embedded
bash permission policy.

## Risks And Rollback

- Risk: generated static Rust for thousands of models increases binary size or compile time.
  Rollback: generate a compact binary/static representation under `OUT_DIR` and decode without TOML
  parsing, or fall back to a single cached `OnceLock` parse while recording the blocker.
- Risk: generated code becomes unreadable.
  Rollback: keep the source TOML as the reviewed artifact and constrain generated code to simple
  static entry tables.
- Risk: build script accidentally performs network work in normal builds.
  Rollback: require explicit `BUILD_MODELS=1`; add tests or source checks around the no-network
  default.

## Required Reads

- `crates/talos-config/src/model.rs`
- `crates/talos-config/build.rs`
- `crates/talos-config/src/models.toml`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/bash_permission_policy.toml`
- `docs/backlog/active/MC-001-model-catalog-modernization.md`
- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
