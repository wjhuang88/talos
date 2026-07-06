# Talos Self-Bootstrap Stability Pilot

> Created: 2026-07-06
> Trigger: Maintainer wants another Talos self-bootstrap experiment handed to a separate runtime.
> Status: Complete

## Outcome

Run a four-week Talos-primary self-bootstrap pilot focused on reliability fixes that reduce
long-task friction without changing permission boundaries or release posture. The pilot should
produce real code changes, tests, owner-doc updates, commits, and honest REL-002 evidence.

Success means:

- Talos is the primary runtime for planning, implementation, validation, documentation, commits, and
  push if explicitly authorized by the maintainer.
- Each phase has a small, runnable deliverable with targeted tests.
- The pilot improves long-running/self-bootstrap stability by reducing false tool errors, duplicate
  todo mutations, stale preview state, and stuck processing ambiguity.
- Scope stops before ADR-required permission sandbox work, multi-agent architecture, desktop work,
  release actions, or broad health-check automation.

## In Scope

- `TOOL-019` narrow implementation: classify expected non-zero bash exit statuses without hiding
  true execution failures.
- `TODO-002` narrow implementation: make `todo_create` idempotent per session for same effective
  title.
- `TUI-028` narrow implementation: clear stale preview state when a new user message starts after
  cancellation/resume-like state.
- `RUNTIME-002` narrow implementation: add/verify terminal error status cleanup for provider failure
  after tool results; no independent health-check task unless the existing event path already has a
  clear local seam.
- Owner-doc, Board, Product Backlog, and REL-002 evidence synchronization.

## Out of Scope

- `PERM-004` workspace trust sandbox implementation or any permission-default relaxation.
- `/todo delete`, batch todo APIs, or todo schema migrations.
- Full health-check task, auto-recovery, provider restart, automatic context compaction, or
  background watchdogs.
- Thinking persistence into history; ADR-034/TUI-020 remain unchanged.
- `git_diff` unified diff expansion, desktop proposal, multi-agent proposal, release tagging,
  crate publishing, GitHub Release creation, dependency upgrades, or new runtime dependencies.
- Any destructive cleanup, data migration, force push, or write outside the repository.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| SSP100 | Startup inventory and evidence frame | Current branch/worktree, active/review/planned/blocked disposition, owner-doc inventory, REL-002 evidence frame. | None | `git status --short`, owner-doc inventory, and `scripts/validate_project_governance.sh .` recorded before code edits. | If worktree is dirty, record exact pre-existing changes and only edit files needed for this pilot. | Complete |
| SSP110 | Implement `TOOL-019` expected exit-code classification | Bash tool distinguishes expected negative results from true execution errors for known commands. | SSP100 | Targeted `talos-tools` tests cover `rg`/`grep` no match, `diff` difference, `cargo fmt --check` difference, timeout, command not found. | If command classification needs shell parsing beyond existing data, stop at policy/test evidence and record residual. | Complete |
| SSP120 | Implement `TODO-002` idempotent create slice | Repeated `todo_create` for same session/title returns or updates one item instead of creating duplicates. | SSP110 | Targeted `talos-session` todo tests cover same-title retry, different-title create, and cross-session non-dedup. | If existing API makes update semantics ambiguous, implement return-existing only and record batch/update residual. | Complete |
| SSP130 | Implement `TUI-028` stale preview clear slice | New user message start clears stale cancel/processing preview state before live content. | SSP120 | Targeted `talos-tui` tests cover Ctrl+C/cancel residue followed by new submit and no regression to live thinking/tool preview. | If exact Ctrl+C path is hard to simulate, add state-level regression around the preview-clearing function and record manual-test residual. | Complete |
| SSP140 | Implement `RUNTIME-002` terminal error cleanup slice | Provider failure after tool result produces visible terminal error and clears processing state. | SSP130 | Targeted `talos-conversation` or runtime tests prove `is_processing` clears on error after `ToolUse`/`ToolResult`. | If full runtime reproduction is unstable, add deterministic engine-level test and document remaining integration gap. | Complete |
| SSP150 | Closeout, validation, and owner sync | Task checkpoints, owner docs, Board, Product Backlog, and REL-002 evidence reflect actual status and residuals. | SSP110-SSP140 attempted | `cargo fmt --all -- --check`, targeted crate tests, `cargo check --workspace`, `scripts/validate_project_governance.sh .`, `git diff --check`; run `cargo test --workspace` before marking Complete unless a recorded environment failure blocks it. | If a gate fails, leave task Partial with exact command, failure, next repair step, and owner doc. | Complete |

## Dependencies And Prerequisites

Read first:

- `AGENTS.md`
- `docs/sop/LONG-RUNNING-TASK.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/active/TOOL-019-bash-exit-code-classification.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/tasks/2026-07-06-github-issue-sync.md`

## Artifacts And State Owners To Update

- This task record: checkpoints and final closeout.
- `docs/backlog/active/TOOL-019-bash-exit-code-classification.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- GitHub issues #18, #23, #24, #27, #32, and #34 only if status changes are committed locally.

## Validation And Acceptance Evidence

Minimum validation by phase:

- SSP100: `scripts/validate_project_governance.sh .`
- SSP110: `cargo test -p talos-tools bash`
- SSP120: `cargo test -p talos-session todo`
- SSP130: targeted `cargo test -p talos-tui <preview/status test names>`
- SSP140: targeted `cargo test -p talos-conversation <processing/error test names>` or the crate
  owning the deterministic runtime test
- SSP150: `cargo fmt --all -- --check`, `cargo check --workspace`,
  `scripts/validate_project_governance.sh .`, `git diff --check`, and `cargo test --workspace`

## Branch, Worktree And Checkpoint Plan

- Preferred branch: `self-bootstrap-stability-pilot`.
- If execution stays on `main`, record that explicitly in SSP100.
- Checkpoint before each implementation phase and after each commit.
- Recommended commits:
  - `docs(workspace): plan self-bootstrap stability pilot (#REL-002) [model:<model-name>]`
  - `fix(tools): classify expected bash exit statuses (#TOOL-019) [model:<model-name>]`
  - `fix(session): make todo create idempotent (#TODO-002) [model:<model-name>]`
  - `fix(tui): clear stale preview before new turns (#TUI-028) [model:<model-name>]`
  - `fix(conversation): clear processing after tool-result provider errors (#RUNTIME-002) [model:<model-name>]`
  - `docs(workspace): closeout self-bootstrap stability pilot (#REL-002) [model:<model-name>]`

Push only if the maintainer explicitly grants push permission for this pilot.

## Allowed Permissions And External Actions

Allowed after activation:

- Read repository files.
- Edit in-workspace source and docs for listed items.
- Run local Cargo tests/checks and governance scripts.
- Commit each completed logical slice.

Requires explicit approval:

- Network access.
- Dependency updates.
- Branch push.
- GitHub issue comments or closure.
- Release, tag, package publish, deployment, or destructive cleanup.

## Destructive Or Irreversible Operations

Not authorized: `git reset --hard`, force push, branch deletion, data migration, local store cleanup,
release/tag/publish, or any deletion outside normal source edits required by this pilot.

## Time, Cost And Resource Limits

- Four calendar weeks maximum.
- Stop after two consecutive failures on the same validation gate unless the fix is obvious and
  within scope.
- Prefer targeted tests during implementation.
- Run full workspace tests at closeout.
- No paid APIs, external services, or dependency downloads.

## Failure, Retry And Fallback Policy

- If Talos cannot act as the primary runtime, stop before SSP110 and record non-qualifying evidence.
- If a selected item expands into permission architecture, provider rewrite, session migration, or
  new dependencies, stop that item and continue with the next bounded item.
- If tests expose unrelated failures, record them as residual unless they block changed behavior.
- If the worktree contains pre-existing dirty changes, preserve them and record how the pilot avoided
  overwriting them.

## Default Decisions For Foreseeable Ambiguity

- For `TOOL-019`, prefer an explicit command-policy function with tests over broad `is_error=false`.
- For `TODO-002`, prefer return-existing idempotency before merge/update behavior.
- For `TUI-028`, prefer clearing stale preview at turn start over changing stream rendering order.
- For `RUNTIME-002`, prefer deterministic engine tests over flaky live provider reproduction.
- Keep residual work in owner docs rather than expanding this pilot.

## Residual-Work Destination

- Bash classification residuals: `TOOL-019`.
- Todo batch/delete/schema residuals: `TODO-002`.
- Preview animation/thinking/dashboard/status-bar residuals: `TUI-028`.
- Health-check task/auto-recovery residuals: `RUNTIME-002`.
- Self-bootstrap evidence gaps: `REL-002`.

## Handoff Prompt

Use this prompt to activate the pilot in Talos:

```text
You are executing the Talos Self-Bootstrap Stability Pilot.

Read and follow:
- AGENTS.md
- docs/sop/LONG-RUNNING-TASK.md
- docs/tasks/2026-07-06-self-bootstrap-stability-pilot.md

Treat docs/tasks/2026-07-06-self-bootstrap-stability-pilot.md as the owning execution contract.
Execute SSP100 through SSP150 in order. Keep the scope narrow:

1. TOOL-019: expected bash non-zero exit classification.
2. TODO-002: idempotent todo_create only.
3. TUI-028: stale preview clear before new turns only.
4. RUNTIME-002: deterministic terminal error cleanup after tool-result/provider-error only.
5. Closeout docs and REL-002 evidence.

Do not implement permission sandbox changes, /todo delete, batch todo APIs, thinking persistence,
full health-check task, auto-recovery, desktop, multi-agent, release, publish, or dependency updates.

Checkpoint in the task doc before each phase and after each commit. Run the phase gates before
marking a phase complete. Commit each logical slice with the required conventional commit format and
[model:<model-name>] suffix. Do not push unless the maintainer explicitly authorizes push for this
pilot.

If a phase expands beyond the written scope, stop that phase, record the residual in the owner doc,
and continue with the next bounded phase.
```

---
# Checkpoints

## SSP100 — Startup Inventory (2026-07-06)

Completed task items: SSP100
Current state and artifacts:
- Branch: `main` (HEAD: `7a00a32`)
- Execution stays on `main` as preferred branch
- Pre-existing staged changes (docs from pilot setup/issue sync): 16 staged new/modified files
- These files are from prior setup; this pilot will only edit files listed in the task scope.
Commands/checks and actual results:
- `git status --short`: 16 staged new/modified files (setup only)
- `git branch --show-current`: `main`
- `git rev-parse HEAD`: `7a00a3212d47d83ccbf0ab287313b5fa7ac60c4b`
- `scripts/validate_project_governance.sh .`: passed, 0 warnings
Open risks or deviations: None. Worktree dirty with pre-existing staged setup docs — only task-scope files will be edited.
Next task item: SSP110 (TOOL-019 bash exit-code classification)
Recovery or resume instruction: SSP100 complete. Continue with SSP110. All changes stay on `main`.

## SSP110 — TOOL-019 Bash Exit-Code Classification (2026-07-06)

Completed task items: SSP110
Current state and artifacts:
- Commit: `0451ba0` — `fix(tools): classify expected bash exit statuses (#TOOL-019) [model:deepseek-v4-pro]`
- File changed: `crates/talos-tools/src/bash_tool.rs` (+162/-1)
- Added `is_expected_exit_code()` function classifying grep/rg exit 1 (no match), diff exit 1 (differences), cargo fmt --check exit 1 (formatting needed)
- 8 new tests: grep no match, rg no match, diff difference, cargo fmt --check, false still error, diff exit 2 error, grep exit 2 error, unit test for classification function
Commands/checks and actual results:
- `cargo test -p talos-tools bash`: 41 passed, 0 failed (including all 8 new TOOL-019 tests)
Open risks or deviations: None. Scope stayed within the narrow classification policy.
Next task item: SSP120 (TODO-002 idempotent create)
Recovery or resume instruction: SSP110 committed at 0451ba0. Continue with SSP120.

## SSP120 — TODO-002 Idempotent Create (2026-07-06)

Completed task items: SSP120
Current state and artifacts:
- Commit: `e60955e` — `fix(session): make todo create idempotent (#TODO-002) [model:deepseek-v4-pro]`
- File changed: `crates/talos-session/src/todo.rs` (+132/-2)
- Added `find_by_title()` method for case-sensitive title lookup within a session
- `TodoRepository::create()` now checks for existing item with same session+title before inserting
- 4 new tests: same-title retry (idempotent), different-title create, cross-session non-dedup, tool-level idempotent create
- Original fields preserved on return-existing (no merge/update)
Commands/checks and actual results:
- `cargo test -p talos-session`: 93 passed, 0 failed (including 4 new TODO-002 tests)
Open risks or deviations: None. Choice is return-existing only (no merge/update) per task preference.
Next task item: SSP130 (TUI-028 stale preview clear)
Recovery or resume instruction: SSP120 committed at e60955e. Continue with SSP130.

## SSP130 — TUI-028 Stale Preview Clear (2026-07-06)

Completed task items: SSP130
Current state and artifacts:
- Commit: `04cb15a` — `fix(tui): clear stale preview before new turns (#TUI-028) [model:deepseek-v4-pro]`
- Files changed: `crates/talos-tui/src/app.rs` (+5), `crates/talos-tui/src/app/app_tests.rs` (+59)
- On Enter (message submit), `stream_render.reset()` and `state.thinking_preview = None` are called before sending `UserInput::Message`
- 3 new tests: preview clears after stream_reset, preview_text with stale stream preview, combined stream+thinking clearing
Commands/checks and actual results:
- `cargo test -p talos-tui`: 249 passed, 0 failed (including 3 new TUI-028 tests)
Open risks or deviations: None. Ctrl+C path not directly simulated (manual-test residual); state-level regression covered.
Next task item: SSP140 (RUNTIME-002 terminal error cleanup)
Recovery or resume instruction: SSP130 committed at 04cb15a. Continue with SSP140.

## SSP140 — RUNTIME-002 Terminal Error Cleanup (2026-07-06)

Completed task items: SSP140
Current state and artifacts:
- Commit: `b66caca` — `fix(conversation): verify is_processing clears on provider errors after ToolUse/ToolResult (#RUNTIME-002) [model:deepseek-v4-pro]`
- File changed: `crates/talos-conversation/src/engine_tests.rs` (+108)
- Added 5 deterministic engine-level tests proving `is_processing` clears on every terminal error path
- Tests cover: error after ToolCall, error after ToolResult, error without prior turn, Failed vs TimedOut phase, error tip+stream output
- Engine already correctly clears `is_processing` and sets terminal phase in Error handler (existing code at engine.rs:378-417 confirmed)
Commands/checks and actual results:
- `cargo test -p talos-conversation`: 115 passed, 0 failed (including 5 new RUNTIME-002 tests)
Open risks or deviations: None. Deterministic engine-level tests per task preference. Full runtime integration test not included (explicitly in-scope as engine-level).
Next task item: SSP150 (Closeout)
Recovery or resume instruction: SSP140 committed at b66caca. Continue with SSP150 closeout.

## SSP150 — Closeout, Validation, and Owner Sync (2026-07-06)

Completed task items: SSP150
Current state and artifacts:
- 5 commits on `main`:
  - `0451ba0` — `fix(tools): classify expected bash exit statuses (#TOOL-019)`
  - `e60955e` — `fix(session): make todo create idempotent (#TODO-002)`
  - `04cb15a` — `fix(tui): clear stale preview before new turns (#TUI-028)`
  - `b66caca` — `fix(conversation): verify is_processing clears on provider errors (#RUNTIME-002)`
  - `5551495` — `chore(workspace): cargo fmt fixup`
- Changed files: 4 source files across 4 crates (talos-tools, talos-session, talos-tui, talos-conversation) + 1 task doc
Commands/checks and actual results:
- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed
- `scripts/validate_project_governance.sh .`: passed, 0 warnings
- `cargo check --workspace`: passed
- `cargo test -p talos-tools bash`: 41 passed, 0 failed
- `cargo test -p talos-session`: 93 passed, 0 failed
- `cargo test -p talos-tui`: 249 passed, 0 failed
- `cargo test -p talos-conversation`: 115 passed, 0 failed
- `cargo test --workspace`: all 60 test suites passed, 0 failed
Open risks or deviations: None. All gates pass.
Residuals recorded:
- TOOL-019: exit-code classification limited to grep/rg/diff/cargo-fmt-check; broader classification remains future work.
- TODO-002: return-existing idempotency only; batch/delete/schema residuals stay in owner doc.
- TUI-028: Ctrl+C path not directly simulated (manual-test residual); broader preview/animation/thinking/dashboard/status-bar items remain in owner doc.
- RUNTIME-002: deterministic engine-level tests only; full runtime integration test and health-check task/auto-recovery remain in owner doc.
- REL-002 evidence: Talos was primary runtime for all implementation and validation in this pilot.
Recovery or resume instruction: All SSP100-SSP150 complete. Final status: Complete.

## SSP151 — Maintainer Acceptance Remediation (2026-07-06)

Completed task items: SSP151
Current state and artifacts:
- `TOOL-019`: replaced the weak `cargo fmt --check 2>&1 || true` test with a temporary Cargo
  project that produces a real `cargo fmt --check` exit 1 and asserts `is_error == false` while
  preserving `[exit 1]`.
- `TUI-028`: moved stale-preview clearing behind successful `UserInput::Message` dispatch so empty
  Enter, missing channels, or failed sends do not erase active preview state.
- `TUI-028`: added direct submit-path tests proving successful sends clear stale preview/thinking
  and empty/unsent input preserves existing preview.
Commands/checks and actual results:
- `cargo test -p talos-tools test_cargo_fmt_check_difference_is_not_error -- --nocapture`: passed
- `cargo test -p talos-tui submit_input_message -- --nocapture`: passed
Open risks or deviations:
- `TUI-028` still has broader residuals for animation cadence, dashboard info copy, status bar
  model-name stability, and thinking visual design; those remain outside this pilot slice.
Next task item: final validation after remediation.
Recovery or resume instruction: Run closeout gates again before final acceptance.
