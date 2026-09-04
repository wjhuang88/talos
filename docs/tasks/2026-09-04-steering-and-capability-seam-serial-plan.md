# Steering And Capability Seam Serial Development Plan

**Status**: Active coordination / I207 Complete / Closed / I208 Active / Claimed
**Published plan date**: 2026-09-04
**Outcome**: Deliver the remaining steering follow-ups and then prepare the progressive capability compatibility seam in a controlled serial sequence.
**Work mode**: Standard
**Deferred validation tracker**: None; human/device evidence remains a per-child gate and is not deferred by this plan.

This is a coordination and recovery ledger. I207 is Complete / Closed via closeout merge
`2edb914f`; I208 claim/activation is effective via governance PR #487 merge `75ca8057`; I246
remains unclaimed and unauthorized until its own claim merges.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Serial coordination for I207/TUI-049 -> I208/TUI-050 -> I246/CAP-001-P0. Each child retains its own implementation authority, claim, PR, review and closeout. |
| Claimed At | Not applicable |
| Source Issue | #267 and #467 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Establish an effective claim for the current child before implementation; advance only after its terminal disposition and checkpoint. |

## Startup Contract

### In Scope

- I207 / TUI-049: shared left/right padding for wrapped steering rows.
- I208 / TUI-050: steering insertion at explicit model-response/tool-call boundaries.
- I246 / CAP-001-P0 / #467: current-code characterization and compatibility seam preparation.
- Owner-first synchronization, focused validation, exact-head review and merge-time CAS for each child.

### Out Of Scope

- No combined implementation PR, shared claim, Desktop implementation, GPUI, capability resolver,
  dynamic provider, Bundle installation, Browser connector, release or publication.
- No direct Arborium redesign in I207/I208; no steering behavior change in I246.
- No activation of CAP-001 parent, DESKTOP-001, SESSION-009, RUNTIME-006 or DESKTOP-002.

### Non-Terminal Inventory At Selection

| Work | State | Disposition |
|---|---|---|
| I207 / TUI-049 | Complete / Closed | Implementation PR #483 merged as `ca3b2fa7`; native-terminal acceptance passed; closeout merged as `2edb914f`. |
| I208 / TUI-050 | Active / Claimed | Governance claim PR #487 merged as `75ca8057`; implementation starts from that merge and is locally converging. |
| I246 / CAP-001-P0 | Planned / Unclaimed | Third child; depends on CAP-001 decision or explicit narrowing and follows steering work. |
| I164 | Paused / superseded | Preserve; do not resume. |
| CAP-001 parent | Refinement / Unclaimed | Preserve; parent is not an implementation unit. |
| DESKTOP-001 | Deferred / Unclaimed | Preserve; mock/real Desktop work remains separately governed. |

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| S0 | Establish current-main inventory and claim for I207 | Effective I207 claim and active owner | Clean current main; no overlapping steering implementation | Governance PR merged, exact-head CI/review/CAS pass | Keep I207 Planned/Unclaimed | Complete (`8ff4c6f1`) |
| S1 | Implement I207 / TUI-049 | Padding-correct steering wrapping with focused and terminal evidence | S0 | Owner acceptance, local locked checks, exact-head review/CI, merge and closeout | Preserve existing layout and record residual | Complete (`ca3b2fa7`) |
| S2 | Establish I208 claim after I207 disposition | Effective I208 claim and active owner | S1 terminal; current-main refresh | Governance PR #487 merged and CAS pass | Keep I208 Planned/Unclaimed | Complete (`75ca8057`) |
| S3 | Implement I208 / TUI-050 | Deterministic boundary insertion and ordering evidence | S2 | Event-sequence tests, cancellation/error/restart evidence, exact-head review/CI, merge and closeout | Keep accepted queue custody and defer timing change | Active / local convergence |
| S4 | Establish I246 claim after steering sequence | Effective I246 claim and active owner | S3 terminal; CAP-001 decision/narrowing; overlap inventory | Governance PR merged and CAS pass | Keep CAP-001-P0 Refinement/Unclaimed | Planned |
| S5 | Implement I246 / CAP-001-P0 | Characterization, compatibility seam and Desktop handoff | S4 | Focused behavior/dependency evidence, docs, exact-head review/CI, merge and closeout | Preserve current paths and route unresolved decisions to child ADRs | Planned |
| S6 | Final ledger closeout | All three child owners and derived views synchronized | S1, S3, S5 | Existing completion evidence, residuals recorded, claim closed | Leave task Partial with explicit residual owner | Planned |

## Shared Validation And Ownership Rules

- Each child starts from the latest `main` after its own claim merge; no implementation branch exists before that point.
- Local convergence is the normal edit loop; one stable candidate PR per child, with related reviewer corrections batched locally.
- I207 owns steering layout/padding; I208 owns steering custody/timing; I246 owns text/language and Plugin compatibility seams.
- Root Cargo files, Cargo.lock, `talos-conversation`, CI workflows and any shared presentation API are overlap-gated and require explicit coordination.
- Owner documents are updated first, then this ledger, iteration index, Product Backlog and Board.
- Every Complete child records an existing implementation commit; a status-only commit cannot self-certify.

## Branch, Checkpoint And Recovery Plan

- Target branch: `main`; short-lived child branches only after effective claims.
- Append a checkpoint after each child claim, stable candidate, merge and closeout.
- At interruption, resume from this table and the active child owner; refresh `main`, claims and open PRs before acting.
- Never restore unrelated historical stashes or worktrees as part of this plan.

## Residual Destination

CAP-001-A/B/C, BUNDLE/TEXT/LANG/DIST/BROWSER, SESSION-009, RUNTIME-006 and Desktop implementation
remain separate owners. Any unresolved steering behavior receives a new bounded child rather than
expanding I207 or I208.

## Completion Evidence

- I207 Completion Commit: `ca3b2fa7ffb1ca14b82d1acf6af6be147368e6fe` (implementation merge for
  PR #483; closeout merged as `2edb914f139a6b3954f790d3fc2889eac16d4599`).
- I208 and I246 Completion Commit: Pending.
- This coordination record cannot self-certify child implementation completion.
