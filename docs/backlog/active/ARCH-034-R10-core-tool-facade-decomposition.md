# ARCH-034-R10: Core Tool Facade Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F20 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | I179 (Planned; claim PR #167) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing result/presentation, authorization, `AgentTool`, contribution/registry, and protocol implementations from `talos-core/src/tool.rs` into private responsibility modules behind the unchanged public `talos_core::tool` facade; preserve every public path/name, visibility, trait default, object-safety property, serialization/schema shape, authorization normalization/comparison rule, registry replacement/collision/validation semantic, diagnostic, macro, dependency, and protocol parse/config behavior. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #167 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if exact public-path/API/serialization/trait/registry/authorization/protocol equivalence cannot be proven; any API redesign or semver change requires a separate story, ADR, and migration plan. |
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
