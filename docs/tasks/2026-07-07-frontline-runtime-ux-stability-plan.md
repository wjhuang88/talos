# 2026-07-07 Frontline Runtime And UX Stability Plan

**Status**: Planned
**Created**: 2026-07-07
**Timebox**: 16 weeks / roughly 4 months
**Owner boundary**: frontline implementation package; maintainer or senior agent reviews phase closeouts
**Trigger**: maintainer requested a new four-month long-running task package for frontline development.

## Outcome

Deliver a low-ambiguity implementation package that improves Talos day-to-day stability and
observability without asking the receiving developer to make architecture, security, permission, or
release decisions. The work is intentionally bounded to runtime stuck-state recovery, TUI status and
preview feedback, todo mutation reliability, write/diff result visibility, provider usage accounting,
and remote MCP documentation/test hardening.

This is a delegation contract. It does not authorize permission-default changes, sandbox/process
changes, release tags, crate publishing, remote deployment, broad architecture rewrites, or new
provider protocol families.

## In Scope

- `RUNTIME-002` follow-through for visible stuck-processing recovery and deterministic tests.
- `TUI-028` preview/status feedback reliability items that are display-only or state-reset-only.
- `TODO-002` mutation reliability where behavior is already specified by the owner doc.
- `TOOL-015` and `TOOL-018` bounded write/edit/diff visibility improvements.
- `PROVIDER-001`, `TUI-017`, and `TUI-018` usage/context display fixes with focused tests.
- Remote MCP user documentation and local fixture tests for the already-implemented `sse` and
  `streamable_http` transports.
- Monthly checkpoints with exact commands, test results, commits, residuals, and recovery notes.

## Out Of Scope

- Permission reuse, approval defaults, bash policy, workspace trust sandbox, `PERM-004`, or any
  approval/sandbox/process-hardening implementation.
- Binary session-log migration or changing the default session storage format.
- Plugin runtime expansion, executable hooks, remote plugin install, marketplace behavior, or
  write-capable plugin tools.
- Native Git replacement, `gix` upgrades, Git push/pull/fetch implementation, or host-Git fallback
  removal.
- New model provider families, OAuth/device-flow credential systems, paid API calls, or
  network-dependent tests.
- Streamable HTTP resumable sessions, long-lived MCP server-to-client notification channels, or MCP
  protocol redesign.
- Release tags, GitHub Releases, crate publish, installer signing, website deployment, or domain/DNS
  changes.
- Broad refactors outside the owner files named by each task.

## Required Reads

The receiving developer must read these files before making changes:

1. `AGENTS.md`
2. `docs/sop/LONG-RUNNING-TASK.md`
3. `docs/sop/ITERATION-WORKFLOW.md`
4. `docs/sop/GIT-WORKFLOW.md`
5. `docs/sop/DOC-CHECK.md`
6. `docs/BOARD.md`
7. `docs/backlog/PRODUCT-BACKLOG.md`
8. `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
9. `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
10. `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
11. `docs/backlog/active/TOOL-015-write-edit-result-visibility.md`
12. `docs/backlog/active/TOOL-018-diff-output-and-rendering.md`
13. `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`
14. `docs/backlog/active/TUI-017-context-usage-percentage.md`
15. `docs/backlog/active/TUI-018-context-limit-million-format.md`
16. `docs/tasks/2026-07-07-mcp-remote-http-sse-streamable.md`
17. `docs/reference/config.reference.toml`

Read any additional owner doc named by an individual task before editing that task.

## Operating Rules

- Work in task order. Do not skip ahead unless the task fallback explicitly permits continuing.
- Before each task, write a short local checklist naming expected files, tests, and owner docs.
- Change owner docs before derived views such as `docs/BOARD.md`.
- Keep code edits localized. If the fix needs a cross-crate API redesign, stop and record a blocker.
- Commit only at phase boundaries: FS04, FS08, FS12, and FS16, unless maintainer asks for smaller
  commits.
- Use conventional commit messages with `[model:<model-name>]`.
- Never claim full validation unless the exact command passed in this worktree.
- Tests using MCP remote transports must use local loopback fixtures only. Do not require internet or
  external MCP services.
- If validation cannot run, record the command, failure summary, environment assumption, and fallback
  validation actually run.

## Ordered Task Items

| ID | Week | Theme | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---:|---|---|---|---|---|---|---|
| FS00 | 1 | Start | Inventory Board/backlog/task state and append a kickoff checkpoint to this file. | Current task disposition and first-owner confirmation. | None | `scripts/validate_project_governance.sh .` and `git diff --check` pass. | If owner docs conflict, record conflict and stop. | Planned |
| FS01 | 1 | Runtime | Audit `RUNTIME-002` residual paths for stuck `processing` state. | Short table of terminal event paths and existing tests. | FS00 | Existing engine tests are linked; missing integration surfaces are listed. | If behavior is unclear, write the gap and stop before coding. | Planned |
| FS02 | 2 | Runtime | Add runtime-level integration coverage for processing state clearing after terminal provider/tool errors. | Deterministic test proving UI/runtime does not remain stuck after error. | FS01 | Targeted runtime/conversation tests pass. | If integration harness is too broad, add the narrowest engine/runtime test and record residual. | Planned |
| FS03 | 3 | Runtime | Add visible stuck-state recovery/status signal without changing provider semantics. | User-visible status or diagnostic event when a turn reaches a terminal error/timeout. | FS02 | Tests prove normal success path is unchanged and error path clears processing. | If display path requires TUI refactor, keep runtime event only and record TUI residual. | Planned |
| FS04 | 4 | Closeout | Month 1 closeout for runtime stuck-state package. | Checkpoint with commits, tests, residuals, recovery instructions. | FS01-FS03 | `cargo fmt --all -- --check`, targeted tests, `cargo check --workspace`, governance, `git diff --check`. | Close as Partial with exact failing gate and owner. | Planned |
| FS05 | 5 | TUI | Complete remaining `TUI-028` preview reset/animation/status inventory. | Table mapping issues #24-#28/#31 to implemented, residual, or out-of-scope. | FS04 | Owner doc updated before Board. | If issue mapping conflicts, stop and ask. | Planned |
| FS06 | 6 | TUI | Implement preview/status feedback fixes that are strictly display-state changes. | Preview clears stale content, status messages identify waiting-for-model vs waiting-for-tool where data exists. | FS05 | Focused TUI tests pass. | If a fix needs new agent protocol events, record residual instead. | Planned |
| FS07 | 7 | Todo | Complete `TODO-002` batch/delete/schema reliability tasks already specified by owner doc. | Idempotent create remains intact; update/delete/batch behavior is tested. | FS04 | `cargo test -p talos-session`, targeted CLI/TUI todo tests pass. | If schema migration is required, stop and request senior review. | Planned |
| FS08 | 8 | Closeout | Month 2 closeout for TUI preview/status and todo reliability. | Checkpoint with validation evidence and residual owners. | FS05-FS07 | `cargo test -p talos-tui`, `cargo test -p talos-session`, `cargo check --workspace`, governance. | Close as Partial with exact residuals. | Planned |
| FS09 | 9 | Tool Output | Implement `TOOL-015` write/edit result visibility. | `write` shows path/byte count/bounded preview; `edit` shows bounded diff. | FS08 | Tool and TUI display tests prove full model payload remains available. | If permission semantics would change, stop. | Planned |
| FS10 | 10 | Diff | Implement bounded diff rendering improvements for `TOOL-018`. | Diff output renders with added/removed styling and bounded length. | FS09 | Tests cover edit diff and read-only git diff surfaces without changing Git permission boundaries. | If Git execution path is involved, limit to rendering and record Git residual. | Planned |
| FS11 | 11 | Provider Usage | Complete `PROVIDER-001` OpenAI-compatible streaming usage accounting. | Usage-only stream chunks are captured; accounting is surfaced to existing usage model. | FS08 | Provider unit tests cover include_usage request option and usage-only chunk handling. | If provider API behavior is uncertain, use fixture-only tests and record uncertainty. | Planned |
| FS12 | 12 | Closeout | Month 3 closeout for tool/diff/provider usage package. | Checkpoint with commits, tests, and residuals. | FS09-FS11 | `cargo test -p talos-tools`, `cargo test -p talos-tui`, `cargo test -p talos-provider`, `cargo check --workspace`, governance. | Close as Partial with exact failing gate. | Planned |
| FS13 | 13 | Status Bar | Implement `TUI-017` context usage percentage using provider/model limits when known. | Status bar shows bounded context percentage without panics when usage or limit is absent. | FS11 | TUI tests cover known limit, unknown limit, zero/invalid data, and OpenAI-compatible usage. | If usage remains unavailable, implement display fallback only and record dependency. | Planned |
| FS14 | 14 | Status Bar | Implement `TUI-018` million-token context format. | `1M ctx`/`2M ctx` formatting for million-scale limits; lower values unchanged. | FS13 | Focused formatting tests pass. | If formatting helper is shared, add tests before editing. | Planned |
| FS15 | 15 | MCP Docs/Test | Harden remote MCP docs and local fixtures for `sse` and `streamable_http`. | README/config docs show stdio, SSE, Streamable HTTP; local tests cover missing URL and loopback fixture paths. | FS12 | `cargo test -p talos-mcp`, config doc review, no internet-dependent tests. | If protocol features beyond request/response are needed, record out-of-scope residual. | Planned |
| FS16 | 16 | Final Closeout | Final handoff and documentation sync. | Final checkpoint, owner-doc status updates, residual list, commit references. | FS00-FS15 | `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, governance, `git diff --check`. | Close as Partial only with exact failed gate and owner for every residual. | Planned |

## Detailed Acceptance Standards

### Runtime Stuck-State Recovery

- A turn must not remain visually or internally `processing` after terminal error, timeout,
  cancellation, or failed tool/provider response.
- Tests must prove both the final state and the visible/diagnostic signal.
- Normal successful streaming behavior must not regress.
- Do not add background watchdog threads or polling loops unless an owner doc explicitly requires
  them. Prefer deterministic state transitions already present in the runtime.

### TUI Preview And Status Feedback

- Display fixes must be orthogonal to model/provider protocol behavior.
- If the UI can distinguish waiting for model response from waiting for tool result using existing
  state, it should render that distinction.
- Stale preview content must clear on new submit, cancellation, terminal error, and resume where
  owner docs require it.
- No hidden errors may be swallowed merely to keep the UI quiet; terminal failures must be visible.

### Todo Mutation Reliability

- Idempotent create behavior must remain intact.
- Batch/delete/update semantics must be deterministic and covered by tests.
- Schema changes require explicit owner-doc acceptance and migration tests. If that becomes
  necessary, stop before implementation.
- Session-scoped behavior must remain session-scoped; do not add cross-session todo mutation.

### Tool And Diff Visibility

- Tool execution payloads visible to the model must remain full unless the owner doc says otherwise.
- UI/scrollback output may be bounded, but omitted counts or truncation markers must be accurate.
- `write` output must never dump large file contents unbounded.
- `edit` and diff display must use existing semantic rendering helpers where possible.
- Read-only Git surfaces must stay read-only; do not add Git mutation behavior.

### Provider Usage And Context Display

- OpenAI-compatible streaming requests should opt into usage accounting only where the provider
  protocol supports it.
- Usage-only chunks must update accounting without creating bogus assistant text.
- Status bar context percentage must degrade gracefully when usage or context limit is missing.
- Million-unit context formatting must not change smaller values.

### Remote MCP Docs And Tests

- Remote MCP tests must use local loopback fixtures only.
- Docs must distinguish:
  - `transport = "stdio"`
  - `transport = "sse"`
  - `transport = "streamable_http"`
  - `transport = "http"` as a compatibility alias for Streamable HTTP
- Auth examples must prefer `auth_token_env` or `authorization_env`, not inline secrets.
- Streamable HTTP resumable sessions and long-lived server-to-client notifications remain out of
  scope for this frontline package.

## Validation Matrix

Baseline commands:

```sh
cargo fmt --all -- --check
cargo check --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Targeted commands by area:

```sh
cargo test -p talos-conversation
cargo test -p talos-tui
cargo test -p talos-session
cargo test -p talos-tools
cargo test -p talos-provider
cargo test -p talos-mcp
```

Full closeout commands:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Optional only when the maintainer asks or the local environment already supports it:

```sh
cargo clippy --workspace -- -D warnings
```

## Branch, Commit, And Checkpoint Plan

- Work on the maintainer-provided branch/worktree.
- Commit at FS04, FS08, FS12, and FS16 after reviewing `git diff --cached`.
- Use one logical commit per phase unless a phase naturally separates docs-only and code changes.
- Do not push unless explicitly instructed by the maintainer.
- Append checkpoint sections to this file at every phase closeout.

Checkpoint template:

```text
## Checkpoint FS<N> - <Title> (<date>)

Completed task items:
Commits:
Changed files:
Validation:
Open deviations:
Residual owner:
Next item:
Recovery instructions:
```

## Stop-And-Ask Conditions

Stop and ask the maintainer before continuing if any task appears to require:

- Permission, approval, sandbox, process-hardening, bash policy, or workspace trust changes.
- Network-dependent tests, external MCP services, real provider credentials, or paid API calls.
- Session storage default changes, JSONL compatibility changes, or binary format migration.
- New runtime dependencies not already approved by owner docs.
- Public API breaking changes or semver-affecting crate changes.
- Rewriting command architecture, TUI architecture, provider protocol architecture, or MCP protocol
  architecture.
- Release, publish, tag, deploy, DNS, signing, or installer actions.
- Guessing product behavior not stated in this plan or the owner docs.

## Default Decisions For Foreseeable Ambiguity

- If a task is already implemented, do not rewrite it. Add missing tests/evidence and mark the item
  complete with commit references.
- If an owner doc and this plan conflict, owner doc wins; record the conflict in this task file.
- If full workspace tests are slow but targeted tests pass, do not claim full validation. Record the
  targeted commands and leave full validation to phase closeout.
- If a UI screenshot cannot be captured, record deterministic text/snapshot evidence and the exact
  test that covers the behavior.
- If a fix requires high-risk scope, stop and convert the finding into a residual with owner doc,
  file references, and reproduction steps.

## Residual Work Destination

- Runtime stuck-state residuals: `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- TUI preview/status residuals: `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
- Todo reliability residuals: `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- Tool/diff residuals: `docs/backlog/active/TOOL-015-write-edit-result-visibility.md` or
  `docs/backlog/active/TOOL-018-diff-output-and-rendering.md`
- Provider usage/status residuals: `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`,
  `docs/backlog/active/TUI-017-context-usage-percentage.md`, or
  `docs/backlog/active/TUI-018-context-limit-million-format.md`
- MCP remote residuals: create a new backlog item only if the residual changes scope beyond the
  2026-07-07 fast-track request/response transport implementation.

## Checkpoint FS00 - Kickoff Inventory (2026-07-07)

Completed task items:
- FS00: Board/backlog/task state inventoried; no owner-doc conflicts requiring stop-and-ask.

Commits:
- None. Doc-only kickoff checkpoint; first code commit lands at FS04 per the branch plan.

Changed files:
- `docs/tasks/2026-07-07-frontline-runtime-ux-stability-plan.md` (this checkpoint).

Inventory and disposition:

| Owner doc | Status at kickoff | FS item | Disposition |
|---|---|---|---|
| `RUNTIME-002-turn-health-and-stuck-processing.md` | In Progress (SSP140: engine-level `is_processing` verification complete with 5 deterministic tests in `engine_tests.rs`) | FS01-FS03 | Real work remains: runtime-level integration coverage (FS02) and visible stuck-state recovery signal (FS03). Engine-level clearing is already proven; do not rewrite it. |
| `TUI-028-preview-status-feedback-reliability.md` | In Progress (SSP130: stale preview clear complete) | FS05-FS06 | Remaining preview reset / waiting-for-model vs waiting-for-tool / animation items stay in the owner doc for inventory (FS05) and strictly display-state fixes (FS06). |
| `TODO-002-todo-mutation-reliability.md` | In Progress (SSP120: idempotent create complete; `find_by_title()` added; 4 tests) | FS07 | Remaining batch / delete / schema reliability items are owned by the owner doc. Stop and request senior review if a schema migration becomes necessary. |
| `TOOL-015-write-edit-result-visibility.md` | Complete (2026-07-01, I076/T104; 22 file-tool tests, 200 unit, 4 TUI tool_result tests) | FS09 | Per Default Decisions: do not rewrite. Verify bounded preview/diff behavior still passes targeted tests, add evidence pointer, mark FS09 complete with commit reference at FS12 closeout. |
| `TOOL-018-diff-output-and-rendering.md` | Planned | FS10 | Real work: bounded diff rendering for `edit` and `git_diff` with added/removed styling, preserving read-only Git boundaries. |
| `PROVIDER-001-openai-streaming-usage.md` | Complete (2026-07-01, I076/T101; `parse_sse_stream_retains_usage_only_chunk` regression test, 48 unit + 4 integration + 2 doc tests) | FS11 | Per Default Decisions: do not rewrite. Verify `include_usage` request option and usage-only chunk handling still pass targeted tests, add evidence pointer at FS12 closeout. |
| `TUI-017-context-usage-percentage.md` | Complete (2026-07-01, I076/T103; 14 status-bar tests) | FS13 | Per Default Decisions: do not rewrite. Plan acceptance adds "zero/invalid data" and "OpenAI-compatible usage" test coverage — add only missing cases at FS13, do not redo existing formatting. |
| `TUI-018-context-limit-million-format.md` | Complete (2026-07-01, I076/T102; 14 status-bar tests cover M, k, raw, none) | FS14 | Per Default Decisions: do not rewrite. Verify acceptance cases (`1M ctx`, `2M ctx`, `200k ctx`) still pass and record evidence at FS14. |
| `2026-07-07-mcp-remote-http-sse-streamable.md` | Complete (commit `a7cc14c`: stdio + sse + streamable_http + http alias implemented) | FS15 | Implementation shipped. FS15 work is docs/test hardening only: README/config docs distinguishing the four transports and local loopback fixture tests for missing URL and existing fixture paths. No internet-dependent tests. |

Conflict check:
- No owner doc conflicts with the plan. The "already complete" items (TOOL-015, PROVIDER-001, TUI-017, TUI-018, MCP remote) are handled by the plan's Default Decisions: do not rewrite; add missing tests/evidence and mark complete with commit references.
- BOARD.md lists this plan as `Planned` with the correct exclusion gate; no permission, sandbox, release, storage-format, plugin, Git transport, or protocol redesign is authorized here.
- Branch: working on `main` (1 commit ahead of `origin/main`, the plan-file commit `7f6ea5d`). No maintainer-provided branch/worktree was named; proceeding on `main` with commits only at FS04/FS08/FS12/FS16 per the branch plan.

Validation (baseline, before any code change):
- `git diff --check`: PASS (clean working tree).
- `scripts/validate_project_governance.sh .`: PASS, 0 warnings.

Open deviations:
- None. The "already complete" owner docs are not a deviation; they are handled by Default Decisions and recorded in the per-task evidence at the corresponding FS closeout.

Residual owner:
- None new at FS00. Per-task residuals stay in their owner docs as listed under Residual Work Destination above.

Next item:
- FS01: audit `RUNTIME-002` residual paths for stuck `processing` state. Produce a short table of terminal event paths and existing tests; list missing integration surfaces. Owner reads: `crates/talos-conversation/src/engine.rs`, `crates/talos-conversation/src/engine_tests.rs`, `crates/talos-cli/src/tui_bridge.rs`, `crates/talos-tui/src/app.rs`, ADR-006.

Recovery instructions:
- Owning record: this file (`docs/tasks/2026-07-07-frontline-runtime-ux-stability-plan.md`).
- Git state at FS00 close: branch `main`, HEAD `7f6ea5d` (plan-file commit), working tree has only this checkpoint edit pending.
- Resume by reading this checkpoint, then start FS01 by auditing `crates/talos-conversation/src/engine.rs` terminal event handlers against the 5 existing engine-level tests in `engine_tests.rs`.
- Completion gate for FS00 (governance + `git diff --check`) must pass before advancing to FS01.

## Checkpoint FS04 - Month 1 Closeout: Runtime Stuck-State (2026-07-07)

Completed task items:
- FS00: Board/backlog/task state inventoried; kickoff checkpoint appended; no owner-doc conflicts.
- FS01: Terminal event paths audited; 6 terminal paths mapped to existing tests; MaxTokens clearing
  gap identified and documented in `RUNTIME-002` owner doc; missing integration surfaces listed.
- FS02: MaxTokens stuck-processing bug fixed in `engine.rs` (clear `is_processing` on any
  non-`ToolUse` stop reason); 2 engine tests + 3 conversation-loop integration tests added.
- FS03: Visible signal chain verified as already implemented at TUI level (`preview_text_for_state`,
  status-bar phase text, error Tip coloring); 2 integration tests added proving the conversation
  loop forwards visible error Tip + Error stream on terminal failure, and that the normal EndTurn
  success path is unchanged.

Commits:
- (will be created at this checkpoint — see `git log` after commit)

Changed files:
- `crates/talos-conversation/src/engine.rs` — MaxTokens clearing fix (1 condition change + comment).
- `crates/talos-conversation/src/engine_tests.rs` — 2 new engine tests (MaxTokens, ToolUse continuation).
- `crates/talos-cli/src/tests.rs` — 5 new conversation-loop integration tests + 2 test helpers.
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md` — FS01 audit table,
  FS02-FS03 execution evidence, status update, Required Reads expanded.
- `docs/tasks/2026-07-07-frontline-runtime-ux-stability-plan.md` — FS00 kickoff checkpoint,
  this FS04 closeout checkpoint.

Validation (all run in this worktree on 2026-07-07):
- `cargo fmt --all -- --check`: PASS (clean).
- `cargo check --workspace`: PASS.
- `cargo test -p talos-conversation`: PASS, 117 tests (115 original + 2 new), 0 failed.
- `cargo test -p talos-cli --bin talos`: PASS, 159 tests (154 original + 5 new), 0 failed.
- `scripts/validate_project_governance.sh .`: PASS, 0 warnings.
- `git diff --check`: PASS.

Open deviations:
- None. The MaxTokens clearing change is a bug fix within RUNTIME-002 acceptance ("every terminal
  path clears `is_processing`"), not a provider-semantics change. Normal success path is proven
  unchanged by `conversation_loop_normal_end_turn_success_path_unchanged`.

Residual owner:
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md` owns the optional
  `UserInput::Cancel` through `tui_bridge` integration test and any future health-check task.
  These are recorded as residuals in the owner doc and are not blockers for FS04 closeout.

Next item:
- FS05: Complete remaining `TUI-028` preview reset/animation/status inventory. Owner doc:
  `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`. Map issues #24-#28/#31 to
  implemented, residual, or out-of-scope.

Recovery instructions:
- Owning record: this file.
- Git state at FS04 close: branch `main`, HEAD will be the FS04 commit (after `git commit`).
- Resume by reading this checkpoint, verifying `cargo test -p talos-conversation -p talos-cli` still
  passes, then starting FS05 by reading the TUI-028 owner doc and inventorying its remaining items.
- If the MaxTokens fix needs revisiting, the key file is
  `crates/talos-conversation/src/engine.rs` `TurnEnd` handler (the `!matches!(stop_reason, StopReason::ToolUse)`
  condition); the regression guard is `turn_end_tool_use_keeps_processing_for_continuation` in
  `engine_tests.rs`.

## Checkpoint FS08 - Month 2 Closeout: TUI Preview/Status + Todo Reliability (2026-07-07)

Completed task items:
- FS05: TUI-028 inventory complete. All 6 issues mapped: #24-#28 already implemented (stale preview
  clear, 50ms animation timer, dashboard System-source rendering, truncate_str model formatting,
  thinking label animation), #31 out-of-scope (thinking persistence decision gap, ADR-034).
- FS06: No display-state code work needed — all TUI-028 acceptance items were already implemented.
  Per Default Decisions, did not rewrite. Verified via 251 TUI tests.
- FS07: `/todo delete <id> --confirm` implemented (TodoCommandAction::Delete, short-ID resolution,
  ambiguity detection, --confirm guard). `TodoRepository::create_batch` added with idempotent
  deduplication. UUID hiding verified already implemented.

Commits:
- FS04 commit: `3d3c3dd` (Month 1 closeout).
- FS08 commit: (will be created at this checkpoint).

Changed files (since FS04):
- `crates/talos-conversation/src/types.rs` — `TodoCommandAction::Delete { id, confirm }`.
- `crates/talos-conversation/src/engine.rs` — `parse_todo_command` delete subcommand + --confirm.
- `crates/talos-conversation/src/engine_tests.rs` — 3 delete parse tests.
- `crates/talos-cli/src/todo_view.rs` — `handle_todo_delete` + `resolve_todo_id` + 3 view tests.
- `crates/talos-session/src/todo.rs` — `create_batch` method + 4 batch tests.
- `README.md` — `/todo delete` slash-command table row.
- `docs/backlog/active/TUI-028-*.md` — FS05 issue inventory.
- `docs/backlog/active/TODO-002-*.md` — FS07 execution evidence.
- `docs/tasks/2026-07-07-frontline-runtime-ux-stability-plan.md` — this checkpoint.

Validation (all run in this worktree on 2026-07-07):
- `cargo fmt --all -- --check`: PASS (clean).
- `cargo check --workspace`: PASS.
- `cargo test -p talos-tui`: PASS, 251 tests (249 unit + 2 doc), 0 failed.
- `cargo test -p talos-session`: PASS, 97 tests (93 original + 4 batch), 0 failed.
- `cargo test -p talos-conversation`: PASS, 120 tests (117 + 3 delete parse), 0 failed.
- `cargo test -p talos-cli --bin talos`: PASS, 162 tests (159 + 3 delete view), 0 failed.
- `scripts/validate_project_governance.sh .`: PASS, 0 warnings.
- `git diff --check`: PASS.

Open deviations:
- None. `/todo delete` is the first mutating user slash command; it requires `--confirm` as the
  acceptance demands. Batch agent tool registration is a residual, not a deviation.

Residual owner:
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md` owns the batch agent tool
  registration residual and the README help-text clarification for mutating vs read-only `/todo`
  subcommands.
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md` owns the #31 thinking
  persistence decision gap.

Next item:
- FS09: Implement `TOOL-015` write/edit result visibility. Owner doc says Complete (I076/T104);
  per Default Decisions, verify existing implementation + add evidence pointer at FS12 closeout.

Recovery instructions:
- Owning record: this file.
- Git state at FS08 close: branch `main`, HEAD will be the FS08 commit (after `git commit`).
- Resume by reading this checkpoint, verifying targeted tests pass, then starting FS09 by reading
  `docs/backlog/active/TOOL-015-write-edit-result-visibility.md`.
- If `/todo delete` needs revisiting, the key files are `crates/talos-conversation/src/types.rs`
  (Delete variant), `crates/talos-conversation/src/engine.rs` (parse_todo_command delete branch),
  and `crates/talos-cli/src/todo_view.rs` (handle_todo_delete + resolve_todo_id).
