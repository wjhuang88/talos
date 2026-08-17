# Iteration I159: `talos-tools` Lightweight Feature Boundary

> Document status: Complete
> Published plan date: 2026-07-26
> Planned objective: `talos-tools` defaults to local read/search and heavy families are true opt-in Cargo features while CLI behavior remains unchanged.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `talos-tools` defaults to local read/search and heavy families are true opt-in Cargo features while CLI behavior remains unchanged.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline release-governance session 2026-08-14 |
| Work Slice | ARCH-031-A / I159 only: implement real `talos-tools` Cargo feature boundaries, lightweight file-read/search defaults and explicit CLI `coding` selection while preserving product tool and permission behavior. No shared composition, runtime preset, sandbox policy, version bump, publication, tag or release. |
| Claimed At | 2026-08-14 |
| Source Issue | None |
| Governance Claim PR | #235 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #235 head `11619e13ca6c854b4db737a9978767436a19ab9f` passed CI `31789567122`, independent natural-person approval `5292115807`, both governance validators and merge-time CAS, then merged to `main` as `fa635b4eaadd4b55939322f89acfda4522489ab7`. |
| Implementation PR | #236 (merged as `f79c1ead1cd3a547797dea3666295f510d88a13d`) |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Closed after exact-head CI `31801484313`, independent approval `5293622712`, merge-time CAS and PR #236 merge `f79c1ead`; I160 requires its own effective claim before implementation. |

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
| 2026-08-14 | Pre-commit local validation | Draft PR #236 implements the approved feature model and explicit downstream selections. Local feature, default/coding, workspace, product-inventory, CLI-smoke and external-consumer checks passed before implementation commit `d886917e`. The local collaboration validator was run without `GITHUB_BASE_REF`/`COLLABORATION_VALIDATION_BASE`, so it compared only the final commit and missed the branch-wide active ARCH-031 parent edit; this record does not claim exact-head preflight success. |
| 2026-08-14 | Exact-head CI/review correction | CI `31794297165` and independent review `5292595210` on `d886917e` found the same blocker: release preflight stopped before Cargo validation because the changed active ARCH-031 Epic lacked a Collaboration Claim block. ARCH-031 now records the established Unclaimed Epic-parent metadata; the corrected head must rerun base-bound validation, full preflight and fresh independent review. |
| 2026-08-14 | Corrected local validation | With the ARCH-031 Epic-parent metadata present, `COLLABORATION_VALIDATION_BASE=origin/main` makes the collaboration validator inspect the complete branch diff and report 0 warnings. The same base-bound full release preflight completes successfully, and no-feature/default/image/shell/coding checks pass after the cfg simplification. Commits `34c09b14` and `57bc1585` record the correction; push, exact-head CI and fresh independent review remain pending. |
| 2026-08-14 | Merge and completion | Corrected exact head `33a2c6ffad0e5c473baf41c14e704dfd19fcd0c9` passed CI `31801484313` 5/5 and independent natural-person review `5293622712`; merge-time CAS confirmed unchanged head/base/checks/review and PR #236 merged as `f79c1ead1cd3a547797dea3666295f510d88a13d`. Completion cites the pre-existing implementation commits below; this closeout does not self-certify. |

## Verification Evidence

- Baseline: at claim merge `fa635b4e`, `cargo tree --locked -p talos-tools --depth 1` included
  `arborium`, `gix`, `image`, `libc`, `reqwest`, `rust-websearch`, `scraper`, `similar`, and
  `talos-sandbox` as direct normal dependencies.
- Feature checks passed for no features, the default, every individual family, required
  combinations, and `coding`; default tests passed 41 tests, while `coding` passed 320 unit tests,
  15 document-boundary tests, and 3 integration-hardening tests.
- Default dependency evidence: `cargo tree --locked -p talos-tools --depth 1` excludes every heavy
  dependency named above. The remaining `sha2`/`uuid` dependencies serve the default read snapshot
  contract; local search dependencies serve the default `search` family.
- Downstream selection: CLI=`coding`, MCP handshake fixture=`file-write + shell`, runtime fixture=
  `file-write`; `talos-agent` had no source consumer and its unused dependency was removed.
- Focused product evidence: exact sorted registry inventory test passed; locked workspace check and
  build passed; real `cargo run -p talos-cli --locked -- --mock --print --no-init --no-context
  "I159 coding feature product smoke"` completed successfully.
- Minimal consumer: a standalone package with only
  `talos-tools = { path = "<checkout>/crates/talos-tools" }` imports `ReadTool` and `GlobTool`,
  constructs both from `PathBuf::from(".")`, and passed `cargo check --offline`. No feature is
  selected, so this exercises the true default surface without relying on a retained temp path.
- Local Rust validation before `d886917e`: `cargo fmt --all -- --check`, workspace check/build,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo test --workspace --locked` passed.
- Exact-head governance/release validation at `d886917e`: failed. CI `31794297165` stopped in the
  collaboration validator before Cargo commands because the changed active ARCH-031 Epic lacked a
  claim block.
- Corrected working-tree validation: `COLLABORATION_VALIDATION_BASE=origin/main` collaboration
  validation reports 0 warnings; project governance reports 0 warnings; base-bound
  `./scripts/release_preflight.sh` completes successfully. Commits `34c09b14` and `57bc1585` record
  the correction.
- Exact-head CI: run `31801484313` at `33a2c6ff` completed 5/5 successfully, including
  `Format + Check + Clippy + Test` and `Windows Rust workspace`.
- Independent review: approval comment `5293622712` rechecked the complete base-to-head diff,
  feature seams, dependency tree, governance truth and status boundaries.

## Completion Evidence

- Completion Commit: `d886917e45d5ca0f110e111b966cd379485e3580`,
  `34c09b142766c70ac62ef24424ed035f2fa921a5`.
- Accepted implementation head: `33a2c6ffad0e5c473baf41c14e704dfd19fcd0c9`.
- Implementation PR #236 merge: `f79c1ead1cd3a547797dea3666295f510d88a13d`.
- This status-only closeout commit is not completion evidence.

## Variance And Residuals

- Planning variance resolved before activation: the initial PR #235 readiness decision incorrectly
  treated `document_extract` as dependency-free relative to `file-read`. ARCH-031-A now records the
  existing `scraper` edge and the separate default-off `document` feature. No implementation
  variance or residual is authorized by this correction.
- Release residual: the pre-existing `scraper 0.22`/`0.27` duplicate remains unchanged and is owned
  by I162 publication-closure reconciliation; I159 does not perform dependency upgrades.

## REL-002 Execution Record

- Primary executor/runtime: Codex / GPT-5 mainline release-governance session.
- External assistance: independent natural-person exact-head review through shared account
  `@wjhuang88`, with role separation disclosed in comment `5293622712`.
- Planning/editing/testing/docs/commit/push ownership: executing Agent prepared the claim,
  implementation, evidence correction and push; the independent reviewer performed detached
  source/CI/runtime-boundary verification; merge-time CAS was executed separately after approval.
- Qualification verdict: I159 qualifies only its bounded feature-boundary deliverable; REL-002
  self-bootstrap qualification remains NO-GO.

## Retrospective

- Outcome: lightweight `file-read + search` defaults and real opt-in heavy capability gates landed
  without changing the CLI product tool inventory.
- Documentation: crate docs, EN/zh-CN README migration notes, SDK contract, publication matrix,
  Story, iteration, parent and derived views were synchronized.
- Lessons: branch-level governance validation must bind the target branch merge-base; recorded as
  EVOLUTION lesson 49. The pre-existing `scraper` duplication remains explicitly owned by I162.
