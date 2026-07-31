# Iteration I169: Batched Steering Turn

> Document status: Review
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
| 2026-07-30 | Change control | PR #64/#68 review reclassified the initial batch drain as a core asynchronous data-flow change. The prior narrow implementation is invalidated, not completed. The product objective remains Issue #50, but acceptance expands to the review's G0-G13 safety gates. ADR-056 records the selected bridge-accepted/transactional/actor-arbitrated design before replacement implementation. |
| 2026-07-30 | Review disposition | PR #64 is closed only as duplicate/superseded by Draft PR #68. Reviews `4819565218` and `4819746766`, comment `5132132605`, and migrated #68 comment `5132551226` remain binding. |
| 2026-07-30 | Replacement implementation | Added ADR-056 structured submissions, actor-owned FIFO arbitration without implicit preemption, distinct per-item provider/persistence messages, source-aware scheduler input, bridge identity/sequence validation, transactional prepare/reserve/send/start-ack/commit, bounded queue/batch/context checks, and paused Cancel/Error behavior. |
| 2026-07-30 | Deterministic validation | Core/agent/conversation and all 290 CLI binary tests pass, including distinct-message capture, attachment/kind separation, send acknowledgement, cancellation terminal handling, and a 1,000-step fixed-seed queue invariant stress test. |
| 2026-07-30 | Windows baseline audit | Repaired Unix-only image fixture compilation, CRLF artifact comparison, and hard-coded `/tmp` permission fixtures. Full workspace tests now stop at 27 pre-existing `talos-tools` Windows failures caused by unavailable `sh` plus Unix path/permission output assumptions. Process-boundary behavior is outside TUI-041 and remains a Ready-for-Review blocker. |
| 2026-07-31 | Scope correction | Removed all I170 process/fixture changes from PR #68 and moved them to independent Draft PR #78. ADR-056 was returned from Accepted to Proposed/Review because requested changes are not maintainer acceptance. |
| 2026-07-31 | Review correction | Bound lifecycle events to sender generation and session identity, removed uncorrelated start fallback, added bounded identity dedupe and actor aggregate limits, implemented paused user priority, completed cancel/shutdown/preview behavior, centralized mutation gates, and added full provider-request budgeting for initial and continuation calls. Implementation evidence: `1c81e040`. |
| 2026-07-31 | Main integration | Rebased the six I169-only commits onto exact `main@37e369b1`; seven mainline contribution commits are ancestors rather than duplicated PR commits. |

## Closure Ledger

- Requested outcome: develop GitHub Issue #50 under project governance.
- Artifacts: TUI-041, I169, compatible engine/bridge implementation, focused tests, user docs, and
  synchronized indexes.
- Existing assets preserved: I145/TUI-026 history, ADR-039/049 boundaries, main's I158 Active and
  I168 Complete states, I164 pause, I159-I162 blockers, and unrelated work outside this worktree.
- Validation: focused locked tests, workspace check/Clippy, governance validation, diff review, and
  a passing combined #68+#78 locked workspace test.
- Remote evidence: GitHub API and reconstructed signed Git objects verified exact `main@37e369b1`;
  the branch is six commits ahead and zero behind that head.
- Implementation evidence: `1c81e040` contains the PR-review corrections on top of the rebased
  transactional implementation. I170 evidence is owned only by Draft PR #78.
- Remote destination: Draft PR #68; push, issue synchronization, and CI evidence are pending.

## Verification Evidence

- PASS — pre-rebase `cargo check --workspace --locked`.
- PASS — pre-rebase `cargo clippy --workspace --locked -- -D warnings`.
- PASS — pre-rebase `cargo test -p talos-core -p talos-agent -p talos-cli --locked`, including 304
  `talos-agent` tests, 293 CLI binary tests, integration tests, and doctests.
- PASS — deterministic actor tests cover duplicate IDs, lost EQ acknowledgement, aggregate queue
  limits, pending shutdown rejection, paused scheduler/user priority, pre-start context rejection,
  and attachment metadata limits.
- PASS — deterministic bridge tests cover rejection-before-queue-ack, lifecycle correlation,
  cancellation terminal handling, batching, tool-use non-terminal behavior, and attachments.
- PASS — post-rebase `cargo fmt --all -- --check`, `cargo check --workspace --locked`, and
  `cargo clippy --workspace --locked -- -D warnings`.
- PASS — post-rebase `cargo test -p talos-core -p talos-agent -p talos-cli --locked`.
- PASS — governance validator (0 warnings), scale assessment (high-risk/release-managed/worktree),
  and `git diff --check`.
- EXPECTED DEPENDENCY FAILURE — standalone PR #68 `cargo test --workspace --locked` stops while
  compiling the Unix-only `talos-provider` symlink fixture on Windows. Those fixes are intentionally
  excluded from #68 and owned by Draft PR #78.
- PASS — integration head `0383221d` (PR #68 implementation `1c81e040` plus the five I170 code/test
  commits from PR #78) completes `cargo test --workspace --locked` with exit 0.
- PENDING — exact-head CI, rebuilt real-TUI transcript, and maintainer review.

## Completion Evidence

- Completion Commit: pending. Do not mark Complete without a commit that includes the compatibility
  repair, tests, docs, and already-recorded validation evidence.

## Variance And Residuals

- The candidate code arrived before governance intake; I169 records rather than conceals that
  variance.
- I169 is in Review because implementation evidence exists but post-rebase validation, real-TUI
  acceptance, CI, and maintainer review remain pending. Windows process/path remediation is
  independently owned by Draft PR #78, so no completion claim is made here.
- The original baseline described one `A\n\nB\n\nC` string and excluded scheduler, session switch,
  attachment, persistence, and context interactions. Maintainer review proved those exclusions do
  not remove real runtime coupling. They are accepted as a material same-objective correction;
  the immutable original baseline remains above, while ADR-056 and TUI-041 own the expanded gates.

## PR #64/#68 G0-G13 Execution Gate

- G0: ADR-056 is Proposed/Review; TUI-041, TUI-026, ADR-049, and iteration evidence are synchronized.
- G1: bridge-accepted cutoff documented and deterministically tested with both channels ready.
- G2: active session/turn/sequence and actor-owned submission correlation enforced.
- G3: explicit Idle/Submitting/Running/Cancelling/PausedAfterFailure state transitions tested.
- G4-G5: unique staged ownership and prepare/reserve/send/ack/commit rollback proven for SQ
  closed/full/timeout/sender change.
- G6-G8: recoverable items, input kind, multiline text, preview, slash, and per-item attachments.
- G7: live UI, actor messages, successful durable replay, memory/hook/usage turn semantics aligned.
- G9: scheduler and session mutation ordering/isolation deterministic with no implicit preemption.
- G10: item/queue/batch/context hard limits and visible non-truncating overflow behavior.
- G11: content-free structured lifecycle/stale/rollback diagnostics.
- G12: barrier/oneshot/paused-time matrix plus fixed-seed stress invariant test.
- G13: post-rebase locked format/check/Clippy, focused tests, and governance checks pass. Standalone
  Windows workspace portability is isolated in PR #78 and passes in combined validation; exact-head
  CI and rebuilt real-TUI transcript remain required.

## Retrospective

- Pending.
