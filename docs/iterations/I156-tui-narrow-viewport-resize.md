# Iteration I156: TUI Narrow-Viewport And Resize Robustness

> Document status: Planned
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
2. `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
3. this iteration
4. the selected Story
5. governing ADRs/specifications in Required Reads
6. the exact source files named by the Story

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
   - prove the real binary/TUI/SDK path, not library-only behavior.
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
| YYYY-MM-DD | Activation | Record dependency inventory, baseline SHA, primary executor/runtime, and activation decision. |

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
