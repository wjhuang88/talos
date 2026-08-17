# PERM-007: Model-Assisted Auto Permission Decisions

> The filename retains its Goal-era path for stable references; the governed target now covers a
> configurable cross-surface `auto` mode rather than Goal mode only.

| Field | Value |
|---|---|
| Story ID | PERM-007 |
| Type | Permission / Security Epic |
| Priority | P1 |
| Status | Refinement — scope change requires ADR-011 revision, threat model and bounded child decomposition |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | None |
| Depends On | PERM-006-A/B/C structured decision and authoritative execution pipeline; ADR-011 revision; existing Deny precedence |

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
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Independently review and accept an ADR that revises or supersedes ADR-011, then claim one bounded child before implementation. |

## Identity / Goal / Value

Reduce repeated low-risk human approval interruptions through a configurable cross-surface `auto`
mode without letting a model bypass Talos permission, authorization, grant, workspace or execution
boundaries. Goal mode is one consumer of this shared capability, not its security scope.

## Scope

- Define a redacted structured input and validated output for model-assisted risk classification.
- Define the canonical configuration field for `auto`, its default-on target, persistence and
  precedence against a per-session `/auto` toggle; exact naming and migration remain ADR work.
- Define `/auto` as an explicit TUI/session command that reports enabled, disabled, policy, model,
  timeout and degradation state without changing the underlying permission authority.
- Define equivalent behavior for Goal, interactive CLI/TUI, headless, Runtime and MCP entrypoints
  when their permission context and resources are equivalent.
- Preserve configured Deny, hard boundaries and fail-closed behavior as non-overridable.
- Define timeout, unavailable-model, malformed-output, uncertainty and human-escalation behavior.
- Define auditable policy/model versions, bounded decisions and final authorization outcomes.
- Decompose decision/threat-model, configuration and command surface, implementation and
  cross-surface conformance work before Ready.

## Exclusions

- No arbitrary model-generated permission rules, permanent grants or widened resource scopes.
- No sandbox rewrite, bypass of `PermissionController`, or implementation before the revised ADR
  explicitly accepts the proposed default-on target.
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
- ADR-011 currently says Guardian/model assistance is disabled by default and cannot auto-approve
  write-capable tools in its first implementation. A new ADR must explicitly revise or supersede
  that decision before the requested default-on `auto` mode can become Ready.
- The new ADR must define maximum automatic authority, privacy, audit, timeout, configuration and
  session-override precedence, headless behavior, migration and rollback policy.

## Uncertainty And Validation Path

Refine the exact risk classes that can be automated, prove the model cannot enlarge the structured
request, and define deterministic adversarial fixtures before selecting the first child. The
default-on request is a product target, not an accepted safety decision until ADR-011 is revised.

## State / Status Owners

- Epic scope and acceptance: this file.
- Remote request and discussion: Issue #188.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

If implemented, document the global `auto` policy, `/auto` control, model identity, default and
session overrides, escalation behavior and audit visibility across every supported surface. Do not
present the capability as shipped while this owner remains Refinement.

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

## Acceptance For Behavior / Technical Work

- Hard Deny and out-of-policy requests cannot be converted to Allow by any model output.
- Unavailable, malformed, conflicting or uncertain model results fail closed to human confirmation
  or Deny according to the accepted ADR.
- A configured `auto` mode defaults and a `/auto` session override are explicit, inspectable and
  deterministic; disabling `auto` restores the existing human-approval path.
- Equivalent requests have equivalent authorization semantics across CLI, TUI, Goal, headless,
  Runtime and MCP surfaces.
- Audit evidence is useful without storing credentials, secrets or complete sensitive inputs.
- The revised ADR, bounded children and cross-surface security tests complete with independent
  security review before any default-on behavior is claimed.

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
