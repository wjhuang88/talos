# PERM-007-G: Higher-Risk Automatic Writes

| Field | Value |
|---|---|
| Story ID | PERM-007-G |
| Type | Permission / Safety Story |
| Status | In Progress (proposed by #765; effective only on merge) |
| Source | [GitHub Issue #563](https://github.com/wjhuang88/talos/issues/563) |
| Selected Iteration | I308 / #765, proposed activation only |
| Collaboration Claim | Delegated to the single I308 claim in #765; ineffective until merge |

Issue #563 tracks future diff-aware writes, rollback/backup gates, and impact analysis for
higher-risk automatic permission decisions. This owner is intake-only; W0/W1 behavior remains
unchanged and all higher-risk operations remain human-required until separately governed.

## I308 Selection Proposal — 2026-10-26

[I308](../../iterations/I308-builtin-auto-review.md) proposes joint ownership of H and G
for handoff T8. Proposed claim/activation in #765 is ineffective until target-branch merge.
It preserves the complete existing acceptance, including actual invocation-local effectful
approval and G's diff/impact/backup/recovery requirements. Historical higher-risk human gates
remain until concrete implementation is independently reviewed and validated. No implementation
or universal Auto coverage is claimed by selection. The iteration holds the single claim record;
these Story owners delegate execution tracking to it once effective.

Current proposal supersedes historical Unclaimed/intake-only descriptions only upon #765 merge.
No runtime policy changes occur in this governance PR; higher-risk eligibility still requires implementation.
