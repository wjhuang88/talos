# MEM-002: Conversation Context Continuity

## Outcome

Talos agents maintain conversation history across turns. The LLM receives prior messages
from the current session so it can reason about what was previously discussed.

## Status

Complete. Delivered through I024; history wiring, JSONL-backed resume, visible TUI resume
hydration, workspace-scoped implicit resume, and layers 1-3 compaction evidence are recorded in
the iteration. Accepted residuals are tracked by MEM-003 and MEM-004.

## Priority

P0. Blocks daily use — the agent cannot sustain a multi-turn conversation without context.

## Problem

`Agent::run_inner()` starts fresh every turn with only the current user message. Session
JSONL storage and a 5-layer compaction module exist but are never wired into the turn loop.
Every LLM call is amnesiac, causing hallucinations and broken user trust.

## Existing Infrastructure

| Component | Location | Status |
|-----------|----------|--------|
| Session JSONL storage + `read_messages()` | `talos-session/src/lib.rs` | Built, unused by agent |
| 5-layer Compactor (budget/trim/microcompact/collapse/autocompact) | `talos-agent/src/compaction.rs` | Built, never called |
| TokenEstimator | `talos-agent/src/token.rs` | Built, unused |
| `Message` type (User/Assistant/Tool) | `talos-core/src/message.rs` | Defined |
| `ConversationEngine.messages` | `talos-conversation/src/engine.rs` | TUI display only |

## Approach

Wire existing infrastructure into the agent turn loop:

1. CLI modes create `SessionManager`, load/resume session, read history via `read_messages()`
2. History passed to `AppServerSession` at initialization
3. Session actor prepends history before calling `agent.run_streaming()`
4. `run_inner()` accepts history, inserts before current user message
5. Compaction triggered when history exceeds token budget threshold
6. After each turn, new messages appended to in-memory history and written to JSONL

This is the Working Memory + Episodic Memory layer from ADR-016. Semantic Memory
consolidation remains in I019.

## Acceptance Criteria

- [x] Agent receives conversation history in every turn (not just the current message)
- [x] History is loaded from session JSONL on startup via `SessionManager`
- [x] Compaction is triggered when history exceeds token budget (`Compactor::should_compact`)
- [x] New turns are persisted to session JSONL after completion
- [x] All modes in I024 scope pass history through the pipeline; print mode remains documented
  single-turn behavior
- [x] Resume (`-c` / `-r`) loads full conversation context from prior session
- [x] False-history risk is reduced by using current-session JSONL history instead of relying on
  the model to infer prior turns
- [x] `cargo test --workspace` passes

## Out of Scope

- Semantic memory consolidation (I019)
- Procedural memory extraction (I019)
- Vector/graph retrieval (I019/I020)
- Cross-session memory (future)
- Fork identity fix (#ARCH-S6) — separate from context wiring

## Risks

| Risk | Mitigation |
|------|------------|
| Token budget overflow from long sessions | Existing 5-layer Compactor handles this |
| `read_messages()` loses tool_calls fidelity | Accept fidelity gap in first slice; improve serialization later |
| Session JSONL read performance on large files | Bounded by Compactor; worst case loads then compacts |
| API surface change on `SessionOp::Submit` or `Agent::run_inner` | Backward-compatible: history is optional |

## Required Reads

- `docs/decisions/016-layered-memory-architecture.md` — four-layer memory model
- `docs/proposals/session-context-contamination.md` — original P0 investigation
- `crates/talos-session/src/lib.rs` — Session, SessionManager, read_messages()
- `crates/talos-agent/src/compaction.rs` — Compactor, TokenEstimator
- `crates/talos-agent/src/lib.rs` — Agent struct, run_inner()
- `crates/talos-agent/src/session.rs` — AppServerSession, run_turn_with_forwarding
- `crates/talos-cli/src/main.rs` — run_tui_mode, run_conversation_loop

## Dependencies

- I023 Complete (TUI state model — provides stable event-driven architecture)
- I019 (Semantic Memory) depends on this — I024 is a prerequisite

## Residual Work Destination

Semantic consolidation and cross-session memory remain in MEM-001 / I019. LLM compaction layers
4-5 and 50-turn heavy-tool proof are tracked by MEM-003. First-class workspace/session topology,
stable workspace identity, and same-workspace multi-session awareness are tracked by MEM-004.
