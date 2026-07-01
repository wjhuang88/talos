# 2026-07-01 Self-Bootstrap Rehearsal: T40 Code Slice

**Rehearsal number**: 2
**Plan item**: T52
**Session date**: 2026-07-01
**Runtime**: Talos 0.2.0 on Darwin arm64
**Change type**: small code change
**External assistance**: labeled below

## Objective

Attempt a small code change (adding `ToolProvenance::Plugin` variant across 3 crates) and evaluate
whether Talos could serve as the primary development runtime for this class of work.

## Scope

- **In scope**: Record evidence for the T40 code slice (enum variant + 3 match arms + 8 tests across
  talos-core, talos-conversation, talos-tui); evaluate Talos's ability to drive this workflow.
- **Out of scope**: Claim REL-002 compliance; assert Talos was the primary runtime.

## Environment

- Talos version: `talos 0.2.0`
- Provider/model used: external opencode agent, model glm-5.2; Talos was not the primary agent
  runtime for this rehearsal.
- Workspace: `/Users/GHuang/WorkSpace/RustProjects/talos`
- Starting commit: `acde17a` (Month-2 closeout)
- Ending commit: `f4423b9` (T40 delivery)

## Execution Record

| Step | Tool(s) used | Outcome | Notes |
|---|---|---|---|
| 1 | External agent: codebase exploration (grep, read, explore subagents) | success | Mapped ToolProvenance definition, all match sites, rendering paths, existing test patterns. |
| 2 | External agent: code edits (4 files) | success | Added Plugin variant to talos-core/tool.rs; updated 3 exhaustive match arms; added 8 tests. |
| 3 | External agent: validation (cargo check/clippy/test/fmt/governance) | success | All gates passed on first attempt — compiler-enforced exhaustiveness caught all match sites. |
| 4 | `cargo run -p talos-cli -- --version` | success | Runtime smoke: `talos 0.2.0`. |
| 5 | External agent: owner-doc updates + commit | success | Updated PLUGIN-001, ADR-028 correction, four-month plan, BOARD.md. |

## External Assistance

| Step | What was needed | Who provided it | Why Talos could not |
|---|---|---|---|
| 1 | Multi-file codebase exploration and pattern mapping | External opencode agent (explore subagents) | Talos lacks parallel explore/grep agents and multi-angle codebase analysis capabilities. |
| 2 | Precise multi-file code edits (enum variant + match arms + tests) | External opencode agent | Talos's edit tool exists but lacks the orchestration layer for coordinated multi-crate changes, test generation, and compiler-error-driven fix loops. |
| 3 | Validation orchestration (fmt → check → clippy → test → governance) | External opencode agent | Talos has bash tool but lacks the structured validation pipeline and error-recovery workflow. |
| 5 | Owner-doc governance updates and conventional-commit formatting | External opencode agent | Talos lacks governance-aware documentation tooling and commit workflow integration. |

## Validation Evidence

- `cargo check --workspace`: pass (T40 commit f4423b9).
- `cargo test --workspace`: pass — 1264+ tests across all crates, 0 failed (Month-3 session close).
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `cargo run -p talos-cli -- --version`: `talos 0.2.0`.

## Commit

- Commit SHA: `f4423b9`
- Commit message: `feat(core): add ToolProvenance::Plugin variant and rendering paths (#T40) [model:glm-5.2]`
- Files changed: 9 (4 source/test + 5 docs), +255/-8 lines.

## Gaps Exposed

| Gap | Severity | Blocking REL-002? | Recommended fix |
|---|---|---|---|
| Talos cannot orchestrate multi-crate code changes (explore → edit → validate → commit). | high | yes | Integrate structured tool orchestration: parallel explore, compiler-error-driven edit loops, validation pipeline. TODO-001 (session todo list) partially addresses planning; TUI-016 addresses command UX. |
| Talos lacks governance-aware commit tooling (conventional commits, owner-doc-first ordering, BOARD sync). | high | yes | Add git-commit tool with conventional-commit template and governance validation pre-commit hook. |
| Talos cannot generate tests from patterns (existing test style → new variant tests). | medium | no | Add test-generation capability or improve agent prompting for test-first workflows. |
| Talos's edit tool is basic (single replacement, no AST-aware refactoring). | medium | no | Consider tree-sitter-aware edit tool (TOOL-008 Phase 2 enables this infrastructure). |
| Session-level task tracking (TODO-001) not yet implemented. | medium | no | TODO-001 (issue #8) would let the agent track multi-step plans within a session. |

## Assessment

- **Self-bootstrap coverage**: ~5%. Talos provided CLI/runtime smoke validation only. All planning,
  code editing, test generation, validation orchestration, and governance documentation were performed
  by the external opencode agent. This is marginally worse than T38 (10%) because T52 attempted code
  changes (harder) rather than docs-only (easier), exposing more gaps.
- **Would this rehearsal satisfy REL-002?**: No. REL-002 requires Talos as the primary development
  runtime. The gap is not tool availability (Talos has read/write/edit/bash/grep) but orchestration:
  the agent loop cannot yet coordinate multi-file changes, validation pipelines, and governance
  workflows without external orchestration.
- **Ready for the next rehearsal level?**: Conditionally. T61 (third rehearsal) should attempt an
  architecture-sensitive slice. Before that, TODO-001 (session todo list) and improved agent prompting
  for multi-step workflows should land. The critical path is: better agent orchestration → more of the
  session driven by Talos → higher self-bootstrap coverage.

## Progress Since T38

| Dimension | T38 (rehearsal 1) | T52 (rehearsal 2) | Trend |
|---|---|---|---|
| Change type | documentation-only | small code change | harder |
| Files touched | 1 (evidence record) | 9 (source + tests + docs) | broader |
| External assistance | 100% (external Codex agent) | 95% (external opencode agent) | marginal |
| Gaps identified | 3 | 5 | more exposed |
| Validation | deferred to T39 | full workspace test passed | better |

## Recovery

- To resume: `git checkout f4423b9` and read this evidence record.
- Next rehearsal (T61) should attempt: an architecture-sensitive slice with at least one step driven
  through Talos runtime paths (e.g., using `talos-runtime` SDK to execute a tool call that produces a
  code change). The goal is to move from ~5% to >15% self-bootstrap coverage.
