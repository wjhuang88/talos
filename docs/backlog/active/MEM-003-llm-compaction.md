# MEM-003: LLM-Based Compaction (Layers 4-5)

| Field | Value |
|-------|-------|
| Story ID | MEM-003 |
| Priority | P3 |
| Status | In Progress / Partial (T4 delegated to I312; #817 merged and effective) |
| Depends On | MEM-002/I024 completed lower-layer baseline; higher-layer residuals remain here |
| Blocks | None |
| Origin | I024 Day 4 audit gap |

## Current implementation and acceptance mapping — 2026-11-01

The plan and older snapshots below are preserved history. ADR-096 replaces the
old raw-history mutation architecture with a shared, structured checkpoint over
the fully budgeted request projection. I312 is active after #817; implementation
is local and has no merged completion evidence yet.

| Original acceptance | Current replacement and evidence boundary |
|---|---|
| Actor calls `Compactor::compact()` / all five layers | Deliberately superseded by ADR-096: Actor automatic/manual paths use the Agent's shared checkpoint pipeline, retaining raw history. The legacy model-assisted API now refuses missing request authority and has no separate summarizer. The literal old call requirement is not claimed satisfied. |
| 50 heavy-tool turns remain bounded | `session/checkpoint_acceptance_tests.rs::fifty_tool_turns_preserve_semantics_raw_export_and_durable_restart` exercises actual Actor turns, checkpoints and admitted request bounds. This is deterministic fixture evidence, not real-model quality assurance. |
| Summary failures do not crash/retry storm | Shared bounded stream decoding, checkpoint attempt ledger and manual/automatic failure regressions replace the old independent circuit breaker. |
| Existing session tests pass | Retained Actor/session regressions must pass on the final candidate; legacy tests expecting unsafe API success are replaced by explicit refusal/input-preservation regressions. |
| Simulated 50-turn integration | The same real Actor fixture covers semantic markers, raw export and durable restart; text-heavy manual/fork fixtures cover sequential folding. |

The deprecated Compactor signatures remain callable, but always fail before
provider invocation; this is a behavior-breaking next-minor migration, not a
compatible preservation of legacy summary behavior. Manual refusal returns the
original messages. Deterministic utility methods do not confer session authority.
Full preflight, independent final review, platform CI and merge remain required.
MEM-003 is not Complete; I312 owns remaining evidence and user-reference sync.

## Problem

The `Compactor` implements 5 progressive compaction layers, but only layers 1-3 (budget, trim, microcompact) are wired into the session actor's turn loop. Layers 4-5 (collapse, autocompact) require calling the LLM provider to summarize conversation history, but the session actor does not have access to the provider reference.

Layers 1-3 are sufficient for short-to-medium sessions under typical message sizes. The previously cited 40-50 turn boundary is an assumption, not verified evidence. Layers 4-5 are needed for extreme long sessions (>50 turns with heavy tool use) where the accumulated context exceeds the model limit even after budget/trim/microcompact.

## Approach

### Option A: Expose provider through Agent

Add `Agent::compact_history(&self, history: Vec<Message>) -> Result<Vec<Message>>` that delegates to the internal provider. The session actor calls this method instead of directly accessing the provider.

**Pros**: Minimal architecture change; encapsulation preserved.
**Cons**: Agent takes on compaction responsibility.

### Option B: Pass provider reference to session actor

Restructure `AppServerSession::new` to accept a provider reference (via `Arc`) alongside the `Agent`.

**Pros**: Clean separation of concerns.
**Cons**: Duplicates the provider reference; both Agent and session actor hold it.

### Option C: Async compaction task

Spawn a separate async task that has access to the provider and compacts history on demand.

**Pros**: Non-blocking; clean separation.
**Cons**: More complex coordination; history needs to be swapped atomically.

## Acceptance Criteria

- `AppServerSession` calls `Compactor::compact()` (all 5 layers) instead of individual layer methods
- A 50-turn session with heavy tool use does not exceed the model token limit
- LLM summarization failures do not crash the session — circuit breaker prevents retry storms
- Existing session tests continue to pass
- New integration test: simulated 50-turn session proves compaction keeps context bounded

## Current State

Layers 1-3 are wired in `session.rs:106-112`:
```rust
if self.compactor.should_compact(&self.history) {
    let compacted = self.compactor.apply_budget(self.history.clone());
    let compacted = self.compactor.apply_trim(compacted);
    let compacted = self.compactor.apply_microcompact(compacted);
    self.history = compacted;
}
```

Layers 4-5 are implemented in `compaction.rs` (`apply_collapse`, `apply_autocompact`) but not called.

## Current source reconciliation — 2026-10-30

The wiring description above is historical: both current Session submission paths
call Compactor::compact with the Agent provider. This does not prove the five
acceptance rows complete. Planned I312 consumes their higher-layer convergence
and 50-turn verification within T4, under MEM-005's request-budget policy. No
second summarizer is authorized. I312 remains unclaimed before atomic activation.

## Atomic T4 activation proposal — #817

Execution is delegated to I312, proposed Claimed/Active in #817 and ineffective
until target-branch merge. Original acceptance remains unchanged. Prior preparation
status is historical; no implementation exists yet. #390/T5 remains outstanding.
