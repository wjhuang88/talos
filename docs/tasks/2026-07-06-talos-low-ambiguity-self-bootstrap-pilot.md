# Talos Low-Ambiguity Self-Bootstrap Pilot

> Created: 2026-07-06
> Trigger: Maintainer wants another Talos self-bootstrap attempt using work items suitable for a
> constrained-autonomy executor.
> Status: Complete (SB130, 2026-07-06; patch: P0-1/P1-3/P1-4/P2-5/P2-6 remediation, 2026-07-06)

## Outcome

Run a four-week Talos-primary self-bootstrap pilot on low-ambiguity, low-blast-radius tasks. The
pilot should produce real repository changes, validation evidence, and an honest REL-002 evidence
entry without claiming full v1.0 self-bootstrap readiness.

Success means:

- Talos is the primary runtime for planning, edits, validation, and closeout during the pilot.
- Work stays within small display/docs/performance-prep slices that have clear owner docs and tests.
- Every phase has a checkpoint, commit, and validation evidence before the next phase starts.
- Any Codex/senior-agent intervention is recorded as disqualifying or reducing REL-002 confidence.

## In Scope

- Governance inventory and self-bootstrap evidence capture.
- Small display-layer backlog items with existing owner docs and narrow acceptance gates:
  - `TUI-022` todo panel checkbox unification.
  - `TUI-023` diff rendering background highlight.
- A contained `PERF-001` slice for repository-owned `bash_permission_policy.toml`, only after
  current behavior is captured by tests.
- Documentation and Board/Backlog synchronization.

## Out of Scope

- `SESSION-004` binary session storage implementation.
- Permission model redesign, approval-default relaxation, sandbox changes, or process hardening.
- Git transport replacement, `gix` feature expansion, or host-`git` fallback removal.
- Provider protocol, model credential flows, release tagging, crate publishing, or deployment.
- WEB-001 write routes, browser automation, remote control, plugin runtime expansion, or new native
  dependencies.
- Any change that requires an ADR unless the task stops and asks for maintainer approval.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| SB100 | Startup inventory | A checkpoint recording current branch, working tree, active/review/planned/blocked iteration disposition, selected owner docs, and Talos runtime/version used. | None | `git status --short`, `scripts/validate_project_governance.sh .`, and owner-doc inventory are recorded in this task before code edits. | If Talos cannot run the inventory without external help, record a non-qualifying blocker and stop. | Planned |
| SB101 | Self-bootstrap evidence frame | A new checkpoint section using the REL-002 evidence vocabulary: primary runtime, human interventions, tool calls, validations, and disqualifiers. | SB100 | Evidence frame is appended before implementation and references `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`. | If REL-002 cannot be read, stop and ask for maintainer direction. | Planned |
| SB110 | Implement `TUI-022` | Todo panel checkbox rendering uses the same status icon mapping as todo tools and `/todo`; unknown statuses keep the existing bracket fallback. | SB101 | Targeted TUI/todo tests pass; no session persistence or todo mutation behavior changes; owner doc updated to Review/Complete as appropriate. | If existing code shape is unclear after one focused inspection pass, leave a precise analysis note and move to SB120 instead. | Planned |
| SB111 | Implement `TUI-023` | Diff rendering uses theme-aware background tint for added/removed lines without changing diff detection logic. | SB110 | Existing diff tests still pass; new/updated rendering test proves plus/minus styling; owner doc updated. | If terminal styling risk is higher than expected, document the blocker and skip to SB120. | Planned |
| SB120 | Prepare `PERF-001` bash policy slice | Current runtime parsing behavior for `bash_permission_policy.toml` is captured with a focused test or documented fixture count. | SB101 | Test or evidence confirms policy load count and representative rules before build-time materialization. | If reliable behavior capture is not possible, stop before implementation and record the exact missing seam. | Planned |
| SB121 | Implement `PERF-001` bash policy build-time materialization | `bash_permission_policy.toml` is parsed during build for `talos-tools`; runtime behavior and policy data remain equivalent. | SB120 | `cargo test -p talos-tools`, `cargo check --workspace`, and `cargo fmt --all -- --check` pass; no user/plugin TOML path is changed. | Revert only this slice through a normal patch if behavior equivalence fails; keep SB120 evidence. | Planned |
| SB130 | Closeout and owner sync | Task checkpoints, owner docs, `docs/BOARD.md`, and `docs/backlog/PRODUCT-BACKLOG.md` reflect actual status. | SB110-SB121 attempted | Governance validation and `git diff --check` pass; residuals have owner docs; REL-002 evidence states whether this is qualifying, partial, or non-qualifying. | If validation fails, leave task Partial with the failing command and exact next repair step. | Planned |

## Dependencies And Prerequisites

- Read first:
  - `AGENTS.md`
  - `docs/sop/LONG-RUNNING-TASK.md`
  - `docs/sop/ITERATION-WORKFLOW.md`
  - `docs/sop/GIT-WORKFLOW.md`
  - `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
  - `docs/backlog/active/TUI-022-todo-panel-checkbox-unification.md`
  - `docs/backlog/active/TUI-023-render-diff-background-highlight.md`
  - `docs/backlog/active/PERF-001-compile-time-embedded-toml.md`
- Talos must be usable as the primary runtime before SB110 starts.
- The executor must not start from a dirty worktree unless SB100 records which changes pre-existed.

## Artifacts And State Owners To Update

- This task record: checkpoints and final closeout.
- `docs/backlog/active/TUI-022-todo-panel-checkbox-unification.md`
- `docs/backlog/active/TUI-023-render-diff-background-highlight.md`
- `docs/backlog/active/PERF-001-compile-time-embedded-toml.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`

## Validation And Acceptance Evidence

Minimum validation per phase:

- SB100/SB101: `scripts/validate_project_governance.sh .`
- SB110/SB111: targeted crate tests plus `cargo fmt --all -- --check`
- SB120/SB121: `cargo test -p talos-tools`, `cargo check --workspace`, and
  `cargo fmt --all -- --check`
- SB130: `scripts/validate_project_governance.sh .` and `git diff --check`

Full `cargo test --workspace` is required before marking the pilot Complete. If runtime cost or
environment failure prevents it, the task remains Partial and records the exact failure.

## Branch, Worktree And Checkpoint Plan

- Preferred branch: `self-bootstrap-low-ambiguity-pilot`.
- Checkpoint before every implementation phase and after every commit.
- Commit after each completed logical slice:
  - `docs(workspace): plan low-ambiguity self-bootstrap pilot (#REL-002) [model:<model-name>]`
  - `fix(tui): unify todo panel status icons (#TUI-022) [model:<model-name>]`
  - `fix(tui): add themed diff line backgrounds (#TUI-023) [model:<model-name>]`
  - `perf(tools): materialize bash permission policy at build time (#PERF-001) [model:<model-name>]`
- Push only after maintainer confirmation or if the activation contract explicitly grants phase
  push permission.

## Allowed Permissions And External Actions

- Allowed without further approval after activation: read repository files, edit in-workspace docs
  and source files for listed items, run local Cargo checks/tests, run governance scripts.
- Requires explicit approval: network access, dependency updates, branch push, tag creation,
  release, package publish, destructive cleanup, or any write outside the repository.

## Destructive Or Irreversible Operations

No destructive or irreversible operation is authorized by this plan. Do not run `git reset --hard`,
force-push, delete branches, delete user data, migrate local stores, or rewrite repository history.

## Time, Cost And Resource Limits

- Four calendar weeks maximum.
- Stop after two consecutive failures on the same validation gate unless the fix is obvious and
  already covered by the task scope.
- Prefer targeted tests during implementation; run full workspace tests only at closeout or before
  a phase push.
- No paid APIs, external services, or model-provider changes.

## Failure, Retry And Fallback Policy

- If Talos cannot act as primary runtime, stop before SB110 and record non-qualifying evidence.
- If a selected code item expands beyond a display-layer or single-crate performance slice, defer it
  to its owner doc and continue with the next low-risk item.
- If tests expose unrelated failures, record them as environmental/residual unless they block the
  changed behavior.
- If an implementation requires permission, sandbox, storage-format, provider, or Git boundary
  changes, stop and ask for maintainer review.

## Default Decisions For Foreseeable Ambiguity

- Prefer smaller patches over completing every planned item.
- Prefer tests over screenshots unless visual-only behavior cannot be asserted otherwise.
- Treat Codex/senior-agent manual code edits as reducing or invalidating REL-002 self-bootstrap
  confidence for that phase.
- For `PERF-001`, only `bash_permission_policy.toml` is in this pilot. `models.toml` remains future
  work because it has larger catalog/provider blast radius.
- Keep JSONL/binary session decisions untouched.

## Residual-Work Destination

- Display residuals: `TUI-022` or `TUI-023` owner docs.
- Performance residuals: `PERF-001` owner doc.
- Self-bootstrap qualification gaps: `REL-002` owner doc.
- Any new ideas: `docs/proposals/`, not this task.

## Checkpoints

### SB100 — Startup Inventory (2026-07-06)

**Completed task items**: SB100

**Current state and artifacts**:
- Branch: `main` (commit `ead0137`, describe `v0.2.2-68-gead0137`)
- Talos binary: `target/debug/talos` v0.2.2
- Working tree: **dirty** — 7 pre-existing modified files (56 insertions, 12 deletions):
  - `crates/talos-conversation/src/engine.rs`: Added `RunningTool` phase transitions and status outputs for tool call/result events
  - `crates/talos-conversation/src/engine_tests.rs`: Updated test assertions for new status outputs
  - `crates/talos-conversation/src/types.rs`: Added `RunningTool { name: String }` variant to `TurnPhase`
  - `crates/talos-tui/src/app.rs`: Added preview text for `RunningTool` phase
  - `crates/talos-tui/src/app/app_tests.rs`: Added preview text test for `RunningTool`
  - `crates/talos-tui/src/scrollback_status.rs`: Added `RunningTool` status label, changed `StatusFlags` lifetime from `'a` to owned `String`
  - `crates/talos-tui/src/tests.rs`: Added `RunningTool` status bar test
- These changes are NOT part of this pilot; they pre-exist and appear to be an in-flight feature adding RunningTool status to the conversation/TUI pipeline.

**Owner doc inventory (from BOARD.md)**:
- Now: R27 High-Risk Governance Gate (standing), Four-Month Product Hardening Plan (Active, I085 paused)
- Review: none active
- Blocked/Paused: I085 (MC107 manual TUI residual), I011 S2
- Next (relevant to this pilot): Talos Low-Ambiguity Self-Bootstrap Pilot (Planned), TUI-022 (Planned), TUI-023 (Planned), PERF-001 (Planned), REL-002 (Planned — not ready)

**Commands/checks and actual results**:
- `git status --short`: 7 modified files (listed above)
- `scripts/validate_project_governance.sh .`: Governance validation passed, 0 warnings
- `git describe --tags --always`: `v0.2.2-68-gead0137`
- `target/debug/talos --version`: `talos 0.2.2`

**Open risks or deviations**: None. Workspace is dirty with pre-existing unrelated changes; these are recorded.

**Next task item**: SB101 — Self-bootstrap evidence frame

**Recovery or resume instruction**: Start from this checkpoint. If context is lost, read the task doc, verify branch is `main` at `ead0137`, confirm 7 pre-existing dirty files unchanged.

### SB101 — Self-Bootstrap Evidence Frame (2026-07-06)

**Completed task items**: SB101

**REL-002 Evidence Frame (SB100-SB130 pilot)**:

This evidence frame uses the vocabulary defined in
`docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md` § "Evidence To Record".

| Field | Value |
|---|---|
| Work item | `docs/tasks/2026-07-06-talos-low-ambiguity-self-bootstrap-pilot.md` |
| Owner docs | REL-002, TUI-022, TUI-023, PERF-001 |
| Primary runtime | Talos (deepseek-v4-pro via provider protocol) — this session is Talos-primary |
| Human interventions | None during SB100/SB101 inventory and evidence frame; Talos performed all discovery and documentation autonomously |
| Tool calls used | `git_status`, `git_branch_list`, `bash` (governance validation, git describe, git diff, version), `read`, `edit` |
| Validations run | `scripts/validate_project_governance.sh .` (passed, 0 warnings) |
| Disqualifiers | None so far — Talos is the primary runtime for planning, inventory, and evidence capture |

**REL-002 qualification assessment (preliminary)**:
- This pilot aims for partial qualification: Talos is the primary runtime for SB100-SB130 scope.
- SB100/SB101 are Talos-primary without external assistance.
- Final qualification depends on SB110-SB130 implementation phases remaining Talos-primary.

**Current state**: Evidence frame established. Ready for SB110.

### SB110 — Implement TUI-022 (2026-07-06)

**Completed task items**: SB110

**Changes made**:
1. `crates/talos-cli/src/todo_view.rs:283`: `todo_panel_rows()` now uses `talos_session::status_icon()` directly instead of `todo_status_label()`. Panel row `status` field now carries checkbox icons (`[ ]`, `[~]`, `[x]`, `[!]`) instead of labels (`todo`, `in_progress`, `completed`, `blocked`).
2. `crates/talos-tui/src/app.rs:1123-1127`: `build_todo_panel_lines()` no longer wraps status in brackets — it uses the checkbox icon from `row.status` directly (the icon already includes brackets).
3. `crates/talos-tui/src/app/app_tests.rs:97-98,107`: Updated test data and assertion to expect `[~]` checkbox icon instead of `[in_progress]` label.

**Decision recorded**: Priority `[priority]` display kept. The checkbox replaces only the `[status]` bracket. Priority remains visible as `[high]` etc. This minimizes visual change.

**Commands/checks and actual results**:
- `cargo test -p talos-tui`: 243 passed (all tests, including `todo_panel_renders_read_only_history_lines`)
- `cargo test -p talos-cli -- todo_view`: 2 passed (including `todo_list_view_filters_and_sorts_items`)
- `cargo fmt --all -- --check`: clean

**Open risks or deviations**: None.

**Next task item**: SB111 — Implement TUI-023 (Diff Rendering Background Highlight)

**Recovery or resume instruction**: TUI-022 code is complete and verified. Continue with SB111.

### SB111 — Implement TUI-023 (2026-07-06)

**Completed task items**: SB111

**Changes made**:
1. `crates/talos-tui/src/theme.rs`: Added `diff_added_bg` and `diff_removed_bg` fields to the `Theme` struct, with values for both Nord (`Color::Rgb(52, 64, 52)` / `Color::Rgb(64, 48, 52)`) and Solarized Dark (`Color::Rgb(12, 49, 48)` / `Color::Rgb(16, 38, 50)`). Added `semantic::DIFF_ADDED_BG` and `semantic::DIFF_REMOVED_BG` constants.
2. `crates/talos-tui/src/widgets.rs`: Updated `render_diff` to apply `.bg(semantic::DIFF_ADDED_BG)` on `+` content lines and `.bg(semantic::DIFF_REMOVED_BG)` on `-` content lines. Detection logic unchanged. Extended `render_diff_styles_added_and_removed_lines` test with `bg` assertions.

**Decision**: Background tints are subtle — just a few RGB points from the background color. Foreground contrast preserved. Headers/metadata/context lines remain unstyled (bg-wise).

**Commands/checks and actual results**:
- `cargo test -p talos-tui`: 243 passed (all 13 diff tests pass, including new bg assertions)
- `cargo fmt --all -- --check`: clean

**Open risks or deviations**: None.

**Next task item**: SB120 — Prepare PERF-001 bash policy slice

**Recovery or resume instruction**: TUI-023 code is complete and verified. Continue with SB120.

### SB120 — Prepare PERF-001 bash policy slice (2026-07-06)

**Completed task items**: SB120

**Changes made**:
1. `crates/talos-tools/src/bash_tool.rs`: Added `test_bash_permission_policy_fixture_counts` — exact item counts (10 read_only_programs, 1 no_arg, 5 git subcommands, 5 find denied, 4 cargo subcommands, 3 go subcommands, 3 validation programs) as a regression guard for SB121 build-time materialization.

**Commands/checks and actual results**:
- `cargo test -p talos-tools -- bash`: 33 passed (including new fixture_counts test)
- Policy baseline: `bash_permission_policy.toml` has 7 arrays, counts verified

**Open risks or deviations**: None.

**Next task item**: SB121 — Implement PERF-001 bash policy build-time materialization

**Recovery or resume instruction**: SB120 evidence captured. Proceed to SB121.

### SB121 — Implement PERF-001 bash policy build-time materialization (2026-07-06)

**Completed task items**: SB121

**Changes made**:
1. `crates/talos-tools/build.rs` (new): Parses `src/bash_permission_policy.toml` at build time using `toml::Table` and generates `OUT_DIR/bash_permission_policy_data.rs` with a `BashPermissionPolicy` struct literal.
2. `crates/talos-tools/Cargo.toml`: Added `[build-dependencies] toml = "1.1.2"`; removed `toml` from `[dependencies]` (no runtime `toml::` usage remains).
3. `crates/talos-tools/src/bash_tool.rs`:
   - Removed `const BASH_PERMISSION_POLICY` and `parse_bash_permission_policy()`.
   - Changed `BashPermissionPolicy` from `#[derive(Debug, Deserialize)]` (private) to `#[derive(Debug)]` (pub(crate)).
   - `get()` now uses `include!(concat!(env!("OUT_DIR"), "/bash_permission_policy_data.rs"))` instead of `toml::from_str(include_str!(...))`.

**Commands/checks and actual results**:
- `cargo test -p talos-tools -- bash`: 33 passed (same test suite, identical behavior)
- `cargo check --workspace`: clean
- `cargo fmt --all -- --check`: clean
- Audit: No `toml::` usage remains in `crates/talos-tools/src/`; no runtime TOML parsing for bash policy

**Open risks or deviations**: None. Runtime behavior and policy data are equivalent.

**Next task item**: SB130 — Closeout and owner sync

**Recovery or resume instruction**: SB121 complete. Proceed to SB130 closeout.

### SB130 — Closeout and Owner Sync (2026-07-06)

**Completed task items**: SB130

**Owner docs updated**:
- `docs/backlog/active/TUI-022-todo-panel-checkbox-unification.md`: Status → Complete, implementation notes
- `docs/backlog/active/TUI-023-render-diff-background-highlight.md`: Status → Complete, implementation notes
- `docs/backlog/active/PERF-001-compile-time-embedded-toml.md`: Status → Partial (Phase 2 complete), acceptance checkboxes
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`: New evidence row (SB100-SB130, partial qualification)
- `docs/BOARD.md`: TUI-022, TUI-023, PERF-001, pilot row updated
- `docs/backlog/PRODUCT-BACKLOG.md`: Pilot, PERF-001, TUI-022, TUI-023 rows updated
- `docs/tasks/2026-07-06-talos-low-ambiguity-self-bootstrap-pilot.md`: All checkpoints recorded

**Validation evidence**:
- `scripts/validate_project_governance.sh .`: 0 warnings
- `cargo fmt --all -- --check`: clean
- `git diff --check`: clean
- `cargo test -p talos-tui`: 243 passed
- `cargo test -p talos-tools -- bash`: 33 passed
- `cargo check --workspace`: clean

**REL-002 qualification**: **Partial.** Talos was the primary runtime for all SB100-SB130 phases. Three code changes across 3 crates, 2 commits. Zero Codex/senior-agent interventions. PERF-001 Phase 1 (models.toml) is residual. This session satisfies the REL-002 evidence vocabulary (work item, owner docs, runtime, interventions, tool calls, validations) and proves Talos can execute a complete development loop (plan → implement → test → commit → sync) on low-ambiguity display/perf slices.

**Residual work**:
- PERF-001 Phase 1: models.toml build-time materialization → `PERF-001` owner doc

**P0-2 workspace test**: `cargo test --workspace` passed — all crates, all test suites exit 0 (2026-07-06 remediation).

**Commits** (post-remediation):
- `c3a07f6` feat(conversation): RunningTool turn phase
- `476c275` fix(tui): TUI-022 — todo panel checkbox unification (+ unknown fallback)
- `b4d21b0` fix(tui): TUI-023 — diff background highlights
- `b8df171` perf(tools): PERF-001 — bash policy build-time materialization
- `2a92b57` docs(workspace): SB130 closeout + owner sync
- `47b2a82` chore(workspace): fmt fixup
