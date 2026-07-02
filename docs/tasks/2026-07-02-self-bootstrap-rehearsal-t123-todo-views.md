# 2026-07-02 Self-Bootstrap Rehearsal: Todo Read-Only Views

**Rehearsal number**: 2
**Plan item**: T123
**Session date**: 2026-07-02
**Runtime**: Talos 0.2.0 validation planner on macOS; Codex remained the primary executor
**Change type**: user-facing feature slice
**External assistance**: Codex performed planning, editing, validation execution, documentation, git,
and issue sync

## Objective

Use the completed validation planning surface during a real Talos repository change and record what
Talos could and could not do for the self-bootstrap loop. The concrete slice was I078/T122:
read-only `/todo` slash/TUI views for active-session todos.

## Scope

- **In scope**: `talos validate plan --profile workspace`; T122 implementation evidence from
  `fd853e3`; governance and issue sync evidence.
- **Out of scope**: claiming a REL-002 qualifying Talos-on-Talos session; executing validation
  commands through Talos; autonomous editing through Talos.

## Environment

- Talos version: `talos 0.2.0`
- Provider/model used: N/A for Talos runtime; Codex used the current session model.
- Workspace: `/Users/GHuang/WorkSpace/RustProjects/talos`
- Starting commit: `fd853e3471f5f13c9f2d6f3c92e94f492186c907`

## Execution Record

| Step | Tool(s) used | Outcome | Notes |
|---|---|---|---|
| 1 | `./target/debug/talos validate plan --profile workspace` | success | Talos listed required workspace checks without executing them. |
| 2 | `./target/debug/talos validate plan --profile workspace --json` | success | JSON output included fmt/check/test/governance checks and prerequisite findings. |
| 3 | Codex terminal/tools | success | Codex had already executed the T122 implementation and validation commands. |
| 4 | GitHub CLI | success | Codex synced issue #8 with T122 Review status. |

## External Assistance

| Step | What was needed | Who provided it | Why Talos could not |
|---|---|---|---|
| T122 implementation | Code edits, tests, docs, commit, push, issue sync | Codex | Talos does not yet run as the primary repository-editing agent in this environment. |
| Validation execution | `cargo`/governance command execution and evidence synthesis | Codex | T108 only implemented read-only validation planning; Phase 2 execution is not implemented. |
| GitHub issue sync | `gh issue comment` | Codex | No Talos-owned issue-sync tool or GitHub connector path is available. |

## Validation Evidence

- `talos validate plan --profile workspace`: passed as read-only plan generation.
- `talos validate plan --profile workspace --json`: passed; JSON named fmt/check/test/governance
  checks and reported workspace/governance prerequisites.
- T122 validation already recorded in `docs/iterations/I078-month3-session-todo-memory-thinking.md`:
  `cargo fmt --all -- --check`; `cargo test -p talos-conversation`; `cargo test -p talos-cli`;
  `cargo test -p talos-tui`; `cargo clippy -p talos-conversation -p talos-cli -p talos-tui --
  -D warnings`; `cargo check --workspace`; `scripts/validate_project_governance.sh .`.

## Commit

- Commit SHA: `fd853e3471f5f13c9f2d6f3c92e94f492186c907`
- Commit message: `feat(cli): add read-only todo slash views (#T122,#8) [model:gpt-5]`
- Files changed: 18 files; conversation slash parsing/types, CLI todo view runtime module, TUI
  panel rendering, README, and governance docs.

## Gaps Exposed

| Gap | Severity | Blocking REL-002? | Recommended fix |
|---|---|---|---|
| Validation planner cannot execute or store command evidence. | High | Yes | Implement Phase 2 explicit validation execution with allowlisted profiles and recorded evidence. |
| Talos is not yet the primary code-editing/runtime agent for repository work. | High | Yes | Continue T123/T132 rehearsals only as gap evidence until Talos can drive edits and tools directly. |
| GitHub issue sync still depends on external CLI use. | Medium | No for local development; yes for unattended governance parity | Add an issue-sync owner path or document issue sync as external release-governance assistance. |

## Assessment

- **Self-bootstrap coverage**: approximately 15%. Talos produced the validation plan; Codex performed
  implementation, validation execution, docs, git, push, and issue sync.
- **Would this rehearsal satisfy REL-002?**: No. It is useful gap evidence, but Codex remained the
  primary executor and Talos only provided read-only validation planning.
- **Ready for the next rehearsal level?**: Yes, with conditions. T132 should target an
  architecture-sensitive slice only after a Phase 2 validation execution surface or a clearly labeled
  external-execution gap record is available.

## Recovery

- To resume: `git checkout fd853e3471f5f13c9f2d6f3c92e94f492186c907` and read this evidence record.
- Next rehearsal should attempt: validation evidence capture for one allowlisted profile, or an
  architecture-sensitive read-only planning/review slice with every external action labeled.
