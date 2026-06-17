# I026: Approval UX + Git Tools + Prompt Optimization

**Status**: Active
**Started**: 2026-06-17
**Depends On**: TOOL-002 (Approval bug), ARCH-002 (Prompt template/cache), GIT-001 (Git tools)

## Outcome

Fix approval UX, implement comprehensive Git tools (read + write), add dynamic prompt
templates with cache optimization, and validate project documentation.

## Confirmed Decisions

| Decision | Choice |
|----------|--------|
| Git write ops (push/pull/checkout) | host git fallback, error if git not found |
| Approval rendering | inline below tool call line |
| Dynamic template slots | tool_protocol_hint, workspace_info, model_info, datetime |
| datetime placement | append prompt section (end of prompt) to minimize cache breakage |
| Cache control | Anthropic explicit cache_control + OpenAI message ordering stability |
| Git auto-commit | No — all git ops are explicit LLM tool calls |
| Tree tool format | ASCII tree with box-drawing characters |

## Stories

### Group A: Approval UX

#### S1: Approval-Tool Call Ordering Fix (~100 LOC, P1)

Approval prompt appears immediately after the tool call line, before tool executes.

### Group B: Git Tools

#### S2: GIT-001 P0 — Read-Only Git Tools (~350 LOC, P1)

Tools: git_status, git_diff, git_log, git_show, git_branch_list
Implementation: gix crate v0.84+ with minimal features
Permission: ToolNature::Read (auto-allow)

#### S3: GIT-001 P2 — Write Git Tools (~400 LOC, P2)

Tools:
- git_add (gix index native)
- git_commit (gix native)
- git_push (host git fallback)
- git_pull (host git fallback)
- git_checkout (host git fallback)

Permission: ToolNature::Write/Execute (Ask). No auto-commit.
Host git invoked directly (no sh -c), allowlisted shapes, structured args.

### Group C: Prompt Optimization

#### S4: Dynamic Prompt Template (~100 LOC, P2)

Convert identity.txt to template with slot variables.
Template engine: simple key substitution via HashMap.

#### S5: Prompt Cache Control (~120 LOC, P2)

Anthropic: emit cache_control markers on system message content blocks.
OpenAI: ensure stable message ordering for prefix caching.

### Group D: Small Tools + Validation

#### S6: TOOL-003 P3 — Tree Tool (~50 LOC, P3)

ASCII tree visualization with box-drawing characters.
Parameters: path (optional), max_depth (optional, default 3).
Permission: ToolNature::Read (auto-allow).

#### S7: ARCH-002 Phase 1 — Documentation Validation (Research, P1)

Verify all documentation against actual implementation.

## Exit Criteria

- [ ] All 7 stories complete
- [ ] cargo clippy --workspace -- -D warnings passes
- [ ] cargo test --workspace passes
- [ ] BOARD.md updated with final state
