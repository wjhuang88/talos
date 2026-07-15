# Iteration I126: Schedule Inspection And Control

> Document status: Complete (2026-07-15)
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
| 2026-07-15 | Maintainer re-review (`7b5a0ab`) | Promotion rejected again; I126 remains Review. The full locked validation ladder is green, and recurring metadata, user-visible privacy, and an Agent/session list/cancel happy path are improved. Acceptance proof is still incomplete: the fixture neither proves no post-cancel fire nor exercises Deny with positive before/after snapshots; list-cap/omitted-count behavior lacks a multi-task regression; recurring timing is tested after only one tick rather than multiple ticks; and the new ratatui tests render constant strings directly through `Paragraph`, bypassing the actual tool-result scrollback path while using disjunctive assertions that do not retain every required semantic. Owner evidence also still describes the superseded 60-character preview and old test set. I127 remains blocked. |
| 2026-07-15 | Closure | Current working tree closes both review rounds: recurring snapshots are refreshed after three paused-time ticks; the user-visible list hides message content and proves its 20-row cap plus omitted count; an approved Agent/session cancellation is advanced past its deadline with no scheduled submission, while an independently denied cancellation is followed by a second-session list that still shows the task; and all 40/60/80/120-column checks construct `ToolResultDisplay`, use `build_tool_result_scrollback_lines`, and render a ratatui Buffer. I126 is Complete. I127 is Planned and unblocked, not activated. |

## Verification Evidence

- **SF120**: `ListScheduledTasksTool` is Read/Allow and shows task ID, kind, and next timing only. It never renders message content, caps output at 20 task rows, and reports omitted rows. Tests cover empty, populated, privacy, and a 21-task cap regression. Commits `ccd5f43`, `5f13637`, plus closure corrections.
- **SF121**: `CancelScheduledTaskTool` is Execute/Ask and returns Cancelled/NotFound. Approved cancellation is proved through an Agent/session fixture and no later scheduled submission; a separate real-session fixture proves configured Deny leaves the task visible before and after the denied request. Commits `ccd5f43`, `7b5a0ab`, plus closure corrections.
- **SF122**: README documents the no-message-content list policy. TUI tests build actual `ToolResultDisplay` scrollback and render ratatui Buffers at 40/60/80/120 columns for list, Cancelled, NotFound/already-finished, unavailable, and empty outcomes.
- **SF123**: 3 CLI permission regression tests remain: cancel Deny blocked, cancel Ask print-mode auto-deny, and list Read auto-allowed. Fixture-provider runtime coverage now includes approved list/cancel/no-fire and denied cancel/list-after flows.
- **Permission**: list is Read/Allow (auto-allowed in print mode), cancel is Execute/Ask (blocked by Deny and print-mode Ask).
- **Race safety**: existing `actor_recurring_cancel_race_no_duplicate` and `actor_recurring_shutdown_no_duplicate` cover cancellation/shutdown races.
- **Bounded output**: list omits message content entirely, returns at most 20 rows, and includes an omitted-count marker.
- **Iteration validation ladder** (2026-07-15):
  - `cargo fmt --all -- --check` — pass
  - `cargo clippy --workspace --locked -- -D warnings` — pass
  - `cargo test --workspace --locked` — pass (55 focused scheduler + 5 focused schedule TUI; full workspace green)
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

## Maintainer Re-review Findings

The remediation commits `5f13637` and `7b5a0ab` fix the stale first-tick metadata behavior, hide
message content from the user-visible list, cap displayed rows at 20, add an already-fired result,
and add an Agent/session list/cancel happy path. The following blocking evidence remains:

1. **Cancellation permission/lifecycle proof is still incomplete.**
   `fixture_provider_list_cancel_full_lifecycle` observes a listed task and a `cancelled` result,
   but it does not advance past the deadline or inspect the session queue to prove no later fire.
   It also contains no Deny flow and no positive list-before/list-after assertion that a denied
   cancellation leaves the same task unchanged. The CLI Deny test still targets an unspawned
   scheduler and a nonexistent `sched_1`, so it cannot establish unchanged task state.

2. **Narrow-terminal proof still bypasses the product rendering path.** The new tests create a
   ratatui `Buffer`, but pass hard-coded strings directly to `Paragraph`; they never construct a
   `ToolResultDisplay` or call `build_tool_result_scrollback_lines`. Assertions such as
   `cancelled || sched_1` and `not found || sched_99` permit one required semantic to disappear.
   Render the actual list/cancel tool-result scrollback at 40/60/80/120 columns and require task
   ID, state, and actionable outcome together for every applicable result.

3. **Bounded/recurring regressions are not discriminating enough.** The privacy test creates one
   task, so it does not prove the 20-row cap or omitted-count marker. The recurring timing test
   checks only the first tick although the prior review explicitly required multiple ticks. Add
   multi-task and multi-tick paused-time coverage that would fail if either behavior regresses.

4. **Owner evidence is stale.** Verification Evidence and Retrospective still claim a raw
   60-character preview, four old TUI tests, and no need for new rendering proof; they do not
   identify `5f13637`/`7b5a0ab` or the current test set. Reconcile the owner record after the test
   gaps above are closed.

Re-run focused Agent/CLI/TUI tests and the full locked validation ladder after correction. Remove
the unused `tc` fixture closure so workspace tests and release preflight are warning-free. Do not
activate I127 before I126 reaches Complete.

## Maintainer Re-review Resolution

All four re-review findings are resolved by the closure changes recorded above. The unused fixture
closure was removed; focused scheduler and TUI tests, the full locked workspace ladder, release
preflight, governance validation, scale assessment, and diff checks are green. I126 is promoted to
Complete; I127 is unblocked but remains Planned until its own activation inventory and Gate 0.

## Retrospective

- All 4 stories delivered in one session. The 4-tool Vec factory pattern
  (`create_scheduler_tools`) cleanly handled the new list/cancel tools
  with zero composition root changes — the builders iterate the Vec.
- The `#[allow(dead_code)]` on `ScheduleCommand` can now be removed since
  all variants (RegisterOneShot, RegisterRecurring, Cancel, List, Shutdown)
  are constructed by production tools or the actor.
- Narrow-terminal proof uses the existing scrollback formatter, then renders the resulting text
  to ratatui Buffers; direct constant-string rendering is not sufficient evidence.
