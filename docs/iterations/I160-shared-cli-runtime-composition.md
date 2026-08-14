# Iteration I160: Shared CLI And Runtime Internal Composition

> Document status: Active (2026-08-15)
> Published plan date: 2026-07-26
> Planned objective: CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: CLI and runtime adapters share one internal composition implementation with equivalent product behavior and separate public entrypoints.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline session` |
| Work Slice | ARCH-031-B / I160 only: shared internal CLI/runtime composition with separate public entrypoints and behavior-equivalence evidence; no preset, fallback, version, tag or publication. |
| Claimed At | 2026-08-14 |
| Source Issue | None |
| Governance Claim PR | #238 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed release prerequisite; `@wjhuang88` is the shared GitHub account and natural-person separation is limited. PR #238 exact head `edcbe47f81798480447962048fe4f50bb69fdba1` passed CI `31815122170`, independent approval `5295372157`, and merge-time CAS before merge `71faf8440466668daeef0afd0e779be072978b01` established the claim on `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-15 |
| Handoff / Release Condition | Execute only ARCH-031-B/I160 from `main@71faf844` through an independently reviewed exact-head implementation PR; release/version/tag/publication and I161-I162 remain excluded. |

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
