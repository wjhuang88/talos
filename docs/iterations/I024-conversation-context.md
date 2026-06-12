# I024: Conversation Context Continuity

**User can**: Have a multi-turn conversation where the agent remembers all previous messages
in the session, correctly references prior context, and does not hallucinate false memories.

## Status: Active

## Activation Note (2026-06-12)

I024 is active because recent work has already landed parts of S1-S4. The next implementer
must start with a state audit before claiming completion: compare current code and tests
against each story below, then update this document with per-story evidence.

## Scope

Wire existing session storage and compaction infrastructure into the agent turn loop so
every LLM call receives conversation history. This is the Working Memory + Episodic Memory
layer from ADR-016, split from I019 as a P0 prerequisite.

## Decision Record

ADR-016 defines four memory layers. The user decided to split the implementation:
- **I024** (this iteration): Working Memory + Episodic Memory — history loading, turn wiring,
  compaction, JSONL persistence
- **I019** (future): Semantic Memory — fact consolidation, contradiction metadata, bounded
  semantic retrieval

Rationale: Working/Episodic is a prerequisite for Semantic. The P0 context gap makes TUI mode
unreliable for multi-turn conversations.

## Selected Stories

- [ ] #I024-S1: Agent API accepts conversation history (implementation landed; evidence audit pending)
- [ ] #I024-S2: AppServerSession loads and passes history (implementation landed; evidence audit pending)
- [ ] #I024-S3: CLI modes integrate SessionManager and JSONL persistence (implementation landed; resume/runtime audit pending)
- [ ] #I024-S4: Compaction wired into turn loop (implementation landed; long-session audit pending)
- [ ] #I024-S5: Tests and runtime verification (not complete)

## Stories Detail

### S1: Agent API accepts conversation history

Modify `Agent::run_inner()` to accept optional `Vec<Message>` history. Prepend history
before the current user message in the `messages` vector. Update `run_streaming()` signature
accordingly.

**Files**: `crates/talos-agent/src/lib.rs`

**Acceptance**:
- `run_inner()` has a `history: Vec<Message>` parameter (or equivalent)
- History messages appear before the current user message in the provider call
- All existing tests continue to pass (history defaults to empty vec)
- New unit test: agent with 3-message history responds contextually

### S2: AppServerSession loads and passes history

Add in-memory history tracking to `AppServerSession`. On each turn, prepend accumulated
history before calling `run_streaming`. After each turn, append the turn's user message and
assistant response to the history.

**Files**: `crates/talos-agent/src/session.rs`, `crates/talos-core/src/session.rs`

**Acceptance**:
- `AppServerSession` maintains `history: Vec<Message>` across turns
- History is passed to agent on every `SessionOp::Submit`
- After turn completion, new messages are added to history
- `SessionConfig` gains optional `initial_history: Vec<Message>` for session resume
- Session actor tests verify multi-turn history accumulation

### S3: CLI modes integrate SessionManager and JSONL persistence

Add `SessionManager` to TUI and inline modes. On startup: create or resume session, load
history via `session.read_messages()`, pass to session actor. After each turn: append new
messages to JSONL via `session.append()`.

**Files**: `crates/talos-cli/src/main.rs`

**Acceptance**:
- `run_tui_mode()` creates `SessionManager`, loads/resumes session
- History passed to `AppServerSession` at initialization
- After each turn, new messages persisted to JSONL
- Resume (`-c` / `-r`) loads full conversation context
- Inline and interactive modes also wired
- Print mode (`-p`) remains single-turn (no history needed)

### S4: Compaction wired into turn loop

Trigger `Compactor::should_compact()` before each provider call. If history exceeds
threshold, run compaction pipeline. Wire `TokenEstimator` for token counting.

**Files**: `crates/talos-agent/src/session.rs` or `crates/talos-agent/src/lib.rs`

**Acceptance**:
- `Compactor::should_compact()` called before provider call
- `Compactor::compact()` applied when history exceeds 80% of model limit
- Long sessions (>20 turns) do not exceed token budget
- Unit test: 50-turn session compacts correctly

### S5: Tests and runtime verification

Integration tests for multi-turn context, resume, and compaction. Runtime verification
in TUI mode.

**Acceptance**:
- Test: 3-turn conversation — agent references messages from turns 1-2
- Test: resume session — agent has full history
- Test: long session triggers compaction without errors
- Runtime: TUI mode multi-turn conversation verified
- `cargo test --workspace` passes

## Acceptance Criteria (Iteration-level)

- [ ] Agent receives conversation history in every turn
- [ ] History loaded from session JSONL on startup
- [ ] Compaction triggered when history exceeds token budget
- [ ] New turns persisted to session JSONL
- [ ] All modes (TUI, inline, interactive) wired
- [ ] Resume loads full conversation context
- [ ] `cargo test --workspace` passes

## Non-Goals

- No Semantic Memory consolidation (I019)
- No Procedural Memory (I019)
- No vector/graph retrieval (I019/I020)
- No cross-session memory
- No fork identity fix (#ARCH-S6)
- No `read_messages()` fidelity improvement (tool_calls serialization)

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Token budget overflow | LLM call fails | Existing 5-layer Compactor |
| `read_messages()` loses tool_calls | Context incomplete for tool-heavy sessions | Accept in first slice; tool_calls still in JSONL |
| `SessionOp` API change breaks callers | Compilation errors | Backward-compatible: history optional |
| Large JSONL load latency | Slow session resume | Compactor bounds the context window |

## Dependencies

- I023 Complete (✅)
- ADR-016 (Accepted)

## Required Reads

- `docs/backlog/active/MEM-002-conversation-context-continuity.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `docs/proposals/session-context-contamination.md`
- `crates/talos-session/src/lib.rs`
- `crates/talos-agent/src/compaction.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-agent/src/session.rs`
- `crates/talos-cli/src/main.rs`
