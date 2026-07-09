# 2026-07-07 Frontline Runtime And UX Stability Plan

**Status**: Partial after 2026-07-08 issue audit
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

## 2026-07-08 Issue-Audit Correction

The original FS16 closeout overclaimed completion for several issue-linked items. The completed
work remains useful, but this plan is no longer considered fully complete:

- `#18` / `RUNTIME-002` remains open because the provider request-dispatch hang before response
  headers is not fixed. OpenAI-compatible and Anthropic providers still use `reqwest::Client::new()`
  without a request-dispatch timeout protecting `send().await`.
- `#28` is reopened as `#39`: Dashboard availability is still a persistent System scrollback line
  with a redundant `[System]` prefix, not a transient tip notification.
- `#24` and `#31` have implementation-adjacent evidence only; they need real runtime/visual
  evidence before their UX claims can be treated as closed.
- `#26` is split to `TUI-029` and remains Planned / decision-required.

Follow-up execution lives in `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`,
starting with I107's corrective queue.

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
| FS00 | 1 | Start | Inventory Board/backlog/task state and append a kickoff checkpoint to this file. | Current task disposition and first-owner confirmation. | None | `scripts/validate_project_governance.sh .` and `git diff --check` pass. | If owner docs conflict, record conflict and stop. | Complete |
| FS01 | 1 | Runtime | Audit `RUNTIME-002` residual paths for stuck `processing` state. | Short table of terminal event paths and existing tests. | FS00 | Existing engine tests are linked; missing integration surfaces are listed. | If behavior is unclear, write the gap and stop before coding. | Partial after 2026-07-08 audit |
| FS02 | 2 | Runtime | Add runtime-level integration coverage for processing state clearing after terminal provider/tool errors. | Deterministic test proving UI/runtime does not remain stuck after error. | FS01 | Targeted runtime/conversation tests pass. | If integration harness is too broad, add the narrowest engine/runtime test and record residual. | Complete |
| FS03 | 3 | Runtime | Add visible stuck-state recovery/status signal without changing provider semantics. | User-visible status or diagnostic event when a turn reaches a terminal error/timeout. | FS02 | Tests prove normal success path is unchanged and error path clears processing. | If display path requires TUI refactor, keep runtime event only and record TUI residual. | Complete |
| FS04 | 4 | Closeout | Month 1 closeout for runtime stuck-state package. | Checkpoint with commits, tests, residuals, recovery instructions. | FS01-FS03 | `cargo fmt --all -- --check`, targeted tests, `cargo check --workspace`, governance, `git diff --check`. | Close as Partial with exact failing gate and owner. | Partial after 2026-07-08 audit |
| FS05 | 5 | TUI | Complete remaining `TUI-028` preview reset/animation/status inventory. | Table mapping issues #24/#25/#27/#28/#31 to their current dispositions, and #26 to a separate decision-required owner. | FS04 | Owner doc updated before Board. | If issue mapping conflicts, stop and ask. | Partial after 2026-07-08 audit |
| FS06 | 6 | TUI | Implement preview/status feedback fixes that are strictly display-state changes. | Preview clears stale content, status messages identify waiting-for-model vs waiting-for-tool where data exists. | FS05 | Focused TUI tests pass. | If a fix needs new agent protocol events, record residual instead. | Partial after 2026-07-08 audit |
| FS07 | 7 | Todo | Complete `TODO-002` batch/delete/schema reliability tasks already specified by owner doc. | Idempotent create remains intact; update/delete/batch behavior is tested. | FS04 | `cargo test -p talos-session`, targeted CLI/TUI todo tests pass. | If schema migration is required, stop and request senior review. | Complete |
| FS08 | 8 | Closeout | Month 2 closeout for TUI preview/status and todo reliability. | Checkpoint with validation evidence and residual owners. | FS05-FS07 | `cargo test -p talos-tui`, `cargo test -p talos-session`, `cargo check --workspace`, governance. | Close as Partial with exact residuals. | Partial after 2026-07-08 audit |
| FS09 | 9 | Tool Output | Implement `TOOL-015` write/edit result visibility. | `write` shows path/byte count/bounded preview; `edit` shows bounded diff. | FS08 | Tool and TUI display tests prove full model payload remains available. | If permission semantics would change, stop. | Complete |
| FS10 | 10 | Diff | Implement bounded diff rendering improvements for `TOOL-018`. | Diff output renders with added/removed styling and bounded length. | FS09 | Tests cover edit diff and read-only git diff surfaces without changing Git permission boundaries. | If Git execution path is involved, limit to rendering and record Git residual. | Complete |
| FS11 | 11 | Provider Usage | Complete `PROVIDER-001` OpenAI-compatible streaming usage accounting. | Usage-only stream chunks are captured; accounting is surfaced to existing usage model. | FS08 | Provider unit tests cover include_usage request option and usage-only chunk handling. | If provider API behavior is uncertain, use fixture-only tests and record uncertainty. | Complete |
| FS12 | 12 | Closeout | Month 3 closeout for tool/diff/provider usage package. | Checkpoint with commits, tests, and residuals. | FS09-FS11 | `cargo test -p talos-tools`, `cargo test -p talos-tui`, `cargo test -p talos-provider`, `cargo check --workspace`, governance. | Close as Partial with exact failing gate. | Complete |
| FS13 | 13 | Status Bar | Implement `TUI-017` context usage percentage using provider/model limits when known. | Status bar shows bounded context percentage without panics when usage or limit is absent. | FS11 | TUI tests cover known limit, unknown limit, zero/invalid data, and OpenAI-compatible usage. | If usage remains unavailable, implement display fallback only and record dependency. | Complete |
| FS14 | 14 | Status Bar | Implement `TUI-018` million-token context format. | `1M ctx`/`2M ctx` formatting for million-scale limits; lower values unchanged. | FS13 | Focused formatting tests pass. | If formatting helper is shared, add tests before editing. | Complete |
| FS15 | 15 | MCP Docs/Test | Harden remote MCP docs and local fixtures for `sse` and `streamable_http`. | README/config docs show stdio, SSE, Streamable HTTP; local tests cover missing URL and loopback fixture paths. | FS12 | `cargo test -p talos-mcp`, config doc review, no internet-dependent tests. | If protocol features beyond request/response are needed, record out-of-scope residual. | Complete |
| FS16 | 16 | Final Closeout | Final handoff and documentation sync. | Final checkpoint, owner-doc status updates, residual list, commit references. | FS00-FS15 | `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, governance, `git diff --check`. | Close as Partial only with exact failed gate and owner for every residual. | Complete |

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
| `TOOL-018-diff-output-and-rendering.md` | Complete | FS10 | Real work: bounded diff rendering for `edit` and `git_diff` with added/removed styling, preserving read-only Git boundaries. |
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
  `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`. Map issues
  #24/#25/#27/#28/#31 to current dispositions, and keep #26 outside TUI-028 unless the
  reasoning-history decision boundary changes.

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
- FS05: TUI-028 inventory originally marked #24/#25/#27/#28/#31 implemented. Later audits
  corrected this: #27 has implementation evidence; #25 is still open because current code only
  animates a gradient label, not the requested two-color three-segment ripple; #28 is reopened as
  #39; #24/#31 need runtime/visual evidence; #26 is tracked by TUI-029 as decision-required.
- FS06: Original closeout claimed no display-state code work was needed. 2026-07-08 audit corrected
  this: #28/#39 still needs a transient dashboard notification, and #24/#31 need real
  runtime/visual evidence. The 251 TUI tests did not prove those UX claims.
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
- `docs/backlog/active/TUI-029-thinking-history-archive.md` owns the #26 thinking persistence
  decision gap.

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

## Checkpoint FS12 - Month 3 Closeout: Tool/Diff/Provider Usage (2026-07-07)

Completed task items:
- FS09: TOOL-015 verified complete (I076/T104). 22 file-tool tests + 7 TUI tool_result tests pass.
  Per Default Decisions, no rewrite; evidence confirmed.
- FS10: TOOL-018 scrollback diff rendering implemented. `tool_display.rs` now applies semantic
  +/- styling (green/red foreground) for `edit`/`diff` tool results and unified diff markers.
  False-positive prevention: prose with `-`/`+` bullets is NOT styled for non-diff tools.
  3 new tests. `git_diff` unified diff content recorded as residual (needs deeper gix API work).
- FS11: PROVIDER-001 verified complete (I076/T101). 4 usage tests pass:
  `include_usage` request option, usage-only chunk handling, null-usage guard, SSE retention.

Commits:
- FS12 commit: (will be created at this checkpoint).

Changed files (since FS08):
- `crates/talos-tui/src/tool_display.rs` — `is_diff_content` + `diff_line_style` + rendering loop
  modification + 3 tests.
- `docs/backlog/active/TOOL-018-diff-output-and-rendering.md` — FS10 evidence + residuals.
- `docs/tasks/2026-07-07-frontline-runtime-ux-stability-plan.md` — this checkpoint.

Validation (all run in this worktree on 2026-07-07):
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test -p talos-tools`: PASS, 255 tests, 0 failed.
- `cargo test -p talos-tui`: PASS, 254 tests (252 unit + 2 doc), 0 failed.
- `cargo test -p talos-provider`: PASS, 73 tests, 0 failed.
- `scripts/validate_project_governance.sh .`: PASS, 0 warnings.
- `git diff --check`: PASS.

Open deviations:
- None. TOOL-015 and PROVIDER-001 were already complete per Default Decisions. TOOL-018 scrollback
  rendering is new work; git_diff unified content is a residual, not a deviation.

Residual owner:
- `docs/backlog/active/TOOL-018-diff-output-and-rendering.md` owns the git_diff unified diff
  content residual and the HistoryAttrs background-color limitation.

Next item:
- FS13: TUI-017 context usage percentage. Verified complete (I076/T103). 16 status_bar tests pass
  including known/unknown limit and compact mode. Per Default Decisions, verify + add any missing
  edge-case test at FS13.

Recovery instructions:
- Owning record: this file.
- Git state at FS12 close: branch `main`, HEAD will be the FS12 commit.
- Resume by reading this checkpoint, verifying `cargo test -p talos-tui` passes, then starting
  FS13 by confirming TUI-017 status_bar tests and proceeding to FS15 MCP docs/tests.

## Checkpoint FS16 - Final Closeout (2026-07-07)

Completed task items (FS00-FS15):
- FS00: Kickoff inventory; no owner-doc conflicts; baseline gates passed.
- FS01: RUNTIME-002 terminal event paths audited; MaxTokens clearing gap identified.
- FS02: MaxTokens stuck-processing bug fixed (engine.rs); 2 engine + 3 conversation-loop tests.
- FS03: Visible signal chain verified (Tip/Error Stream/Status); 2 integration tests.
- FS04: Month 1 closeout — commit `3d3c3dd`.
- FS05: TUI-028 inventory — originally marked #24/#25/#27/#28/#31 implemented; later audits
  corrected #25 to open implementation gap, #28 to #39 open, and #24/#31 to evidence gaps. #26
  split to TUI-029 as decision-required.
- FS06: Original closeout claimed no display-state work was needed. 2026-07-08 audit corrected
  this: #28/#39 still needs implementation, and #24/#31 need runtime/visual evidence.
- FS07: `/todo delete <id> --confirm` implemented; `TodoRepository::create_batch` added.
- FS08: Month 2 closeout — commit `d76cac4`.
- FS09: TOOL-015 verified complete (22 file-tool + 7 TUI tests).
- FS10: TOOL-018 scrollback diff rendering + git_diff unified diff content implemented; 4 tests.
- FS11: PROVIDER-001 verified complete (4 usage tests).
- FS12: Month 3 closeout — commit `afc27a2`.
- FS13: TUI-017 verified complete (16 status_bar tests; division-by-zero guard exists).
- FS14: TUI-018 verified complete (million format + sub-million preserved).
- FS15: MCP remote docs updated (README: 4 transports + auth examples); 10 MCP tests pass.
- FS16 revision: closed TODO-002 batch tool registration gap (TodoCreateBatchTool registered in
  print + TUI registries) and TOOL-018 git_diff unified diff gap (similar::TextDiff::unified_diff
  with HEAD-blob retrieval). Updated all task-table statuses; synced BOARD/backlog derived views.
- FS16 revision 2: closed remaining acceptance gaps from second review. Added `todo_update_batch`
  tool (TODO-002 batch update acceptance). Added git_diff `staged` mode (HEAD vs index via `:path`
  rev syntax) and `path` parameter filtering (TOOL-018 staged + path-filtered acceptance). Ref-to-ref
  comparison formally deferred via documented acceptance change in TOOL-018 owner doc. 1767 tests
  pass.

Commits (this plan):
- `3d3c3dd` — FS01-FS04: MaxTokens fix + runtime integration coverage.
- `d76cac4` — FS05-FS08: /todo delete + batch create + TUI-028/TODO-002 closeout.
- `afc27a2` — FS09-FS12: scrollback diff rendering + TOOL-018 evidence.
- `8199e19` — FS13-FS16: MCP docs + initial closeout.
- FS16 revision commit: (this checkpoint).

Changed files (cumulative across all phases):
- `crates/talos-conversation/src/engine.rs` — MaxTokens clearing fix + /todo delete parsing.
- `crates/talos-conversation/src/engine_tests.rs` — 8 new tests (MaxTokens, ToolUse, delete parse).
- `crates/talos-conversation/src/types.rs` — TodoCommandAction::Delete variant.
- `crates/talos-cli/src/tests.rs` — 7 conversation-loop integration tests.
- `crates/talos-cli/src/todo_view.rs` — /todo delete handler + short-ID resolution + 3 tests.
- `crates/talos-cli/src/registry.rs` — TodoCreateBatchTool registration + test assertion.
- `crates/talos-session/src/todo.rs` — create_batch method + TodoCreateBatchTool + 7 tests.
- `crates/talos-session/src/lib.rs` — TodoCreateBatchTool + TodoCreateBatchInput exports.
- `crates/talos-tui/src/tool_display.rs` — diff rendering + 3 tests.
- `crates/talos-tools/src/git.rs` — git_diff unified diff via similar::TextDiff + 1 integration test.
- `README.md` — /todo delete row + MCP 4-transport docs.
- `docs/BOARD.md` — frontline plan + RUNTIME-002/TODO-002/TUI-028/TOOL-018 status sync.
- `docs/backlog/PRODUCT-BACKLOG.md` — frontline plan + TODO-002/TOOL-018 status sync.
- `docs/backlog/active/RUNTIME-002-*.md` — FS01 audit + FS02-FS03 evidence.
- `docs/backlog/active/TUI-028-*.md` — FS05 issue inventory.
- `docs/backlog/active/TODO-002-*.md` — FS07 evidence + status update.
- `docs/backlog/active/TOOL-018-*.md` — FS10 evidence + status update.
- `docs/tasks/2026-07-07-frontline-runtime-ux-stability-plan.md` — all checkpoints + table status.

Final validation (all run in this worktree on 2026-07-07):
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test --workspace`: PASS, 1767 tests, 0 failed.
- `scripts/validate_project_governance.sh .`: PASS, 0 warnings.
- `git diff --check`: PASS.

Open deviations:
- `#18` request-dispatch timeout was not fixed by FS04; it was later resolved in I107 by
  `dispatch_timeout_secs` plus provider, agent, and conversation-loop tests.
- `#28` was not fully fixed and is reopened as `#39`; System-source scrollback output is not the
  requested transient dashboard notification.
- `#24` and `#31` need real runtime/visual evidence before the UX claims can be closed.

Fallback-permitted residuals (explicitly allowed by each task's fallback column, not deviations):
- RUNTIME-002/PROVIDER-002: #18 request-dispatch timeout was an open P0 residual from this plan and
  is resolved by I107. Optional health-check task remains as a secondary residual.
- TODO-002: no residual. `todo_create_batch` and `todo_update_batch` are implemented and registered
  in both print and TUI tool registries.
- TOOL-018: `git_diff` ref-to-ref comparison formally deferred via documented acceptance change
  (see TOOL-018 owner doc). The deferred work is retained as `TOOL-020` so it is not lost during
  later planning. Unstaged, staged, and path-filtered modes are implemented and tested.
  HistoryAttrs background-color limitation is a display constraint.
- TUI-028: #25 thinking ripple animation and #28/#39 transient dashboard notification are open;
  #24/#31 need runtime/visual evidence.
- TUI-029: #26 thinking persistence into history is a decision gap (ADR-034/TUI-020 revision
  required). The plan's acceptance explicitly defers this: "Persisting thinking content into
  history is not implemented unless ADR-034/TUI-020 are explicitly revised."

Recovery instructions:
- Owning record: this file.
- Git state at final closeout: branch `main`, HEAD is the FS16 revision commit.
- The plan is Partial after audit. Resume residual work from I107 in the 2026-07-08 Talos
  self-bootstrap plan.
- The original FS16 validation recorded 1767 workspace tests passing, but the 2026-07-08 issue
  audit found product gates still open. No push, release, tag, publish, or deployment occurred.
