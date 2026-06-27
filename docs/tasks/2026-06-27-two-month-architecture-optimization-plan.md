# 2026-06-27 Two-Month Architecture Optimization Long Task

**Status**: In Progress
**Owner area**: Architecture health and production-module decomposition
**Source**: User request to continue architecture optimization with an approximately two-month
execution plan and implement it
**Start date**: 2026-06-27
**Target horizon**: About 8 weeks

## Outcome

Reduce architecture corrosion across production modules through ordered, behavior-preserving
decomposition slices. The task is complete when the currently known oversized production roots and
material duplicate logic are either eliminated, decomposed below the review-risk threshold, or
owned by explicit residual stories with acceptance, validation gates, and a deferral reason.

## In Scope

- Follow-up decomposition for CLI and TUI residual roots:
  - `crates/talos-cli/src/mode_runners.rs`
  - `crates/talos-tui/src/app.rs`
- Agent root and supporting modules:
  - `crates/talos-agent/src/lib.rs`
  - `crates/talos-agent/src/caching.rs`
  - `crates/talos-agent/src/token.rs`
- Conversation, provider, exploration, tools, and storage production roots when they have clear
  behavior-preserving boundaries:
  - `crates/talos-conversation/src/engine.rs`
  - `crates/talos-provider/src/openai.rs`
  - `crates/talos-exploration/src/lib.rs`
  - `crates/talos-tools/src/git.rs`
  - `crates/talos-session/src/sqlite.rs`
- Test-suite partitioning only when the test file itself blocks review or compile feedback.
- Repeated logic that creates maintenance risk, including duplicated state transitions,
  builder/setter boilerplate, formatting, parsing, permission setup, request assembly, or error
  mapping that should be centralized in a local helper or shared module.
- Backlog, iteration, Board, README, and task synchronization for each completed slice.

## Out of Scope Without Separate Approval

- Permission semantics, sandbox escape boundaries, or process-hardening behavior.
- Provider protocol changes, request/response schema changes, or model behavior changes.
- New runtime dependencies.
- Network access, dependency installation, destructive cleanup, migration, commit, push, tag, or
  release.
- Broad rewrites that combine decomposition with product behavior changes.

## Two-Month Execution Plan

| Week | Focus | Candidate Owner Stories | Completion Gate |
|---|---|---|---|
| 1 | Stabilize current architecture baseline and continue CLI/TUI residuals. | ARCH-022, ARCH-023 | Current dirty architecture changes remain validated; one more CLI or TUI residual slice closes. |
| 2 | Finish the selected CLI/TUI residual path or record explicit residuals. | ARCH-022/023 follow-up | Targeted crate tests, workspace gates, governance validation. |
| 3 | Agent root decomposition. | New ARCH story | `talos-agent/src/lib.rs` loses one clear responsibility without API or prompt behavior changes. |
| 4 | Conversation engine decomposition. | New ARCH story | `talos-conversation/src/engine.rs` splits registry/command handling/render-output helpers without user-visible command changes. |
| 5 | Provider adapter decomposition. | New ARCH story | `talos-provider/src/openai.rs` splits request/stream/error helpers without protocol changes. |
| 6 | Exploration/tools production roots. | New ARCH stories | One storage/ingestion/tool root is reduced with targeted tests and workspace gates. |
| 7 | Secondary production roots and test-suite partitioning. | New ARCH stories | Remaining large production roots have owners; test-only splits happen only where useful. |
| 8 | Final audit, residual closure, and roadmap sync. | Long-task closure | Large-file audit, dependency audit, owner docs synchronized, residual stories explicit. |

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| M0 | Baseline audit and plan creation | This two-month plan, current line-count audit, and residual target map. | Prior architecture debt burn-down complete | Plan recorded and Board/backlog synchronized. | Keep as planning-only if current dirty tree is inconsistent. | Complete |
| M1 | CLI residual boundary map | Select the next ARCH-022 slice from `mode_runners.rs`. | M0 | Owner story/iteration names exact flow and validation. | Defer CLI if boundaries imply behavior churn. | Complete |
| M2 | CLI residual implementation | Extract one remaining CLI mode-runner flow or helper. | M1 | `cargo test -p talos-cli --quiet`, workspace gates, governance, diff check. | Stop after one safe helper. | Complete |
| M3 | TUI residual boundary map | Select the next ARCH-023 slice from `app.rs`. | M0 | Owner story/iteration names frame/cursor/input risk. | Defer if visual behavior is too entangled. | Complete |
| M4 | TUI residual implementation | Extract one low-risk TUI app helper group. | M3 | `cargo test -p talos-tui --quiet`, workspace gates, governance, diff check. | Stop before cursor/frame behavior changes if not provable. | Complete |
| M5 | Agent root map | Map `talos-agent/src/lib.rs` into config, prompt, turn, tools, memory, and cache responsibilities; identify duplicated setup/setter logic. | M2 or M4 | Owner story/iteration created. | Keep as residual only if root and duplication are both stable enough. | Complete |
| M6 | Agent root implementation | Extract one low-risk agent root responsibility and centralize any repeated local logic in that slice. | M5 | `cargo test -p talos-agent --quiet`, workspace gates. | Stop after pure helper extraction. | Complete |
| M7 | Conversation engine map/implementation | Split one command/registry/output helper group. | M0 | `cargo test -p talos-conversation --quiet`, workspace gates. | Plan-only if UI behavior risk is high. | Complete |
| M8 | Provider adapter map/implementation | Split one OpenAI adapter helper group without request semantics changes. | M0 | Provider targeted tests and workspace gates. | Map-only if protocol behavior cannot be frozen. | Complete |
| M9 | Exploration/tools/storage map/implementation | Pick one clear production root from exploration/tools/session storage. | M0 | Targeted crate tests and workspace gates. | Register residual owner. | Planned |
| M10 | Secondary audit and residual registration | Re-run large-file audit and register remaining owners. | M2-M9 | No unowned production root above threshold except explicit exclusions. | Mark Partial with residuals. | Planned |
| M11 | Final closure | Close the two-month task or record continuation. | M10 | All owner docs synchronized and validation evidence recorded. | Mark Partial with exact blockers. | Planned |

## Initial Baseline Audit

Largest current Rust files after the prior architecture debt burn-down:

- Test-heavy or test-only:
  - `crates/talos-agent/src/tests.rs` — 1861 lines.
  - `crates/talos-conversation/src/engine_tests.rs` — 1443 lines.
  - `crates/talos-session/src/tests.rs` — 1224 lines.
  - `crates/talos-config/src/tests.rs` — 1065 lines.
  - `crates/talos-memory/src/tests.rs` — 1053 lines.
- Production roots to consider:
  - `crates/talos-cli/src/mode_runners.rs` — 1778 lines.
  - `crates/talos-tui/src/app.rs` — 1118 lines.
  - `crates/talos-exploration/src/lib.rs` — 1070 lines.
  - `crates/talos-provider/src/openai.rs` — 1001 lines.
  - `crates/talos-session/src/sqlite.rs` — 983 lines.
  - `crates/talos-conversation/src/engine.rs` — 960 lines.
  - `crates/talos-agent/src/lib.rs` — 914 lines.
  - `crates/talos-tools/src/git.rs` — 868 lines.
- Security-sensitive exclusions for implementation without separate review:
  - `crates/talos-permission/src/lib.rs` — 1370 lines.
  - `crates/talos-sandbox/src/lib.rs` and hardening modules.

## Validation Requirements

Every implementation slice must record:

- Targeted crate tests.
- `cargo fmt --all -- --check`.
- `cargo check --workspace`.
- `cargo clippy --workspace -- -D warnings`.
- `cargo test --workspace --quiet`.
- `scripts/validate_project_governance.sh .`.
- `git diff --check`.
- Before/after line counts for the touched root module.
- Duplicate-logic disposition: either extracted, intentionally left local with a reason, or
  registered as a residual owner.

## Checkpoint and Recovery Rules

- Append a checkpoint after every completed task item.
- If interrupted, resume from the first task item whose status is not Complete.
- Do not mix product behavior changes into architecture cleanup.
- Do not commit, push, tag, release, install dependencies, or access the network under this task
  unless separately authorized.

## Checkpoints

| Date | Completed task items | Current state and artifacts | Commands/checks and actual results | Open risks or deviations | Next task item | Recovery or resume instruction |
|---|---|---|---|---|---|---|
| 2026-06-27 | M0 started. | Two-month architecture optimization plan created from the post-burn-down large-file audit. Current dirty tree already contains completed architecture, skill, README, and governance work from this context; continue without reverting unrelated work. | `find crates ... | xargs wc -l | sort -nr | head -60`, `git status --short --branch`, and Board/backlog inspection completed. | Commit/push requested earlier was superseded by the user's latest request to continue architecture optimization. | M1 | Resume by selecting the next ARCH-022 CLI residual slice from `crates/talos-cli/src/mode_runners.rs`. |
| 2026-06-27 | M0, M1, M2. | ARCH-024/I069 extracted CLI inline mode into `crates/talos-cli/src/mode_inline.rs`; `mode_runners.rs` now re-exports inline mode and dropped from 1778 to 1500 lines. | `cargo test -p talos-cli --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Remaining CLI residual stays under ARCH-022; no behavior change, dependency, release, commit, push, network, or destructive action. | M3 | Resume by mapping the next ARCH-023 TUI residual slice from `crates/talos-tui/src/app.rs`, with frame/cursor behavior treated as visual-risk sensitive. |
| 2026-06-27 | M3, M4. | ARCH-025/I070 extracted TUI exit-summary formatting into `crates/talos-tui/src/app_summary.rs`; `app.rs` dropped from 1118 to 1005 lines. | `cargo test -p talos-tui --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Frame rendering, cursor placement, viewport sizing, input handling, approval flow, and scrollback flushing were intentionally untouched. | M5 | Resume by mapping `crates/talos-agent/src/lib.rs` and deciding whether one low-risk root responsibility should move under a child module. |
| 2026-06-27 | M5, M6. | ARCH-026/I071 extracted Agent construction/configuration into `crates/talos-agent/src/configuration.rs`; `lib.rs` dropped from 914 to 655 lines and repeated prompt-builder mutation was centralized. | `cargo test -p talos-agent --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Turn-loop, tool execution, provider request, permission, sandbox, and hook behavior were intentionally untouched. | M7 | Resume by mapping `crates/talos-conversation/src/engine.rs` for command/registry/output helper extraction and duplicate-logic cleanup. |
| 2026-06-27 | M7. | ARCH-027/I072 extracted conversation command registry metadata/completion into `crates/talos-conversation/src/command_registry.rs`; `engine.rs` dropped from 960 to 739 lines. | `cargo test -p talos-conversation --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Slash command dispatch, TUI menu rendering, CLI behavior, aliases, and availability rules were intentionally untouched. | M8 | Resume by mapping `crates/talos-provider/src/openai.rs` for request/stream/error helper extraction and duplicate-logic cleanup. |
| 2026-06-28 | M8. | ARCH-028/I073 extracted OpenAI request DTOs/body assembly/redaction into `crates/talos-provider/src/openai_request.rs`; `openai.rs` dropped from 1001 to 848 lines. | `cargo test -p talos-provider --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | HTTP send/retry, endpoint URL behavior, SSE parsing, usage extraction, text-tool fallback, Anthropic provider, and provider protocol fields were intentionally untouched. | M9 | Resume by selecting one clear production root from exploration/tools/session storage for behavior-preserving extraction. |
