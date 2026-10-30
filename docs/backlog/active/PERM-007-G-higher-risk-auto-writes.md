# PERM-007-G: Higher-Risk Automatic Writes

| Field | Value |
|---|---|
| Story ID | PERM-007-G |
| Type | Permission / Safety Story |
| Status | Complete / Closed |
| Source | [GitHub Issue #563](https://github.com/wjhuang88/talos/issues/563) |
| Selected Iteration | I308 / #765, activated at `8f0ce2340` |
| Collaboration Claim | Delegated to the effective I308 claim in merged #765 |

## Completion evidence — 2026-10-30

Status: Complete. Completion Commit: `5dea4925a143151071ea231779c100a75734e6fe`.
I308's unchanged seven acceptance rows and PERM-007-H/G scope passed independent
closure review against implementation head `70d05dfe835d6d3f8d521539f513eeefd9e5cc41`.
CI `41007199388` passed all eight exact-head jobs, including full locked preflight,
native Windows workspace/CLI smoke and Linux/Windows Desktop. PR #809 merged
following exact-head static permission/security/API approval and merge-time CAS.
The reviewer independently checked CI but did not run tests; shared external
account identity remains disclosed. Earlier Partial/pending sections are history.

Recovery is opt-in, bounded and target-bound, with stale-target/replay refusal and
partial-failure receipts; no universal or cross-process rollback is promised.
Unavailable/protected search stays independently owned by I293. Standalone MCP
requires an explicitly injected interactive pipeline/intent for Auto; otherwise
it remains headless. Unix process tests execute real jobs; non-Unix cancellation
fixtures cover pending jobs. These are explicit supported boundaries, not claims
that every operation has identical Auto behavior on every platform.

Only I308/T8 and PERM-007-H/G close here. MODEL-014 → T4 → T5 → T1 → T6 remain
unimplemented/incomplete, and the complete handoff goal remains Active.

## Current Execution — 2026-10-26

Activation #765 is merged at `8f0ce2340`. [I308](../../iterations/I308-builtin-auto-review.md)
records implementation and validation; full H/G acceptance remains incomplete.
#770 (`687b84a93b4a9f40ede1eb600151c0a22a8f2d59`) merged complete-image
model review and invocation-local Auto execution for supported text edits/deletion,
with opt-in bounded backup/recovery. #771 adds policy/path evidence; #772
(`881a27b890e5310055380e6e3f5b0da78f9fdd3c`) adds recovery permission,
changed-target and pre-admission cancellation evidence. Started-write rollback,
universal identity protection and full G acceptance are not established.
The selection-proposal paragraphs below preserve the published baseline and are historical.

Issue #563 remains open for the complete higher-risk write acceptance. Earlier
intake-only/human-required descriptions below are historical, superseded only
for the concrete supported operations above.

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
