# PERM-007-F: Generic Shell Auto Classifier

**Status**: Refinement

| Field | Value |
|---|---|
| Story ID | PERM-007-F |
| Type | Permission / Security / Runtime Story |
| Priority | P0 |
| Status | Refinement / Unclaimed |
| Parent Epic | PERM-007 (closed; separately governed follow-up) |
| Source | GitHub Issue #462 |
| Selected Iteration | None |
| Depends On | I241, ADR-012, ADR-040, ADR-069; reconcile Issues #56/#57 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned; intake only, no implementation authority |
| Claimed At | Not applicable |
| Source Issue | #462 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | Establish a reviewed decision and a separate effective implementation claim before production changes. |

## Goal And Boundary

Reduce routine shell approval prompts through a generic model classifier rather than per-command
exceptions. Deterministic Deny/Ask, permission/admission, sandbox and secret boundaries remain
authoritative. This intake record changes no Rust, Cargo, config, UI, release or publication state.

## Next Step

Prepare a decision-only classifier context/precedence contract, threat matrix, nonterminal inventory
and authority reconciliation with Issues #56/#57. Unknown or unsafe actions remain human-required
or denied until separately reviewed implementation exists.
