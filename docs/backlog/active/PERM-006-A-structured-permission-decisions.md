# PERM-006-A: Structured Permission Requests, Contexts, And Decision Reports

| Field | Value |
|---|---|
| Story ID | PERM-006-A |
| Type | Permission / Technical Story |
| Priority | P0 |
| Status | Review |
| Source | [GitHub Issue #53](https://github.com/wjhuang88/talos/issues/53) |
| Selected Iteration | I189 (Review) |
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
| Authorization Evidence | Claim PR #197 merged as `0df88638409027849e5bf4ba13ef72d2e96b9b90` after exact-head CI `31554958547`, independent security approval comment `5261239200` bound to `b4f23ec2255c60723c7d1abae3084a24c3bb5899`, and merge-time CAS. Activation PR #351 merged as `20cfcce4e72be3da4e3efc1190ee498975e7476b` after exact-head CI `32500829272`, independent Agent-role security/governance approval `5372336921`, and merge-time CAS. |
| Implementation PR | #356 |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Review implementation commit `6b577d6afcb05230c821214902b9067c45c767a9` through PR #356 with fresh exact-head CI, independent security/code review and merge-time CAS. A later owner-first closeout may cite that pre-existing implementation commit; this Review state does not authorize PERM-006-B/C/D/E or PERM-007 behavior. |

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
Do not present this Story as shipped while implementation review and owner-first closeout remain pending.

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

## 2026-08-22 Activation And API Decision Checkpoint

- Activation PR #351 merged as `20cfcce4` after exact-head CI `32500829272`, independent
  Agent-role security/governance approval `5372336921` and merge-time CAS. I189 is Active/Claimed.
- Read-only API assessment found that public mutable `PermissionEngine` fields erase the source
  identity needed to distinguish configured rules from runtime grants. Guessing from rule shape,
  vector position or a hidden sentinel would produce untrustworthy provenance.
- ADR-065 is the required pre-1.0 API/migration decision already anticipated by the Published
  Baseline. It changes no permission outcome, approval routing, serialized configuration, grant
  lifetime, hook DTO, workspace version or release state. Implementation waits for ADR acceptance.

## 2026-08-22 Implementation Review Checkpoint

- ADR-065 was Accepted through PR #355 merge `9579df7a` after exact-head CI `32508015164`,
  independent Agent-role security/API review `5373150265` and merge-time CAS.
- Implementation commit `6b577d6afcb05230c821214902b9067c45c767a9` adds the bounded structured
  request/context/report evaluator and preserves the claim exclusions. Local focused and dependent
  crate tests plus the complete repository release preflight passed; an independent local red-team
  review approved the corrected closed reason dimension and MCP/plugin redaction coverage.
- I189 is Review/Claimed through implementation PR #356. Completion remains pending fresh
  exact-head CI, independent security/code review, merge-time CAS and a later owner-first closeout.
