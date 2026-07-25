# Iteration I160: Shared CLI And Runtime Internal Composition

> Document status: Planned
> Published plan date: 2026-07-26
> Planned objective: CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-031-B` | `ARCH-031` | `Refinement/Blocked` | `I159` Complete and ARCH-031-B updated to Ready | CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints. |

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

- implement the owner and responsibility map in ARCH-031-B;
- add print/TUI/MCP/runtime equivalence tests;
- keep adapters thin;
- remove duplicated paths only after proof.

### Forbidden Changes

- no new crate;
- no RuntimePreset or SandboxFallbackPolicy;
- no permission/sandbox default change;
- no tool feature additions;
- no public CLI library promise;
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
   - create one logical commit referencing `ARCH-031-B` and `I160`.

### Non-Goals

- no new crate;
- no RuntimePreset or SandboxFallbackPolicy;
- no permission/sandbox default change;
- no tool feature additions;
- no public CLI library promise;
- no tag/release.

### Acceptance

All unchecked Acceptance items in `ARCH-031-B` that belong to this iteration must be satisfied. Do not
invent acceptance criteria here.

### Planned Validation

Focused composition/equivalence tests plus full locked validation.

### Runtime Evidence

Record:
- print, TUI, MCP tool-set equivalence;
- real product read and permission-gated tool;
- external/minimal runtime build using the shared path.

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

- sharing requires runtime to depend on CLI/TUI;
- a new crate appears necessary;
- permission behavior differs;
- mode-specific behavior is undocumented;
- hidden global state is required.

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
