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

- Focused tests: pending
- Full locked validation: pending
- Runtime evidence: pending
- Governance validation: pending

## Completion Evidence

- Completion Commit: pending
- Do not cite a status-only documentation commit as implementation completion.
- Keep `Review`, `Partial`, or `Blocked` if implementation, runtime evidence, CI, or human acceptance is pending.

## Variance And Residuals

- None recorded at planning time.

## REL-002 Execution Record

- Primary executor/runtime: pending
- External assistance: pending
- Planning/editing/testing/docs/commit/push ownership: pending
- Qualification verdict: pending; do not assume qualification.

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
