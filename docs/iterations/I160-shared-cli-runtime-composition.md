# Iteration I160: Shared CLI And Runtime Internal Composition

> Document status: Complete (2026-08-15)
> Published plan date: 2026-07-26
> Planned objective: CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline session` |
| Work Slice | ARCH-031-B / I160 only: shared internal CLI/runtime composition with separate public entrypoints and behavior-equivalence evidence; no preset, fallback, version, tag or publication. |
| Claimed At | 2026-08-14 |
| Source Issue | None |
| Governance Claim PR | #238 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed release prerequisite; `@wjhuang88` is the shared GitHub account and natural-person separation is limited. PR #238 exact head `edcbe47f81798480447962048fe4f50bb69fdba1` passed CI `31815122170`, independent approval `5295372157`, and merge-time CAS before merge `71faf8440466668daeef0afd0e779be072978b01` established the claim on `main`. |
| Implementation PR | #240 |
| Last Updated | 2026-08-15 |
| Handoff / Release Condition | None — I160 is complete. I161/I162 and release/version/tag/publication remain separately governed. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-031-B` | `ARCH-031` | `Refinement/Blocked` | `I159` Complete and ARCH-031-B updated to Ready | CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints. |

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
   - prove the real CLI composition path and an embedded runtime fixture using the same internal composition.
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
| 2026-08-14 | Dependency readiness | I159/ARCH-031-A is Complete through existing implementation evidence `d886917e`/`34c09b14`, CI `31801484313`, approval `5293622712` and merge `f79c1ead`. I160 moves from Blocked to Planned/Unclaimed; no activation or implementation authority is created. |
| 2026-08-14 | Claim preparation | Finalized claim PR #238 starts from `main@1b129c951df22a7de63e14735e02b1e8a79a9cd7`; the proposed claim remains ineffective until its `Claimed` record merges to `main`. |
| 2026-08-15 | Activation | PR #238 exact head `edcbe47f81798480447962048fe4f50bb69fdba1` passed CI `31815122170`, independent approval `5295372157`, and merge-time CAS, then merged as `71faf8440466668daeef0afd0e779be072978b01`. The implementation worktree `/private/tmp/talos-i160-impl` and branch `feat/runtime-I160-shared-composition` start at that exact claim merge. I160 is the sole Active iteration. I164 remains Paused; I188/I189/I195/I196 remain Planned and unactivated; I161/I162/I203 remain Blocked. Open PRs #120/#121, #227, #228 and #233 do not own this Work Slice. Primary executor is `Codex / GPT-5 mainline session`; no Rust/Cargo change existed at activation. |
| 2026-08-15 | Baseline and owner selection | Exact source baseline is `main@2b76b4e9`. Existing contribution factories are in `talos-tools/src/contributions.rs`; duplicate profile selection remains in CLI registry builders while `RuntimeBuilder::new()` remains minimal and caller-tool driven. The shared owner is a focused internal module in `talos-runtime`, consumed by CLI through an explicit bridge; no new crate, preset, fallback, permission-default change, or default tool expansion is authorized. |
| 2026-08-15 | Implementation checkpoint | Added the optional `talos-runtime` `shared-composition` feature and contribution-group owner; CLI print/TUI/MCP adapters now consume the shared groups while retaining existing wrappers and product additions. `RuntimeBuilder::new()` is unchanged; `.shared_tools()` is explicit. No preset, fallback, permission default, or release/publication work was added. |
| 2026-08-15 | Completion | Implementation commit `0524e82fa700892cb77bf378139c47b92a64693c` was merged through PR #240 as `97556149` after exact-head CI `31824945312` and independent approval `5296616991`. Owner-first closeout and derived-view synchronization merged through PR #241 as `2d48bd2c`; I160 is Complete. I161/I162 and release/version/tag/publication remain separate gates. |

## Verification Evidence

- Focused tests: `cargo test --locked -p talos-runtime --features shared-composition` (22 passed); `cargo test --locked -p talos-cli registry::tests::` (29 registry tests passed).
- Full locked validation: local `cargo test --locked --workspace` and `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` passed; exact implementation-head CI `31824945312` passed all five required jobs, including Windows Rust workspace.
- Runtime evidence: `tests::shared_composition_runtime_executes_read_tool` builds `RuntimeBuilder::new().shared_tools()` and executes the real shared `read` tool against a workspace fixture; composition inventory and MCP exclusion tests pass.
- Governance validation: `COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .` and `scripts/validate_project_governance.sh .` both passed with 0 warnings; `git diff --check` passed.

## Completion Evidence

- Completion Commit: `0524e82fa700892cb77bf378139c47b92a64693c` (pre-existing implementation commit; this closeout status commit is not used as its own evidence).
- Implementation PR #240 merged as `97556149e38e5bd52e1722792ad0662bbe95eda4` after exact-head CI `31824945312` and independent approval `5296616991`.
- Derived-view closeout PR #241 merged as `2d48bd2c5431e332d2106af9855b11199888b179` after exact-head CI and independent approval bound to `80607e617fc417c0cbfcc03ad38a75c506e39ca3`.
- Do not cite a status-only documentation commit as implementation completion.

## Variance And Residuals

- None recorded at planning time.

## REL-002 Execution Record

- Primary executor/runtime: pending
- External assistance: pending
- Planning/editing/testing/docs/commit/push ownership: pending
- Qualification verdict: pending; do not assume qualification.

## Retrospective

- Outcome: Complete; shared contribution selection is explicit and behavior-equivalent across CLI product and runtime adapters.
- Documentation: runtime SDK and architecture contracts record the opt-in `shared-composition` bridge and preserved default behavior.
- Lessons: owner-first status synchronization and YAML parseability were verified before derived-view closeout; release work remains gated by I161/I162.
