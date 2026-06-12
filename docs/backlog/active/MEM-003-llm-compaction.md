# MEM-003: LLM-Based Compaction (Layers 4-5)

| Field | Value |
|-------|-------|
| Story ID | MEM-003 |
| Priority | P3 |
| Status | Planned |
| Depends On | MEM-002/I024 in Review; close or explicitly accept residuals before activation |
| Blocks | None |
| Origin | I024 Day 4 audit gap |

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
