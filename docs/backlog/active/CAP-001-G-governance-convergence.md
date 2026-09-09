# CAP-001-G: Capability Architecture Governance Convergence

| Field | Value |
|---|---|
| Story ID | CAP-001-G |
| Type | Governance Story |
| Parent | CAP-001 |
| Status | In Progress / Claimed (proposed; ineffective until #518 merges) |
| Selected Iteration | I253 |
| Source | Issue #466 |
| Depends On | ADR-072 Accepted; CAP-001-P0 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | CAP-001-G architecture audit, terminology migration contract, child decomposition and documentation synchronization only. |
| Claimed At | 2026-09-09 |
| Source Issue | #466 |
| Governance Claim PR | #518 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized single-developer unattended completion of #499 then #466 and accepted ADR-072. Independent Agent-role review and exact-head CI remain required; shared GitHub identity does not prove natural-person separation. Proposed claim is ineffective until merge. |
| Implementation PR | Not started |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Effective governance claim required; no CAP-001-B/C or runtime authorization. |

## Goal And Acceptance

Deliver the architecture-governance outcome explicitly requested by #466, not all downstream
product implementations and not a descriptor-only substitute. The complete scope, exclusions,
requirements-to-evidence acceptance, validation and documentation targets are the published
[I253 plan](../../iterations/I253-capability-governance-convergence.md).
Each governance acceptance bullet in #466 must have concrete evidence; missing downstream
features require real child owners/Issues and must not be described as implemented.

## Required Reads

- [CAP-001](CAP-001-progressive-capability-provider-architecture.md)
- [I253](../../iterations/I253-capability-governance-convergence.md), including its Required Reads.
- [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md)
- Full Issue #466 body/comments and the #467 compatibility evidence.

## State Owners And Residuals

This file owns Story state; I253 owns execution. Parent CAP-001 owns architecture and child map.
Later runtime, API, schema and domain work remains with separately claimed children.
Complete requires an already-existing governance implementation commit plus full acceptance evidence.

## 2026-09-09 Atomic Claim Candidate

PR #518 proposes this governance Story In Progress / Claimed and I253 Active / Claimed
atomically. Neither is effective before merge to main. All child implementation scopes
remain unclaimed; no runtime, Cargo, API/schema or permission implementation is authorized.
The I253 checkpoint records the current inventory and next exact-head CI/review/CAS gate.
