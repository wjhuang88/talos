# Iteration I157: Provider Removal And Credential Clear

> Document status: Planned
> Published plan date: 2026-07-26
> Planned objective: A user can remove a provider entry or clear one credential through `talos config unset ... --confirm` without hand-editing TOML.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A user can remove a provider entry or clear one credential through `talos config unset ... --confirm` without hand-editing TOML.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `MODEL-010` | `None` | `Ready` | `I156` Complete; provider config baseline unchanged | A user can remove a provider entry or clear one credential through `talos config unset ... --confirm` without hand-editing TOML. |

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

- add the CLI-only `ConfigCommand::Unset` surface;
- implement dotted-key provider entry removal and API-key clear;
- enforce `--confirm`;
- perform one atomic write on success;
- prove active-provider removal remains picker-recoverable;
- update EN/zh-CN/config reference docs.

### Forbidden Changes

- no TUI `/connect` delete action;
- no bulk/wildcard deletion;
- no environment variable deletion;
- no second secret store;
- no model-catalog change;
- no unrelated config refactor;
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
   - prove the real `talos config unset` CLI path and the subsequent startup/model-picker recovery path.
5. **Documentation and owner sync**
   - update Story, iteration, parent, Board, and user/reference docs named by the Story.
6. **Validation and commit**
   - run focused and full validation;
   - inspect staged diff;
   - create one logical commit referencing `MODEL-010` and `I157`.

### Non-Goals

- no TUI `/connect` delete action;
- no bulk/wildcard deletion;
- no environment variable deletion;
- no second secret store;
- no model-catalog change;
- no unrelated config refactor;
- no tag/release.

### Acceptance

All unchecked Acceptance items in `MODEL-010` that belong to this iteration must be satisfied. Do not
invent acceptance criteria here.

### Planned Validation

```bash
cargo test --locked -p talos-config
cargo test --locked -p talos-cli
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

### Runtime Evidence

Use an isolated HOME/config fixture and record:
- custom provider removal;
- builtin-backed credential clear/disconnect wording;
- missing `--confirm` leaves file byte-identical;
- active-provider removal followed by startup or `/model` picker recovery;
- logs/config list do not reveal cleared credentials.

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

- atomic write cannot be achieved with existing config save path;
- active-provider removal panics or requires changing product policy;
- public config API break is required without an ADR;
- implementation needs TUI plumbing or secret-store redesign.

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

- 2026-07-28 priority shift: maintainer selected and activated I164/TUI-038
  after I163 completed. I157 remains Planned, its published scope and baseline
  are unchanged, and it is deferred until I164 disposition rather than
  superseded.

## REL-002 Execution Record

- Primary executor/runtime: pending
- External assistance: pending
- Planning/editing/testing/docs/commit/push ownership: pending
- Qualification verdict: pending; do not assume qualification.

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
