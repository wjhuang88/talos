# Iteration I125: Recurring Scheduled Follow-Ups

> Document status: Review
> Published plan date: 2026-07-13
> Planned objective: add bounded recurring follow-ups without catch-up bursts or permission reuse
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a user can request a session-scoped recurring follow-up and observe bounded fires

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SF110 | SCHED-001 | Ready | I124 Complete | bounded recurring actor behavior |
| SF111 | SCHED-001 | Ready | SF110 | Execute/Ask `schedule` tool |
| SF112 | SCHED-001 | Ready | SF111 | missed-tick/race/permission regressions |
| SF113 | SCHED-001 | Ready | SF112 | docs and configured-provider walkthrough |

### Scope

- interval-in-seconds recurrence with documented bounds and `MissedTickBehavior::Delay`;
- no immediate surprise tick unless the user-visible contract explicitly says so;
- each fired turn follows the existing queue and permission pipeline.

### Non-Goals

- persistence, cron/calendar syntax, direct tools, background daemon, permission redesign.

### Acceptance

- A valid approved recurrence fires at the documented cadence without catch-up bursts.
- Cancellation/shutdown races produce no duplicate post-cancel turn.
- Approval of recurrence never becomes approval for later write/network/execute tools.

### Planned Validation

- paused-time cadence and missed-tick tests;
- fixture-provider recurring-turn test;
- standard workspace/preflight/governance ladder.

### Documentation To Update

- README scheduling behavior/limits; SCHED-001; index/backlog/Board.

### Risks And Rollback

- Risk: interval first-tick semantics or queue pressure creates unexpected repeated work.
- Rollback: keep `schedule` unregistered while retaining I124 one-shot behavior.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-13 | Planning | Blocked on I124 Complete; no activation claimed. |
| 2026-07-14 | Dependency update | I124 reached Complete. I125 is unblocked and remains Planned; no activation or implementation is claimed. Run the required activation inventory and gates before starting work. |
| 2026-07-14 | Activation | Gate 0 passed (workspace clean, rustc 1.97.0, cargo metadata OK, governance 0 warnings, release preflight passed). No other Active iteration exists (I124 Complete). Security boundary unchanged from I124 — `schedule` tool is `ToolNature::Execute` (default Ask), same permission architecture. I125 activated; SF110-SF113 ready for implementation. |
| 2026-07-14 | SF110 | `feat(agent): add recurring interval behavior with MissedTickBehavior::Delay (#SF110)` — ScheduleKind::Recurring, RegisterRecurring command, handle_register_recurring with interval_at + MissedTickBehavior::Delay. 8 tests: bounds, no immediate first tick, cadence, cancel stops, no burst, invalid interval. |
| 2026-07-14 | SF111 | `feat(agent): expose schedule tool and update factory API (#SF111)` — ScheduleTool (Execute/Ask), factory renamed to create_scheduler_tools returning Vec. Builder signatures changed to Vec. All 10 composition roots + 13 test call sites updated. |
| 2026-07-14 | SF112+SF113 | `feat(i125): schedule permission regression + fixture-provider recurring proof + docs (#SF112,#SF113)` — schedule Deny/Ask regression tests, recurring fixture-provider test through real Agent/session path, README + SCHED-001 updated. |
| 2026-07-14 | Closeout | All 4 stories delivered. Validation ladder passes: fmt, clippy (-D warnings), test workspace, release preflight, governance 0 warnings, diff check. |

## Verification Evidence

- **SF110**: 8 recurring behavior tests (bounds, no immediate tick, cadence ×3 fires, cancel stops, no burst, invalid interval). Commit `4ac0708`.
- **SF111**: ScheduleTool (Execute/Ask) wired into all 10 composition roots. Factory renamed to `create_scheduler_tools`. Commit `9f2f22f`.
- **SF112**: `schedule_denied_by_permission_does_not_execute` and `schedule_ask_in_print_mode_auto_denies` prove Deny and unresolved Ask cannot register recurring tasks. Commit `e2acff1`.
- **SF113**: `fixture_provider_recurring_fires_through_session_pipeline` proves recurring fire reaches provider through full Agent/session path. README updated. Commit `e2acff1`.
- **Permission regression**: schedule approval never becomes approval for later tools — same architecture as I124 (SessionOp::Submit carries only String).
- **Cadence/burst evidence**: `actor_recurring_fires_at_cadence` (3 consecutive interval fires), `actor_recurring_no_catch_up_burst` (bounded fire count after large time advance), `actor_recurring_cancelled_stops_firing` (no post-cancel fire).
- **Iteration validation ladder** (2026-07-14):
  - `cargo fmt --all -- --check` — pass
  - `cargo clippy --workspace --locked -- -D warnings` — pass
  - `cargo test --workspace --locked` — pass (39 scheduler + 189 CLI tests)
  - `./scripts/release_preflight.sh` — pass
  - `scripts/validate_project_governance.sh .` — 0 warnings
  - `git diff --check` — clean

## Variance And Residuals

- List/cancel UX is owned by I126.

## Retrospective

- All 4 stories delivered in one session. The `MissedTickBehavior::Delay`
  + `interval_at(now + interval, interval)` pattern cleanly prevents both
  surprise immediate ticks and catch-up bursts.
- Factory renamed from `create_delay_tool_and_scheduler` to
  `create_scheduler_tools` (returning Vec) to support the growing tool set.
  ADR-041 covers the renamed public API.
- Recurring tasks do not send `fired_tx` notifications — entries persist in
  the HashMap until Cancel/Shutdown. Queue-closed recurring tasks become
  stale entries cleaned up on Shutdown. This is acceptable for I125; I127
  can add prompt lifecycle management.
