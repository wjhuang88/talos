# 2026-06-27 Architecture Debt Burn-down Long Task

**Status**: Complete
**Owner area**: Architecture residual cleanup
**Source**: User request to plan a larger ordered task for the remaining technical debt after
ARCH-012 through ARCH-016
**Current baseline**: Memory, config, CLI helper, and TUI scrollback slices are complete through
I063.

## Outcome

Reduce the remaining high-friction oversized production modules into focused, behavior-preserving
modules, with each slice independently validated and recorded. The task should leave no known
architecture debt from the current oversized-module audit unplanned; any item not completed must
have an explicit residual owner and reason.

## In Scope

- `crates/talos-cli/src/mode_runners.rs` deeper split.
- `crates/talos-tui/src/app.rs` event/frame/cursor boundary split.
- `crates/talos-agent/src/compaction.rs` policy/layer/status split.
- `crates/talos-agent/src/prompt.rs` prompt builder/template/cache-boundary split.
- `crates/talos-agent/src/session.rs` session runtime/turn/compaction integration split.
- Required backlog, iteration, task, Board, and README/doc synchronization for completed slices.

## Out of Scope

- Permission semantics or sandbox policy changes.
- Provider protocol behavior changes.
- New runtime dependencies.
- Feature work such as new commands, new memory behavior, new compaction layers, or web UI work.
- Commit, push, tag, release, network spend, destructive cleanup, or migration.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Refresh architecture inventory | Current line-count and responsibility map for remaining target files; decide exact owner story IDs before coding. | None | Inventory recorded in this task; no non-terminal owner contradiction in `docs/BOARD.md` and `docs/iterations/README.md`. | Stop before coding and update only the plan if targets have changed. | Complete |
| T1 | CLI mode runner map | Promote the next CLI owner story and map `mode_runners.rs` flows into print, inline/TUI, session command, and shared runtime responsibilities. | T0 | Story/iteration created with behavior-preserving acceptance and targeted CLI tests named. | Keep as design-only story if runner boundaries are unclear. | Complete |
| T2 | CLI flow split | Extract the clearest `mode_runners.rs` flow modules, likely print/inline/session-command helpers, without changing CLI behavior. | T1 | `mode_runners.rs` reduced materially; `cargo test -p talos-cli`, workspace check/clippy/test, governance, and diff check pass. | Stop after one safe helper slice if flow-level split risks behavior churn. | Complete |
| T3 | TUI app boundary map | Promote a TUI app owner story and map `app.rs` responsibilities: event handling, frame assembly, component stack, cursor/terminal management, and agent-event routing. | T0 | Story/iteration created with TUI targeted tests and visual-risk notes. | Keep as design-only if app responsibilities are too entangled for a single slice. | Complete |
| T4 | TUI app split | Extract one or more low-risk `app.rs` modules, prioritizing pure frame/cursor helpers before event mutation. | T3 | `cargo test -p talos-tui`, workspace check/clippy/test, governance, and diff check pass; no viewport/cursor behavior change. | Stop after pure helper extraction; leave event-loop split to a new story. | Complete |
| T5 | Agent compaction boundary map | Promote an agent compaction owner story and classify compaction types, policy decisions, deterministic layers, LLM-deferred layers, status/reporting, and tests. | T0 | Story/iteration created; hidden-output and prompt-cache constraints recorded. | Do not code if boundaries imply behavior changes or MEM-003/MEM-007 decisions. | Complete |
| T6 | Agent compaction split | Extract behavior-preserving compaction modules, likely `types`, `policy`, `layers`, `status`, and tests. | T5 | Existing compaction tests plus workspace gates pass; no change to compaction trigger semantics or hidden-output behavior. | Stop at type/policy extraction if layer execution has hidden coupling. | Complete |
| T7 | Agent prompt boundary map | Promote a prompt owner story and map prompt responsibilities: system prompt composition, memory section, stable prefix/cache boundary, hidden output, provider-sensitive fields, and tests. | T0, T6 | Story/iteration created; cache-stability and hidden-output constraints named. | Defer if MODEL-003/MEM-007 changes would make the split unstable. | Complete |
| T8 | Agent prompt split | Extract prompt builder/template/cache-boundary helpers without changing prompt text or provider request payloads. | T7 | Prompt snapshot/cache-stability tests and workspace gates pass; no README-visible behavior change unless docs need clarification. | Stop after pure helper extraction if generated prompt diffs appear. | Complete |
| T9 | Agent session boundary map | Promote a session owner story and map turn orchestration, session context, compaction integration, memory prompt injection, cancellation, and persistence touchpoints. | T0, T6, T8 | Story/iteration created; risk notes cover persistence and cancellation. | Keep as plan-only if session split would alter runtime behavior. | Complete |
| T10 | Agent session split | Extract one low-risk session runtime/helper slice, avoiding persistence and permission changes unless separately approved. | T9 | Agent/session tests plus workspace gates pass; no persistence, cancellation, or memory behavior regression. | Stop after read-only helper extraction; leave runtime mutation split to a follow-up. | Complete |
| T11 | Final architecture audit | Re-run line-count and dependency audit; update residual owners or mark the long task complete. | T2, T4, T6, T8, T10 | All required owner docs synchronized; workspace gates, governance validation, and final checkpoint recorded. | Mark Partial with explicit residual stories if any slice is intentionally deferred. | Complete |

## Dependencies and Prerequisites

- ARCH-012 through ARCH-016 remain complete and must not be reopened.
- Each implementation phase must create or select its own owner story/iteration before code edits.
- Security-sensitive crates (`talos-permission`, `talos-sandbox`) are deliberately excluded from
  this long task.
- Agent compaction/prompt/session work must preserve ADR-016 memory boundaries, ADR-023 credential
  masking, hidden-output filtering, and prompt-cache stability.

## Artifacts and State Owners to Update

- `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md` — this long-task owner.
- New `docs/backlog/active/ARCH-017+...` stories as slices are activated.
- New `docs/iterations/I064+...` iteration records as slices are activated.
- Per-slice task records under `docs/tasks/`.
- `docs/backlog/PRODUCT-BACKLOG.md`.
- `docs/BOARD.md` after owner docs.
- `docs/iterations/README.md`.
- README files only when a completed slice changes user-facing behavior or architecture claims.

## Validation and Acceptance Evidence

Every implementation slice must record:

- Targeted crate gate, for example `cargo test -p talos-cli --quiet` or
  `cargo test -p talos-tui --quiet`.
- `cargo fmt --all -- --check`.
- `cargo check --workspace`.
- `cargo clippy --workspace -- -D warnings`.
- `cargo test --workspace --quiet`.
- `scripts/validate_project_governance.sh .`.
- `git diff --check`.
- Before/after line counts for touched oversized modules.

The long task is complete only when all required task items are complete or intentionally deferred
to named residual owners with validation evidence for the completed slices.

## Branch, Worktree and Checkpoint Plan

- Use the current worktree.
- Do not create branches, commits, tags, pushes, or releases unless the user separately requests
  them.
- Append a checkpoint to this task after every task item, before switching target areas, and before
  stopping for any reason.
- If interrupted, resume from the first task item whose status is not Complete.

## Allowed Permissions and External Actions

- Allowed: edit files in this repository, run Cargo checks/tests, run project governance scripts,
  inspect Git status/diff.
- Not allowed without separate explicit approval: network access, dependency installation,
  destructive cleanup, commit, push, tag, release, migration, or edits outside this repository.

## Destructive or Irreversible Operations

None are authorized. The plan is refactor-only and documentation-only. No `git reset`, force-push,
data deletion, migration, or release operation is allowed under this task.

## Time, Cost and Resource Limits

- Prefer small, independently verified slices over one broad edit.
- If a single slice exceeds one coherent refactor boundary, stop after the safe sub-slice and record
  a residual.
- Full workspace tests are required before marking a slice Complete, even if they are slower.
- No external paid services or network-dependent validation.

## Failure, Retry and Fallback Policy

- If targeted tests fail, repair within the slice before continuing.
- If workspace gates fail because of unrelated dirty work, record evidence and stop for user
  direction rather than reverting unrelated changes.
- If a split reveals behavior changes, revert only the agent-authored slice or stop with Partial;
  never mask behavior drift as architecture cleanup.
- If an area proves too coupled for safe extraction, complete the mapping task and create a smaller
  residual owner story.

## Default Decisions for Foreseeable Ambiguity

- Prefer behavior-preserving module extraction over API changes.
- Preserve existing public and crate-local call paths with re-exports when that avoids churn.
- Extract pure helpers before mutation-heavy event loops or persistence paths.
- Avoid changing tests unless imports must follow a moved module.
- Avoid new abstractions unless a moved module has a clear single responsibility.

## Residual-work Destination

Residual architecture work goes into new `ARCH-*` backlog items linked from
`docs/backlog/PRODUCT-BACKLOG.md`, not into free-form notes or this task alone.

## Checkpoints

| Date | Completed task items | Current state and artifacts | Commands/checks and actual results | Open risks or deviations | Next task item | Recovery or resume instruction |
|---|---|---|---|---|---|---|
| 2026-06-27 | Planning record created. | Long-task baseline exists; no implementation item has started. | Pending validation after doc synchronization. | Execution still requires consolidated confirmation before status can become In Progress. | T0 | Resume by confirming execution scope, then run T0 inventory and update this checkpoint. |
| 2026-06-27 | T0 complete. | Inventory confirms remaining ordered targets still match plan: `mode_runners.rs` 1912, `tui/app.rs` 1503, `agent/compaction.rs` 1447, `agent/prompt.rs` 1232, `agent/session.rs` 1150. First implementation owner will be ARCH-017/I064 for CLI print flow extraction. | `wc -l` and `rg` function map completed; no code changed for T0. | Existing worktree is already dirty from prior architecture and skill/runtime changes; continue without reverting unrelated changes. | T1 | Resume by creating ARCH-017/I064 owner docs, then extract the print-mode flow from `mode_runners.rs`. |
| 2026-06-27 | T1 and T2 complete via ARCH-017/I064. | `run_print_mode` moved to `crates/talos-cli/src/mode_print.rs`; `mode_runners.rs` re-exports the entrypoint and is reduced from 1912 to 1778 lines. | Passed: `cargo clippy -p talos-cli -- -D warnings`, `cargo test -p talos-cli --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, `git diff --check`. | CLI still has inline/TUI/session command flow residuals, but the long-task order moves next to TUI app boundary mapping. | T3 | Resume with T3: map `crates/talos-tui/src/app.rs` responsibilities and create the TUI app owner story before code edits. |
| 2026-06-27 | T3 complete; T4 started. | TUI app map identifies the safest first split as `ScrollbackLine` + `StreamRenderState` extraction from `crates/talos-tui/src/app.rs` into `app_stream.rs`, preserving `crate::app::*` re-exports for existing tests. | `rg` function/type map and line-count audit completed. | Event loop, frame assembly, and cursor management remain out of scope for this slice. | T4 | Resume by creating ARCH-018/I065 owner docs, then extract stream rendering state into `app_stream.rs`. |
| 2026-06-27 | T4 complete via ARCH-018/I065. | `ScrollbackLine`, `StreamRenderState`, and `SPINNER_FRAMES` moved to `crates/talos-tui/src/app_stream.rs`; `app.rs` re-exports compatibility names and is reduced from 1503 to 1118 lines. | Passed: `cargo check -p talos-tui`, `cargo clippy -p talos-tui -- -D warnings`, `cargo test -p talos-tui --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, `git diff --check`. | Event-loop/frame/cursor/input residuals remain, but the long-task sequence moves next to agent compaction mapping. | T5 | Resume with T5: map `crates/talos-agent/src/compaction.rs` into type/policy/layer/status/test responsibilities before code edits. |
| 2026-06-27 | T5 and T6 complete via ARCH-019/I066. | `crates/talos-agent/src/compaction.rs` now keeps module documentation and public re-exports only, reduced from 1447 to 41 lines. Focused child modules own constants, policy, public types/status, engine logic, and tests under `crates/talos-agent/src/compaction/`. README files now note the behavior-preserving architecture cleanup. | Passed: `cargo test -p talos-agent --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, `git diff --check`. | Compaction behavior was intentionally unchanged; MEM-003 LLM proof work and MEM-007 active context compression remain separate residuals. | T7 | Resume with T7: map `crates/talos-agent/src/prompt.rs` responsibilities and create the prompt owner story before code edits. |
| 2026-06-27 | T7 and T8 complete via ARCH-020/I067. | `crates/talos-agent/src/prompt.rs` now keeps module documentation and public re-exports only, reduced from 1232 to 64 lines. Focused child modules own embedded prompt assets, public DTO/cache marker types, section metadata, builder behavior, and tests under `crates/talos-agent/src/prompt/`. | Passed: `cargo test -p talos-agent --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`. | Prompt text, section order, cache marker byte ranges, stable/dynamic prefix behavior, hook behavior, and memory section placement were intentionally unchanged. | T9 | Resume with T9: map `crates/talos-agent/src/session.rs` responsibilities and create the session owner story before code edits. |
| 2026-06-27 | T9 and T10 complete via ARCH-021/I068. | `crates/talos-agent/src/session.rs` now keeps `AppServerSession` actor-loop ownership and is reduced from 1150 to 193 lines. `session/turn.rs` owns turn forwarding and `session/tests.rs` owns the existing async test suite. | Passed: `cargo test -p talos-agent --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`. | Session operation matching, history commits, deterministic pre-turn compaction, skill context gating, cancellation behavior, and turn completion events were intentionally unchanged. | T11 | Resume with T11: run final architecture audit, synchronize residual owners, and close or mark the long task partial. |
| 2026-06-27 | T11 complete. | Final audit confirms this task's targeted roots now stand at: `mode_runners.rs` 1778, `app.rs` 1118, `compaction.rs` 41, `prompt.rs` 64, `session.rs` 193, `talos-memory/src/lib.rs` 39, `talos-config/src/lib.rs` 28, and `scrollback.rs` 756. Remaining large CLI/TUI app roots are registered as ARCH-022 and ARCH-023. | Passed final audit commands: `wc -l` target audit, repository large-file scan, `cargo fmt --all -- --check`, `scripts/validate_project_governance.sh .`, and `git diff --check`. Prior implementation gates for T2/T4/T6/T8/T10 all passed workspace check/clippy/test. | `talos-agent/src/tests.rs`, `talos-conversation/src/engine_tests.rs`, and other test-heavy files remain large but are test suites, not current production root-module corrosion. `talos-permission` and `talos-sandbox` remain security-sensitive and outside this task. | Complete | Long task is closed. Resume future architecture work through ARCH-022 or ARCH-023, or through a fresh audit if priorities change. |
