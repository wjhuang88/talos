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
