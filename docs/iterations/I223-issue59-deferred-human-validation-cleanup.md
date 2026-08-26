# Iteration I223: Issue #59 Deferred Human Validation Cleanup

> Document status: Review / Claimed
> Published plan date: 2026-08-23
> Planned objective: resolve every deferred Unix/Windows/manual acceptance row accumulated by the
> Issue #59 TOOL-024-B/C/D chain against its exact implementation head and final integrated main.
> Baseline rule: preserve this evidence-only target; changed behavior uses a new implementation owner.
> MVP deliverable: Issue #378 contains terminal evidence for V59-B1/C1/D1/D2/FINAL, with every pass
> synchronized owner-first and every failure transferred to a separately governed corrective owner.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 validation session |
| Work Slice | Evidence-only execution and reconciliation of Issue #378 rows V59-B1, V59-C1, V59-D1, V59-D2 and V59-FINAL against their existing implementation heads and final integrated main. No Rust/Cargo, product behavior, security policy, release, Dashboard, `/auto` or Desktop authority. |
| Claimed At | 2026-08-26 |
| Source Issue | #378 |
| Governance Claim PR | #405 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer active goal directs Issue #59 completion. Claim PR #405 exact head `77090baea4b729defa9ecea718ad8699fd1b6eb6` passed CI `32942271783`, independent Agent-role approval `5422073546` and merge-time CAS, then merged as `ab7508883100f17260eb8b0a54002c07373395bd`. |
| Implementation PR | None |
| Last Updated | 2026-08-26 |
| Handoff / Release Condition | Close only after every Issue #378 row has exact environment/command/result evidence or a separately governed corrective owner. All rows now have terminal evidence; evidence merge and owner-first closeout remain pending. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| Issue #378 validation cleanup | TOOL-024 / Issue #59 | Planned / Unclaimed | Exact B/C/D heads and final integrated main available | One evidence-only closure packet; no behavior changes |

### Scope

- Run and record every Issue #378 manual/device row against its source head and final integrated main.
- Synchronize source owners first after passes.
- Create separately governed corrective owners for failures before closing the tracker.

### Non-Goals

- No production Rust/Cargo behavior, security policy, release or unrelated acceptance.
- No inference of a pass from CI, Agent review or an unchecked tracker row.

### Acceptance

- Every tracker row records exact command/environment/result as Pass or names a corrective owner.
- Issue #378 closes only after every row is terminal.
- Issue #59 closes only after B/C/D owners and this cleanup are terminal on `main`.

### Planned Validation

- Issue #378 row-by-row evidence and SHA audit.
- Owner/Board/backlog/manifest consistency and both governance validators.
- `git diff --check`; no production-code diff.

### Documentation To Update

- Issue #378, each source owner, TOOL-024 parent, Issue #59 and derived views.

### Risks And Rollback

- Stale binaries or wrong heads create false acceptance: rebuild/check SHA before each row and
  invalidate mismatched evidence.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-23 | Planned cleanup reservation | Created for Deferred Human Validation scheduling only; remains Unclaimed and inactive. |
| 2026-08-26 | Claim preparation | All B/C/D implementation owners are Complete/Closed on `main@a600bd41`. Proposed evidence-only activation binds V59-B1 to `8671edf4`, V59-C1 to `60b0367c`, V59-D1 to `d4d7cb25`, V59-D2 to `a5fbc22e` and V59-FINAL to the post-closeout integrated baseline `a600bd41`; no pass is inferred from those merges. |
| 2026-08-26 | Activation | Claim PR #405 merged as `ab750888`; I223 is effective and is the sole Active Issue #59 iteration. No product-code authority was added. |
| 2026-08-26 | Evidence execution | V59-B1, V59-C1, V59-D1 and V59-D2 passed. V59-FINAL was rechecked through the terminal Windows device walkthrough below. |

## Verification Evidence

| Row | State | Exact evidence |
|---|---|---|
| V59-B1 | Pass | macOS arm64, integrated `main@ab750888`: rebuilt `talos --no-init --repl --no-context -w /private/tmp/talos-i228-impl` started `printf 'v59-cancel-ready\\n'; sleep 300` via `bash(background=true, timeout_secs=600)`, returned job `job_408197bf-f825-4f78-a391-b5860e3454f7`, and `process(cancel)` emitted one `Cancelled` terminal with `cleanup_outcome=terminated`. Process-table check found no matching shell/sleep; foreground `printf 'v59-after-cancel-ok\\n'` exited 0. A separate 60-second run emitted `TimedOut` with terminated cleanup. |
| V59-C1 | Pass | The same real CLI session exercised `process list`, `status`, and `read(cursor=0)` for `job_7f6a1435-1bae-4556-b750-79a4b8ca745c`; read returned `v59-ready\\n` and `next_cursor=10`. Locked focused tests on `ab750888` passed 11/11 supervisor/process cases, including cursor advancement, in-chunk resume, 32-KiB clamp/eviction and unknown-job fail-closed behavior. |
| V59-D1 | Pass | Real Windows Server 2025 evidence at D1 candidate `835578635daa1eebc76e79ca893296baeed6b35a` (same tree as merge `d4d7cb25`), CI `32849330531`, job `97807941106`: cancel, timeout, shutdown and PowerShell grandchild-reap tests passed. Integrated `main@ab750888`, CI `32943409893`, job `98098984864`, repeated all four successfully. |
| V59-D2 | Pass | Unix real CLI passed start/list/status/read/cancel/terminal and remained usable for a foreground command. Windows Server 2025 hosted runner at disposable harness head `4c477a9790a7a344dfbf7305b162ab5b9426c94c`, workflow run `32958236636`, job `98144594163`, passed real interactive CLI approval flow, background start/list/status/read (19-byte `v59-windows-ready`), cancel with exactly one `Cancelled` terminal and foreground PowerShell continuation. Artifact `i223-windows-device-4c477a9790a7a344dfbf7305b162ab5b9426c94c` was uploaded. The harness branch/workflow was disposable and never merged. |
| V59-FINAL | Pass | Final integrated main behavior was rechecked through the Windows Server 2025 device walkthrough above against source tree `main@ab750888` (product tree unchanged); B/C/D interactions, exact-one terminal projection, output delivery and post-cancel foreground usability all passed. |

## Completion Evidence

- Completion Commit: Pending.
- Evidence-only status commit cannot self-certify missing runtime acceptance.

## Variance And Residuals

- Disposable Windows runner attempts are retained as variance evidence: early failures were harness encoding, exit-marker and assertion-boundary defects; the terminal run passed all product assertions. No implementation variance or product-code correction was introduced.

## 2026-08-26 Terminal Windows Evidence Checkpoint

The final disposable device candidate `4c477a9790a7a344dfbf7305b162ab5b9426c94c` ran on Windows
Server 2025 in workflow `32958236636`, job `98144594163`, with all steps successful. The uploaded
artifact contains the CLI transcript and fixture-provider evidence. The provider independently
recorded `read=v59-windows-ready` and `foreground=v59-windows-foreground-ok`; the CLI transcript
contains the background receipt, list/status/read projection, exactly one `terminal: Cancelled`,
and `process request handled`. The disposable branch `test/i223-windows-device` and its workflow
files were used only to obtain this evidence and are not product changes.

## Retrospective

- Exact runner logs prove Windows process ownership and the interactive projection against the integrated product tree.
