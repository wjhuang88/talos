# Iteration I124: One-Shot Scheduled Follow-Up

> Document status: Review
> Published plan date: 2026-07-13
> Planned objective: deliver one safe, session-scoped delayed follow-up through the normal queue
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a user can ask Talos to follow up after a bounded delay, and it fires exactly once

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SF100 | SCHED-001 | Ready | permission matrix | internal command/event contract and labeled source |
| SF101 | SCHED-001 | Ready | SF100 | cancellation-aware one-shot actor |
| SF102 | SCHED-001 | Ready | SF101 | Execute/Ask `delay` tool wired in supported modes |
| SF103 | SCHED-001 | Ready | SF102 | deterministic and fixture-provider runtime proof |

### Scope

- bounded seconds-based one-shot scheduling; session/process lifetime only;
- existing queue, cancellation, registry, permission, and status mechanisms;
- Execute/Ask registration and fresh permission decisions for later tool calls.

### Non-Goals

- recurrence, list/cancel surface, persistence, cron/calendar time, direct tool execution;
- permission defaults, public APIs, session encoding, dependencies, release work.

### Acceptance

- Given a valid delayed request, when it is approved and time advances, then one visibly labeled
  follow-up enters the normal session flow exactly once.
- Given Deny, invalid duration, shutdown, or a closed queue, then no follow-up fires and Talos does
  not panic or retry forever.
- Given the follow-up asks for another tool, then that tool receives its normal independent
  permission decision.

### Planned Validation

- focused agent/tool/CLI tests with paused Tokio time;
- fixture-provider conversation test through the real engine;
- repository standard validation ladder from the program plan.

### Documentation To Update

- `README.md` scheduling section and SCHED-001 owner record;
- iteration index, product backlog summary, and Board.

### Risks And Rollback

- Risk: injection looks like fresh user authority or survives shutdown.
- Rollback: keep the tool unregistered; actor remains unreachable until all permission/lifecycle
  tests pass.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-13 | Planning | Published as Planned; no implementation or activation claimed. |
| 2026-07-13 | Activation | Gate 0 passed (rustc 1.97.0, governance 0 warnings, release preflight passed). Security note recorded at `docs/reference/I124-PRE-ACTIVATION-SECURITY-NOTE-2026-07-13.md` proving Execute/Ask for mutation tools, Read for list, Deny precedence, and fire-time re-evaluation — no `talos-permission` change needed. I124 activated; SF100-SF103 ready for implementation. |
| 2026-07-13 | SF100 | `feat(agent): add scheduler command/event contract and source labels (#SF100)` — crate-private types in `talos-agent/src/scheduler.rs`: ScheduleCommand, ScheduledTaskInfo, ScheduleKind, SchedulerHandle, source labels, duration bounds, task ID generator. 10 unit tests. |
| 2026-07-13 | SF101 | `feat(agent): implement cancellation-aware scheduler actor (#SF101)` — SchedulerActor with tokio::select! loop, RegisterOneShot/Cancel/List/Shutdown handlers, fired-task cleanup via unbounded channel, CancellationToken support. 9 actor behavior tests (paused time). |
| 2026-07-13 | SF102 | `feat(agent): expose delay tool and wire scheduler into all roots (#SF102)` — DelayTool (ToolNature::Execute), create_scheduler()/PendingSchedulerActor two-phase API, wired into all 9 composition roots (print, inline, TUI, RPC, interactive, session new/resume/fork, model switch). MCP mode excluded. |
| 2026-07-13 | SF103 | `test(agent): prove one-shot firing, permission isolation, and edge cases (#SF103)` — 10 DelayTool tests: nature, valid input, all rejection paths, unavailable scheduler, end-to-end fire+inject proof, one-shot-once invariant, permission isolation proof. |
| 2026-07-13 | Closeout | All 4 stories delivered. Validation ladder passes: fmt, clippy (-D warnings), test --workspace, release preflight, governance 0 warnings, diff --check. |

## Verification Evidence

- **SF100**: 10 unit tests for types, validation, labeling, ID generation. Commit `4a25747`.
- **SF101**: 9 actor behavior tests with paused Tokio time covering fire, cancel, shutdown, cancellation token, invalid duration, list, fired cleanup, unknown cancel, closed-queue degradation. Commit `c25906c`.
- **SF102**: DelayTool exposed as Execute/Ask. Wired into all 9 applicable composition roots. `cargo check --workspace --locked` passes. Commit `8b5b350`.
- **SF103**: 10 DelayTool tests including end-to-end fire+inject proof and permission isolation proof. 29 total scheduler tests pass. Commit `eb30553`.
- **Permission regression**: `delay_tool_nature_is_execute` and `delay_tool_end_to_end_permission_is_fresh_per_call` prove the delay tool is Execute/Ask and that injected messages carry no permission grant.
- **Closed-queue/cancel/shutdown evidence**: `actor_closed_session_queue_no_panic`, `actor_cancelled_one_shot_does_not_fire`, `actor_shutdown_aborts_all_tasks`, `actor_cancel_token_aborts_all_tasks` prove safe degradation.
- **Iteration validation ladder** (2026-07-13):
  - `cargo fmt --all -- --check` — pass
  - `cargo clippy --workspace --locked -- -D warnings` — pass
  - `cargo test --workspace --locked` — pass
  - `./scripts/release_preflight.sh` — pass
  - `scripts/validate_project_governance.sh .` — 0 warnings
  - `git diff --check` — clean

## Variance And Residuals

- Recurrence and operator controls are owned by I125-I126.

## Retrospective

- All 4 stories delivered in one session. The two-phase composition pattern
  (`create_scheduler` + `PendingSchedulerActor`) solved the chicken-and-egg
  problem of needing `sq_tx` (from session creation) before building tools.
- Dead-code warnings on `Cancel`/`List`/`Shutdown` variants are expected —
  these are I125/I126 infrastructure and are consumed in tests.
- Model switch (`model_lifecycle.rs`) recreates the scheduler on rebuild.
  Task persistence across model switches is deferred to I127.
