# Frontline Execution Package: Session-Scoped Scheduled Follow-Ups

**Status**: Ready for assignment; no iteration activated
**Program plan**: `docs/tasks/2026-07-13-four-month-scheduled-followups-plan.md`
**Ordered iterations**: I124, I125, I126, I127
**Checkpoint owner**: assigned developer

## Start Gate

1. Read `AGENTS.md`, this file, the program plan, `docs/BOARD.md`, SCHED-001, I124, and
   `docs/reference/AUTONOMY-PERMISSION-MATRIX-2026-07-04.md`.
2. Confirm the worktree is clean and based on updated `main`. Do not stash, reset, or overwrite
   changes belonging to someone else.
3. Run the repository-pinned toolchain and baseline gates:

```bash
git status -sb
rustc --version
cargo metadata --locked --no-deps --format-version 1
scripts/validate_project_governance.sh .
./scripts/release_preflight.sh
```

4. Inspect current composition roots before editing. Record every place built-in tools are
   registered; missing one is a blocking product defect, not deferred cleanup.
5. Produce a short security note proving `delay`/`schedule`/`cancel` resolve as Execute/Ask,
   `list_scheduled_tasks` is Read, Deny wins, and fire-time tool calls are evaluated normally.
6. Only then mark I124 Active. Do not activate I125-I127 early.

## Implementation Map

| Concern | Expected owner | Constraint |
|---|---|---|
| actor and task metadata | `talos-agent` | one owner; cancellation-aware; no panic on channel close |
| tool schemas/adapters | `talos-tools` | Execute for mutation, Read for list; bounded input/output |
| internal messages | existing core/conversation session types where possible | avoid public API expansion; escalate if unavoidable |
| composition | `talos-cli` registries/mode roots | identical availability in supported interactive modes |
| rendering | `talos-tui` tool display | display-only; no permission or persistence logic |
| prompt/docs | embedded prompt assets and README | describe session-only behavior and permission boundary |

Do not create a second event bus, scheduler registry, or mutable state copy. Reuse the existing
ordered session flow and tool registry.

## Required Test Matrix

- one-shot: fires once; cancelled before fire; shutdown before fire; invalid durations;
- recurring: first fire timing; delayed missed ticks; no burst; cancellation race;
- actor: command channel closes; session queue closes/fills; task completion removes metadata;
- permission: Ask by default for every mutation, explicit Deny, list remains read-only, later tool
  call gets a fresh decision;
- surface: all intended registries expose the same schemas; list/cancel results are bounded;
- runtime: fixture provider registers/fires/lists/cancels through the real conversation path;
- TUI: 40/60/80/120-column semantic buffer assertions for schedule output;
- lifecycle: no scheduled fire after shutdown and no leaked Tokio tasks in the test harness.

Prefer `tokio::time::pause/advance` or `#[tokio::test(start_paused = true)]`. Do not use long
wall-clock sleeps or timing tolerances that hide races.

## Work And Commit Order

Use one SF story per logical commit:

```text
feat(agent): add one-shot scheduled follow-up actor (#SF101) [model:<model-name>]
feat(tools): expose ask-gated delay tool (#SF102) [model:<model-name>]
```

Before each commit, stage only that story, run its focused tests, review `git diff --cached`, scan
for secrets, and run `git diff --check`. At iteration close run the full validation ladder from the
program plan and record actual counts/evidence in the active iteration.

## Checkpoint Template

Append a row after every story, failure, or handoff:

| Time | Story | Branch/commit | State | Validation | Changed files | Blocker/retry | Next exact action |
|---|---|---|---|---|---|---|---|

Allowed states: Not Started, In Progress, Review, Complete, Blocked. Retry an unchanged failed
command at most twice. Record the first actionable error and safe fallback.

## Recovery Procedure

After interruption, read the latest checkpoint, current iteration, its diff, and `git status -sb`.
Run the last focused test before editing. If the checkpoint and code disagree, code plus test
evidence wins temporarily; reconcile the owner doc before continuing. Never infer completion from
checkboxes alone.

## Authority Boundary

The assignee may implement the active iteration, tests, fixtures, and affected docs. Stop before
permission-policy changes, persistence, direct scheduled tool calls, public API/session format
changes, dependencies, remote surfaces, destructive Git operations, push/PR, or release actions.

## Checkpoints

| Time | Story | Branch/commit | State | Validation | Changed files | Blocker/retry | Next exact action |
|---|---|---|---|---|---|---|---|
| 2026-07-13 | Planning handoff | `main` | Ready | planning inventory completed; implementation not claimed | plan/package/I124-I127/governance docs | I124 security note and Gate 0 still required | Assignee reruns Start Gate and activates I124 only. |
| 2026-07-13 | Gate 0 | `main` `a3f17ad` | Pass | git clean (ahead 1 = planning commit); rustc 1.97.0 == pinned; cargo metadata coherent; governance 0 warnings; release preflight passed | none | none | Proceed to security note and I124 activation. |
| 2026-07-13 | Security note | `main` | Pass | I124 pre-activation security note recorded at `docs/reference/I124-PRE-ACTIVATION-SECURITY-NOTE-2026-07-13.md`; all six claims proven with code evidence; no talos-permission change needed | `docs/reference/I124-PRE-ACTIVATION-SECURITY-NOTE-2026-07-13.md` | none | Activate I124 and begin SF100. |
| 2026-07-13 | SF100 | `main` `4a25747` | Complete | 10 unit tests pass; scheduler types defined | `crates/talos-agent/src/scheduler.rs`, `crates/talos-agent/src/lib.rs` | none | Implement SF101 actor. |
| 2026-07-13 | SF101 | `main` `c25906c` | Complete | 9 actor tests pass (paused time); 234 agent tests pass | `crates/talos-agent/src/scheduler.rs`, `crates/talos-agent/Cargo.toml` | none | Expose SF102 delay tool. |
| 2026-07-13 | SF102 | `main` `8b5b350` | Complete | cargo check workspace passes; CLI tests pass; 9 composition roots wired | `crates/talos-agent/src/scheduler.rs`, `crates/talos-cli/src/{registry,mode_print,mode_inline,mode_runners,mode_interactive,session_handlers,model_lifecycle}.rs` | none | Prove SF103 with fixture-provider test. |
| 2026-07-13 | SF103 | `main` `eb30553` | Complete | 29 scheduler tests pass; end-to-end fire+inject+permission proof | `crates/talos-agent/src/scheduler.rs` | none | I124 closeout: validation ladder + doc sync. |
| 2026-07-13 | I124 closeout | `main` | Review | fmt/clippy(-D warnings)/test/release preflight/governance/diff all pass | `docs/iterations/I124-*.md`, `docs/BOARD.md`, `docs/iterations/README.md`, `docs/tasks/2026-07-13-scheduled-followups-execution-package.md`, `crates/talos-agent/src/scheduler.rs` (dead-code allow fixes) | none | I124 Review complete; I125 blocked until maintainer promotes to Complete. |
| 2026-07-14 | I124 maintainer review | `main` `80f5841` | Review | commit history, fmt, clippy, workspace tests, release preflight, and governance all pass; acceptance/security review fails | review records only | raw `DelayTool` bypasses approval wrappers in all 9 roots; no fixture-provider/real-session proof; unapproved public scheduler API; three I127 limitations not enumerated | Fix the four review findings, rerun the full ladder, and request I124 re-review. Do not activate I125. |
| 2026-07-14 | I124 maintainer re-review | `main` `68c24cf` | Review | 30 scheduler tests, 2 focused CLI permission tests, fmt, clippy, workspace tests, release preflight, governance, and diff check pass | review records only | fresh follow-up decision not distinguished; two public exports remain; new dev-dependency lacks approval; queue wait described as time-bounded without timeout; one ignored doctest lacks tracking issue | Correct the five findings, rerun the ladder, and request another I124 re-review. Do not activate I125. |
| 2026-07-14 | I124 maintainer third review | `main` `7fe1d17` | Review | focused scheduler/CLI permission tests, fmt, clippy, workspace tests, release preflight, governance, and diff check pass | review records only | distinct-Deny test can pass without the scheduled turn occurring; ADR-041 does not complete baseline change control for the two public exports | Add positive proof that the second scheduled turn ran; complete and index an accepted ADR/owner-doc change decision or remove the exports; then request another I124 review. Do not activate I125. |
| 2026-07-14 | I124 closure | `main` working tree after `7fe1d17` | Complete | positive scheduled-turn + fresh-Deny focused test, fmt, workspace Clippy, workspace tests, release preflight, governance, and diff check pass | scheduler test, ADR-041/index, I124/security/SCHED/index/Board/checkpoint docs | none | I124 Complete. I125 is Planned and unblocked; run its activation gate before starting it. |
