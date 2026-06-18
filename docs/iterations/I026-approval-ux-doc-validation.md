# I026: Approval UX + Git Tools + Prompt Optimization

**Status**: Complete
**Started**: 2026-06-17
**Closed**: 2026-06-18
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

**Status**: Complete.

Implemented as inline scrollback approval rendering plus an agent event-flow fix. During provider
streaming, full `ToolCall` display events are buffered; `run_streaming()` emits each `ToolCall`
immediately before that tool executes, then emits its `ToolResult` before moving to the next tool.
This gives the TUI the desired `ToolCall -> ToolApprovalRequest(if Ask) -> ToolResult` ordering.

### Group B: Git Tools

#### S2: GIT-001 P0 — Read-Only Git Tools (~350 LOC, P1)

Tools: git_status, git_diff, git_log, git_show, git_branch_list
Implementation: gix crate v0.66 with minimal features
Permission: ToolNature::Read (auto-allow)

**Status**: Complete.

#### S3: GIT-001 P2 — Write Git Tools (~400 LOC, P2)

Tools:
- git_add (gix index native)
- git_commit (gix native)
- git_push (host git fallback)
- git_pull (host git fallback)
- git_checkout (host git fallback)

Permission: ToolNature::Write/Execute (Ask). No auto-commit.
Host git invoked directly (no sh -c), allowlisted shapes, structured args.

**Status**: Complete.

### Group C: Prompt Optimization

#### S4: Dynamic Prompt Template (~100 LOC, P2)

Convert identity.txt to template with slot variables.
Template engine: simple key substitution via HashMap.

**Status**: Complete.

Implemented in `crates/talos-agent/src/prompt.rs`. Supported slots include
`tool_protocol_hint`, `workspace_info`, `model_info`, and `datetime`. Stable slots render in
`prompts/identity.txt`; `datetime` renders in the dynamic runtime section after cacheable prompt
markers to avoid invalidating the stable prefix.

#### S5: Prompt Cache Control (~120 LOC, P2)

Anthropic: emit cache_control markers on system message content blocks.
OpenAI: ensure stable message ordering for prefix caching.

**Status**: Complete.

System prompt cache markers now travel on `Message::System`. Anthropic converts them to top-level
`system` text blocks with `cache_control: { type: "ephemeral" }`; OpenAI keeps the system message
first and does not emit provider-specific cache-control fields.

### Group D: Small Tools + Validation

#### S6: TOOL-003 P3 — Tree Tool (~50 LOC, P3)

ASCII tree visualization with box-drawing characters.
Parameters: path (optional), max_depth (optional, default 3).
Permission: ToolNature::Read (auto-allow).

**Status**: Complete.

#### S7: ARCH-002 Phase 1 — Documentation Validation (Research, P1)

Verify all documentation against actual implementation.

**Status**: Complete for active owner docs touched by this iteration.

Validation findings fixed in this pass:
- I026 status now reflects S1-S7 completion and records the completed approval-ordering fix.
- ARCH-002 prompt-template/cache section now reflects the implemented S4/S5 behavior.
- GIT-001 and Product Backlog summaries now reflect delivered Git tools instead of planned-only
  status.
- BOARD.md now derives from the updated owner docs.
- README now reflects the current built-in tool count and I026 prompt/cache/Git scope.

Residual documentation debt:
- A full historical ADR-by-ADR audit remains part of broader ARCH-002 Phase 1/Phase 2 work. This
  iteration synchronized the active planning and user-facing docs required for the delivered
  changes.

## Exit Criteria

- [x] All 7 stories complete
- [x] cargo clippy --workspace -- -D warnings passes
- [x] cargo test --workspace passes
- [x] BOARD.md updated with final state

## Residual Work

- **ARCH-007 — Workspace `clippy --all-targets` cleanup**: the I026 verification command
  `cargo clippy --workspace -- -D warnings` (lib + bin targets only) passes, but
  `cargo clippy --workspace --all-targets -- -D warnings` surfaces ~35 pre-existing
  `clippy::unwrap_used` warnings in `crates/talos-conversation/src/engine_tests.rs` (and
  possibly elsewhere) because `[workspace.lints.clippy] unwrap_used = "warn"` is set in the
  root `Cargo.toml`. These warnings predate I026 and are not caused by any I026 change, but the
  verification scope gap is now registered as a follow-up cleanup story. See
  `docs/backlog/active/ARCH-007-clippy-all-targets-cleanup.md`.

## Verification Log

- `cargo check --workspace` — passed after core/provider prompt cache metadata changes.
- `cargo test -p talos-agent prompt::tests` — passed.
- `cargo test -p talos-provider build_request_body` — passed.
- `cargo test -p talos-agent test_streaming_tool_events_are_interleaved_per_tool` — passed.
- `cargo test -p talos-agent` — passed.
- `cargo test -p talos-conversation` — passed.
- `cargo test -p talos-tui` — passed.
- `cargo check --workspace` — passed after approval event-flow changes.
- `cargo clippy --workspace -- -D warnings` — passed.
- `cargo test --workspace` — passed (1 ignored timing-sensitive test).
- `scripts/validate_project_governance.sh .` — passed with 0 warnings.

## Closure Verification (2026-06-18)

Re-ran the two unchecked exit-criteria commands before marking Complete:

- `cargo clippy --workspace -- -D warnings` — passed (5.54s, no warnings).
  Scope note: this is the lib + bin target scope; `--all-targets` test-target failures are
  pre-existing and registered as ARCH-007.
- `cargo test --workspace` — passed. All suites green (142 + 91 + 81 + 63 + 55×2 + 46 + 42 +
  36 + 33 + 26 + 25×2 + 13 + 4 + 3 + 2×4 + 1×8 + 0×6 across all crates). One pre-existing
  ignored timing-sensitive test retained.

No I026-introduced regressions found. The iteration is closed; ARCH-007 tracks the only
residual verification scope gap.
