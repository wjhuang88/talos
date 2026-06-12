# I024: Conversation Context Continuity

**User can**: Have a multi-turn conversation where the agent remembers all previous messages
in the session, correctly references prior context, and does not hallucinate false memories.

## Status: Review

## Review Gap: Visible History Hydration (2026-06-12)

The `-c`/`--continue` path already restored JSONL messages into `SessionConfig.initial_history`,
so the next provider call received prior conversation context. The missing piece was visible TUI
history: the TUI history area started empty because `Tui::new()` did not hydrate scrollback from
the same restored messages.

This is a UX gap in I024 rather than a new memory feature: users can continue a session, but the
screen looks like a fresh conversation. Closeout requires the TUI to render restored user,
assistant, and tool-result messages into scrollback without re-submitting them to the agent and
without appending duplicate JSONL entries.

**Resolution**: Implemented TUI visible-history hydration from the same `initial_history` loaded
from JSONL. Restored messages render as completed scrollback blocks, reuse the existing stream
prefix and Markdown/table rendering path, and do not enter the agent SQ or append duplicate JSONL
records. Verification: `cargo test -p talos-tui`, `cargo check --workspace`,
`cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, and
`scripts/validate_project_governance.sh .` all pass on 2026-06-12.

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

- [x] #I024-S1: Agent API accepts conversation history (**Done** — evidence below)
- [x] #I024-S2: AppServerSession loads and passes history (**Done** — evidence below)
- [x] #I024-S3: CLI modes integrate SessionManager and JSONL persistence (**Done** — evidence below)
- [x] #I024-S4: Compaction wired into turn loop (**Done** — layers 1-3; layers 4-5 deferred to MEM-003)
- [x] #I024-S5: Tests and runtime verification (**Done** — 658 workspace tests pass)

## Stories Detail

### S1: Agent API accepts conversation history — **Done**

**Evidence** (2026-06-12 audit):

| Acceptance Criterion | Status | Evidence |
|---|---|---|
| `run_inner()` has a `history: Vec<Message>` parameter | ✅ | `lib.rs:367-370` — `run_inner(user_message, history, event_tx)` |
| History messages appear before the current user message in the provider call | ✅ | `lib.rs:394-397` — `let mut messages = history; messages.push(Message::User { content: full_message });` |
| All existing tests continue to pass | ✅ | `lib.rs:334` — `run()` passes `vec![]` as default |
| New unit test: agent with 3-message history | ✅ | `session.rs` — `test_initial_history_from_jsonl_resume` creates a JSONL-backed session, resumes it through `SessionManager`, and verifies prior user+assistant messages reach the provider before the new user message. |

**Gap**: No remaining S1 gap after the JSONL-backed resume test; direct `Agent::run_streaming` tests still use empty history, but the provider-facing path is covered through `AppServerSession`.

### S2: AppServerSession loads and passes history — **Done**

**Evidence** (2026-06-12 audit):

| Acceptance Criterion | Status | Evidence |
|---|---|---|
| `AppServerSession` maintains `history: Vec<Message>` across turns | ✅ | `session.rs:35` — `history: Vec<Message>` field; L59 initialized from `config.initial_history` |
| History is passed to agent on every `SessionOp::Submit` | ✅ | `session.rs:118` — `let history = self.history.clone();` → L128 passed to `TurnForwarding` → L222 `agent.run_streaming(message, history, event_tx)` |
| After turn completion, new messages are added to history | ✅ | `session.rs:162-174` — `commit_finished_turn()` pushes user + assistant messages |
| `SessionConfig` gains optional `initial_history: Vec<Message>` | ✅ | `core/session.rs` — `initial_history` field in `SessionConfig` |
| Session actor tests verify multi-turn history accumulation | ✅ | `session.rs:772-882` — `test_multi_turn_with_history`: 3 turns, verifies 3rd call has history from turns 1-2 |
| Interrupt preserves history | ✅ | `session.rs:884-969` — `test_interrupt_after_success_preserves_history`: verifies user msg + assistant response survive interrupt |

**Gap**: `commit_finished_turn` only saves `user_msg` + `assistant(content, tool_calls: vec![])`. Tool call messages (`Message::Tool { result }`) are NOT committed to in-memory history. This means tool-heavy sessions have incomplete history for subsequent turns. This is a known fidelity gap documented in I024 Non-Goals: "No `read_messages()` fidelity improvement (tool_calls serialization)".

### S3: CLI modes integrate SessionManager and JSONL persistence — **Done**

**Evidence** (2026-06-12 audit):

| Acceptance Criterion | Status | Evidence |
|---|---|---|
| `run_tui_mode()` creates `SessionManager`, loads/resumes session | ✅ | `main.rs:792-841` — creates `SessionManager`, handles `--session`/`--continue`/`--resume`/new |
| History passed to `AppServerSession` at initialization | ✅ | `main.rs:842-849` — `session.read_messages()` → `initial_history` → `SessionConfig` |
| After each turn, new messages persisted to JSONL | ✅ (TUI only) | TUI: `main.rs:862-926` — bridge forwarder persists assistant msg; user_msg_tx wrapper persists user msg. Inline: `main.rs:1096-1134` — persists both. |
| Resume (`-c` / `-r`) loads full conversation context | ✅ | `main.rs:842` — `session.read_messages().unwrap_or_default()` loads from JSONL |
| Inline and interactive modes also wired | ✅ Inline / ⚠️ Interactive | Inline: `main.rs:1011-1052` — has SessionManager + initial_history. Interactive: `main.rs:1172-1346` — has SessionManager + initial_history + JSONL persist in `event_loop.rs` (not audited in this session). |
| Print mode (`-p`) remains single-turn | ⚠️ Partially | `main.rs:610-616` — `print_mode: true`, `initial_history: vec![]`. Print mode does NOT wire JSONL persistence for its turn, but since it's single-turn this is acceptable. |

**Gaps**:
1. **Print mode JSONL persistence**: Print mode creates a session but never persists the user message or assistant response to JSONL. This is acceptable per I024 ("Print mode remains single-turn") but means print-mode turns are invisible to future `-c` resumes.
2. **Interactive mode event_loop persistence**: Verified and fixed during Day 3; `assistant_persisted` prevents double writes when both `TurnEnd` and `TurnCompleted::Success` arrive.

### S4: Compaction wired into turn loop — **Review with deferred residual**

**Evidence** (2026-06-12 audit):

| Acceptance Criterion | Status | Evidence |
|---|---|---|
| `Compactor::should_compact()` called before provider call | ✅ | `session.rs:107` — `if self.compactor.should_compact(&self.history)` |
| `Compactor::compact()` applied when history exceeds 80% of model limit | ⚠️ Partially | `session.rs:108-111` — applies layers 1-3 (budget, trim, microcompact). Layers 4-5 (collapse, autocompact) are NOT called — they require a provider reference which the session actor doesn't pass. |
| Long sessions (>20 turns) do not exceed token budget | ⏭️ Deferred | Layers 1-3 are wired; 50-turn heavy-tool validation depends on layers 4-5 and is tracked by MEM-003. |
| Unit test: 50-turn session compacts correctly | ⏭️ Deferred | Tracked by MEM-003 acceptance criteria. |

**Gaps**:
1. **Layers 4-5 not wired**: The `session.rs` compaction code only calls `apply_budget`, `apply_trim`, `apply_microcompact`. Layers 4-5 (`apply_collapse`, `apply_autocompact`) require an `&dyn LanguageModel` provider reference that isn't available in the current architecture (the session actor holds an `Arc<Agent>`, not a provider reference).
2. **No 50-turn integration test**: Need a test proving compaction keeps context bounded over many turns.

### S5: Tests and runtime verification — **Review**

| Acceptance Criterion | Status |
|---|---|
| Test: 3-turn conversation — agent references messages from turns 1-2 | ✅ `session.rs:772-882` |
| Test: resume session — agent has full history | ✅ `session.rs::test_initial_history_from_jsonl_resume` |
| Test: long session triggers compaction without errors | ⏭️ Deferred to MEM-003 |
| Runtime: TUI mode multi-turn conversation verified | ⚠️ Not manually re-run in this correction; covered by session/CLI wiring tests, TUI visible-history hydration tests, and remains Review evidence to collect before Complete |
| `cargo test --workspace` passes | ✅ Passes on 2026-06-12 |

## Day 1 Audit Summary (2026-06-12)

### Fully Done
- S1: Agent `run_inner()` accepts `Vec<Message>` history
- S2: `AppServerSession` maintains history, passes to agent, commits after turns

### Gaps Remaining
1. **S3 gap**: Verify interactive mode `event_loop.rs` JSONL persistence
2. **S3 gap**: Print mode doesn't persist to JSONL (acceptable, documented)
3. **S4 gap**: Compaction layers 4-5 (collapse/autocompact) not wired — requires provider access
4. **S4 gap**: No 50-turn compaction integration test
5. **S5 gap**: No resume-from-JSONL integration test
6. **S5 gap**: No runtime TUI multi-turn verification recorded
7. **S5 gap**: `cargo test --workspace` not yet run this session

### Priority for Day 2
1. Verify interactive mode `event_loop.rs` persistence
2. Wire compaction layers 4-5 OR document deferral with rationale
3. Run `cargo test --workspace` + `cargo clippy --workspace -- -D warnings`

## Day 2-4 Progress (2026-06-12)

### Day 2: History Pipeline — ✅ Complete
- Interrupt path: `SessionOp::Interrupt` → `cancel_token.cancel()` + `commit_finished_turn()` (session.rs:144-151)
- Shutdown path: `SessionOp::Shutdown` → `commit_finished_turn()` + break (session.rs:152-157)
- `commit_finished_turn` only commits when `JoinHandle` resolves with `Some(String)` — interrupted turns correctly produce `None` and are not committed
- Existing tests cover: `test_multi_turn_with_history` (3-turn history accumulation), `test_interrupt_after_success_preserves_history` (interrupt preserves history), `test_concurrent_submit_and_interrupt` (cancel path)

### Day 3: JSONL Persistence & Resume — ✅ Complete
- **TUI mode**: Bridge forwarder persists assistant on `TurnCompleted::Success` (main.rs:872-886). User msg wrapper persists before forwarding (main.rs:907-926). No duplicate writes.
- **Inline mode**: User msg persisted after submit (main.rs:1096-1102). Assistant persisted on `TurnCompleted::Success` (main.rs:1126-1137). No duplicate writes.
- **Interactive mode** (`event_loop.rs`): **Found and fixed duplicate persist bug**. Both `AgentEvent::TurnEnd` and `SessionEvent::TurnCompleted::Success` fire `AgentCompleted`, causing double JSONL writes. Fixed with `assistant_persisted` boolean guard.
- **Resume-from-JSONL test added**: `test_initial_history_from_jsonl_resume` in session.rs creates a JSONL-backed session, resumes it through `SessionManager`, loads `read_messages()`, and verifies prior user+assistant messages reach the provider.
- All 140 talos-agent tests pass + 13 doctests. All talos-cli tests pass.

### Day 4: Compaction — Deferred to MEM-003
- Layers 1-3 (budget, trim, microcompact) are wired and sufficient for short-to-medium sessions under typical message sizes; 40-50 turn heavy-tool claims remain unproven until MEM-003
- Layers 4-5 (collapse, autocompact) require `&dyn LanguageModel` provider reference through the session actor
- Architecture options documented in `docs/backlog/active/MEM-003-llm-compaction.md`
- No 50-turn integration test added (layers 4-5 are the ones that matter for that scale)
- This is a deliberate scope cut to close I024 without architecture thrash

### Updated S3 Status
- ✅ TUI mode JSONL persistence verified
- ✅ Inline mode JSONL persistence verified
- ✅ Interactive mode JSONL persistence verified (bug fixed)
- ✅ Resume test added and passing

### Updated S4 Status
- ✅ Compaction layers 1-3 wired and functional
- ⏭️ Layers 4-5 deferred to MEM-003 (P3 backlog item)

## Acceptance Criteria (Iteration-level)

- [x] Agent receives conversation history in every turn (S1 ✅)
- [x] History loaded from session JSONL on startup (S3 ✅ — TUI, inline, interactive)
- [x] Compaction triggered when history exceeds token budget (S4 ✅ — layers 1-3; layers 4-5 deferred to MEM-003)
- [x] New turns persisted to session JSONL (S3 ✅ — all modes)
- [x] All modes (TUI, inline, interactive) wired (S3 ✅)
- [x] Resume loads full conversation context (S5 ✅ — `test_initial_history_from_jsonl_resume`, JSONL-backed)
- [x] `cargo test --workspace` passes (Day 5 — 658 tests, 0 failures)
- [x] `cargo check --workspace` clean (Day 5)
- [x] `cargo clippy --workspace -- -D warnings` clean (Day 5)

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
