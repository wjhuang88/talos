# Iteration I161: Sandbox Fallback And Coding Preset

> Document status: Blocked
> Published plan date: 2026-07-26
> Planned objective: Embedders have fail-closed sandbox fallback choices and an explicit coding preset that cannot weaken permission or sandbox constraints.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Embedders have fail-closed sandbox fallback choices and an explicit coding preset that cannot weaken permission or sandbox constraints.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-031-C` | `ARCH-031` | `Refinement/Blocked` | `I160` Complete; ARCH-031-C Ready; security reviewer assigned | Embedders have fail-closed sandbox fallback choices and an explicit coding preset that cannot weaken permission or sandbox constraints. |

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

- implement ARCH-031-C API and matrix;
- use shared composition;
- add typed fallback approval context;
- obtain independent security review;
- update SDK contract only after implementation.

### Forbidden Changes

- no permission-default relaxation;
- no policy in `talos-sandbox`;
- no new sandbox backend;
- no unrelated builder cleanup;
- no new crate;
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
   - prove the embedded runtime security matrix for Deny, Ask, AllowUnsandboxed, and the coding preset.
5. **Documentation and owner sync**
   - update Story, iteration, parent, Board, and user/reference docs named by the Story.
6. **Validation and commit**
   - run focused and full validation;
   - inspect staged diff;
   - create one logical commit referencing `ARCH-031-C` and `I161`.

### Non-Goals

- no permission-default relaxation;
- no policy in `talos-sandbox`;
- no new sandbox backend;
- no unrelated builder cleanup;
- no new crate;
- no publish/tag/release.

### Acceptance

All unchecked Acceptance items in `ARCH-031-C` that belong to this iteration must be satisfied. Do not
invent acceptance criteria here.

### Planned Validation

Run ARCH-031-C security matrix tests plus full locked validation and SDK docs.

### Runtime Evidence

External fixture or focused embedded tests must prove Deny, headless Ask deny, scoped Ask approval,
AllowUnsandboxed with permission Deny, and coding preset behavior.

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

- fallback approval cannot be distinguished from normal approval;
- `Deny` precedence changes;
- policy must move into `talos-sandbox`;
- caller override order is unclear;
- security review is unavailable or rejects the design.

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
