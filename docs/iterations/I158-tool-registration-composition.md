# Iteration I158: Tool Registration Composition Consolidation

> Document status: Review
> Published plan date: 2026-07-26
> Planned objective: Print, TUI, and MCP tool registries are assembled from one explicit contribution model with preserved permission and capability behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Print, TUI, and MCP tool registries are assembled from one explicit contribution model with preserved permission and capability behavior.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Released |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / active maintainer session 2026-07-31 |
| Work Slice | Correct the interactive contribution migration without eagerly constructing excluded tools; add final deterministic Print/TUI/MCP inventory and set-equivalence evidence; synchronize I158 acceptance, verification, delivery state, and derived views after implementation merge. |
| Claimed At | 2026-07-31 |
| Source Issue | None |
| Governance Claim PR | #112 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer-authorized active session on 2026-07-31; no independent reviewer is presently available; claim merge requires exact-head CI, both governance validators, merge-time overlap/dependency CAS, and no unresolved blocking feedback. |
| Implementation PR | #102 merged as `9d2926ed04a6c4666d7895fbb6bdb4099907daf8`; #105 merged as `ec4d918f1fb72b0ab2ddbdcaa24809cc61707d14` |
| Last Updated | 2026-07-31 |
| Handoff / Release Condition | Released after the bounded #102/#105 work slice and review synchronization. Resume only through a new compatible claim that resolves the scheduler/status contribution decision and documentation/finding residuals. |

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
| 2026-07-31 | Implementation | Contribution/collision contract and owning-crate factories landed through #77, #82, #85, #93, #95, #98, #99, #100, #101, and #102. Interactive composition was rebuilt from effective Claim #112 and merged as `9d2926ed04a6c4666d7895fbb6bdb4099907daf8` without constructing excluded `exec` or `document_extract`. |
| 2026-07-31 | Evidence | Deterministic exact Print/TUI/MCP profile inventory evidence merged through #105 as `ec4d918f1fb72b0ab2ddbdcaa24809cc61707d14`. Exact-head CI run `30626150159` passed macOS release preflight and the Windows installer fixture. |
| 2026-07-31 | Review disposition | Implementation and profile-equivalence slices are merged. I158 moves to Review, not Complete: scheduler/status contribution ownership and final ARCHITECTURE/TOOL-003/F01 documentation disposition remain unresolved. Claim #112 is Released after this bounded synchronization. |

## Verification Evidence

- Focused tests: contribution source/inventory tests, symbol-only isolation, selective interactive profile construction, exact 21-tool interactive inventory, and deterministic full Print/TUI/MCP inventories passed in their implementation PRs.
- Full locked validation: exact-head CI passed for #100 (`30623638710`), #102 (`30625383505`), and #105 (`30626150159`); the final implementation evidence commit is `ec4d918f1fb72b0ab2ddbdcaa24809cc61707d14`.
- Runtime/composition evidence: the Print/TUI/MCP builders expose exact sorted inventories; duplicate registration reports both sources; the full Print composition executes snapshot-aware `read`; existing permission, presentation, `read_image`, plugin, MCP, and one-shot continuation tests remained green.
- Governance validation: project-governance and Collaboration Claim validators passed on Claim #112 and each post-claim implementation slice.

## Completion Evidence

- Completion Commit: not yet assigned; delivery remains Review.
- Existing implementation/evidence commits on `main`: `9d2926ed04a6c4666d7895fbb6bdb4099907daf8` and `ec4d918f1fb72b0ab2ddbdcaa24809cc61707d14`; earlier contribution chain is recorded under Actual Activation And Execution.
- Complete is blocked until the remaining acceptance and documentation residuals below are resolved and revalidated. This status-only synchronization does not cite itself as implementation completion.

## Variance And Residuals

- The accepted contribution model is implemented for tool-owning groups and product profiles, but scheduler tools and the CLI-owned MCP `status` tool remain explicit raw registrations. Architecture/maintainer review must either record these as justified runtime/profile exceptions or migrate them through focused authoritative contributions before Complete.
- `docs/reference/ARCHITECTURE.md`, developer-facing TOOL-003 extension guidance, ARCH-034-F01 disposition, and parent-program closure language still require a dedicated documentation/finding closeout slice.
- No user-visible inventory or permission behavior changed. TUI-037 remains blocked because its gate requires I158 Complete or Paused; Review is not sufficient.

## REL-002 Execution Record

- Primary executor/runtime: GPT-5.6 Thinking through connected GitHub repository tooling
- External assistance: none
- Planning/editing/testing/docs/commit/push ownership: primary executor; GitHub Actions supplies repository validation evidence
- Qualification verdict: non-qualifying for REL-002 until Talos itself is the primary execution runtime

## Retrospective

- Outcome: implementation and deterministic profile-equivalence evidence merged; iteration is in Review pending final exception/documentation disposition.
- Documentation: owner and derived operating views synchronized by this review transition; architecture, extension guidance, and F01 closure remain explicit residuals.
- Lessons: profile exclusion must prevent construction, not merely filter after eager factory creation; environment-dependent registry inputs need explicit deterministic test seams.
