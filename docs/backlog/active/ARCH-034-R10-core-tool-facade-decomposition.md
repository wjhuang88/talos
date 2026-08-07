# ARCH-034-R10: Core Tool Facade Decomposition

> Document status: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F20 |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I179 (Complete; Implementation PR #168) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing result/presentation, authorization, `AgentTool`, contribution/registry, and protocol implementations from `talos-core/src/tool.rs` into private responsibility modules behind the unchanged public `talos_core::tool` facade; preserve every public path/name, visibility, trait default, object-safety property, serialization/schema shape, authorization normalization/comparison rule, registry replacement/collision/validation semantic, diagnostic, macro, dependency, and protocol parse/config behavior. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #167 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #168 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed after implementation PR #168 merged; any public path/API/serialization/trait/registry/authorization/protocol, dependency, or behavior change requires a separate story, ADR, and migration plan where applicable. |
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

## Execution Evidence

- Claim PR #167 finalized head `168d96b0ea9aedb9d3850c800f0cedddb09e76ef` passed CI run `31183822345` and merged as `9a5419e496db4f059ed841917d8ee9f099d377f6` after merge-time CAS.
- `tool.rs` is now a 26-line stable facade over six private responsibility modules; the original public paths remain available through private-module re-exports.
- Focused `talos-core` tests, locked workspace check/Clippy/tests, release preflight, source-literal/name equivalence, public-path probes, and private-module layout probes pass.
- Source implementation commit `63d494c5` was submitted in Draft PR #168 before the accepted exact head and squash merge recorded below.

## Completion Evidence

- Completion Commit: `dafc9be08736aee91e0f9cdd92e5226930808061`
- Implementation PR #168 squash-merged at `dafc9be08736aee91e0f9cdd92e5226930808061` from source implementation `63d494c5` and accepted exact Head `7b646a4d33cb17a21258e31d86ce1fe8d01b1929`.
- Exact-head CI `31189425069` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke checks.
- Merge-time CAS confirmed base `9a5419e496db4f059ed841917d8ee9f099d377f6`, head `7b646a4d33cb17a21258e31d86ce1fe8d01b1929`, no blocking reviews/comments, and no overlapping claim or implementation PR.

## Rollback / Residual

Revert if any public path or diagnostic changes. API redesign requires an ADR and migration plan.
