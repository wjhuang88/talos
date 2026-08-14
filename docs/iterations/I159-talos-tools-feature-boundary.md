# Iteration I159: `talos-tools` Lightweight Feature Boundary

> Document status: Blocked
> Published plan date: 2026-07-26
> Planned objective: `talos-tools` defaults to local read/search and heavy families are true opt-in Cargo features while CLI behavior remains unchanged.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `talos-tools` defaults to local read/search and heavy families are true opt-in Cargo features while CLI behavior remains unchanged.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-031-A` | `ARCH-031` | `Refinement/Blocked` | `I158` Complete and ARCH-031-A updated to Ready | `talos-tools` defaults to local read/search and heavy families are true opt-in Cargo features while CLI behavior remains unchanged. |

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

- implement exactly ARCH-031-A;
- feature-gate optional dependencies, modules, exports, tests, and downstream manifests;
- keep product tool inventory through explicit `coding` feature.

### Forbidden Changes

- no tool behavior changes;
- no preset/fallback;
- no composition redesign;
- no sibling crates;
- no version bump;
- no publish/tag/release.

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
   - prove the full Talos product build with explicit coding features and a minimal default-only consumer build.
5. **Documentation and owner sync**
   - update Story, iteration, parent, Board, and user/reference docs named by the Story.
6. **Validation and commit**
   - run focused and full validation;
   - inspect staged diff;
   - create one logical commit referencing `ARCH-031-A` and `I159`.

### Non-Goals

- no tool behavior changes;
- no preset/fallback;
- no composition redesign;
- no sibling crates;
- no version bump;
- no publish/tag/release.

### Acceptance

All unchecked Acceptance items in `ARCH-031-A` that belong to this iteration must be satisfied. Do not
invent acceptance criteria here.

### Planned Validation

Run the complete feature/build matrix in ARCH-031-A plus full locked validation.

### Runtime Evidence

Record:
- `cargo tree` default-only absence of heavy deps;
- real CLI/TUI product tool inventory unchanged;
- minimal default-only consumer build.

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

- feature boundaries require behavior change;
- a disabled family leaks a public re-export or hard dependency;
- product inventory cannot be preserved;
- public break exceeds ADR-052.

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
| 2026-08-14 | Dependency disposition | TUI-037/I202 reached Complete through implementation commit `6d3f85ea9f7e76f617ec9716f17ecdd0f9dd0772` and PR #230 merge `e0cc782a475c2e5baceb31f2a125f1e268af7ecf`, satisfying the required independent TUI disposition. I159 remains Blocked and unactivated because selected Story ARCH-031-A is still Refinement/Blocked and not Ready; no I159 claim or implementation authority is created. |

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
