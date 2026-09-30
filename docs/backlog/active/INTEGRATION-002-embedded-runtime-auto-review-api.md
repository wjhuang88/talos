# INTEGRATION-002: Embedded Runtime Auto Review API

**Status**: Refinement / Unclaimed
**Type**: Runtime SDK / Product API Story
**Parent Epic**: None

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Define a supported embedded Runtime composition for opt-in Auto review with human fallback; no implementation authority |
| Claimed At | Not applicable |
| Source Issue | #623 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Intake only; no implementation authorization |
| Implementation PR | Not started |
| Last Updated | 2026-09-30 |
| Handoff / Release Condition | ADR/API contract and bounded owner required before implementation |

## Goal And Boundary

Expose a supported `talos-runtime` API that lets an embedding host explicitly compose the existing
model-assisted Auto review, its bounded policy/context, and an existing human `ApprovalHandler`.
The Runtime must not read CLI configuration implicitly, grant permissions by default, or require
the host to depend on `talos-cli`/`talos-tui` or construct internal actors.

## Scope For Refinement

- Public builder/configuration types for explicit Auto enable/disable and session state.
- Host-selected evaluator model, risk scope, budget, and restricted context.
- Fail-closed fallback to the host's human handler for uncertainty, timeout, cancellation,
  invalid/unavailable model responses, and unsupported risk classes.
- Redacted, typed review outcome events bound to the session and tool call.
- Compatibility for existing builder calls when Auto is not configured.

## Exclusions

No default authorization, CLI global-config loading, new permission policy, browser/MCP/business
operation expansion, implicit durable grants, recursive tool calls, or implementation in this
intake record.

## Dependencies And Decisions

- Existing Runtime approval-handler and headless policy contracts.
- PERM-007 / ADR-064 and the shared bounded model-invocation contracts.
- A new ADR/API contract must define public types, lifecycle invalidation, exact-once consumption,
  redaction, and semver migration before the story becomes Ready.

## Acceptance For Refinement

- [ ] Owner identifies affected crates and public API surface.
- [ ] Given Auto is not configured, existing Runtime approval behavior remains unchanged.
- [ ] Given Auto is explicitly configured, uncertain/unsupported/error outcomes reach human approval
      or deny when no human handler exists.
- [ ] Session/account/policy changes invalidate in-flight and cached decisions.
- [ ] Typed outcome events expose no secrets or model reasoning.
- [ ] API examples, migration notes, focused tests, security/API review, and release evidence are
      listed before implementation claim.

## Required Reads

- Issue [#623](https://github.com/wjhuang88/talos/issues/623)
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/decisions/064-*.md` (ADR-064 Runtime/embedding boundary)
- `docs/backlog/active/PERM-007-F-generic-shell-effect-classification.md`
- `docs/sop/REQUIREMENT-INTAKE.md`

## Residual

This is separate from the completed I286 Desktop provider-failure evidence and from I287/H6. No
Desktop four-week status changes are implied.
