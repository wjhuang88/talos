# MEM-010: User-Origin-Only Global Memory Admission

| Field | Value |
|---|---|
| Story ID | MEM-010 |
| Type | Memory / Security Bug Story |
| Priority | P0 |
| Status | Ready — narrow admission correction; pending iteration selection |
| Source | [GitHub Issue #114](https://github.com/wjhuang88/talos/issues/114) |
| Selected Iteration | None |
| Depends On | MEM-001 foundation; complementary to MEM-011 scope architecture |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #114 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Prevent assistant, system, and tool transcript entries from becoming new global semantic memories in the production rule-based consolidation path; only original user-authored entries may be admitted.

## Scope

- Change production extractor role eligibility to user-only.
- Add mixed-role, assistant-only, memory-keyword, sensitive-content, duplicate, and evidence-reference regressions.
- Document that existing assistant-derived rows are not silently deleted by this fix.

## Exclusions

- No LLM extractor, scoring redesign, automatic consolidation, existing-row cleanup, sharing scopes, or session-format change.

## Dependencies

MEM-001 foundation; complementary to MEM-011 scope architecture

## Decision Links And Constraints

- Transcript persistence remains unchanged; this is an admission boundary only.
- Evidence for newly inserted memories must resolve to user-authored entries.
- ADD-only, deduplication, limits, sensitivity filtering, and prompt formatting stay unchanged.

## Uncertainty And Validation Path

This is a bounded security correction and can be selected directly after baseline tests confirm the current role gate.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #114.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Ready.

## Required Reads

- docs/backlog/active/MEM-001-layered-memory-foundation.md
- docs/backlog/active/MEM-011-extensible-memory-scopes.md
- crates/talos-memory/src/consolidation.rs
- crates/talos-cli/src/memory_cli.rs

## Acceptance For Behavior / Technical Work

- User entries remain eligible; assistant/system/tool entries produce no candidates.
- Mixed sessions insert only user-sourced candidates and evidence.
- Sensitive filtering, limits, duplicate handling, prompt retrieval, and TLOG behavior remain unchanged.
- Focused memory and full governance/workspace gates pass.

## Residual Destination

Auditing or deleting existing assistant-derived rows requires a separate migration/inspection Story.
