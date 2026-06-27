# ARCH-013: Config Module Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I060
**Source**: Follow-up to user correction that ARCH-012 only split one oversized module
**Depends on**: ARCH-012 complete; ADR-023 inline API key boundary

## Problem

After ARCH-012, `crates/talos-config/src/lib.rs` remained a 2083-line module combining public
configuration types, error definitions, credential file I/O, config load/save behavior, provider
defaults, environment substitution, and tests. That file was the largest remaining production
module and contained security-sensitive API-key masking/persistence behavior from ADR-023.

## Corrosion Judgment

| Signal | Evidence | Judgment |
|---|---|---|
| Oversized module | `talos-config/src/lib.rs` was 2083 lines before this slice. | Concrete split candidate. |
| Responsibility mixing | Types, `Config` behavior, credential I/O, built-in provider defaults, env substitution, and tests lived together. | Boundary corrosion present. |
| Security-sensitive behavior | Inline `api_key` persistence and display masking rely on clear type/debug boundaries. | Split without changing semantics. |
| Public API risk | Downstream crates import `talos_config::Config`, `ProviderConfig`, `McpConfig`, logging types, and `ConfigError`. | Preserve root re-exports. |

## Scope

- Split `talos-config/src/lib.rs` into focused modules without changing public `talos_config::*`
  imports.
- Preserve config schema, default values, API-key persistence, Debug masking, env substitution,
  built-in provider defaults, opencode/agents imports, and model-limit resolution behavior.
- Move tests out of the crate root.
- Record residual oversized modules for future owner stories.

## Non-Goals

- No config schema change.
- No new provider defaults.
- No API-key storage or masking policy change.
- No decomposition of `talos-cli/src/mode_runners.rs` or TUI/agent modules in this slice.

## Acceptance Criteria

- [x] `talos-config/src/lib.rs` is reduced to a small module/re-export root.
- [x] Public imports such as `talos_config::Config`, `talos_config::ConfigError`,
  `talos_config::ProviderConfig`, `talos_config::McpConfig`, and logging config types remain valid.
- [x] Config behavior tests pass after the split.
- [x] Residual oversized-module inventory is updated.

## Implementation Notes

- `lib.rs`: crate docs, module declarations, public re-exports.
- `error.rs`: `ConfigError`.
- `types.rs`: config DTOs, logging/MCP/RPC structs, provider protocol/model config.
- `credentials.rs`: credentials file path/load/save behavior.
- `config.rs`: `Config` behavior, validation, API-key lookup, model/provider resolution.
- `builtin.rs`: built-in provider defaults.
- `env.rs`: home directory and `${ENV_VAR}` substitution helpers.
- `tests.rs`: existing config tests migrated out of the root.

## Verification Evidence

- 2026-06-27: `cargo test -p talos-config` passed: 89 unit tests and 1 doc-test.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

The largest remaining production modules after ARCH-013 are:

- `crates/talos-cli/src/mode_runners.rs` (2062 lines): next concrete production candidate.
- `crates/talos-tui/src/scrollback.rs` (1614 lines): remains under ARCH-011 promotion rules unless
  display work forces a split.
- `crates/talos-tui/src/app.rs` (1503 lines): input/rendering state split candidate.
- `crates/talos-agent/src/compaction.rs` (1447 lines): policy/execution/testability candidate.
- `crates/talos-permission/src/lib.rs` (1370 lines): security-sensitive; split only with dedicated
  permission-boundary review.
