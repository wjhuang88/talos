# Iteration I156: TUI Narrow-Viewport And Resize Robustness

> Document status: Active
> Published plan date: 2026-07-26
> Planned objective: Tool-call summaries remain visible at narrow widths and resize never leaks viewport-fixed UI into scrollback.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Tool-call summaries remain visible at narrow widths and resize never leaks viewport-fixed UI into scrollback.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-035` | `None` | `Ready` | `TUI-034` complete; no conflicting active iteration | Tool-call summaries remain visible at narrow widths and resize never leaks viewport-fixed UI into scrollback. |

### Start Here

Read in order:

1. `AGENTS.md`
2. `docs/sop/START-ITERATION.md`
3. `docs/sop/ITERATION-WORKFLOW.md`
4. `docs/sop/CHANGE-CONTROL.md`
5. `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
6. this iteration
7. the selected Story
8. governing ADRs/specifications in Required Reads
9. the exact source files named by the Story

The selected Story owns scope and acceptance. This iteration owns activation, execution evidence,
variance, and completion state.

### Authorized Scope

- implement only TUI-035 Fix 1, Fix 2, and Fix 3;
- reuse existing display-width wrapping;
- add focused narrow-width, CJK, and resize tests;
- record a real Alacritty wide-to-narrow walkthrough.

### Forbidden Changes

- no alternate-screen/full renderer rewrite;
- no scrollback storage redesign;
- no tool-result policy change;
- no Desktop work;
- no unrelated TUI cleanup;
- no new unsafe/native dependency;
- no tag/release.

### Implementation Slices

1. **Baseline**
   - inspect current code and record current behavior;
   - run focused baseline tests;
   - list expected files before editing.
2. **Tests**
   - add failing focused tests for the Story acceptance;
   - do not rewrite unrelated tests.
3. **Minimum implementation**
   - implement the smallest change satisfying the selected Story.
4. **Runtime wiring**
   - prove the real Talos inline TUI path, including narrow width and live resize behavior.
5. **Documentation and owner sync**
   - update Story, iteration, parent, Board, and user/reference docs named by the Story.
6. **Validation and commit**
   - run focused and full validation;
   - inspect staged diff;
   - create one logical commit referencing `TUI-035` and `I156`.

### Non-Goals

- no alternate-screen/full renderer rewrite;
- no scrollback storage redesign;
- no tool-result policy change;
- no Desktop work;
- no unrelated TUI cleanup;
- no new unsafe/native dependency;
- no tag/release.

### Acceptance

All unchecked Acceptance items in `TUI-035` that belong to this iteration must be satisfied. Do not
invent acceptance criteria here.

### Planned Validation

```bash
cargo test --locked -p talos-tui
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

### Runtime Evidence

Record:
- width 40/80/120/160 tool-call wrapping;
- widths 1/2/3 bounded and no panic;
- CJK arguments;
- Alacritty continuous wide-to-narrow drag;
- no duplicated hint/status row or staircase fill;
- widen and height-shrink regression.

### Documentation To Update

- selected Story;
- this iteration;
- parent Epic/Story if its actual state changes;
- `docs/BOARD.md` derived view;
- `docs/backlog/PRODUCT-BACKLOG.md` compact row if state changes;
- `docs/iterations/README.md`;
- user/reference docs named by the Story.

### Risks And Rollback

- Preserve the previous runnable path until the new path has focused and runtime equivalence evidence.
- Roll back the iteration commit if a security, permission, persistence, or product-mode regression is found.
- Do not hide a failed gate by weakening acceptance or deleting tests.

### Stop And Escalate Conditions

- fixing the issue requires abandoning the targeted DECSTBM approach;
- bottom-pane/history ownership is not identifiable;
- a change affects transcript/export/session semantics;
- tests require product behavior outside TUI-035.

If a stop condition occurs:

1. stop editing;
2. record the exact code/document conflict under Variance And Residuals;
3. keep the iteration `Blocked` or `Review`;
4. do not create a speculative workaround;
5. request maintainer/architecture input.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-26 | Activation | Baseline `065d801`. Full Active/Review/Planned/Blocked inventory completed. No conflicting Active iteration exists. All Review and prior Planned items have explicit dispositions. TUI-035 is Ready. TUI-034 Complete (`616ff11`), TUI-025 Complete, ADR-035 Accepted dependency evidence verified. Source-path baseline remains compatible with Fix 1/2/3. Activation decision: GO. Primary activation executor/runtime: `glm-5.2` (Sisyphus orchestrator). Activation-only work does not qualify as REL-002 implementation evidence. |

### Activation Inventory — 2026-07-26

| Category | Items | Disposition |
|---|---|---|
| Active | None before activation | I156 may become the sole Active iteration |
| Review | None | No unresolved Review iteration |
| Planned | I156, I157 | I156 activate; I157 remain Planned (defer until I156 Complete) |
| Blocked | I158, I159, I160, I161, I162 | Blockers: I158 on ADR-053 Proposed; I159 on I158; I160 on I159; I161 on I160 + security review; I162 on I161 + readiness gate |
| Paused/Partial | None | — |
| Ambiguous | None | All owner states resolved |

- Baseline SHA: `065d801`
- Selected Story: `TUI-035`
- Story status before activation: `Ready`
- Dependencies:
  - TUI-034: Complete — maintainer Alacritty walkthrough 2026-07-24; Completion Commit `616ff11` (P4 `dd6e090`)
  - TUI-025: Complete (2026-07-04)
  - ADR-035: Accepted (2026-07-03)
- Source baseline (all present, no architectural rewrite):
  - `crates/talos-tui/src/tool_display.rs`: `build_tool_call_scrollback_line` (L454), `build_tool_result_scrollback_lines` (L233)
  - `crates/talos-tui/src/app.rs`: tool-call site (L621), tool-result site (L632), `wrap_scrollback_line` flush (L792), `Event::Resize(_, _) => {}` no-op (L1253)
  - `crates/talos-tui/src/app_stream.rs`: `wrap_scrollback_line` (L76)
  - `crates/talos-tui/src/inline_terminal.rs`: `set_viewport_area` (L231), `insert_styled_history` (L338)
  - `crates/talos-tui/src/scrollback_markdown.rs`: `append_fill_segment` (L336)
- Primary activation executor/runtime: `glm-5.2` (Sisyphus orchestrator)
- Intended implementation executor/runtime: pending (recorded when implementation begins)
- External assistance: none
- Decision: `GO`

## Verification Evidence

- Focused tests: passed — `cargo test --locked -p talos-tui` → 404 passed, 0 failed (added tool-call width matrix 40/80/120/160, CJK wrap, continuation alignment, styling preservation; wrap edge 0/1/2/3/4; `resize_clear_action` decision matrix incl. zero-height safety).
- Full locked validation: passed — `cargo fmt --all -- --check` clean; `cargo check --workspace --locked` clean; `cargo clippy --workspace --locked -- -D warnings` clean; `cargo test --workspace --locked` → 2431 passed, 0 failed.
- Runtime evidence: automated gates green; real Alacritty walkthrough PENDING (human gate, not yet performed).
- Governance validation: passed — `scripts/validate_project_governance.sh .` 0 warnings; `git diff --check` clean.

## Implementation Record

- Implementation baseline: `2f9de9f` (ancestor `065d801` activation baseline).
- Implementation Commit: `2f9de9f` — `fix(tui): harden narrow viewport and resize rendering (#TUI-035)`.
- Correction Commit: `5d11926` — `fix(tui): make extreme-width rendering renderer-accounted (#TUI-035)`. Removed terminal-autowrap dependency at viewport widths 1/2/3: `wrap_scrollback_line` now splits every line to fit the viewport (width 0 → empty; width > 0 → each row ≤ viewport), with CJK substitution marker at width 1 and anti-prefix-only continuation. Added `prepare_history_rows` helper for testability. Builder→renderer chain and physical row accounting regressions added.
- History-Preservation Commit: `dc28392` — `fix(tui): preserve history across unrenderable widths (#TUI-035)`. Fixed two data-integrity defects: (1) width 0 no longer consumes pending scrollback (`flush_pending_scrollback` returns early before `mem::take`); (2) CJK at width 1 is deferred (not permanently substituted). `prepare_history_rows` returns `HistoryPreparation { ready, deferred }`; lines with unrenderable scalars are deferred with full segments/style/fill intact, restored losslessly when width becomes renderable. Fill-bearing overflow also deferred. 417 focused tests, 2444 workspace.
- FIFO/Recovery Commit: `db85a3e` + `b24d9e0` — `fix(tui): preserve pending history order and failure recovery (#TUI-035)`. Fixed two correctness defects: (1) FIFO reorder — `prepare_history_rows` now uses `ready_prefix`/`deferred_suffix` with a `blocked` flag: once the first unrenderable line is encountered, ALL subsequent lines enter `deferred_suffix` regardless of their own renderability, preserving strict insertion order. (2) Insertion failure loss — `flush_pending_scrollback` tracks `committed_count` per logical line; on I/O failure, only the uncommitted physical-row tail is restored (not the full logical.original), preventing duplication of already-flushed rows. `PreparedLogicalLine { original, physical_rows }` retains the original for lossless recovery. `flush_prepared_with_writer` test seam with `MockWriter` covers plain insert failure, styled insert failure, retry-after-failure exactly-once, and partial-physical-row no-duplication. 424 focused tests, 2451 workspace.
- Semantic-Row Recovery Commit: `6909675` — `fix(tui): preserve all uncommitted physical history rows (#TUI-035)`. Removed `.filter(|r| !r.text.is_empty())` from both production (`flush_pending_scrollback`) and test helper (`flush_prepared_with_writer`). Empty-text rows carry semantics via segments, fill, bg, and style attrs — they are not empty facts. Recovery now restores the full `physical_rows[committed_count..]` slice without any content-based filtering. 5 new tests: empty physical row preserved, fill-only preserved, styled-empty preserved, retry-exactly-once with empty row, narrower-width rewrap of recovered suffix. 429 focused tests, 2456 workspace.
- Files changed: `crates/talos-tui/src/{tool_display,app,app_stream,inline_terminal,scrollback}.rs`.
- Decisions:
  - Fix 1: `build_tool_call_scrollback_line` → `build_tool_call_scrollback_lines(display, viewport_width)`, reuses `wrap_to_display_width`; styled prefix on row 0, dim continuation indent on subsequent rows; `MIN_TOOL_CALL_ARGS_BUDGET=20` floor mirrors tool-result.
  - Fix 2: `wrap_scrollback_line` returns the line as-is below `MIN_WRAP_WIDTH=4` (covers width 0/1/2/3) instead of shredding; content preserved, no panic, no runaway rows.
  - Fix 3: `Event::Resize` → `InlineTerminal::notify_resize()` (forces full clear+repaint on next draw); new pure `resize_clear_action(previous, next)` clears viewport rows on width-shrink so stale wide bottom-pane content cannot remain or leak into scrollback.
- Invariant: the viewport-fixed bottom hint/status/composer pane is rendered only via `terminal.draw()`, never via `insert_history` — confirmed structurally; Fix 3 strengthens resize cleanup.
- Maintainer Alacritty walkthrough: pending.
- Completion status: NOT YET ELIGIBLE — automated gates passed, human gate pending; no Completion Commit recorded.

## Completion Evidence

- Completion Commit: pending
- Do not cite a status-only documentation commit as implementation completion.
- Keep `Review`, `Partial`, or `Blocked` if implementation, runtime evidence, CI, or human acceptance is pending.

## Variance And Residuals

### Architecture Correction — 2026-07-27

**Classification:** in-scope correction. The published objective remains unchanged:
tool-call summaries must remain visible at narrow widths and fixed UI must never
leak into scrollback. Real-terminal evidence disproved the original targeted
DECSTBM approach, so this correction changes implementation ownership rather
than the deliverable or acceptance target. ADR-054 records the proposed
alternate-screen, app-owned transcript renderer.

#### Manual Alacritty Walkthrough — Failed

Observed:

- repeated copies of the fixed `Enter to send...` hint row;
- stale composer rows persisted above the newly drawn composer;
- stale model/status rows entered terminal scrollback;
- duplicates accumulated after repeated width and height resize.

Conclusion:

- the current primary-screen DECSTBM inline renderer does not satisfy TUI-035;
- I156 remains Active;
- TUI-035 remains In Progress.

- Screenshot evidence: maintainer-provided Alacritty resize capture, 2026-07-27.
- Planned correction: ADR-054 Proposed; move history facts into an application-owned
  transcript and render history plus fixed panes in one alternate-screen frame.

- The real-terminal resize walkthrough (continuous wide→narrow drag, width 1/2/3 extreme, height-only shrink, widen) remains the mandatory human gate before I156 can move to Complete.
- `resize_clear_action` is tested as a pure decision function; the full end-to-end "no history duplication during drag" is structurally guaranteed (bottom pane never calls `insert_history`) plus the resize-triggered full repaint, but final visual confirmation belongs to the human gate.

## REL-002 Execution Record

- Primary executor/runtime: `glm-5.2` (Sisyphus orchestrator) — pre-recorded at activation; implementation performed by the same runtime.
- External assistance: none.
- Planning/editing/testing/docs/commit/push ownership: Sisyphus orchestrator (`glm-5.2`); push authorized separately by maintainer.
- Qualification verdict: pending; this iteration is not automatically REL-002-qualifying.

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
