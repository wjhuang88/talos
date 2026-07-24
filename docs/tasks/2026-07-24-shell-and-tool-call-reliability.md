# Long Task: Shell And Tool-Call Reliability Package

| Field | Value |
|---|---|
| Created | 2026-07-24 |
| Status | Planned (deferred — hand off after the current iteration closes) |
| Owner on handoff | (assign developer/agent at dispatch) |
| Author | Sisyphus [model:claude-opus-4.8] |
| Source | User request 2026-07-24: bundle two hang bugs + shell cross-platform/timeout work into one dispatchable package |

## Outcome

The shell/command execution tools and the text-based tool-call path are reliable:
no turn or command hangs indefinitely, and the `bash` tool works on Windows.
Specifically:

1. A model that reuses tool-call ids (e.g. `"id":"call_0"`) over a text-protocol
   session can no longer wedge the turn (PROVIDER-004).
2. The `bash` tool's timeout is a hard deadline that continuous output cannot
   reset (TOOL-023-A).
3. The execution timeout defaults to 300s, is configurable per-call and globally
   (TOOL-023-B).
4. On Windows the `bash` tool runs PowerShell under a Windows-appropriate name
   (TOOL-023-C).

## In Scope

- The four backlog stories below, each already carrying full detail and
  acceptance:
  - `docs/backlog/active/PROVIDER-004-text-tool-call-id-collision.md`
  - `docs/backlog/active/TOOL-023-A-bash-timeout-fix.md`
  - `docs/backlog/active/TOOL-023-B-configurable-timeout-default.md`
  - `docs/backlog/active/TOOL-023-C-windows-powershell.md`
- Epic context: `docs/backlog/active/TOOL-023-cross-platform-shell-and-timeout.md`

## Out Of Scope

- Windows resource hardening (CPU/memory rlimits, Job Objects) — excluded per
  ADR-007; PowerShell children get env sanitization only.
- Raising the 600s max timeout clamp — deferred; only the default becomes 300s.
- `exec` shell replacement — `exec` is argv-only and cross-platform already.
- Fixing model behavior — Talos must tolerate duplicate/placeholder tool-call ids
  regardless of what the model emits.
- Any push, tag, release, or deployment (see permissions below).

## Ordered Task Items

Execution order prioritizes the two hang bugs (user's active pain), then the
enhancements. See the table under "Task Item Template" below.

## Dependencies And Prerequisites

- Dispatch ONLY after the current iteration closes (user instruction).
- T1 (PROVIDER-004) and T2 (TOOL-023-A) are independent of each other (different
  files) and may proceed in either order or in parallel; both must precede sign-off.
- T3 (TOOL-023-B) depends on T2 (a configurable/raised default is meaningless
  while the bash timer can be reset).
- T4 (TOOL-023-C) depends on T2 (shares the repaired bash execution path).

## Artifacts And State Owners To Update

- The four story files (status + `Completion Commit:` on completion).
- `docs/backlog/active/TOOL-023-cross-platform-shell-and-timeout.md` (Epic children table).
- `docs/backlog/PRODUCT-BACKLOG.md` rows for each story.
- `docs/BOARD.md` (mirror, after owner docs).
- User-facing docs where a story requires: `README.md` / `README.zh-CN.md`,
  `docs/reference/config.reference.toml`, site capabilities (TOOL-023-B/C).
- New ADR for the Windows shell substitution (TOOL-023-C).

## Validation And Acceptance Evidence

Per item, the story's own acceptance is authoritative. Package-level gate:

- `cargo fmt --all -- --check` clean
- `cargo clippy --workspace --locked -- -D warnings` → 0 warnings
- `cargo test --workspace --locked` → 0 failures
- `scripts/validate_project_governance.sh .` → 0 warnings
- New repro tests exist for each hang bug (they must fail/hang against pre-fix code).

## Branch, Worktree And Checkpoint Plan

- Work on `main` or a feature branch per the developer's team convention; if a
  branch is used, do not merge without the user's review.
- Record a checkpoint (SOP Checkpoint block) at each item boundary and before
  stopping/handing off.
- Commits follow `docs/sop/GIT-WORKFLOW.md`: conventional commits, scope =
  affected crate, `[model:<name>]` for agent-authored commits, one logical change
  per commit.

## Allowed Permissions And External Actions

- ALLOWED: edit code, add/modify tests, run `cargo` (fmt/clippy/test/check),
  run `scripts/validate_project_governance.sh`, create local commits.
- NOT ALLOWED without a fresh, explicit user approval: `git push`, opening PRs,
  tagging, releasing, deploying, publishing, migrations, spending, network
  services beyond dependency fetch, and any destructive/irreversible operation.
- Rationale: permission to edit and commit does NOT imply permission to push or
  release (SOP Consolidated Confirmation).

## Destructive Or Irreversible Operations

None permitted. No force-push, no history rewrite, no tag moves.

## Time, Cost And Resource Limits

- No unattended run exceeding local `cargo test --workspace` time without a
  checkpoint. Stop and checkpoint if a single item exceeds a reasonable local
  build/test cycle without progress.

## Failure, Retry And Fallback Policy

- Each item has a Fallback in the table. On repeated failure of an item's
  completion gate, checkpoint the failure with actual command output and stop for
  user input rather than shotgun-patching.
- A failed provider/shell fix that risks regressing native tool calls or Unix
  shell behavior must be reverted to a known-good state before proceeding.

## Default Decisions For Foreseeable Ambiguity

- PROVIDER-004 fix location: text tool-call path forces a unique id (ignore
  model-supplied id / synthesize), symmetric with native `finalized_tool_call_id`.
  Do NOT merely widen the in-turn duplicate guard (that converts a hang into an
  error, not a working turn).
- TOOL-023-A fix: single-shot deadline (the `exec` detached-reader pattern) or
  wrap the loop in `tokio::time::timeout`; must preserve kill + drain-after-kill
  and the `[timeout]` marker.
- TOOL-023-B default: 300s; max clamp stays 600s; precedence per-call > global
  config > 300s built-in.
- TOOL-023-C: PowerShell (not cmd); Windows tool name `shell`/`powershell`;
  env-scrub-only hardening.

## Residual-Work Destination

- Any non-blocking follow-up (e.g. surfacing dropped orphan tool results as a
  warning in `openai_request.rs`, grandchild-process kill scope on Unix) is
  recorded in the relevant story's residual section, not silently dropped.

## Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T1 | PROVIDER-004: force unique text tool-call ids | Text path (anthropic_stream + openai_sse) assigns unique ids; assistant msg + tool result share the id; repro test for cross-turn `call_0` collision | None | Story acceptance met; repro test passes; `cargo test --workspace --locked` + clippy clean | If unique-id approach regresses native pairing, revert and consult user before widening the duplicate guard | Planned |
| T2 | TOOL-023-A: fix bash timeout defeated by continuous output | `bash_tool.rs` single-shot deadline; kill+drain preserved; repro test with continuous-output command | None | Story acceptance met; timeout fires within bound; tests + clippy clean | If restructuring risks output loss, wrap loop in `tokio::time::timeout` preserving drain; if still failing, checkpoint + stop | Planned |
| T3 | TOOL-023-B: 300s default + `[tools].default_timeout_secs` | bash/exec default 300s; config field wired via configuration.rs; precedence tests; docs updated | T2 | Story acceptance met; precedence + clamp tests pass; README/config.reference updated; tests + clippy clean | If config wiring is broad, ship default change first and record config-field as residual | Planned |
| T4 | TOOL-023-C: Windows PowerShell shell + per-platform name | `#[cfg(windows)]` PowerShell path; Unix unchanged; Windows tool name; env-scrub only; new ADR; docs | T2 | Story acceptance met; both targets compile; name-selection unit test; ADR recorded; manual Windows walkthrough recorded as gate | Windows CI absent — if manual Windows walkthrough unavailable, leave story `Review` pending that gate, do not mark Complete | Planned |

## Handoff Notes

- This package is DEFERRED by user instruction: do not start until the current
  iteration is closed. At dispatch, assign an owner, move Status to `In Progress`,
  and record the consolidated confirmation per SOP before executing.
- All four stories already carry full acceptance detail — read them first
  (`Required Reads` in each). This task file is the execution contract; the
  stories are the specification.
- Two related backlog commits are already on `main`: `7ce9789` (TOOL-023 epic)
  and `54b2d93` (PROVIDER-004). Neither is pushed; confirm remote state at dispatch.

## Checkpoint Log

(Append SOP Checkpoint blocks here at each item boundary during execution.)
