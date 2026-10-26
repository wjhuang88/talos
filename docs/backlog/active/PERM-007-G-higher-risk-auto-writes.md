# PERM-007-G: Higher-Risk Automatic Writes

| Field | Value |
|---|---|
| Story ID | PERM-007-G |
| Type | Permission / Safety Story |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #563](https://github.com/wjhuang88/talos/issues/563) |
| Selected Iteration | None |
| Collaboration Claim | Unclaimed; no implementation or release authorization |

Issue #563 tracks future diff-aware writes, rollback/backup gates, and impact analysis for
higher-risk automatic permission decisions. This owner is intake-only; W0/W1 behavior remains
unchanged and all higher-risk operations remain human-required until separately governed.

## I308 Selection Proposal — 2026-10-26

[I308](../../iterations/I308-builtin-auto-review.md) proposes joint ownership of H and G
for handoff T8. This governance draft is Unclaimed and ineffective until finalized and merged.
It preserves the complete existing acceptance, including actual invocation-local effectful
approval and G's diff/impact/backup/recovery requirements. Historical higher-risk human gates
remain until concrete implementation is independently reviewed and validated. No implementation
or universal Auto coverage is claimed by selection. The iteration holds the single claim record;
these Story owners delegate execution tracking to it once effective.
