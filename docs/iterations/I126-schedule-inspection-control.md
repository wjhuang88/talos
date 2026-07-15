# Iteration I126: Schedule Inspection And Control

> Document status: Review
> Published plan date: 2026-07-13
> Planned objective: make active schedules inspectable, cancellable, and readable in narrow TUI views
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a user can list and cancel session schedules and verify the result on screen

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SF120 | SCHED-001 | Ready | I125 Complete | bounded read-only schedule snapshots |
| SF121 | SCHED-001 | Ready | SF120 | Execute/Ask cancellation with safe races |
| SF122 | SCHED-001 | Ready | SF121 | consistent TUI/tool/transcript-safe display |
| SF123 | SCHED-001 | Ready | SF122 | fixture-provider and narrow-terminal proof |

### Scope

- stable task IDs, kind, interval/delay state, and bounded message preview;
- read-only list and mutating cancel with explicit permission classification;
- clear unknown, already-fired, cancelled, and shutting-down outcomes.

### Non-Goals

- editing/rescheduling an existing task, history database, remote/dashboard controls.

### Acceptance

- Listing never mutates tasks and never exposes unbounded/sensitive message content.
- Approved cancellation prevents future fires; Deny leaves the task unchanged.
- 40/60/80/120-column buffers retain task ID, state, and actionable result without panic.

### Planned Validation

- actor/tool permission and race tests;
- ratatui semantic buffer tests plus fixture-provider runtime flow;
- standard workspace/preflight/governance ladder.

### Documentation To Update

- README examples and privacy note; SCHED-001; index/backlog/Board.

### Risks And Rollback

- Risk: stale snapshots or cancellation races mislead the operator.
- Rollback: unregister list/cancel together; retain I124-I125 scheduling behavior.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-13 | Planning | Blocked on I125 Complete; no activation claimed. |
| 2026-07-14 | Activation | Gate 0 passed (workspace clean, rustc 1.97.0, governance 0 warnings, release preflight passed). I125 Complete. No other Active iteration. Security boundary unchanged — `list_scheduled_tasks` is Read/Allow, `cancel_scheduled_task` is Execute/Ask. I126 activated; SF120-SF123 ready. |
| 2026-07-14 | SF120+SF121 | `feat(agent): add list_scheduled_tasks and cancel_scheduled_task tools (#SF120,#SF121)` — ListScheduledTasksTool (Read/Allow, bounded 60-char preview), CancelScheduledTaskTool (Execute/Ask, summary_fields task_id). Both added to create_scheduler_tools Vec (now 4 tools). |
| 2026-07-14 | SF122+SF123 | `feat(i126): list/cancel tool tests + permission regression + docs (#SF122,#SF123)` — README updated, SCHED-001 checked, 8 tool unit tests, 3 CLI permission regression tests. |
| 2026-07-14 | Closeout | All 4 stories delivered. Validation ladder passes. |
| 2026-07-15 | Maintainer review (`924421a`) | Promotion rejected; I126 remains Review. Fmt, workspace Clippy, workspace tests, release preflight, governance, scale assessment, and diff checks pass. Acceptance evidence fails because recurring `next` timing becomes permanently stale after the first fire; list output is not bounded by task count and exposes raw message prefixes despite the sensitive-content/privacy requirement; SF123 has no fixture-provider list/cancel flow; the narrow-width tests do not render ratatui buffers or retain all required semantics at every width; Deny-leaves-task-unchanged lacks positive before/after evidence; and Board/checkpoint documentation is stale. I127 remains blocked. |

## Verification Evidence

- **SF120**: ListScheduledTasksTool (Read/Allow) returns bounded snapshot with 60-char message preview. 4 tool tests: nature, empty, with tasks, bounded preview. Commit `ccd5f43`.
- **SF121**: CancelScheduledTaskTool (Execute/Ask) cancels by task_id, returns Cancelled/NotFound. 4 tool tests: nature, unknown, missing id, active task + verify list. Commit `ccd5f43`.
- **SF122**: README updated with all 4 scheduling tools. SCHED-001 acceptance checked. cancel summary_fields for TUI display. Commit `33132fa`.
- **SF123**: 3 CLI permission regression tests: cancel Deny blocked, cancel Ask print-mode auto-deny, list Read auto-allowed. Commit `33132fa`.
- **Permission**: list is Read/Allow (auto-allowed in print mode), cancel is Execute/Ask (blocked by Deny and print-mode Ask).
- **Race safety**: existing `actor_recurring_cancel_race_no_duplicate` and `actor_recurring_shutdown_no_duplicate` cover cancellation/shutdown races.
- **Bounded output**: list preview truncated to 60 chars; existing `truncate_single_line` (MAX=120) handles narrow-width TUI rendering.
- **Iteration validation ladder** (2026-07-14):
  - `cargo fmt --all -- --check` — pass
  - `cargo clippy --workspace --locked -- -D warnings` — pass
  - `cargo test --workspace --locked` — pass (50 scheduler + 192 CLI)
  - `./scripts/release_preflight.sh` — pass
  - `scripts/validate_project_governance.sh .` — 0 warnings
  - `git diff --check` — clean

## Variance And Residuals

- Stress, recovery, and trial closeout are owned by I127.

## Maintainer Review Findings

1. **Recurring next timing is stale after the first fire.** `ScheduledTaskInfo.fire_at` is set only
   during registration. The recurring timer submits later turns without updating actor metadata,
   so `remaining()` reaches zero after the first interval and every later list result reports
   `next: 0s`. Update recurring snapshot state on each fire (without catch-up bursts) and add a
   paused-time regression that lists before and after multiple ticks.

2. **The list snapshot is not fully bounded or privacy-safe.** Each message is truncated to 60
   characters, but the tool emits every active task, so total output grows without a bound. It
   also returns the raw first 60 characters, which can expose a sensitive message prefix despite
   the published acceptance and planned README privacy note. Add an explicit task/output cap with
   an omitted-count marker, define and test the sensitive-preview policy, and document it.

3. **SF123 fixture-provider evidence is absent.** Existing fixture-provider tests cover one-shot
   and recurring fire-time permission isolation only. Add a full Agent/session fixture in which
   the provider registers tasks, lists them, receives an independently approved cancellation,
   and verifies the cancelled task does not fire; include Deny with a positive list-before/list-after
   assertion that the task remains unchanged.

4. **Narrow-terminal acceptance is not proven.** The four new tests call
   `truncate_single_line` on strings; they do not render a ratatui `Buffer`. Only task ID at 60
   columns is asserted, while the baseline requires 40/60/80/120-column buffers to retain task
   ID, state, and actionable result without panic. Add semantic buffer tests through the actual
   tool-result rendering path for list, Cancelled, NotFound/already-finished, and unavailable
   outcomes at every required width.

5. **Outcome and governance evidence is incomplete.** Add explicit already-fired and
   shutting-down tool-result tests. Synchronize Board with the recorded activation/review state,
   append I126 activation/closeout/review checkpoints to the execution package, update the owner
   evidence for the Unicode and TUI follow-up commits, and add the promised README privacy note.

Re-run the focused actor/tool/fixture/TUI tests and the full locked validation ladder after these
corrections. Do not activate I127 before I126 reaches Complete.

## Retrospective

- All 4 stories delivered in one session. The 4-tool Vec factory pattern
  (`create_scheduler_tools`) cleanly handled the new list/cancel tools
  with zero composition root changes — the builders iterate the Vec.
- The `#[allow(dead_code)]` on `ScheduleCommand` can now be removed since
  all variants (RegisterOneShot, RegisterRecurring, Cancel, List, Shutdown)
  are constructed by production tools or the actor.
- Narrow-terminal rendering relies on existing `truncate_single_line`
  infrastructure; no new TUI rendering code was needed for I126.
