# MEM-011: Extensible Memory Scopes And Automatic Migration

| Field | Value |
|---|---|
| Story ID | MEM-011 |
| Type | Memory / Architecture Story |
| Priority | P1 |
| Status | Refinement — ADR, schema contract, and migration fixtures required |
| Source | [GitHub Issue #116](https://github.com/wjhuang88/talos/issues/116) |
| Selected Iteration | None |
| Depends On | MEM-001 storage; DATA-001 lifecycle; complementary to MEM-010 admission |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #116 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Reserve first-class, multi-dimensional memory ownership/applicability/visibility scopes and automatically migrate every supported legacy `memory.db` without data loss or retrieval broadening.

## Scope

- Separate origin provenance from retrieval applicability.
- Reserve owner, workspace, project/repository, session, agent/profile, audience, and future dimensions.
- Add explicit retrieval context and versioned ordered transactional migrations.
- Backfill legacy rows with an explicit compatibility scope and expose migration status safely.

## Exclusions

- No immediate scope-aware filtering policy, automatic scope inference, team/cloud sharing, vector dependency, or manual migration requirement.

## Dependencies

MEM-001 storage; DATA-001 lifecycle; complementary to MEM-010 admission

## Decision Links And Constraints

- Existing database content, stable IDs, evidence, contradictions, entities, graph edges, hashes, and timestamps are product data and must be preserved.
- Migration is automatic, ordered, transactional where possible, idempotent, and fail-closed.
- Unknown/malformed scope data never broadens visibility.

## Uncertainty And Validation Path

Select schema representation, compatibility scope, version ledger, rollback behavior, duplicate identity, workspace/project identity, and future-schema rejection in an accepted ADR before Ready.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #116.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/backlog/active/MEM-001-layered-memory-foundation.md
- docs/backlog/active/DATA-001-local-data-lifecycle-storage-hygiene.md
- docs/backlog/active/MEM-010-user-origin-memory-admission.md
- crates/talos-memory/src/store.rs
- crates/talos-memory/src/lib.rs

## Acceptance For Behavior / Technical Work

- Fresh and every supported legacy fixture open at the newest schema automatically.
- Repeated migration is idempotent and failure rolls back without empty replacement.
- Legacy retrieval behavior is intentionally preserved through explicit compatibility scope.
- Identity/relationships/timestamps survive migration and future schemas fail closed without mutation.

## Residual Destination

Scope-aware ranking, sharing UX, and policy activation require separately selected follow-up Stories.
