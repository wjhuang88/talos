# RUNTIME-009: Structured Submission SDK Migration

> Document status: Refinement / Unclaimed

| Field | Value |
|---|---|
| Story ID | RUNTIME-009 |
| Type | Runtime SDK / Control-Path Migration |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #614](https://github.com/wjhuang88/talos/issues/614) |
| Selected Iteration | None |
| Depends On | SESSION-011; ADR-056; ADR-058; RUNTIME-006; Issue #234 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Own the public tracked/idempotent submission and reconciliation migration while retaining submit(String) as a convenience wrapper |
| Claimed At | Not applicable |
| Source Issue | #614 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-29 |
| Handoff / Release Condition | Review public API shape and compatibility, then claim a bounded implementation slice with lost-ack, idempotency and generation-fencing evidence |

## Scope

- Expose reviewed tracked submission and bounded reconciliation semantics through talos-runtime.
- Converge convenience submit and structured callers onto one canonical admission/custody implementation.
- Preserve stable identity, same-ID payload-conflict behavior, generation fencing and admission-vs-completion separation.

## Exclusions

No pending-journal redesign, TLOG merge, automatic replay of ambiguous work, multi-client policy, permission/sandbox change or removal of submit(String) is authorized.

## Acceptance

- Reliable callers can submit idempotently and reconcile uncertain delivery.
- Convenience and tracked APIs share one canonical implementation path.
- Retry/restart/generation tests prove no duplicate transcript entries or side effects.
