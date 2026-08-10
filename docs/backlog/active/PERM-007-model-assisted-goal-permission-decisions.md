# PERM-007: Model-Assisted Goal Permission Decisions

| Field | Value |
|---|---|
| Story ID | PERM-007 |
| Type | Permission / Security Epic |
| Priority | P1 |
| Status | Refinement — ADR, threat model and bounded child decomposition required |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | None |
| Depends On | PERM-006-A/B/C structured decision and authoritative execution pipeline; existing Deny precedence |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #188 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | Accept a security ADR and claim one bounded child before implementation. |

## Identity / Goal / Value

Reduce repeated low-risk human approval interruptions in Goal mode without letting a model bypass
Talos permission, authorization, grant, workspace or execution boundaries.

## Scope

- Define a redacted structured input and validated output for model-assisted risk classification.
- Preserve configured Deny, hard boundaries and fail-closed behavior as non-overridable.
- Define timeout, unavailable-model, malformed-output, uncertainty and human-escalation behavior.
- Define auditable policy/model versions, bounded decisions and final authorization outcomes.
- Decompose decision/threat-model, implementation and cross-surface conformance work before Ready.

## Exclusions

- No arbitrary model-generated permission rules, permanent grants or widened resource scopes.
- No sandbox rewrite, bypass of `PermissionController`, or change to non-Goal defaults.
- No implementation, dependency or public API authorization from this intake record.

## Dependencies

- PERM-006-A structured request/context/report contract.
- PERM-006-B scoped grant semantics where any automatic grant is proposed.
- PERM-006-C authoritative evaluate-to-execute pipeline.
- Independent security review before any protected implementation claim.

## Decision Links And Constraints

- `AGENTS.md` Hard Constraints 4 and 5 remain authoritative.
- Deny precedence, workspace boundaries and headless deny-by-default cannot be weakened.
- Any native/process/model integration failure must degrade to human confirmation or Deny.
- A new ADR must define maximum automatic authority, privacy, audit, timeout and rollback policy.

## Uncertainty And Validation Path

Refine the exact risk classes that can be automated, prove the model cannot enlarge the structured
request, and define deterministic adversarial fixtures before selecting the first child.

## State / Status Owners

- Epic scope and acceptance: this file.
- Remote request and discussion: Issue #188.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

If implemented, document Goal-mode policy, model identity, disable control, escalation behavior and
audit visibility. Do not present the capability as shipped while this owner remains Refinement.

## Required Reads

- `docs/backlog/active/PERM-006-permission-pipeline-convergence.md`
- `docs/backlog/active/PERM-006-A-structured-permission-decisions.md`
- `docs/backlog/active/PERM-006-B-scoped-grant-store.md`
- `docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `crates/talos-permission/`
- `crates/talos-agent/src/tool_execution.rs`

## Acceptance For Behavior / Technical Work

- Hard Deny and out-of-policy requests cannot be converted to Allow by any model output.
- Unavailable, malformed, conflicting or uncertain model results fail closed to human confirmation
  or Deny according to the accepted ADR.
- Equivalent requests have equivalent authorization semantics across CLI, TUI, Goal, headless,
  Runtime and MCP surfaces.
- Audit evidence is useful without storing credentials, secrets or complete sensitive inputs.
- All bounded children complete with independent security review and adversarial tests.

## Residual Destination

Sandbox redesign, general autonomous policy generation and non-Goal approval changes require
separate owners and decisions.
