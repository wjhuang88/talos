# Iteration I158: Tool Registration Composition Consolidation

> Document status: Active
> Published plan date: 2026-07-26
> Planned objective: Print, TUI, and MCP tool registries are assembled from one explicit contribution model with preserved permission and capability behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Print, TUI, and MCP tool registries are assembled from one explicit contribution model with preserved permission and capability behavior.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.


## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Pending: interactive contribution migration, final profile inventory/equivalence evidence, and I158 acceptance/state synchronization |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Active maintainer session on 2026-07-31 authorized preparing the formal claim; ownership remains ineffective until finalized claim merge. |
| Implementation PR | `#102`, `#105` (both must remain Draft until claim merge and branch refresh) |
| Last Updated | 2026-07-31 |
| Handoff / Release Condition | None |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-034-R01` | `ARCH-034` | `In Progress` | ADR-053 Accepted 2026-07-31; activation recorded against `e539537d` | Print, TUI, and MCP tool registries are assembled from one explicit contribution model with preserved permission and capability behavior. |

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

- implement accepted ADR-053 only;
- create additive contribution/collision contracts;
- move authoritative factories to owning crates;
- migrate product profiles with set/wrapper equivalence tests;
- retain old builders until equivalence is proven.

### Forbidden Changes

- no global auto-registration;
- no new composition crate;
- no permission/default behavior change;
- no feature-gate implementation;
- no RuntimePreset or SandboxFallbackPolicy;
- no tool feature additions;
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
   - prove print, TUI, and MCP registry composition and permission-wrapper equivalence.
5. **Documentation and owner sync**
   - update Story, iteration, parent, Board, and user/reference docs named by the Story.
6. **Validation and commit**
   - run focused and full validation;
   - inspect staged diff;
   - create one logical commit referencing `ARCH-034-R01` and `I158`.

### Non-Goals

- no global auto-registration;
- no new composition crate;
- no permission/default behavior change;
- no feature-gate implementation;
- no RuntimePreset or SandboxFallbackPolicy;
- no tool feature additions;
- no tag/release.

### Acceptance

All unchecked Acceptance items in `ARCH-034-R01` that belong to this iteration must be satisfied. Do not
invent acceptance criteria here.

### Planned Validation

```bash
cargo test --locked -p talos-core -p talos-tools -p talos-agent -p talos-cli -p talos-plugin
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

### Runtime Evidence

Record real print/TUI/MCP registry names and one duplicate-name failure containing both sources.
Exercise one read and one permission-gated tool through a real product path.

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

- ADR-053 is not Accepted;
- the chosen contract introduces a dependency cycle;
- permission wrappers cannot remain at the outer composition root;
- current mode inventories are not discoverable;
- implementation requires a new crate.

If a stop condition occurs:

1. stop editing;
2. record the exact code/document conflict under Variance And Residuals;
3. keep the iteration `Blocked` or `Review`;
4. do not create a speculative workaround;
5. request maintainer/architecture input.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-31 | Activation | Baseline `e539537d` (`main` after I168 merge). No other implementation iteration is Active or in Review. ADR-053 Accepted; ARCH-034-R01 moved to In Progress. Primary executor is GPT-5.6 Thinking through the connected GitHub repository workflow. Begin with the additive core contract and red tests; retain all old builders until equivalence evidence passes. |

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

- Primary executor/runtime: GPT-5.6 Thinking through connected GitHub repository tooling
- External assistance: none
- Planning/editing/testing/docs/commit/push ownership: primary executor; GitHub Actions supplies repository validation evidence
- Qualification verdict: non-qualifying for REL-002 until Talos itself is the primary execution runtime

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
