# ARCH-034-R10: Core Tool Facade Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F20 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | Not selected |
| Preserved behavior | Every `talos_core::tool` public path and registry/protocol semantic |

## Problem And Boundary

`talos-core/src/tool.rs` combines result/presentation types, authorization policy, `AgentTool`,
contribution identity, and registry implementation in 1,103 production lines. The public facade is
correct; private source ownership is broad.

## Scope

- Split private implementation files and re-export through the existing `tool` module.
- Add compile-time downstream API probes for current public paths.

## Exclusions

- No public name/path, serialization, trait default, registry collision, dependency, or semver change.

## Acceptance And Validation

- `talos-core` remains dependency-free and every existing downstream import compiles unchanged.
- Registry, contribution, presentation, authorization, and serialization tests pass unchanged.
- Locked workspace, API probes, governance, and diff checks pass.

## Rollback / Residual

Revert if any public path or diagnostic changes. API redesign requires an ADR and migration plan.
