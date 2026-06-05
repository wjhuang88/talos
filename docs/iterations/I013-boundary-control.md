# I013: Boundary Control

**User can**: Continue planning and implementing high-risk runtime features without
silently weakening Talos' permission, provider, logging, or public schema boundaries.

## Status: COMPLETE (2026-06-05)

This front-loaded iteration concentrates the work I would not hand to an external
implementer without first pinning the design. The goal is not to ship Guardian or
exec policy DSL behavior yet; it is to close the decision gaps that could otherwise
turn into permission bypasses or unstable public contracts.

## Story Granularity Audit

- `#I010-S6` Guardian AI sub-agent was too risky as a normal polish story. It is now gated by
  ADR-011 and must not be implemented as a free-form auto-approval feature.
- `#I010-S8` Exec policy DSL was too broad for I010 polish. It is now gated by ADR-012 and must
  not become a shell parser.
- `#I011-S2` Provider plugin architecture mixed config schema, migration, and future dynamic
  loading language. ADR-013 keeps the next slice schema-only.
- `#I012` was too broad as one next iteration. Split follow-up execution into file/search and Git
  iterations so `ToolPack`, search, and embedded Git do not block each other.
- `#ARCH-S8` was phaseable. R1 centralized logging can land without the R2/R3 persistent-output ADR.

## Selected Stories

- [x] Boundary ADRs for Guardian, exec policy DSL, and provider config schema.
- [x] #ARCH-S8 R1: centralized logging initialization and `[log]` config baseline.
- [x] Re-plan upcoming iterations so risk-heavy work is not mixed into product polish.

## Implemented

- ADR-011: [Guardian Approval Boundary](../decisions/011-guardian-approval-boundary.md)
- ADR-012: [Exec Policy DSL Boundary](../decisions/012-exec-policy-dsl-boundary.md)
- ADR-013: [Provider Config Schema Boundary](../decisions/013-provider-config-schema-boundary.md)
- `talos-config` now has a `[log]` config schema with `level`, `format`, and `filter`.
- `talos-cli` now initializes tracing through one `logging::init_logger()` path for CLI and MCP
  server modes.

## Acceptance Criteria

- [x] Guardian backlog item points to ADR-011 before implementation.
- [x] Exec policy DSL backlog item points to ADR-012 before implementation.
- [x] Provider plugin S2 backlog item points to ADR-013 before implementation.
- [x] Logging R1 has a single CLI initialization function and keeps terminal UI output protected
      from stderr log corruption.
- [x] `cargo test -p talos-config -p talos-cli` passes.

## Verification Evidence

- `cargo test -p talos-config -p talos-cli` passed on 2026-06-05.
- `cargo test --workspace` passed on 2026-06-05.
- `cargo clippy -p talos-config -p talos-cli -- -D warnings` passed on 2026-06-05.

## Residual Work

- `#ARCH-S8` R2/R3 file output, rotation, JSON logs, and shared logging crate remain blocked on a
  follow-up ADR.
- Guardian and exec policy DSL behavior remains unimplemented by design.
- Provider plugin config schema implementation moves to I015.
