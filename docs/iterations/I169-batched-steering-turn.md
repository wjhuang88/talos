# Iteration I169: Batched Steering Turn

> Document status: Active
> Published plan date: 2026-07-30
> Planned objective: deliver GitHub Issue #50 by consolidating all steering inputs queued during an
> active turn into one FIFO follow-up turn after authoritative completion.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos binary and conversation-loop regression prove A/B/C are sent as
> one `A\n\nB\n\nC` follow-up submission rather than three sequential turns.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
| --- | --- | --- | --- | --- |
| `TUI-041` | None | Ready | TUI-026/I145; ADR-039; ADR-049 | One ordered batch is submitted after the current turn completes. |

### Non-Terminal Inventory And Disposition

- I168 remains Paused by maintainer request; I169 does not resume, alter, or supersede its provider
  terminal-integrity objective.
- I164 remains Paused with its superseded baseline preserved.
- I158-I162 remain Blocked under their recorded ADR/dependency/security gates.
- No Review implementation iteration was identified in the current index.
- Issue #50 is an explicit maintainer-requested bounded P1 correction with a different objective
  from every paused or blocked iteration, so it receives the new I169 identifier.

### Branch And Worktree

- Branch: `codex/issue-50-batched-steering-input`
- Worktree: `C:/Users/12261/Documents/talos-worktrees/issue-50`
- Merge target: `main`
- Starting local upstream: `origin/main@395a0e02`
- Candidate implementation inherited for review: `6da9d71c`
- Remote freshness: rechecked successfully on 2026-07-30; `origin/main@b5fcaaf3` is the PR base and
  this branch must be rebased onto it before push/PR.

### Scope

- Establish TUI-041 as the Issue #50 owner.
- Audit and repair the existing implementation candidate.
- Preserve the old public single-item drain API while adding runtime batched drain behavior.
- Add engine and conversation-loop tests proving FIFO grouping and exactly one session submit.
- Update EN/zh-CN user documentation and all applicable governance owners.

### Non-Goals

- No queue timing, ownership, projection, persistence, editing, concurrency, provider, permission,
  sandbox, release, or protocol expansion.
- No tag, publish, release, issue closure, push, or PR without the corresponding validated state
  and user authorization.

### Acceptance

- Given A/B/C are queued during an active turn, when authoritative completion arrives, then the
  runtime submits exactly one `A\n\nB\n\nC` user message and clears the queue snapshot.
- Given tool-use ends a provider response without completing the user turn, when messages are
  queued, then no steering message drains.
- Given an external caller uses the prior public drain method, when the workspace builds, then its
  single-item FIFO behavior remains source-compatible.

### Planned Validation

- `cargo test --locked -p talos-conversation drain_steering_queue`
- `cargo test --locked -p talos-cli conversation_loop_batches_all_queued_steering_into_one_submit`
- `cargo test --locked -p talos-cli conversation_loop_keeps_steering_queued_across_provider_tool_end`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `powershell -NoProfile -File scripts/validate_project_governance.ps1 .`
- `git diff --check`
- `cargo build --locked -p talos-cli`
- `target/debug/talos --mock --print --no-init --no-context "steering batch smoke"`

### Documentation To Update

- `docs/backlog/active/TUI-041-batched-steering-turn.md`
- `docs/backlog/active/TUI-026-queued-input-display.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`
- `.agent-governance/manifest.yaml`
- `README.md`
- `README.zh-CN.md`

### Risks And Rollback

- Risk: removing the prior public method breaks downstream Rust callers. Preserve it and test it.
- Risk: a bridge regression still emits multiple session submits. Test the real channel boundary.
- Risk: concatenation changes user intent. Preserve FIFO text exactly and use only an explicit
  blank-line separator; no rewriting or deduplication.
- Rollback: revert I169 implementation commits; the prior one-message-per-turn behavior remains the
  known baseline.

## Actual Activation And Execution

| Date | Type | Record |
| --- | --- | --- |
| 2026-07-30 | Intake | GitHub Issue #50 was read through authenticated GitHub CLI and mapped to TUI-041. |
| 2026-07-30 | Recovery audit | Existing branch commit `6da9d71c` implements batched drain but predates the required Story/iteration owner and removes a public method. It is retained as a candidate and must pass compatibility repair plus full validation. |
| 2026-07-30 | Activation | I168/I164 remain Paused and I158-I162 remain Blocked. User explicitly requested Issue #50 development with the governance skill, so separate-objective I169 becomes the sole Active implementation slice in an isolated worktree. |
| 2026-07-30 | Implementation | Restored the public single-item drain API, retained the batched runtime drain, and added engine plus real conversation-loop regressions. README and governance owners were synchronized. |
| 2026-07-30 | Validation | Focused tests, formatting, locked workspace check, Clippy, governance validation, scale assessment, diff check, CLI build, and rebuilt-binary mock smoke passed. Full workspace tests remain blocked by unrelated Windows baseline failures recorded below. |
| 2026-07-30 | Remote refresh | `git fetch origin` succeeded; `origin/main` advanced from `395a0e02` to `b5fcaaf3`. Rebase and post-rebase focused validation are required before PR creation. |
| 2026-07-30 | Rebase audit | Refreshed main records I168 as Active after maintainer resumption. That later remote state is preserved; I169 remains isolated to this Issue #50 draft PR and does not alter I168 scope or files. |

## Closure Ledger

- Requested outcome: develop GitHub Issue #50 under project governance.
- Artifacts: TUI-041, I169, compatible engine/bridge implementation, focused tests, user docs, and
  synchronized indexes.
- Existing assets preserved: I145/TUI-026 history, ADR-039/049 boundaries, I168 pause, I164 pause,
  I158-I162 blockers, and unrelated `LOCAL-DEV.md` work in the original worktree.
- Validation: focused tests, full locked workspace gates, governance validator, rebuilt binary
  smoke, and diff review.
- Remote evidence: `origin/main@b5fcaaf3` was fetched successfully before PR preparation.
- Residual destination: this iteration and TUI-041; remote sync/PR remains pending unless later
  authorized and network-accessible.

## Verification Evidence

- PASS — `cargo test --locked -p talos-conversation drain_steering_queue`: 4 passed.
- PASS — `cargo test --locked -p talos-cli conversation_loop_batches_all_queued_steering_into_one_submit`: 1 passed.
- PASS — `cargo test --locked -p talos-cli conversation_loop_keeps_steering_queued_across_provider_tool_end`: 1 passed.
- PASS — `cargo fmt --all -- --check`.
- PASS — `cargo check --workspace --locked`.
- PASS — `cargo clippy --workspace --locked -- -D warnings`.
- BLOCKED — `cargo test --workspace --locked` cannot compile the pre-existing
  `talos-provider` test at `crates/talos-provider/src/image_io.rs:198` and `:200` on Windows because
  it unconditionally references `std::os::unix::fs::symlink`.
- BLOCKED — a diagnostic retry excluding `talos-provider` reached tests and exposed a second
  pre-existing Windows line-ending mismatch in
  `talos-memory::benchmark::tests::benchmark_is_byte_stable_and_matches_checked_in_artifact`.
  These unrelated cross-platform defects are not folded into TUI-041.
- PASS — `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate_project_governance.ps1 .`: 0 warnings.
- PASS — `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/assess_project_scale.ps1 .`:
  high-risk, release-managed, worktree required.
- PASS — `git diff --check`.
- PASS — `cargo build --locked -p talos-cli`.
- PASS — rebuilt `target/debug/talos.exe --mock --print --no-init --no-context "steering batch smoke"` returned the mock-provider response.

## Completion Evidence

- Completion Commit: pending. Do not mark Complete without a commit that includes the compatibility
  repair, tests, docs, and already-recorded validation evidence.

## Variance And Residuals

- The candidate code arrived before governance intake; I169 records rather than conceals that
  variance.
- I169 remains Active rather than Review because the repository-mandated full workspace test gate
  is not green on Windows. The two baseline failures require separately scoped remediation or a
  successful run on a supported environment; no completion claim is made here.

## Retrospective

- Pending.
