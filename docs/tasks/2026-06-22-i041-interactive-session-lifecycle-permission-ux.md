# Long-Running Task: I041 Interactive Session Lifecycle & Operation-Scoped Permissions

> Status: In Progress
> Created: 2026-06-22
> Confirmed: 2026-06-22
> Owner iteration: [I041 Interactive Session Lifecycle & Operation-Scoped Permissions](../iterations/I041-interactive-session-lifecycle-permission-ux.md)
> Baseline rule: this confirmed task inventory is preserved; unrelated work goes to residuals.

## Startup Contract

### Outcome

Deliver the published I041 MVP over four weeks:

1. **PERM-002 operation-scoped permission rules** — `PermissionRule` matches on
   `ToolNature` + `resource` (path / domain) instead of tool name. Resource extraction
   from tool input by nature. "Always approve" creates a scoped rule. Old tool_name-only
   rules continue to load and apply unchanged.
2. **SESSION-001-B `/new` and `/resume`** — interactive slash commands that consume the
   SESSION-001-A `SessionTransition` service end-to-end. Workspace-scoped resume
   candidates in deterministic order. Refusal/queue while a turn is active.
3. **SESSION-001-C `/fork`** — slash command that clones durable history into a distinct
   child identity; source session remains byte-for-byte unchanged after activation.

### In Scope

- PERM-002 rule schema, matcher, resource extractor, live-approval scoping.
- PERM-002 default rules migration in `crates/talos-permission/src/lib.rs`.
- `/new` and `/resume` BuiltinCommand registration + SessionTransition consumption.
- `/resume` candidate listing in MEM-004 workspace identity, deterministic ordering.
- `/fork` BuiltinCommand + durable history clone + child activation via SessionTransition.
- Refusal/queue policy when a model/tool turn is active.
- Backward compatibility for tool_name-only permission rules.
- Automated tests, real `talos` binary smoke (`/new`, `/resume`, `/fork`, one
  nature-based allow-once-then-auto scenario), README, governance sync.

### Out Of Scope

- SKILL-002 Level 1/2 activation (separate iteration).
- TUI-008 approval dialog UX (separate iteration; PERM-002 deliverable is engine-level
  scoping, not UI).
- Session deletion, rename, cross-workspace resume, model switching.
- Merge/rebase between sessions, cloud branches, marketplace.
- Regex patterns for resources (glob is sufficient for v1).
- Runtime rule editing UI.
- Public API breaking changes, new `unsafe`, new runtime dependencies.
- Release, tag, deployment, remote service mutation, paid API, or destructive actions.

### Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T1 | PERM-002 matcher foundation | `PermissionRule` gains `nature`, `resource`, `resource_kind` fields; first-match-wins matcher by nature+resource; legacy tool_name rules still apply | None | Unit tests cover first-match-wins for Read/Write/Execute/Network natures and legacy fallback | If nature lookup fails for legacy rule, keep tool_name exact match (current behavior) | Planned |
| T2 | PERM-002 resource extractor | Extract path / host from tool input by nature; integration with `AgentTool::nature()` | T1 | Unit tests for http_request/web_search/read/write/edit/delete/bash | If extraction fails, return `Ask` (visible to user) | Planned |
| T3 | PERM-002 config + defaults | Config format `[[rules]] nature = "..." resource = "..."`; default rules in `crates/talos-permission/src/lib.rs` migrated to nature form; backward-compat load path | T1, T2 | Existing config files load without error; new config round-trips through serde | Keep `tool_name` as optional and match exact tool name when nature absent | Planned |
| T4 | PERM-002 always-approve scoping | Pressing `a` in approval dialog creates a scoped rule (write path / network host) instead of tool-wide rule | T1, T2, T3 | Integration test: `a` on `write src/main.rs` produces a rule for `Write` + path `src/main.rs`; subsequent `write` to same path auto-allowed, different path still asks | Document scope change in README; UI explicitly shows "always for this resource" | Planned |
| T5 | PERM-002 closure + review | PERM-002 acceptance criteria all green; documentation in `docs/backlog/active/PERM-002-operation-scoped-permissions.md` synchronized; tests pass | T1-T4 | `cargo test -p talos-permission`, `cargo test --workspace`, README updated | Leave PERM-002 Partial with explicit blockers | Planned |
| T6 | SESSION-001-B `/new` BuiltinCommand | `/new` registered through CMD-001; consumes `SessionTransition::prepare(New)`; commits on user confirmation; refusal/queue while turn is active | T1-T5 (depends on PERM-002 for write tool approval flow) | Integration test proves `/new` end-to-end: new Agent context, new persistence target, process config preserved | If `SessionTransition` ownership of AppServerSession breaks the test, stop and consult an ADR | Planned |
| T7 | SESSION-001-B `/resume` BuiltinCommand | `/resume` lists workspace-scoped candidates in deterministic order (most-recent first, tie-break on session ID); consumes `SessionTransition::prepare(Resume)`; hydration failures preserve old session | T6 | Integration test covers: two workspaces → only current workspace candidates; tie-break deterministic; hydration failure → old session active | Document ordering rule in README; if MEM-004 hash changes break ordering, re-run integration | Planned |
| T8 | SESSION-001-C `/fork` BuiltinCommand | `/fork` clones durable history boundary into distinct child identity; activates child through `SessionTransition`; source session bytes unchanged | T6 (uses same transition infrastructure) | Integration test: source session JSONL/SQLite byte-for-byte unchanged after `/fork` + 1 child turn | If SQLite row update path is non-bytewise, document a more relaxed "no appended rows on source" invariant | Planned |
| T9 | Real binary smoke | `talos` binary supports `/new`, `/resume`, `/fork` and one PERM-002 allow-once-then-auto scenario | T5-T8 | Mock-provider binary command exits 0; smoke scenario recorded as I041 evidence | Retry twice; if env restricts TTY, use `talos -p` (print) mode and document the limitation | Planned |
| T10 | Full closure and delivery | Workspace green, governance synchronized, I041 → Complete with retrospective | T9 | fmt, check, clippy `-D warnings`, workspace tests, both governance validators, diff check, I041 retrospective | Do not mark Complete; leave Review/Partial with checkpoint and exact failing gate | Planned |

### Dependencies And Prerequisites

- Current HEAD includes commit `bf4dca4` (workspace dependency upgrade).
- I040 Complete: SESSION-001-A infrastructure exists and is verified.
- CMD-001 Complete: first-class BuiltinCommand registry available.
- MEM-004 Complete: workspace identity hash for resume candidate filtering.
- PERM-001 (existing permission engine) and `ToolNature` enum are in place.
- Rust stable toolchain and existing Cargo dependencies are available.
- ADR-005/006 typed session seam and single-consumer flow remain binding.
- ADR-016 durable history authoritative; UI is not the fork source of truth.

### Artifacts And State Owners To Update

- Code: `talos-permission`, `talos-cli` (commands, approval, registry), `talos-core`
  (tool nature), tests only as required.
- Backlog: `PERM-002-operation-scoped-permissions.md`,
  `SESSION-001-B-new-resume.md`, `SESSION-001-C-fork.md` — status fields and acceptance
  boxes synchronized.
- Iteration: `I041-interactive-session-lifecycle-permission-ux.md` — execution
  record, verification evidence, retrospective.
- Owners: Product Backlog, iterations index, Board, Manifest, README, AGENTS.md Task
  Router (if PERM-002 implementation becomes a recurring route), EVOLUTION when a
  reusable lesson appears.
- Task checkpoints: this file after every task item / phase boundary.

### Validation And Acceptance Evidence

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `bash scripts/validate_project_governance.sh .`
- `pwsh -NoProfile -File scripts/validate_project_governance.ps1 .`
- `git diff --check`
- Real `talos --mock` (or print mode) binary proof for `/new`, `/resume`, `/fork`
- Real PERM-002 allow-once-then-auto scenario captured

### Branch, Worktree And Checkpoint Plan

- Recommended branch: `main` (project uses main-only with atomic commits; the
  release-managed profile treats `main` as the work branch).
- Recommended worktree: current workspace on `main`; on-demand worktree only if a
  blocking conflict arises mid-iteration.
- Commit after T1-T3, T4-T5, T6-T7, T8, T9, T10 with conventional-commit messages
  scoped per crate (`feat(permission): ...`, `feat(cli): /new and /resume ...`,
  `feat(cli): /fork ...`, `docs(backlog): PERM-002 / SESSION-001-B/C Complete`,
  `chore(iterations): I041 → Complete`).
- Do not force-push, rebase published history, or modify unrelated user changes.
- Checkpoint append after every task item boundary (see Checkpoint section below).

### Allowed Permissions And External Actions

Proposed authorization:

- Read/edit repository files; run format, build, tests, local fixture processes, and
  governance scripts.
- Make local commits on `main` after gates pass.
- Use network only if Cargo must fetch an already-declared dependency; do not add a
  dependency without ADR/dependency review.
- Push to `origin/main` only if explicitly confirmed per phase boundary.
- No release, tag, deployment, remote service mutation, paid API, or external account
  action.

### Destructive Or Irreversible Operations

None authorized. No force push, history rewrite, user-session deletion, database
migration, release, or deployment. Temporary test files/processes must be isolated
and cleaned up.

### Time, Cost And Resource Limits

- Suggested unattended window per phase: up to 6 hours wall time.
- Suggested iteration window: 4 weeks (2026-06-22 → 2026-07-20).
- Monetary spend: zero.
- Retry a failing deterministic command at most twice after a concrete fix or
  environment change.
- Keep test output/files bounded; do not download optional models, plugins, or large
  assets.

### Failure, Retry And Fallback Policy

- Fix root causes within the confirmed scope; do not weaken tests or permissions to
  obtain green.
- After two failed implementation approaches for the same blocker, record evidence
  and stop that dependency chain.
- Optional work is deferred to the named backlog owner; required gate failure leaves
  the task `Partial` / `Blocked`.
- Stop before public API breaking changes, new `unsafe`, new runtime dependency,
  permission model changes beyond PERM-002 scope, destructive actions, credentials,
  external cost, or contradictory requirements unless an existing ADR clearly
  authorizes the exact action.

### Default Decisions For Foreseeable Ambiguity

- Prefer Rust-native / existing project abstractions over new dependencies.
- Prefer glob patterns over regex for resource matching.
- Prefer preserving the old session on any uncertainty in transition.
- Use `Ask` as the safe default when resource extraction fails.
- Keep `tool_name`-only rules as exact-match fallback for backward compatibility.
- Choose the smallest reversible implementation that delivers the I041 MVP.
- Preserve published iteration baselines; route unrelated findings to residual
  backlog items.

### Residual-Work Destination

- Skill bodies / references activation: SKILL-002.
- Approval dialog UX: TUI-008.
- Runtime rule editing UI: new focused backlog story.
- Session deletion / rename / cross-workspace resume: future SESSION-001 children.
- Regex resource patterns: future PERM-002 follow-up.
- Unresolved architecture/security decisions: a new focused backlog Story and ADR
  when required.

## Consolidated Confirmation

Confirmed by the user on 2026-06-22 with: `确认合同并启动`.

Approved contract (initial):

- Deliver I041 MVP over 4 weeks: PERM-002 + SESSION-001-B + SESSION-001-C.
- Use `main` branch; current workspace; on-demand worktree only if a blocking
  conflict arises.
- Edit, test, commit locally; push to `origin/main` only if explicitly confirmed at
  each phase boundary.
- Atomic commits per crate/scope (PERM-002 → `feat(permission)`, slash commands
  → `feat(cli)`, governance sync → `chore(governance)`/`docs(backlog)`).
- Time box: 4-week iteration (2026-06-22 → 2026-07-20); up to 6 hours per phase.
- Retry policy: at most two concrete repair approaches per blocker; otherwise stop
  and record evidence.
- Zero monetary spend; no release, tag, deployment, migration, or destructive
  operations.
- Read/edit/commit authority is granted; push authority requires per-phase
  confirmation.

### Contract Amendments (2026-06-22, before user departure)

Confirmed by the user on 2026-06-22 with answers to the three departure questions:

1. **Push authority (one-time blanket)**: After phase boundary verification passes
   (T5, T7, T8, T10), push to `origin/main` without per-phase confirmation. Failure
   to pass verification still requires a stop-and-ask.
2. **Force-push exception (this session)**: Forward-only fixes (e.g., missing model
   tag, accidental commit content) are authorized for this session. Substantive
   history rewrites still require per-incident special approval, per AGENTS.md
   Git Rule 5.
3. **Acceptance gate failure**: After two concrete repair attempts, move the failed
   item to Residual Work and continue with the next item. Checkpoint documents
   the exact failure and the next item to start.

## Checkpoints

### Checkpoint 0 - Start

```text
Completed task items: T1-T10 inventoried; consolidated confirmation recorded (initial + 3 contract amendments)
Current state and artifacts: I041 iteration doc published at docs/iterations/I041-interactive-session-lifecycle-permission-ux.md; this task record created at docs/tasks/2026-06-22-i041-interactive-session-lifecycle-permission-ux.md; HEAD = 651f320 (force-pushed commit chain)
Commands/checks and actual results: governance validator passed (0 warnings) after I040 closure + I041 activation + EVOLUTION.md lesson #22
Open risks or deviations: none yet
Next task item: T1-T5 (PERM-002 foundation + extractor + config + always-approve scoping + closure) delegated to a deep agent; results verified at T5 boundary
Recovery or resume instruction: re-read this record; current HEAD = 651f320 (main); next gate = T1 unit tests for nature+resource matching; if T1-T5 fail, fall back to forward-only + push at T5 boundary per amendment #1
```

### Checkpoint 1 - T1-T5 delegated (2026-06-22)

```text
Completed task items: T1-T5 delegated to deep agent bg_10e06d59 (session ses_11134db13ffeEeU6m2qXm3gJN1)
Current state and artifacts: HEAD = f139f01 (contract amendments committed + pushed); 3 contract amendments recorded; T1-T5 implementation in progress
Commands/checks and actual results: pre-delegation governance validator passed; contract amendment commit f139f01 pushed
Open risks or deviations: none yet; user away until next session boundary
Next task item: await T1-T5 completion; verify with cargo test --workspace + cargo clippy --workspace -- -D warnings
Recovery or resume instruction: re-read this record; current HEAD = f139f01; pending background task = bg_10e06d59; if task is still pending, retrieve via background_output; if task completed, verify the 5 commits exist and run cargo test/clippy
```

### Checkpoint 2 - T1-T5 Complete (2026-06-22)

```text
Completed task items: T1, T2, T3, T4, T5 — PERM-002 fully landed
Current state and artifacts: HEAD = 6f4f647 on main; 5 atomic commits ahead of 35e7b66 (pushed); iteration I041 Verification Evidence section populated
Commands/checks and actual results:
- cargo test --workspace: 0 failures (workspace tests + 51 talos-permission tests)
- cargo clippy --workspace -- -D warnings: clean
- PERM-002 backlog: status Refinement -> Complete (I041), acceptance checkboxes ticked
- iteration I041 doc: T1-T5 Verification Evidence recorded
Open risks or deviations: none; all 5 atomic commits carry [model:MiniMax-M3] tag
Next task item: T6 — SESSION-001-B /new BuiltinCommand
Recovery or resume instruction: re-read this record; current HEAD = 6f4f647 (main); if T6/T7/T8 work is paused, the next agent should: 1) re-verify HEAD is 6f4f647; 2) read SESSION-001-A-runtime-transition-service.md for the transition API; 3) read SESSION-001-B-new-resume.md acceptance criteria; 4) consume SessionTransition::prepare(New) and prepare(Resume)
```
