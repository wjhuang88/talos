# Iteration I159: `talos-tools` Lightweight Feature Boundary

> Document status: Active
> Published plan date: 2026-07-26
> Planned objective: `talos-tools` defaults to local read/search and heavy families are true opt-in Cargo features while CLI behavior remains unchanged.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `talos-tools` defaults to local read/search and heavy families are true opt-in Cargo features while CLI behavior remains unchanged.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline release-governance session 2026-08-14 |
| Work Slice | ARCH-031-A / I159 only: implement real `talos-tools` Cargo feature boundaries, lightweight file-read/search defaults and explicit CLI `coding` selection while preserving product tool and permission behavior. No shared composition, runtime preset, sandbox policy, version bump, publication, tag or release. |
| Claimed At | 2026-08-14 |
| Source Issue | None |
| Governance Claim PR | #235 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #235 head `11619e13ca6c854b4db737a9978767436a19ab9f` passed CI `31789567122`, independent natural-person approval `5292115807`, both governance validators and merge-time CAS, then merged to `main` as `fa635b4eaadd4b55939322f89acfda4522489ab7`. |
| Implementation PR | Pending draft creation from `feat/tools-I159-feature-boundary` |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Implement only on `feat/tools-I159-feature-boundary` from claim merge `fa635b4eaadd4b55939322f89acfda4522489ab7`; reach Review through the complete feature matrix, product-parity evidence and an implementation commit. |

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
| 2026-08-14 | Priority and readiness change control | The maintainer selected a v0.8.0 GitHub-first/Cargo-second release before I196 implementation. ARCH-031-A resolved its remaining feature-ownership alternatives and moved to Ready. I159 moves from Blocked to Planned and prepares its own claim; the published objective, exclusions and acceptance remain unchanged. I160-I162 stay blocked in order, I203 stays blocked on I162 GO, and no implementation is activated by this planning record. |
| 2026-08-14 | Dependency-fact review correction | Independent review of PR #235 at `4cd5d6868b42f7efafccf117c78e30173addef01` found that `document_extract` unconditionally compiles existing `scraper 0.27`, so assigning it to default `file-read` contradicted the lightweight-default objective. ARCH-031-A change control now assigns the whole tool to a default-off `document` feature requiring `file-read`, includes it in `coding`, and corrects `tree`, `search_engine`, and `browser_page` source attributions. The published objective, product-parity requirement, exclusions and acceptance remain unchanged; this record still does not activate implementation. |
| 2026-08-14 | Activation | PR #235 head `11619e13ca6c854b4db737a9978767436a19ab9f` passed exact-head CI `31789567122`, independent natural-person approval `5292115807`, both governance validators and merge-time CAS, then merged as `fa635b4eaadd4b55939322f89acfda4522489ab7`. The implementation branch starts exactly there. Pre-activation inventory found no Active or Review iteration; I188/I189/I195 remain Planned/Claimed and unactivated, I196 remains Planned/Claimed on release priority hold, I160-I162/I203 remain Blocked, and superseded I164 remains Paused. Open PRs #233, #228, #227 and archival #120/#121 do not own ARCH-031-A/I159. I159 alone becomes Active; no release, version, tag or publish authority is created. |

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

- Planning variance resolved before activation: the initial PR #235 readiness decision incorrectly
  treated `document_extract` as dependency-free relative to `file-read`. ARCH-031-A now records the
  existing `scraper` edge and the separate default-off `document` feature. No implementation
  variance or residual is authorized by this correction.

## REL-002 Execution Record

- Primary executor/runtime: pending
- External assistance: pending
- Planning/editing/testing/docs/commit/push ownership: pending
- Qualification verdict: pending; do not assume qualification.

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
