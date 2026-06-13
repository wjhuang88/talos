# Two-Week Handoff Plan: Context Closure And TUI Polish

> Status: Active handoff / I024 complete, Week 2 carryover ready
> Published: 2026-06-12
> Horizon: 2026-06-13 through 2026-06-26
> Owner: next implementer
> Baseline rule: preserve this plan as the handoff baseline; material scope changes need a new dated note.

## Status Update (2026-06-13)

I024 is complete. The Week 1 context closure work no longer blocks Week 2. Accepted residuals are
explicitly tracked and should not be rediscovered as hidden I024 blockers:

- LLM compaction layers 4-5 and 50-turn heavy-tool proof → `docs/backlog/active/MEM-003-llm-compaction.md`
- First-class workspace/session topology and same-basename workspace collision protection →
  `docs/backlog/active/MEM-004-workspace-session-topology.md`
- Tree-sitter code analysis remains research-only → `docs/backlog/active/CODE-001-tree-sitter-code-analysis-research.md`

The next external implementer should start with **TUI-005 Logo & Splash Screen** unless the user
explicitly reprioritizes CODE-001, MEM-004, or another backlog item.

## Handoff Correction Checklist

These checks come from issues found while reviewing and correcting previous external/agent work.
Apply them before claiming a slice is done:

1. **Do not claim verification without running it in the current tree.**
   Record the actual command and result. Stale `fmt`, `clippy`, or workspace-test claims caused
   incorrect Review status before.
2. **Owner docs first, Board second.**
   Update the iteration/backlog owner file before changing `docs/BOARD.md`. The Board is derived.
3. **Do not present compatibility stopgaps as final architecture.**
   The current workspace-scoped resume fix filters by workspace basename. The final model is
   MEM-004's `Workspace -> Session[]` topology with stable workspace identity.
4. **Validate runtime paths, not just library units.**
   Previous gaps hid in CLI/TUI integration: restored history reached the provider but did not
   hydrate visible TUI scrollback; slash commands intercepted `/mock-request` before the provider;
   interactive persistence duplicated assistant writes.
5. **Render timing matters in inline TUI.**
   TUI scrollback/history changes must respect the first-frame viewport setup. Flushing history
   before the viewport exists can erase restored lines or create apparent logo spacing bugs.
6. **Session selection must be workspace-scoped unless explicitly global.**
   `--continue` / `--resume` implicit selection must not pick the newest session from another
   workspace. Explicit `--session <id>` and `--fork <id>` may remain ID-based.
7. **Slash commands need an explicit routing decision.**
   Commands that should reach the model must be registered as model-passthrough; otherwise
   `talos-conversation` will return Unknown command before provider logic runs.
8. **Keep symbol semantics stable.**
   Assistant/tool prefix uses `●`. Reserve `○` and `◉` for the future todo-list component.
9. **Keep tree-sitter out of implementation until CODE-001 answers the ADR/dependency questions.**
   The Rust-first hard constraint means grammar/runtime dependencies need an explicit decision.
10. **For visible UX changes, update README or an owner usage doc.**
    Mock request diagnostics and TUI startup behavior are user-visible; docs must move with code.

## Current Recommended Next Slice

Proceed with **TUI-005 sub-slice A: styled scrollback splash foundation**:

- Replace the plain startup banner with a styled, inline-safe splash printed into terminal
  scrollback.
- Preserve the inline-by-default TUI model; do not enter alt-screen for the splash.
- Add narrow-width fallback and tests for render-mode selection/line constraints.
- Record runtime evidence for normal and narrow terminal widths before moving TUI-005 to Review.

Leave these for later unless explicitly reprioritized:

- TUI-006 syntax highlighting beyond rounded code-block borders.
- TUI-007 user-selectable theme system.
- CODE-001 tree-sitter dependency research.
- MEM-003 and MEM-004 memory/session residuals.

## Objective

Hand the next implementer a bounded path that preserves the original two-week baseline, records
the completed P0 conversation-context closeout, and focuses remaining capacity on a small
user-facing TUI polish slice.

Current hard gate: **do not reopen I024 unless a real regression is found**. I024 residuals are
registered in MEM-003/MEM-004. Week 2 work may proceed on TUI-005 after the correction checklist
above is read.

## Original Baseline Current State

Confirmed facts:
- I023 TUI State Model is Complete and is the stable UI/event foundation.
- At publication time, I024 Conversation Context was Active/Review-bound.
- At publication time, `docs/iterations/I024-conversation-context.md` still needed status/evidence
  synchronization before it could be closed.
- Subsequent closeout on 2026-06-13 moved I024 to Complete and registered residuals in MEM-003
  and MEM-004.

Original assumptions to validate on Day 1:
- TUI and inline modes both persist user and assistant turns through `SessionManager`.
- Resume flags (`-c` / `-r`) load JSONL history into `AppServerSession`.
- Interactive legacy mode either uses the same history path or is explicitly documented as
  deferred.
- Compaction behavior is adequate for I024 without implementing semantic memory.

2026-06-13 result:
- TUI, inline, and interactive persistence/resume paths are verified for I024 scope.
- Workspace-scoped implicit resume and visible TUI history hydration were corrected.
- Layers 1-3 compaction are accepted for I024; layers 4-5 are deferred to MEM-003.

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

Read for Week 2 / TUI-005 before coding:
- `docs/backlog/active/TUI-005-logo-splash.md`
- `docs/decisions/018-inline-by-default-tui.md`
- `docs/iterations/I023-tui-state-model.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/inline_terminal.rs`

## Week 1: Close I024 Conversation Context — Complete

### Day 1: State Audit And Plan Repair — Complete

Outcome:
- Establish the true I024 implementation state before writing code.

Tasks:
- Compare I024 acceptance criteria against current code and tests.
- Confirm `docs/iterations/I024-conversation-context.md` reflects the true state.
- Mark each I024 story as Done / In Progress / Not Started with short evidence.
- Check whether `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and
  `docs/BOARD.md` agree on I024 state.

Exit gate:
- A reviewer can tell exactly which I024 acceptance items remain.

### Day 2: History Pipeline Completion — Complete

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

### Day 3: JSONL Persistence And Resume — Complete

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

### Day 4: Compaction And Long-Session Safety — Complete With Residuals

Outcome:
- Long sessions stay inside the model context budget without semantic memory.

Tasks:
- Verify `Compactor::should_compact()` is called before provider calls.
- Add a focused test for long history applying budget/trim/microcompact layers.
- Avoid implementing semantic consolidation, vector search, or cross-session memory.
- Record any `read_messages()` fidelity gap as residual work if tool calls are still lossy.

Exit gate:
- Long-history tests prove context is bounded and recent turns remain available.

### Day 5: Runtime Verification And I024 Review — Complete

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
- I024 moved from Review to Complete on 2026-06-13 with residuals registered in MEM-003/MEM-004.

## Week 2: TUI Splash Slice, Now Unblocked

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
- I024 state remains synchronized across I024 doc, iterations README, backlog, and BOARD.
- Week 1 verification commands are recorded with actual results.
- Week 2 either completes a small TUI-005 slice or records the carryover reason in TUI-005 /
  BOARD without changing the baseline silently.
- README/user-facing docs are updated for any observable behavior change.
- `scripts/validate_project_governance.sh .` passes.
- The final commit history separates planning, memory closure, and TUI polish into logical commits.
