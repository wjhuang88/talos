# 2026-07-13 Four-Month Scheduled Follow-Ups Plan

**Status**: Program plan; ready for assignment; no iteration activated
**Timebox**: approximately 16 weeks
**Execution owner**: one frontline developer, one active iteration at a time
**Program owner**: maintainer for activation, security review, and release decisions

## Objective

Let a user ask Talos to perform a session-scoped follow-up later, repeat it at a bounded interval,
inspect or cancel it, and trust that shutdown/backpressure cannot turn scheduling into a permission
bypass. The result is deliberately an in-process session capability, not a durable cron service.

## Work Hierarchy

```text
Long task: 2026-07-13-scheduled-followups-execution-package.md
  ├─ I124 One-Shot Scheduled Follow-Up ── SF100-SF103
  ├─ I125 Recurring Follow-Up Control ─── SF110-SF113
  ├─ I126 Schedule Inspection And TUI ─── SF120-SF123
  └─ I127 Scheduler Reliability Closeout  SF130-SF133
```

I124-I127 are iterations. SF100-SF133 are stories inside those iterations. Only one iteration may
be Active. This program plan and its execution package are not iterations.

## Why This Is A New Baseline

I028 proposed the same product area but classified all scheduling tools as read-only/auto-allow.
That is incompatible with the current permission architecture: creating, repeating, or cancelling
future agent work mutates session execution. I028 is therefore preserved as a historical Planned
baseline and superseded before implementation. I124-I127 use existing permission natures:

- `delay`, `schedule`, and `cancel_scheduled_task`: `ToolNature::Execute`, default Ask;
- `list_scheduled_tasks`: `ToolNature::Read`;
- Deny always wins; approval of registration never approves tools called by the later turn;
- no change to `talos-permission`, default rules, or reusable approval semantics is authorized.

This is a changed acceptance target, so it uses new iteration IDs rather than rewriting I028.

## Pre-Activation Inventory And Disposition

No new iteration is Active at publication. The following non-terminal records were checked before
selecting I124:

| Item | Recorded state | Disposition |
|---|---|---|
| I018 | Planned header; acceptance delivered by I047 | Defer to the status-audit owner for reconciliation; not selected. |
| I019 | Review; acceptance delivered by I050-I053 | Do not reactivate; status reconciliation remains separate. |
| I020 | Review; S1-S3 delivered, S4 explicitly deferred | Do not reactivate; no vector/graph work selected. |
| I028 | Planned / owner says In Progress, no implementation found | Superseded before implementation by I124-I127 because its auto-allow permission premise is invalid. |
| I048-I055 | Several stale Planned headers; index/evidence says fulfilled or Complete | Do not reactivate; data/memory/exploration delivery remains historical and outside this plan. |
| I056 | Stale Review header; index/release evidence says Complete | Do not reactivate; release closeout remains historical. |
| R0 | Historical Planned header; Board/roadmap says Done | Do not reactivate; remediation gate remains closed. |
| Issue / Doc / Code Status Audit | Review, not an iteration | Retain Review. Gate 0 must correct only scheduling rows needed for activation. |
| ARCH-022 / ARCH-023 | Planned stories, not iterations | Deferred; do not mix architecture cleanup into scheduler delivery. |
| PERM-001 | In Progress high-risk owner | Not selected; this plan cannot implement Guardian/DSL or change permission policy. |
| REL-002 | Planned / NO-GO | Not selected; this work cannot qualify or release v1.0. |

I124 is Planned and ready for Gate 0. I125, I126, and I127 are Planned and sequentially blocked on
their predecessor. The stale historical headers above belong to the existing status-audit Review;
this plan records their disposition but does not rewrite their published execution history.

## Four-Month Delivery Matrix

| Month | Iteration | Stories | Runnable result |
|---|---|---|---|
| 1 | I124 | SF100-SF103 | A model fixture and real configured provider can register a one-shot delayed follow-up; firing creates one labeled turn through the normal session queue. |
| 2 | I125 | SF110-SF113 | A user can request a recurring follow-up with bounded intervals; missed ticks do not burst and registration remains Ask-gated. |
| 3 | I126 | SF120-SF123 | A user can list and cancel session schedules; TUI/tool output shows ID, kind, next timing, and bounded message preview. |
| 4 | I127 | SF130-SF133 | Shutdown, cancellation, queue pressure, error handling, docs, and a clean-HOME trial are reproducible by another operator. |

## Story Map

- SF100: define crate-private scheduler commands/events and explicit source labels without changing
  public semver-bound APIs.
- SF101: implement one-shot actor behavior with `CancellationToken`, checked duration bounds, and
  deterministic IDs.
- SF102: expose `delay` as Execute/Ask and wire it through every applicable composition root.
- SF103: prove one-shot firing with paused-time tests and a fixture-provider runtime test.
- SF110: add recurring interval behavior with minimum/maximum bounds and
  `MissedTickBehavior::Delay`.
- SF111: expose `schedule` as Execute/Ask; a registration approval never grants fire-time tool
  permission.
- SF112: prove no first-tick surprise, no catch-up burst, and no duplicate fire after cancellation.
- SF113: add user documentation and real configured-provider walkthrough evidence without secrets.
- SF120: add read-only list snapshots with stable task IDs and bounded message previews.
- SF121: add Execute/Ask cancellation with unknown/already-finished outcomes that do not panic.
- SF122: render schedule tool calls/results consistently in TUI and transcript-safe output.
- SF123: prove list/cancel behavior through fixture-provider and narrow-terminal tests.
- SF130: harden actor shutdown, channel closure, queue-full/receiver-gone behavior, and task cleanup.
- SF131: add deterministic stress tests using paused Tokio time; no wall-clock sleeps in tests.
- SF132: publish clean-HOME trial and recovery instructions; include unsupported-context behavior.
- SF133: second-operator replay, owner-doc/Board closeout, residual classification, and no release
  action.

## Locked Architecture Decisions

- Reuse Tokio and `CancellationToken`; add no scheduler/cron dependency.
- Scheduler state is owned by one actor and kept only for the current process/session.
- A fire enqueues a typed, visibly labeled scheduled follow-up into the existing session queue. It
  does not invoke a tool directly and does not impersonate a fresh user approval.
- Use checked, bounded seconds. Zero, overflow, and values outside documented bounds fail before
  registration.
- The actor must not hold locks across `.await`; spawned task completion is reported back to the
  actor so metadata cannot leak.
- Queue send failure or shutdown removes the task and emits a bounded error/status event; it never
  retries forever or panics.
- Scheduling content follows the existing session/transcript policy. No new database column,
  session encoding, or cross-restart restoration is allowed.

## Explicit Non-Goals

- Persistent schedules, cron expressions, wall-clock/calendar scheduling, background daemons.
- Direct scheduled tool execution, pre-approved future actions, or permission changes.
- Running after Talos exits, remote scheduling, web/dashboard mutation routes.
- New public API, new native dependency, release/tag/publish/deploy work.

## Monthly Gates

Every iteration must pass focused tests, a fixture-provider runtime scenario, and:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
git diff --check
```

Do not add `--all-targets`. Do not remove `--locked`. I124 additionally requires a security review
of the permission profile and injection source before activation; later iterations re-run those
regressions but may not redesign the policy.

## Stop Conditions

Stop and request maintainer direction if implementation appears to require changing
`talos-permission`, treating a mutating scheduling tool as Read/Internal, persisting schedules,
directly invoking tools at fire time, changing a public API/session format, adding a dependency,
or weakening queue/shutdown assertions. Push, PR, tag, publish, release, and branch cleanup require
separate instructions.
