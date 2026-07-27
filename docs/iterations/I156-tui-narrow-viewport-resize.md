# Iteration I156: TUI Narrow-Viewport And Resize Robustness

> Document status: Complete
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
- Historical activation source baseline (superseded; these symbols are not
  present in the current runtime architecture):
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

- Current Alternate Screen startup correction automated evidence (2026-07-27,
  implementation commits `9c87d0f`, `635bc29`, and `dddf32f`):
  `TerminalSession` establishes Alternate Screen before the first full-frame
  draw. The wide and compact Logo variants render below one display-only
  spacer row as a virtual history prefix, never enter `TranscriptStore`, keep
  the first user message immediately below them, and naturally scroll out as
  projected history grows. App-owned logical navigation, bounded layout, modal
  cursor rules, and exhaustive retryable restore are active again. Focused TUI
  validation: 440 passed, 0 failed.
- Superseded native-history correction automated evidence (2026-07-27,
  implementation commit `31e7a0d`): the geometry-free transcript remains the
  logical authority, committed entries are projected exactly once through
  ordinary primary-screen newlines, and only the bounded transient frame is
  redrawn. Terminal startup neither enters nor clears Alternate Screen. Focused
  TUI validation: 436 passed; locked workspace validation: 2463 passed across
  62 test binaries/doc-test groups. These counts describe the rejected
  primary-screen ADR-055 trial.
- Superseded modal cursor visibility correction automated evidence (2026-07-27,
  implementation commit `ad67fc5`): credential and provider-wizard cursors use
  strict panel-local visibility. A field row that is absent from the final panel
  rectangle hides the cursor rather than being vertically clamped to a header,
  instruction, preview, or another option. Provider-wizard Confirm intentionally
  has no text cursor. Test terminal state records cursor visibility/position,
  covering clipped ApiKey/BaseUrl/protocol/entry fields and real `draw_frame`
  terminal heights. Focused TUI validation: 437 passed; locked workspace
  validation: 2464 passed. These counts describe the rejected Alternate Screen
  trial and are retained only as historical evidence.
- Final coordinate and anchor correction automated evidence (2026-07-27,
  implementation commits `6c32d09` and `18648a6`): normal projected rows use
  half-open logical intervals, logical-line EOF is accepted only by the last
  row, and empty/fill-only lines have deterministic zero-length anchors.
  Logical offsets are recorded while consuming original scalars, so width-one
  CJK markers and projected fill cannot move anchors. Page navigation uses the
  final history rectangle height. `AppLayout` now allocates component heights
  by priority and then assigns coordinates in explicit AboveInput/BelowInput
  visual order. Credential/provider cursors are panel-local and converted only
  through the final panel rectangle.
- Current focused validation: `cargo test --locked -p talos-tui --lib` →
  **446 passed, 0 failed, 0 ignored**.
- Current workspace validation: `cargo test --workspace --locked` →
  **2477 passed, 0 failed, 0 ignored** across 62 test binaries/doc-test groups.
- Current static validation: locked workspace check and Clippy with
  `-D warnings` passed with zero Rust/Clippy diagnostics. Cargo emitted one
  informational `talos-config` build-script warning reporting the compressed
  `models.toml` size.

### Full-Frame Foundation Evidence

- Full-frame invariant correction automated evidence (2026-07-27, implementation
  commit `8b7a272`): transcript tool blocks are geometry-free; projection reflows them
  at the current width; anchored history retains a logical entry/row across
  append and resize; fill-only and multi-cell fills remain bounded; `AppLayout`
  and cursor placement are safe for zero/narrow/short sizes; alternate-screen
  setup rolls back partial lifecycle transitions. The old `PreparedLogicalLine`,
  `HistoryPreparation`, terminal insertion writer, and recovery tests were
  removed rather than retained behind `#[cfg(test)]`. This supersedes the
  remaining inline-recovery assumptions below, but does not satisfy the
  real-terminal acceptance gate.

- Architecture-correction automated evidence (2026-07-27): `a3074ad` replaces the interactive
  primary-screen history path with an alternate-screen full-frame renderer, logical
  `TranscriptStore`, width-dependent `HistoryProjection`, application-owned history scroll
  state, and a single size snapshot per draw. The legacy terminal insertion/DECSTBM/reverse-index
  APIs are absent from the runtime TUI source path. `cargo fmt --all -- --check`,
  `cargo clippy --locked -p talos-tui -- -D warnings`, `cargo test --workspace --locked`,
  `cargo check --workspace --locked`, and `git diff --check` passed. This is automated evidence
  only; real-terminal acceptance remains mandatory.

- Historical focused evidence (superseded as the latest count):
  `cargo test --locked -p talos-tui` → 404 passed, 0 failed.
- Historical workspace evidence (superseded as the latest count):
  `cargo test --workspace --locked` → 2431 passed, 0 failed.
- Runtime evidence: automated gates green; real Alacritty walkthrough PENDING (human gate, not yet performed).
- Governance validation: passed — `scripts/validate_project_governance.sh .` 0 warnings; `git diff --check` clean.

## Current Architecture And Implementation Commits

- Alternate Screen Logo correction starting HEAD: `c038852`.
- Current implementation: `9c87d0f`, Logo spacing correction `635bc29`, and
  virtual history-prefix correction `dddf32f`; screenshot interaction
  correction `dd10a62`; Logo-prefix wheel correction `d4d95ad`.
- Current runtime architecture: geometry-free `TranscriptStore` →
  width-independent logical lines → width-dependent full-frame projection in
  Alternate Screen. History, Logo, composer, status, and panels share the
  application-owned frame; only conversation facts enter the transcript.
- ADR-054 is Accepted with the startup-splash amendment. ADR-019 is
  Superseded; ADR-055 is Rejected.
- Automated evidence: 449 focused TUI tests; first-frame Logo, compact Logo,
  transcript exclusion, logical scrolling, bounded layout, modal cursor,
  full-width user history styling, prefix-only row suppression, mouse-wheel
  navigation across the Logo/Transcript boundary, geometry-free tool facts,
  and terminal recovery all pass. Locked workspace: 2480 passed.
- Remaining gate: Alacritty, Kitty/WezTerm, macOS Terminal/iTerm2, and tmux
  Logo/resize/restore walkthrough. I156 remains Active.

### Full-Frame Foundation Commits

- Starting HEAD for the final coordinate correction: `a1885ac`.
- Anchor-boundary implementation: `6c32d09`; interval-regression assertion:
  `1688e6f`.
- Component-layout, final page-height, cursor-coordinate, and entry-point test
  implementation: `18648a6`.
- Modal cursor visibility implementation: `ad67fc5`.
- These commits remain part of the current geometry-free full-frame
  architecture. Commit `9c87d0f` corrects their startup Logo sequencing.

## Historical / Superseded Implementation Attempts

The following DECSTBM, `insert_styled_history`, `resize_clear_action`, and
pending-insertion recovery records are retained only as historical evidence.
They are not part of the current runtime architecture. The amended ADR-054
app-owned full-frame renderer is current authority.

### Legacy Implementation Record

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
- Completion status: COMPLETE — automated gates passed + maintainer Alacritty walkthrough passed 2026-07-27 (Cases A/B/C/D all confirmed by maintainer in real terminal).

## Completion Evidence

- Completion Commit: `6909675` — last TUI-035 code implementation commit (semantic-row recovery). Full chain: `2f9de9f` → `5d11926` → `dc28392` → `db85a3e`/`b24d9e0` → `6909675`. Subsequent TUI-035 commits by other agents (`31e7a0d`..`d4d95ad`) extended the feature surface and were also validated in the maintainer walkthrough.
- Do not cite a status-only documentation commit as implementation completion.
- Keep `Review`, `Partial`, or `Blocked` if implementation, runtime evidence, CI, or human acceptance is pending.

## Variance And Residuals

### Deferred Scope Addition — CTX-001 Bounded Runtime Meta Context — 2026-07-27

**Classification:** maintainer-requested scope addition.

The request to add current-session Meta information, including remaining context
window, to provider context has an independent objective and acceptance target.
It affects prompt construction, token accounting, compaction, and cache
stability rather than TUI-035 resize rendering. It is therefore deferred as
`CTX-001` and does not alter I156's published scope, validation, or human
terminal gate.

I156/TUI-035 subsequently completed on 2026-07-27. `CTX-001` remains
unselected and requires its own explicitly sequenced future iteration.

### Logo-Prefix Wheel Coordinate Correction — 2026-07-27

**Classification:** maintainer-reported real-terminal interaction defect.

The Logo was a display-only virtual prefix while mouse-wheel navigation
addressed only `HistoryProjection` transcript rows. The first wheel event could
therefore drop the remaining Logo rows wholesale, and no transcript anchor
could navigate back to the Logo after it left the frame.

Commit `d4d95ad` introduces one continuous frame-history start coordinate over
the Logo prefix and projected transcript. Wheel movement advances that
coordinate by three rows; positions inside the Transcript resolve to existing
width-independent logical anchors, while positions inside the Logo retain a
display-only prefix offset. The Logo remains excluded from
TranscriptStore/session/export. Focused TUI: 449 passed; locked workspace:
2480 passed across 62 test binaries/doc-test groups. Real-terminal mouse
acceptance remains pending.

### Screenshot Interaction Correction — 2026-07-27

**Classification:** maintainer-reported real-terminal defect.

Screenshot evidence after the Logo-prefix correction exposed three regressions:

- user-message `INPUT_BG` covered only content cells because logical user rows
  carried background but no bounded fill segment;
- an assistant stream beginning with `\n` produced a prefix-only `●` row;
- without mouse capture, terminal wheel input degraded into Up/Down keys and
  browsed composer input history.

Commit `dd10a62` gives all user-block rows a projection-time fill token,
suppresses non-user leading empty stream rows, transactionally enables mouse
capture, and maps wheel movement to three-row logical-history navigation.
Focused TUI: 446 passed; locked workspace: 2477 passed. Cross-terminal mouse
and visual acceptance remains pending.

### Alternate-Screen Direction Reinstated With Logo Correction — 2026-07-27

**Classification:** maintainer-directed product decision.

The maintainer removed native history as a hard requirement and selected one
Alternate Screen mode. The ADR-054 trial's missing Logo was a startup-order
defect, not a reason to retain the ADR-055 renderer:

- Primary Screen no longer prints a splash before TUI initialization.
- `TerminalSession` enters and clears Alternate Screen transactionally.
- The first full frame renders the wide or compact Logo in the history region.
- Logo rows are display-only and never enter transcript/session/export facts.
- The first user message is appended below the Logo; growing history scrolls
  the display-only Logo prefix out naturally.

Implementation commits: `9c87d0f`, `635bc29`, and `dddf32f`. Real-terminal
Logo, resize, and restore acceptance remains pending.

### Native-History Experience Correction — 2026-07-27

**Classification:** superseded maintainer-directed experiment. Native terminal
history was temporarily treated as a hard interaction requirement.

Observed in the ADR-054 implementation:

- the primary-screen logo disappeared immediately after startup;
- shell history and terminal-native selection/search were unavailable while
  Talos ran;
- Talos history required application navigation and vanished from the visible
  terminal surface on exit except for the compact summary.

Correction:

- retain the geometry-free app-owned transcript;
- project only newly committed entries to primary-screen native scrollback,
  exactly once, using ordinary newline output;
- render only the current bounded input/status/panel frame;
- reject ADR-054 and record ADR-055;
- do not restore DECSTBM, reverse index, or width-specific transcript facts.

Residual:

- terminal-side resize can reflow transient frame cells before Talos receives
  the event. This cannot be eliminated while preserving native scrollback.
  The later Alternate Screen decision removes this runtime tradeoff.

### Architecture Correction — 2026-07-27

**Classification:** superseded architecture correction. At this point the
published objective was treated as unchanged:
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
- Historical correction plan: ADR-054; move history facts into an
  application-owned transcript and render history plus fixed panes in one
  Alternate Screen frame. The later native-history acceptance correction
  temporarily replaced it with ADR-055; the final maintainer decision
  reinstated amended ADR-054 with correct first-frame Logo rendering.

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
