# 2026-07-04 High-Risk Execution Set: gix, Runtime, Governance, Self-Bootstrap

> Status: In Progress
> Created: 2026-07-04
> Owner boundary: direct senior-agent execution required
> Trigger: maintainer requested another high-risk execution task set and explicitly asked to include
> the `gix` upgrade.
> Baseline rule: this file is the execution contract. Append checkpoints instead of replacing the
> plan. Changed objectives use a new task or iteration ID.

## Outcome

Plan and execute the next direct-owned high-risk Talos hardening set, focused on dependency
upgrade risk, Git publication boundaries, runtime validation evidence, mutating governance, and
REL-002 self-bootstrap prerequisites.

This task does not reopen I090-I093. It follows from their closeout and uses new iteration IDs.

## In Scope

- Create four planned iteration shells:
  - I094: `gix` upgrade and Git fallback boundary.
  - I095: runtime validation execution evidence.
  - I096: mutating governance preview/write gates.
  - I097: controlled Talos-primary self-bootstrap rehearsal.
- Include a scoped `gix 0.84.0 -> 0.85.0` upgrade attempt in I094.
- Preserve ADR-010: `gix` remains the preferred pure-Rust Git direction; host `git` remains a
  documented fallback only where `gix` lacks a complete safe workflow.
- Keep REL-002 honest: no `v1.0.0` claim unless evidence is complete.

## Out Of Scope

- No crate publish, release tag, GitHub Release, or version-history mutation.
- No automatic push unless the maintainer explicitly asks at that time.
- No forced migration from host-`git` fallback to `gix` when the workflow is not proven.
- No native Git dependency such as `git2`/libgit2.
- No broad Git porcelain surface, destructive reset/clean/rebase, remote credential workflow, or
  issue-sync automation unless selected into a later scoped iteration.
- No permission-default relaxation, Guardian auto-approval, exec DSL expansion, or scheduled direct
  tool execution.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| B0 | Establish execution set | This task record, I094-I097 planned shells, Board, backlog, and iteration index name the new track. | Maintainer request | Governance validation and `git diff --check` pass. | Keep task in Planned with exact blocker. | Planned |
| B1 | Activate I094 | Start the `gix` upgrade / Git fallback boundary iteration. | B0 | Non-terminal inventory disposition recorded; I094 Active. | Keep I094 Planned if another iteration blocks activation. | Complete |
| B2 | Upgrade `gix` safely | Attempt `gix 0.84.0 -> 0.85.0` with no feature expansion beyond accepted scope. | B1 | `cargo update -p gix`, tool tests, workspace check/clippy/test, unavailable-host fallback tests. | Revert upgrade and record exact API/feature blocker. | Complete |
| B3 | Git fallback audit | Classify `git_push`, `git_pull`, `git_checkout`, add/commit, and future stash/reset/merge/rebase against `gix 0.85.0`. | B2 | GIT-001 matrix updated with keep/replace/defer decisions and tests. | Keep host fallback with replacement trigger. | Complete |
| B4 | Activate I095 | Start runtime validation evidence iteration. | B3 | I094 closed or paused with exact residuals. | Keep I095 Planned. | Complete |
| B5 | Validation execution packet | Add or specify allowlisted validation execution evidence records. | B4 | Command, exit status, output summary, and permission decision are durable and tested. | Ship read-only design if execution cannot be safely bounded. | Complete |
| B6 | Activate I096 | Start mutating governance preview/write gates. | B5 | I095 closed or paused with exact residuals. | Keep I096 Planned. | Complete |
| B7 | Governance mutation packet | Typed plan/preview/write flow for owner-doc updates with validation gates. | B6 | No silent owner-doc mutation; governance validation catches drift. | Keep governance read-only and record blocker. | Complete |
| B8 | Activate I097 | Start controlled self-bootstrap rehearsal. | B7 | I096 closed or paused with exact residuals. | Keep I097 Planned. | Planned |
| B9 | Talos-primary rehearsal | Run one documentation-only Talos-primary rehearsal if runtime/governance gates are ready. | B8 | REL-002 evidence explicitly states primary executor boundary and validation evidence. | Record non-qualifying evidence. | Planned |
| B10 | Final closeout | Residual owners, release posture, Board, backlog, iterations, and handoff synchronized. | B9 | Full workspace gates, governance validation, final checkpoint. | Mark Partial with exact unfinished owners. | Planned |

## Dependencies And Prerequisites

- I090-I093 are complete and must not be reopened for changed objectives.
- I085 remains Paused with MC107 real-terminal `/connect` walkthrough residual.
- I086-I089 remain planned product-hardening shells and are not superseded by this direct-owner set.
- ADR-010 remains binding for Git dependency choices.
- REL-002 remains No-go for `v1.0.0`.

## Artifacts And State Owners To Update

- This task record.
- Iteration shells: I094-I097.
- `docs/iterations/README.md`.
- `docs/backlog/active/GIT-001-embedded-git-tools.md`.
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`.
- `docs/backlog/active/GOV-003-builtin-project-governance.md`.
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`.
- `docs/BOARD.md`.
- `docs/backlog/PRODUCT-BACKLOG.md`.
- ADRs only if dependency, permission, Git publication, or governance semantics change.

## Validation And Acceptance Evidence

Every implementation phase must run:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

I094 must additionally record:

- current `gix` lockfile version and target version;
- feature flags before/after;
- cargo tree impact;
- Git tool targeted tests;
- host-`git` unavailable or retained-fallback behavior;
- operation-by-operation keep/replace/defer decisions.

Planning-only phase gates may use:

```sh
scripts/validate_project_governance.sh .
git diff --check
```

## Branch, Worktree And Checkpoint Plan

- Work in the current worktree unless the maintainer explicitly requests a branch.
- Use one logical commit per completed phase if commits are requested.
- Do not push unless the maintainer explicitly asks at that time.
- Append a checkpoint before moving between B3/B4, B5/B6, B7/B8, and B10.

## Allowed Permissions And External Actions

Allowed by this contract:

- Edit repository files in the workspace.
- Run local build, lint, tests, governance checks, and targeted runtime smoke tests.
- Use network only to inspect public crate metadata for `gix` when needed for the upgrade audit.

Not allowed without separate explicit approval:

- Push commits, tags, or release artifacts.
- Publish crates or GitHub Releases.
- Add major runtime/native dependencies.
- Use credentials, paid services, destructive Git operations, remote plugin install, or marketplace
  behavior.

## Destructive Or Irreversible Operations

No destructive or irreversible production operation is authorized. Destructive behavior is limited
to temporary test fixtures and must be covered by tests.

## Time, Cost And Resource Limits

- Timebox: four planned high-risk iterations.
- Monetary spend: zero.
- Network: public crate/source metadata only, no credentials.
- Retry deterministic failures at most twice after concrete fixes before recording a blocker.

## Failure, Retry And Fallback Policy

- If `gix 0.85.0` breaks current Git tools, either fix within I094 scope or revert the upgrade and
  record the blocker.
- If a host-`git` fallback replacement cannot prove equivalent behavior, keep fallback and record
  the replacement trigger.
- If validation execution cannot be permission-bounded, keep the runtime validation packet design
  only.
- If governance mutation cannot avoid silent drift, keep GOV-003 read-only.
- If Codex remains primary executor for B9, record non-qualifying REL-002 evidence.

## Default Decisions For Foreseeable Ambiguity

- Prefer dependency upgrade without feature expansion.
- Prefer host fallback retention over unsafe or under-tested `gix` workflow replacement.
- Prefer read-only/preview flows before write-capable behavior.
- Prefer explicit No-go release posture over optimistic self-bootstrap claims.

## Residual-Work Destination

- Git dependency/fallbacks: GIT-001 and ADR-010.
- Runtime validation evidence: RUNTIME-001.
- Governance mutation/gates: GOV-003.
- Self-bootstrap/release gate: REL-002.
- Residual architecture roots: ARCH-030.

## Checkpoints

### B0 — Planned Execution Set Drafted (2026-07-04)

Completed task items:

- Drafted the high-risk execution set.
- Included the `gix` upgrade in I094.
- Preserved I090-I093 as complete and REL-002 as No-go.

Commands/checks and actual results:

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Recovery or resume instruction:

- Run `git status --short`.
- Read this file, GIT-001, ADR-010, Board, and I094 before activating any work.

### B1 — I094 Activated (2026-07-04)

Completed task items:

- Activated I094 after B0 was committed and pushed.
- Recorded non-terminal inventory disposition:
  - I085 remains Paused with MC107 real-terminal `/connect` walkthrough residual.
  - I086-I089 remain planned product-hardening shells.
  - I095-I097 remain planned and depend on I094/I095/I096 completion or explicit pause.
  - I090-I093 remain Complete and are not reopened.
- Synchronized I094, Board, iteration index, Product Backlog, and this task.

Current state and artifacts:

- I094 is Active.
- GIT-001 remains P0-P2 complete, P3 planned, and selected for the I094 `gix` upgrade/fallback
  audit.
- No dependency update has been applied yet in B1.

Commands/checks and actual results:

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Next task item:

- B2: attempt `gix 0.84.0 -> 0.85.0` safely and run targeted Git validation.

Recovery or resume instruction:

- Run `git status --short`.
- Read I094, GIT-001, ADR-010, and this B1 checkpoint.

### B2/B3 — gix 0.85 Upgrade And Fallback Audit Closed (2026-07-04)

Completed task items:

- Upgraded `crates/talos-tools` from `gix = "0.84"` to `gix = "0.85"` with the same explicit
  feature list: `basic`, `status`, `revision`, `blob-diff`, `index`, and `sha1`.
- Confirmed `Cargo.lock` resolves `gix 0.85.0`.
- Added a host-`git` unavailable regression for retained fallbacks so missing host `git` returns
  the existing actionable error instead of an ambiguous spawn failure.
- Updated GIT-001 with the operation-by-operation fallback decision matrix for `gix 0.85.0`.
- Closed I094 without permission-default changes, destructive Git operations, push/tag/release
  actions, native Git dependencies, or `gix` network/worktree-mutation feature expansion.

Fallback decisions recorded in GIT-001:

- Read-only local status/diff/log/show/branches: keep native `gix` direction.
- Add/commit: keep structured host-`git` fallback while native write orchestration remains under
  evaluation.
- Push/pull/checkout: keep structured host-`git` fallback; `gix 0.85.0` does not yet provide a
  Talos-ready complete workflow for these contracts.
- Stash/reset/merge/rebase/tags/remotes: defer; require fresh coverage review and destructive
  operation tests before implementation.

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p talos-tools`: passed.
- `cargo test -p talos-tools git`: passed.
- `cargo test -p talos-tools`: passed, 226 unit tests plus 18 integration tests and doctests.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `cargo tree --invert gix@0.85.0 -e features`: passed; no Talos feature expansion beyond the
  existing accepted feature set.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Next task item:

- B4: activate I095 Runtime Validation Evidence.

Recovery or resume instruction:

- Run `git status --short`.
- Read I095, RUNTIME-001, this task, and the I094 closeout before starting B4.

### B4 — I095 Activated (2026-07-04)

Completed task items:

- Activated I095 after I094 was committed and pushed.
- Recorded that I094 closed with `gix 0.85.0`, unchanged feature scope, fallback audit, workspace
  validation, clippy, governance validation, and `git diff --check` passing.
- Synchronized I095, RUNTIME-001, REL-002, Board, iteration index, Product Backlog, and this task.

Current state and artifacts:

- I095 is Active.
- B5 is In Progress.
- Runtime validation evidence remains permission-bounded and explicit. This activation does not
  authorize arbitrary shell policy expansion, scheduled execution, Guardian auto-approval, exec DSL,
  hidden pass/fail, release claim, tag, publish, or permission-default change.

Commands/checks and actual results:

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Next task item:

- B5: add or specify the validation evidence packet.

Recovery or resume instruction:

- Run `git status --short`.
- Read I095, RUNTIME-001, REL-002, and this B4 checkpoint.

### B5 — Validation Evidence Packet Closed (2026-07-04)

Completed task items:

- Added `talos validate run` for built-in allowlisted validation profiles.
- Preserved `talos validate plan` as read-only planning.
- Evidence records include command, required flag, source, exit status, stdout/stderr summaries,
  status, and allowlisted-profile permission decision.
- Updated README, README.zh-CN, RUNTIME-001, REL-002 readiness, release-notes draft, I095, Board,
  Product Backlog, iteration index, and this task.

Actual evidence sample:

- `cargo run -p talos-cli -- validate run --profile governance --json`: passed.
- The emitted `governance` record included command `scripts/validate_project_governance.sh .`,
  `exit_status: 0`, `status: passed`,
  `permission_decision: allowlisted validation profile: governance`, `stderr_summary: <empty>`,
  and stdout summary `Governance validation passed: 0 warning(s).`

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo test -p talos-cli validation`: passed, 8 validation/governance tests.
- `cargo check -p talos-cli`: passed.
- `cargo clippy -p talos-cli -- -D warnings`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Next task item:

- B6: activate I096 Governance Mutation Gates.

Recovery or resume instruction:

- Run `git status --short`.
- Read I096, GOV-003, this task, and the I095 closeout before starting B6.

### B6 — I096 Activated (2026-07-04)

Completed task items:

- Activated I096 after I095 was committed and pushed.
- Recorded that I095 closed with allowlisted validation evidence, README sync, REL-002
  non-qualification posture, workspace validation, clippy, governance validation, and
  `git diff --check` passing.
- Synchronized I096, GOV-003, Board, iteration index, Product Backlog, and this task.

Current state and artifacts:

- I096 is Active.
- B7 is In Progress.
- Scope is the smallest safe governance preview/write gate only. This activation does not
  authorize silent owner-doc edits, broad project-manager automation, web write routes, remote
  dashboard mutation, release claim, publish, tag, or permission-default change.

Commands/checks and actual results:

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Next task item:

- B7: implement or precisely block the governance mutation packet.

Recovery or resume instruction:

- Run `git status --short`.
- Read I096, GOV-003, and this B6 checkpoint.

### B7 — Governance Mutation Packet Closed (2026-07-04)

Completed task items:

- Added `talos governance iteration-record preview/write`.
- Limited writes to appending one row to a single resolved `docs/iterations/I###-*.md` owner doc.
- Required `--confirm-preview` for writes.
- Ran governance validation after write and rolled back the file on validation failure.
- Used the new command to write the I096 validation smoke row.
- Updated README, README.zh-CN, GOV-003, REL-002 readiness, release-notes draft, I096, Board,
  Product Backlog, iteration index, and this task.

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo test -p talos-cli governance_mutation`: passed, 5 governance mutation tests.
- `cargo check -p talos-cli`: passed.
- `cargo clippy -p talos-cli -- -D warnings`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `cargo run -p talos-cli -- governance iteration-record preview --iteration I096 --date 2026-07-04 --record-type validation --record ...`: passed.
- `cargo run -p talos-cli -- governance iteration-record write --iteration I096 --date 2026-07-04 --record-type validation --record ... --confirm-preview`: passed and reported `Validation: passed`.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Next task item:

- B8: activate I097 Controlled Self-Bootstrap Rehearsal.

Recovery or resume instruction:

- Run `git status --short`.
- Read I097, REL-002, this task, and the I096 closeout before starting B8.
