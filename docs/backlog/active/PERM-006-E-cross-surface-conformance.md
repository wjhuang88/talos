# PERM-006-E: Cross-Surface Permission Conformance And Security Gate

| Field | Value |
|---|---|
| Story ID | PERM-006-E |
| Type | Security / Test Story |
| Priority | P0 |
| Status | Blocked — final completion requires PERM-006-C and PERM-006-D |
| Source | [GitHub Issue #57](https://github.com/wjhuang88/talos/issues/57) |
| Selected Iteration | None |
| Depends On | May characterize early; completion blocked by PERM-006-C/D |

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
- Release-preflight integration.

## Exclusions

- No new behavior solely to satisfy tests, live credentials, or broad sandbox penetration suite.

## Dependencies

May characterize early; completion blocked by PERM-006-C/D

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

## Residual Destination

New tools and surfaces must add scenarios to this corpus before claiming permission compatibility.
