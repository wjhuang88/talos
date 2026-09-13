# Iteration I265: Prompt Authority Decision Boundary

| Field | Value |
|---|---|
| Status | Complete / Closed |
| Source | PROMPT-001 / Issue #285 |
| Deliverable | Accepted ADR defining prompt authority, provenance, precedence and compatibility boundary. |
| Depends On | I264 baseline, ADR-033 |
| Excludes | No Rust prompt behavior, provider, SDK, persistence, or harness implementation. |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | ADR-074 decision and migration boundary only |
| Source Issue | #285 |
| Governance Claim PR | Pending |
| Claimed At | Not applicable |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim-only ADR acceptance; implementation remains unauthorized until merge. |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Establish an effective claim before accepting ADR-074 or implementing children. |
| Implementation PR | Not started |

## Acceptance

- ADR-074 records source, scope, authority, provenance, precedence and cache semantics.
- Runtime invariants outrank all advisory and extension contributions.
- Compatibility and migration boundaries are explicit; current behavior remains unchanged.
- Follow-up child slices are independently runnable and claimable.

Completion Commit: e7d6ef8e
