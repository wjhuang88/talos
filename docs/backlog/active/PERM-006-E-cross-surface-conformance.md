# PERM-006-E: Cross-Surface Permission Conformance And Security Gate

| Field | Value |
|---|---|
| Story ID | PERM-006-E |
| Type | Security / Test Story |
| Priority | P0 |
| Status | Blocked / Unclaimed — PERM-006-C is Complete; final completion requires PERM-006-D |
| Source | [GitHub Issue #57](https://github.com/wjhuang88/talos/issues/57) |
| Selected Iteration | None |
| Depends On | PERM-006-C Complete; may characterize early; completion blocked by PERM-006-D |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #57 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Create one deterministic scenario corpus that proves identical permission, approval, grant, authorization, hook, and execution semantics across all supported surfaces.

## Scope

- Canonical scenario/expected-trace model.
- Direct, terminal, TUI, headless, embedded runtime, RPC/MCP, native/MCP/plugin adapters.
- Path, command, network/remote, precedence, projection, and failure corpus.
- Third-party `AgentTool::project_input()` projection documentation and conformance coverage:
  secret-bearing tools must override the default full-input projection.
- Release-preflight integration.

## Exclusions

- No new behavior solely to satisfy tests, live credentials, or broad sandbox penetration suite.

## Dependencies

PERM-006-C is Complete. Characterization may start only under separate authority; completion remains
blocked by PERM-006-D.

## Decision Links And Constraints

- One invocation equals one authoritative evaluation.
- Non-interactive Ask fails closed.
- Private inputs, credentials, and grants do not leak into observers or durable output.

## Uncertainty And Validation Path

Capture and review the intended golden compatibility matrix before changing the pipeline; inconsistencies are explicit decisions, not silently frozen.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #57.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Blocked.

## Required Reads

- docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md
- docs/backlog/active/PERM-006-D-typed-effects-and-resources.md
- crates/talos-permission/tests/
- crates/talos-agent/tests/
- crates/talos-runtime/tests/

## Acceptance For Behavior / Technical Work

- The same scenario corpus drives every applicable surface.
- Decision, grant, authorization, execution count, hook identity, and ordering are asserted.
- Deny precedence and failure-closed cases pass on Unix and Windows where semantics differ.
- The standard release preflight invokes the deterministic gate.
- Public integration documentation states that `AgentTool::project_input()` preserves full input by
  default and that third-party tools with secret-bearing arguments must override it.
- Conformance fixtures prove a secret-bearing override reaches permission evaluation without
  exposing the secret through proposal/final hooks, observer output or durable output.

## Residual Destination

New tools and surfaces must add scenarios to this corpus before claiming permission compatibility.

## 2026-08-23 I221 Residual Ownership Checkpoint

PERM-006-C / I221 completed at implementation commit `49d1546c` through PR #376 merge `f9e6706d`.
Its independent permission/security/API review recorded that the default
`AgentTool::project_input()` preserves complete input. PERM-006-E owns the bounded documentation and
conformance residual for third-party tools whose arguments contain secrets; those tools must
override the projection and prove non-disclosure at permission observer/hook boundaries. E remains
Blocked / Unclaimed on PERM-006-D, with no selected iteration, claim or implementation authority.
