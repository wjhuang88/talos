# Iteration I161: Sandbox Fallback And Coding Preset

> Document status: Complete (2026-08-15)
> Published plan date: 2026-07-26
> Planned objective: Embedders have fail-closed sandbox fallback choices and an explicit coding preset that cannot weaken permission or sandbox constraints.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Embedders have fail-closed sandbox fallback choices and an explicit coding preset that cannot weaken permission or sandbox constraints.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline session` |
| Work Slice | `ARCH-031-C / I161` only: explicit `SandboxFallbackPolicy`, coding preset, typed fallback approval context if required, security matrix tests, runtime evidence, and SDK documentation; no I162 publication or release work. |
| Claimed At | 2026-08-15 |
| Source Issue | None |
| Governance Claim PR | #244 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim-only PR #244 merged at `b570ac27`; implementation PR #250 received independent exact-head security APPROVE on `74c5502d` and merged as `d2b4bdd1`; matrix-closure PR #251 received independent exact-head security APPROVE on `8b3ca5fc` and merged as `da5a43a2`. Shared-account identity limits are disclosed in the review records. |
| Implementation PR | #250; matrix-closure follow-up #251 |
| Last Updated | 2026-08-15 |
| Handoff / Release Condition | None — I161 is complete. I162 remains separately governed and must not be activated without its own claim and readiness evidence. |

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
| 2026-08-15 | Claim preparation | I160/ARCH-031-B is Complete through Completion Commit `0524e82f`; I161 remains unactivated and no Rust/Cargo or release change is authorized. This proposed claim is ineffective until its finalized Claimed record merges to `main`. |
| 2026-08-15 | Security review gate | Claim PR #244 merged as `b570ac27`; I161 remains Blocked because no independent security reviewer is available. Issue #245 requests assignment against the exact main baseline and records the required matrix. No implementation, release, tag, GitHub Release, or Cargo publication is authorized. |
| 2026-08-15 | Activation | Fresh inventory at `main@cabb7fa1`: no Active or Review iteration before this activation; I159/I160 Complete, I161 sole Active iteration, I162/I203 Blocked, I188/I189/I195/I196 Planned/Claimed, and I164 Paused. Issue #245 formal security-review result is recorded with shared-account identity limits; I161 implementation may begin from this exact main, but exact-head independent security approval remains mandatory. No release, tag, GitHub Release, or Cargo publication authority is created. |
| 2026-08-15 | Formal security review result | The maintainer accepted Issue #245 as the formal pre-implementation security review record. `ARCH-031-C` is normative for the complete matrix: permission `Deny` always wins; `AllowUnsandboxed` is usable only when required isolation is unavailable and never bypasses permission; headless `Ask` fails closed; fallback approval is typed, scoped, distinguishable from ordinary approval, and cannot become a permanent broad grant; ordinary `AlwaysApprove` cannot substitute for fallback approval; coding composition cannot weaken permission or sandbox constraints; policy remains outside `talos-sandbox`; and the nine-row Security Test Matrix plus path/network/execute variants are in scope. The reviewer and implementer roles are separate, with shared-account identity limits disclosed. This result authorizes neither implementation completion nor merge: the finalized implementation head still requires an independent exact-head review against the complete matrix. |
| 2026-08-15 | Implementation and security closure | PR #250 merged as `d2b4bdd1` after exact-head APPROVE on `74c5502d` and CI `31873172667`; PR #251 merged as `da5a43a2` after exact-head APPROVE on `8b3ca5fc` and CI `31878744293`. The latter closes all nine matrix focused tests and path/network/execute variants. |
| 2026-08-15 | Completion | I161 is Complete with Completion Commits `74c5502d` and `3ca2ec62`; no release, tag, GitHub Release, or Cargo publication authority is created. I162 may now be separately inventoried and claimed. |

## Verification Evidence

- Focused tests: `talos-agent` full suite plus `talos-runtime` default/shared-composition suites; PR #251 closes matrix rows 4, 6, 9 and path/network/execute variants.
- Full locked validation: exact-head CI `31873172667` (#250) and `31878744293` (#251), both 5/5 SUCCESS; local release-preflight governance/site/installer checks passed.
- Runtime evidence: Deny, headless Ask, scoped Ask, permission precedence, caller tool override, CLI Ask delegation, and all matrix variants are covered by focused tests.
- Governance validation: exact-base project governance and collaboration validators reported 0 warnings.

## Completion Evidence

- Completion Commit: `74c5502d8860316070182c0cf2366d5adf57ea6c` and `3ca2ec62b3e91d88c345f5bba15e986cb31f606c` (pre-existing implementation/test commits; this closeout is not self-evidence).
- Do not cite a status-only documentation commit as implementation completion.
- Keep `Review`, `Partial`, or `Blocked` if implementation, runtime evidence, CI, or human acceptance is pending.

## Variance And Residuals

- M1/M4/N4/N5/N6 remain non-blocking residuals: richer fallback projection, preset collision diagnostics, alias consolidation, explicit bash-only documentation, and fallback observability. They are excluded from I161 completion and require separately governed follow-up work if pursued.

## REL-002 Execution Record

- Primary executor/runtime: Codex / GPT-5 mainline session with the repository-pinned Rust toolchain.
- External assistance: independent security review comments on #250 and #251, with shared-account identity limitations disclosed.
- Planning/editing/testing/docs/commit/push ownership: implementation in #250; matrix tests in #251; owner and derived closeout here.
- Qualification verdict: qualified for I161 completion; no release or publication authority is implied.

## Retrospective

- Outcome: Complete; fail-closed sandbox fallback and explicit coding preset shipped with matrix evidence.
- Documentation: owner, long task, Board, backlog, iterations index, and manifest synchronized after implementation evidence.
- Lessons: exact-head review must repeat after every implementation or evidence-head change; test-only security closure also requires independent review.
