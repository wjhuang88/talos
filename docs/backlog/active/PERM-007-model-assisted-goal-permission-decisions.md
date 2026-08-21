# PERM-007: Model-Assisted Auto Permission Decisions

> The filename retains its Goal-era path for stable references; the governed target now covers a
> configurable cross-surface `auto` mode rather than Goal mode only.

| Field | Value |
|---|---|
| Story ID | PERM-007 |
| Type | Permission / Security Epic |
| Priority | P1 |
| Status | In Progress — PERM-007-A / I218 Complete; PERM-007-B/C/D Blocked |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | I218 Complete / Closed for PERM-007-A; implementation none |
| Depends On | PERM-006-A/B/C structured decision and authoritative execution pipeline; Accepted ADR-064; existing Deny precedence |

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
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | ADR-064 is Accepted through I218. Finish PERM-006-A/B/C, then prepare one bounded PERM-007 child with its own protected-scope claim before implementation. |

## Identity / Goal / Value

Reduce repeated low-risk human approval interruptions through a configurable cross-surface `auto`
mode without letting a model bypass Talos permission, authorization, grant, workspace or execution
boundaries. Goal mode is one consumer of this shared capability, not its security scope.

## Scope

- Define a redacted structured input and validated output for model-assisted risk classification.
- Implement ADR-064's canonical `auto.enabled` default-on attempt mode and its precedence against a
  non-persistent per-session `/auto` override through separately claimed children.
- Define `/auto` as an explicit TUI/session command that reports enabled, disabled, policy, model,
  timeout and degradation state without changing the underlying permission authority.
- Define equivalent behavior for Goal, interactive CLI/TUI, headless, Runtime and MCP entrypoints
  when their permission context and resources are equivalent.
- Preserve configured Deny, hard boundaries and fail-closed behavior as non-overridable.
- Define timeout, unavailable-model, malformed-output, uncertainty and human-escalation behavior.
- Define auditable policy/model versions, bounded decisions and final authorization outcomes.
- Preserve the accepted A decision and deliver configuration/command, resolver and cross-surface
  conformance only through the separately governed B-D child sequence.

## Exclusions

- No arbitrary model-generated permission rules, permanent grants or widened resource scopes.
- No sandbox rewrite, bypass of the authoritative permission pipeline, or behavior implementation
  before PERM-006-A/B/C close and the selected child has an effective protected-scope claim.
- No implementation, dependency or public API authorization from this intake record.

## Dependencies

- PERM-006-A structured request/context/report contract.
- PERM-006-B scoped grant semantics where any automatic grant is proposed.
- PERM-006-C authoritative evaluate-to-execute pipeline.
- Independent security review before any protected implementation claim.

## Governed Child Sequence

| Child | Deliverable | Current State | Gate |
|---|---|---|---|
| PERM-007-A / I218 | Threat model and ADR-011 revision/supersession | Complete / Closed at Completion Commit `a289a07f`; ADR-064 Accepted | No behavior change; evidence retained |
| PERM-007-B | Canonical config plus `/auto` session command | Blocked / Unclaimed | Accepted A and all PERM-006 A-C gates closed |
| PERM-007-C | Bounded model-assisted resolver inside the authoritative Ask path | Blocked / Unclaimed | B closed and separate protected-scope claim |
| PERM-007-D | Cross-surface conformance, rollout and rollback evidence | Blocked / Unclaimed | C closed and human validation where required |

The maintainer requested A early for unattended continuity. Decision work may run alongside I189
because it changes no behavior; B-D cannot bypass the ordered PERM-006 implementation chain.

PERM-007-A child Completion Commit: `a289a07ff97746d877f3a422d15f8044bbf50ab6`.
The parent Epic remains In Progress and does not claim completion from this child evidence.

## Decision Links And Constraints

- `AGENTS.md` Hard Constraints 4 and 5 remain authoritative.
- Deny precedence, workspace boundaries and headless deny-by-default cannot be weakened.
- Any native/process/model integration failure must degrade to human confirmation or Deny.
- Accepted ADR-064 supersedes ADR-011 and defines default-on as attempted bounded assistance, never
  default Allow. Its first automatic class is capability-relative atomic no-clobber creation of a
  new text file; existing-file modification remains human-mediated pending a later CAS decision.
- ADR-064's maximum authority, privacy, audit, timeout, configuration/session precedence, headless,
  migration and rollback rules are normative for B-D and cannot be widened by a child claim.

## Uncertainty And Validation Path

Before selecting B, finish PERM-006-A/B/C and translate ADR-064's fixed create-only class, request
binding and adversarial matrix into runnable acceptance. Any inability to provide a trusted open
parent-directory capability keeps that platform ineligible rather than weakening the contract.

## State / Status Owners

- Epic scope and acceptance: this file.
- Remote request and discussion: Issue #188.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

If implemented, document the global `auto` policy, `/auto` control, model identity, default and
session overrides, escalation behavior and audit visibility across every supported surface. Do not
present the capability as shipped while this parent remains In Progress and B-D are Blocked.

## Required Reads

- `docs/backlog/active/PERM-006-permission-pipeline-convergence.md`
- `docs/backlog/active/PERM-006-A-structured-permission-decisions.md`
- `docs/backlog/active/PERM-006-B-scoped-grant-store.md`
- `docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `crates/talos-permission/`
- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-config/src/types.rs`
- `crates/talos-conversation/src/command_registry.rs`
- `docs/decisions/011-guardian-approval-boundary.md`
- `docs/decisions/064-bounded-model-assisted-auto-permission.md`
- `docs/reference/I218-AUTO-PERMISSION-THREAT-MATRIX.md`

## Acceptance For Behavior / Technical Work

- Hard Deny and out-of-policy requests cannot be converted to Allow by any model output.
- Unavailable, malformed, conflicting or uncertain model results fail closed to human confirmation
  or Deny according to the accepted ADR.
- A configured `auto` mode defaults and a `/auto` session override are explicit, inspectable and
  deterministic; disabling `auto` restores the existing human-approval path.
- Equivalent requests have equivalent authorization semantics across CLI, TUI, Goal, headless,
  Runtime and MCP surfaces.
- Audit evidence is useful without storing credentials, secrets or complete sensitive inputs.
- Accepted ADR-064 remains the fixed boundary; bounded children and cross-surface security tests
  require independent protected-scope review before any default-on behavior is claimed.

## Residual Destination

Sandbox redesign, general autonomous policy generation, arbitrary grant generation and unrelated
permission-policy changes require separate owners and decisions.

## Change-Control Checkpoint — 2026-08-17

The maintainer broadened Issue #188 from Goal-only model-assisted permission decisions to a shared
configurable `auto` mode. The requested product target is default enabled, with `/auto` available to
enable or disable the mode for the active TUI/session, and model assistance available across Goal,
interactive, headless, Runtime and MCP entrypoints when their structured permission context is
equivalent.

This is a scope addition and a security-policy change, not an in-scope correction. The original
Goal-only intake remains historical context; no published iteration is being rewritten and no
implementation iteration is selected. The owner remains Refinement / Unclaimed.

The requested default conflicts with accepted ADR-011, which currently requires model assistance to
be disabled by default and forbids first-version write auto-approval. Before PERM-007 can become
Ready or claim implementation, a new independently reviewed ADR must revise or supersede ADR-011,
define the exact eligible decision classes and mode precedence, and preserve Deny/hard-boundary and
fail-closed rules. This checkpoint grants no configuration, `/auto` command, permission runtime,
public API, dependency or release authorization.
