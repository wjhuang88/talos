# Talos Low-Ambiguity Self-Bootstrap Pilot

> Created: 2026-07-06
> Trigger: Maintainer wants another Talos self-bootstrap attempt using work items suitable for a
> constrained-autonomy executor.
> Status: Planned

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

No execution checkpoints yet. This task is ready for activation after maintainer confirmation.
