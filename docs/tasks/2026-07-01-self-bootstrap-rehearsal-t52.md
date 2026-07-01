# 2026-07-01 Self-Bootstrap Rehearsal: Talos-Driven Code Slice

**Rehearsal number**: 2
**Plan item**: T52
**Session date**: 2026-07-01
**Runtime**: Talos 0.2.0 on Darwin arm64
**Change type**: small code change
**External assistance**: user provided the prompt; Talos executed the code change autonomously

## Objective

Give Talos a small multi-crate code change task and evaluate how much of the workflow Talos can
drive autonomously as the primary development runtime.

## Scope

- **In scope**: Add a `TestVariant` unit variant to `ToolProvenance`, update all exhaustive match
  sites, verify compilation.
- **Out of scope**: Full test coverage, validation pipeline, governance docs, commit workflow (these
  are evaluated as gaps below).

## Environment

- Talos version: `talos 0.2.0`
- Provider/model used: Talos TUI with configured provider/model
- Workspace: `/Users/GHuang/WorkSpace/RustProjects/talos`
- Starting commit: `241d776` (T52+T53 delivery)

## Execution Record

| Step | Performed by | Tool(s) used | Outcome | Notes |
|---|---|---|---|---|
| 1 | User | CLI | Prompted Talos with: "add TestVariant to ToolProvenance, run cargo check, fix errors" |
| 2 | **Talos** | `read`, `edit` | success | Added `TestVariant` variant to `talos-core/src/tool.rs` with doc comment |
| 3 | **Talos** | `edit` (compiler-guided) | success | Updated `plugin_observation_key()` in `talos-conversation/src/engine.rs` |
| 4 | **Talos** | `edit` (compiler-guided) | success | Updated `build_tool_call_scrollback_line()` in `talos-tui/src/tool_display.rs` |
| 5 | **Talos** | `edit` (compiler-guided) | success | Updated `ToolCallBubble::render()` in `talos-tui/src/widgets.rs` |
| 6 | External | `cargo check --workspace` | pass | Post-hoc verification: all 4 files correct, compiles clean |
| 7 | External | `cargo test` (3 crates) | pass | 285 tests passed, 0 failed |

## External Assistance

| Step | What was needed | Who provided it | Why Talos could not |
|---|---|---|---|
| 1 | Task prompt | User | Talos needs a human to define the task |
| 6-7 | Validation pipeline (cargo check/test) | External agent (opencode) | Talos did not autonomously run `cargo check` or `cargo test` after edits |
| — | Tests for the new variant | Not provided | Talos did not generate unit tests for `TestVariant` |
| — | Commit + governance docs | Not provided | Talos did not commit or update owner docs |

## Validation Evidence

- `cargo check --workspace`: pass (all 4 files compile correctly).
- `cargo test -p talos-core -p talos-conversation -p talos-tui`: 285 passed, 0 failed.
- Code diff: 4 files, +5 lines — minimal, correct, no extraneous changes.

## Code Quality Assessment

| Dimension | Score | Notes |
|---|---|---|
| Correctness | ✅ Excellent | All 3 exhaustive match sites found and updated. No missed callsites. |
| Semantics | ✅ Good | Match arms are reasonable: `"test_variant"` key, `None` rendering (no badge). |
| Documentation | ✅ Good | Added `/// A synthetic tool variant used only in tests.` doc comment. |
| Test coverage | ❌ Missing | No unit tests for the new variant. |
| Formatting | ⚠️ Uncertain | Not verified with `cargo fmt --check` by Talos. |

## Gaps Exposed

| Gap | Severity | Blocking REL-002? | Recommended fix |
|---|---|---|---|
| Talos does not autonomously run validation (cargo check/test/clippy/fmt) after code changes. | high | yes | Add a "validate" tool or post-edit validation hook that runs the standard pipeline. |
| Talos does not generate tests for new code. | medium | no | Improve agent prompting for test-first workflows; add test-generation capability. |
| Talos does not commit changes or update governance docs. | high | yes | Add git-commit tool with conventional-commit format and governance validation. |
| Talos does not run `cargo fmt` to enforce style. | low | no | Add format check to the validation pipeline. |

## Assessment

- **Self-bootstrap coverage**: **~45%** for this task. Talos autonomously performed the core
  development work: reading code, adding the variant, finding all match sites via compiler errors,
  and making correct edits across 3 crates. The remaining 55% (tests, validation, commit, governance)
  required external orchestration.
- **Would this rehearsal satisfy REL-002?**: No. REL-002 requires Talos as the **primary** runtime
  for the full development cycle (code → test → validate → commit → document). Talos handled the code
  edit phase but not the validation/commit/documentation phases.
- **Ready for the next rehearsal level?**: Yes, conditionally. T61 (third rehearsal) should attempt
  an architecture-sensitive slice. The critical improvement needed is an autonomous validation loop:
  Talos should run `cargo check` after edits, read the errors, fix them, and iterate until clean.

## Progress Since T38

| Dimension | T38 (rehearsal 1) | T52 (rehearsal 2) | Trend |
|---|---|---|---|
| Change type | documentation-only | small code change (multi-crate) | ✅ harder, succeeded |
| Primary runtime | External (Codex) | **Talos** (for code edits) | ✅ major improvement |
| Self-bootstrap coverage | ~10% | **~45%** | ✅ significant jump |
| Files touched correctly | 1 (evidence record) | 4 (source code across 3 crates) | ✅ |
| Compiler-guided fix loop | n/a | ✅ Talos followed compiler errors to all match sites | ✅ new capability proven |
| Tests generated | n/a | ❌ not generated | needs improvement |
| Validation pipeline | deferred | ❌ not run autonomously | needs improvement |
| Commit + governance | n/a | ❌ not performed | needs improvement |

## Key Insight

Talos's compiler-error-driven fix loop works well. When the Rust compiler reports non-exhaustive
match errors, Talos correctly reads the errors, identifies the file/line, and adds the appropriate
match arm. This is the most promising path toward higher self-bootstrap coverage: the compiler acts
as a structured oracle that guides Talos to exactly the right edit locations.

The gap is not in code editing — it's in the surrounding workflow: validation, testing, committing,
and governance. These are orchestration-level capabilities that need tool support or agent-prompting
improvements.

## Recovery

- To resume: changes are uncommitted in working tree at `241d776`.
- Next rehearsal (T61) should attempt: an architecture-sensitive slice with an autonomous validation
  loop (Talos runs cargo check → reads errors → fixes → repeats until clean → commits).
