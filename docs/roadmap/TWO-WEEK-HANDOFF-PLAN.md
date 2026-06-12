# Two-Week Handoff Plan: Context Closure And TUI Polish

> Status: Planned handoff
> Published: 2026-06-12
> Horizon: 2026-06-13 through 2026-06-26
> Owner: next implementer
> Baseline rule: preserve this plan as the handoff baseline; material scope changes need a new dated note.

## Objective

Hand the next implementer a bounded two-week path that first closes the P0 conversation
context work, then uses remaining capacity on a small user-facing TUI polish slice.

The plan has one hard gate: **do not start the Week 2 feature slice until I024 is in Review
with clean verification evidence**. If I024 is not ready by the end of Day 5, spend Week 2
on I024 hardening and closeout instead.

## Current State

Confirmed facts:
- I023 TUI State Model is Complete and is the stable UI/event foundation.
- I024 Conversation Context is Active in `docs/BOARD.md`.
- `docs/iterations/I024-conversation-context.md` still needs status/evidence synchronization
  before it can be closed.
- Recent commits have implemented part of I024, including history passing and completed-turn
  history preservation, but the full I024 acceptance checklist has not been audited end to end.

Assumptions to validate on Day 1:
- TUI and inline modes both persist user and assistant turns through `SessionManager`.
- Resume flags (`-c` / `-r`) load JSONL history into `AppServerSession`.
- Interactive legacy mode either uses the same history path or is explicitly documented as
  deferred.
- Compaction behavior is adequate for I024 without implementing semantic memory.

## Required Reads

Read these before coding:
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/backlog/active/MEM-002-conversation-context-continuity.md`
- `docs/iterations/I024-conversation-context.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `docs/proposals/session-context-contamination.md`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-agent/src/session.rs`
- `crates/talos-agent/src/compaction.rs`
- `crates/talos-cli/src/main.rs`
- `crates/talos-session/src/lib.rs`

Read for Week 2 only after I024 is in Review:
- `docs/backlog/active/TUI-005-logo-splash.md`
- `docs/decisions/018-inline-by-default-tui.md`
- `docs/iterations/I023-tui-state-model.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/inline_terminal.rs`

## Week 1: Close I024 Conversation Context

### Day 1: State Audit And Plan Repair

Outcome:
- Establish the true I024 implementation state before writing code.

Tasks:
- Compare I024 acceptance criteria against current code and tests.
- Confirm `docs/iterations/I024-conversation-context.md` remains correctly marked Active.
- Mark each I024 story as Done / In Progress / Not Started with short evidence.
- Check whether `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and
  `docs/BOARD.md` agree on I024 state.

Exit gate:
- A reviewer can tell exactly which I024 acceptance items remain.

### Day 2: History Pipeline Completion

Outcome:
- Every relevant runtime path passes prior messages to the agent.

Tasks:
- Verify `Agent::run_streaming()` and `run_inner()` receive history before the current user
  message.
- Verify `AppServerSession` commits successful turns to in-memory history on normal completion,
  late interrupt, and shutdown paths.
- Add or repair tests for 3-turn history accumulation and late interrupt preservation.
- Keep print mode single-turn unless the I024 doc explicitly expands scope.

Exit gate:
- Agent/session tests prove prior user and assistant messages are present in later provider calls.

### Day 3: JSONL Persistence And Resume

Outcome:
- TUI and inline sessions persist and resume conversation context.

Tasks:
- Verify TUI startup creates/resumes a session and loads `session.read_messages()`.
- Verify user and assistant turns append to JSONL once, in order, with no duplicate assistant
  writes on cancellation.
- Verify `-c` / `-r` resume loads prior messages into the agent turn loop.
- Decide and document legacy interactive mode behavior if it cannot be wired safely in this slice.

Exit gate:
- Tests or a deterministic harness prove resume includes prior messages in the next provider call.

### Day 4: Compaction And Long-Session Safety

Outcome:
- Long sessions stay inside the model context budget without semantic memory.

Tasks:
- Verify `Compactor::should_compact()` is called before provider calls.
- Add a focused test for long history applying budget/trim/microcompact layers.
- Avoid implementing semantic consolidation, vector search, or cross-session memory.
- Record any `read_messages()` fidelity gap as residual work if tool calls are still lossy.

Exit gate:
- Long-history tests prove context is bounded and recent turns remain available.

### Day 5: Runtime Verification And I024 Review

Outcome:
- I024 is ready for review, or Week 2 remains on I024 hardening.

Tasks:
- Run:
  - `cargo fmt --all --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test --workspace`
- Runtime verify:
  - TUI three-turn conversation: turn 3 can reference turns 1-2.
  - Resume conversation: new turn sees prior session history.
  - Cancel during processing: no duplicate persisted assistant turn.
- Update I024 evidence, `docs/iterations/README.md`, backlog, BOARD, and README if user-visible
  behavior changed.

Exit gate:
- I024 moves to Review with real evidence, or the plan records why Week 2 remains I024-only.

## Week 2: TUI Splash Slice, Only If I024 Is In Review

### Day 6: TUI Status Cleanup And Slice Confirmation

Outcome:
- Avoid implementing stale TUI work.

Tasks:
- Reconcile TUI backlog drift: `TUI-001` is listed as Planned while I014 says TUI completion is
  Complete. Update owner docs before implementation if this is stale.
- Confirm `TUI-005` is still the next TUI user-facing slice.
- Preserve symbol semantics:
  - Assistant/tool stream prefix: `●`
  - Future todo list: `○` incomplete, `◉` complete
  - Do not reuse `○` / `◉` for unrelated sidebar/status icons.

Exit gate:
- TUI-005 has no stale dependency or symbol conflict.

### Day 7: Splash Rendering Foundation

Outcome:
- Add a reusable splash/logo rendering unit without changing the main event architecture.

Tasks:
- Implement a small logo/splash module under `crates/talos-tui/src/`.
- Prefer ratatui-native rendering; do not add image or Node/Python runtime dependencies.
- Keep inline-by-default behavior; no alt-screen splash.
- Add narrow-terminal fallback if Canvas rendering is too constrained.

Exit gate:
- Unit tests cover render-mode selection and generated line count/constraints.

### Day 8: Startup Integration

Outcome:
- Splash appears at startup and dismisses without breaking input, scrollback, or cursor sync.

Tasks:
- Replace the plain banner path with styled splash output or a short-lived viewport component.
- Auto-dismiss after timeout or first user input.
- Ensure splash output remains compatible with terminal scrollback.
- Ensure raw mode and cursor restoration remain correct.

Exit gate:
- TUI starts normally after splash; user can type immediately or dismiss by typing.

### Day 9: Runtime QA And README Sync

Outcome:
- TUI splash is demonstrably usable, not just compiled.

Tasks:
- Verify desktop-width and narrow-width terminal behavior.
- Check no overlap with input/status rows.
- Update README or usage docs if startup behavior changed.
- Add screenshots/log notes only if the project already keeps such evidence.

Exit gate:
- Runtime evidence is recorded in the relevant iteration/backlog owner.

### Day 10: Closeout Or Carryover

Outcome:
- The handoff cycle ends with truthful status.

Tasks:
- Run:
  - `cargo fmt --all --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test --workspace`
  - `scripts/validate_project_governance.sh .`
- Update `docs/BOARD.md` after owner docs.
- Commit one logical change per slice.
- Record residual work in backlog/iteration docs, not only in chat.

Exit gate:
- Either TUI-005 is in Review with evidence, or remaining work is explicitly recorded.

## Do Not Do In This Two-Week Window

- Do not implement semantic memory, vector/graph retrieval, or cross-session memory.
- Do not add a global pub/sub event bus; ADR-006 keeps that out of bounds.
- Do not introduce new non-Rust runtime dependencies.
- Do not expand provider plugin architecture beyond accepted schema work.
- Do not start I019/I020 before I024 closes and OBS-001 is reassessed.
- Do not use `○` or `◉` outside the future todo list component.

## Handoff Definition Of Done

The handoff is complete when:
- I024 state is synchronized across I024 doc, iterations README, backlog, and BOARD.
- Week 1 verification commands are recorded with actual results.
- Week 2 either completes a small TUI-005 slice or records why I024 consumed the whole window.
- README/user-facing docs are updated for any observable behavior change.
- `scripts/validate_project_governance.sh .` passes.
- The final commit history separates planning, memory closure, and TUI polish into logical commits.
