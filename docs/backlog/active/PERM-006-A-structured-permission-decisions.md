# PERM-006-A: Structured Permission Requests, Contexts, And Decision Reports

| Field | Value |
|---|---|
| Story ID | PERM-006-A |
| Type | Permission / Technical Story |
| Priority | P0 |
| Status | Active — effective only after the I189 activation PR reaches `main` |
| Source | [GitHub Issue #53](https://github.com/wjhuang88/talos/issues/53) |
| Selected Iteration | I189 (Active only after the activation record reaches `main`) |
| Depends On | Parent PERM-006; foundational dependency for PERM-006-B/C |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline implementation session 2026-08-21 |
| Work Slice | Implement only PERM-006-A / I189: add one structured permission request/context/per-facet decision-report evaluator, delegate existing permission entrypoints to it, preserve current Deny/Ask/Allow outcomes and compatibility-visible Deny messages, and add provenance, redaction, fail-closed and order-independence tests. No approval routing, wrapper removal, grant/store, AlwaysApprove, typed-resource, policy, sandbox, PERM-006-B/C/D/E, PERM-007, TOOL-024, ACP or release change. |
| Claimed At | 2026-08-11 |
| Source Issue | #53 |
| Governance Claim PR | #197 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #197 merged as `0df88638409027849e5bf4ba13ef72d2e96b9b90` after exact-head CI `31554958547`, independent security approval comment `5261239200` bound to `b4f23ec2255c60723c7d1abae3084a24c3bb5899`, and merge-time CAS. The current activation still requires fresh exact-head independent security review, CI and CAS before it reaches `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Merge the governance-only I189 activation record after exact-head independent security review, CI and merge-time CAS. Begin implementation only from that activation merge or later `main`; retain a second independent security review for the implementation candidate. |

## Identity / Goal / Value

Add one structured permission request/context/report contract that explains per-facet outcomes and becomes the implementation source for existing evaluation entrypoints without changing shipped decisions.

## Scope

- Structured request, execution context, per-facet report, rule/grant provenance, and safe reasons.
- Compatibility projection to current `PermissionDecision`.
- Order-independent conservative aggregation tests.

## Exclusions

- No approval routing, wrapper removal, AlwaysApprove scope change, or typed-resource migration.

## Dependencies

Parent PERM-006; foundational dependency for PERM-006-B/C

## Decision Links And Constraints

- Any Deny denies; otherwise any Ask asks; otherwise Allow.
- Observer-facing reports exclude private projected fields and secrets.
- Current serialized configuration remains compatible.

## Uncertainty And Validation Path

Decide public type ownership and compatibility in an ADR or explicit pre-1.0 API note before Ready.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #53.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/backlog/active/PERM-006-permission-pipeline-convergence.md
- crates/talos-core/src/tool.rs
- crates/talos-permission/src/lib.rs
- crates/talos-agent/src/tool_execution.rs

## Acceptance For Behavior / Technical Work

- One request-evaluation entrypoint is the source of truth for compatibility methods.
- Reports identify explicit rule, grant, workspace, mode, and default decisions.
- Current behavior matrix remains identical and invalid consequential resources fail closed.

## Residual Destination

Any behavior correction discovered here must become a separately reviewed security fix.
