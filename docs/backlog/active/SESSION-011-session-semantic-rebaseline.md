# SESSION-011: Session Semantic Rebaseline

> Document status: Refinement / Unclaimed

| Field | Value |
|---|---|
| Story ID | SESSION-011 |
| Type | Session Architecture / Migration Governance |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #612](https://github.com/wjhuang88/talos/issues/612) |
| Selected Iteration | None |
| Depends On | ADR-037; ADR-039; ADR-042; ADR-056; ADR-058; SESSION-004; SESSION-009 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Rebaseline Session truth, context, runtime instance, operation/control and settlement semantics while preserving TLOG and durable-session invariants |
| Claimed At | Not applicable |
| Source Issue | #612 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-29 |
| Handoff / Release Condition | Refine architecture truth, compatibility paths and child ownership before selecting any implementation slice |

## Scope

- Preserve current TLOG identity, parent-link, archival-chain, recovery, custody and first-terminal-outcome-wins semantics.
- Separate durable Session facts, model-context provenance, runtime-instance identity, external operation intent and turn settlement conceptually without creating a duplicate transcript authority.
- Reconcile current fork/storage documentation truth before implementation.
- Decompose implementation into bounded child owners with explicit compatibility-removal ownership.

## Exclusions

No TLOG format migration, provider-context rewrite, public API change, replay architecture, task engine, persistence replacement or runtime implementation is authorized by this owner.

## Acceptance

- Current durable/TLOG invariants and superseded documentation are reconciled.
- Truth/context/execution/control/settlement ownership is explicit.
- Every additive compatibility path has a canonical path, removal trigger and named owner.
- Implementation remains split into separately claimable children.
