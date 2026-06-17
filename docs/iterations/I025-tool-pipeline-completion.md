# I025: Tool Pipeline Completion

**Status**: Complete
**Started**: 2026-06-17
**Completed**: 2026-06-17
**Depends On**: TOOL-002 (P0 Complete), TOOL-003 (P0 Complete), CODE-002 (Complete)

## Outcome

Close remaining tool pipeline gaps and deliver the next tier of POSIX tools and TUI rendering
enhancements. This iteration consolidates residual work from TOOL-002, TOOL-003, and the
stream_markdown fence detection into a single trackable iteration.

## Stories

### S1: TOOL-002 P1-P2 Residual

| Field | Value |
|-------|-------|
| Source | TOOL-002 #6, #7, #8, #10 |
| Est | ~150 LOC |
| Priority | P1 |

**Scope**:
- #7: Schema validation on tool inputs before execution
- #8: Tool call deduplication within a single turn batch
- #6: Extract shared ToolCallPipeline in talos-provider (optional, if effort permits)
- #10: Wire `Message::System`/`Message::Context` (optional, if effort permits)

**Acceptance**:
- [ ] Invalid tool inputs rejected with clear error message
- [ ] Duplicate tool calls within a turn are deduplicated
- [ ] `cargo test -p talos-agent` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes

### S2: TOOL-003 P1 — diff + stat

| Field | Value |
|-------|-------|
| Source | TOOL-003 P1 section |
| Est | ~200 LOC |
| Priority | P2 |

**Scope**:
- `diff` tool: compare two files, output unified diff (crate: `similar`)
- `stat` tool: file metadata including MIME type (crate: `infer`)
- Register both in all 4 registry builders
- Permission: auto-allow (read-only)

**Acceptance**:
- [ ] `diff` produces unified diff output
- [ ] `stat` returns size, type, permissions, MIME
- [ ] Both registered in all 4 builders
- [ ] Unit tests for each tool
- [ ] `cargo clippy -p talos-tools -- -D warnings` passes

### S3: Fence Info-String Misdetection Fix

| Field | Value |
|-------|-------|
| Source | TOOL-003 fence bug section |
| Est | ~30 LOC |
| Priority | P1 |

**Scope**:
- Replace `Option<String>` fence_marker with `(String, usize)` to track backtick count
- Add whitespace-only check in `is_matching_fence_close`
- Update `ClassifierState::Holding` fence_marker field type

**Acceptance**:
- [ ] `` ```rust `` opening NOT treated as closing fence
- [ ] `` ```` `` (4 backticks) NOT closed by inner `` ``` `` (3 backticks)
- [ ] Existing code block detection not regressed
- [ ] Tests: `test_fence_info_string_not_closed`, `test_fence_nested_backtick_count`

### S4: Mermaid Code Block Rendering

| Field | Value |
|-------|-------|
| Source | TOOL-003 Mermaid rendering section |
| Est | ~50 LOC |
| Priority | P2 |

**Scope**:
- Add `mermaid-text = "0.56"` to `talos-tui/Cargo.toml`
- Add mermaid check in `render_block_lines()` before syntax highlighting
- New method `render_mermaid_block()` with fallback to plain code block

**Acceptance**:
- [ ] `` ```mermaid `` code blocks render as Unicode box-drawing diagrams
- [ ] Invalid Mermaid source falls back to plain code block display
- [ ] Non-mermaid code blocks unaffected (regression test)
- [ ] Tests: `test_mermaid_block_renders_diagram`, `test_mermaid_fallback_on_invalid_syntax`

### S5: ToolNature Attribute (Optional)

| Field | Value |
|-------|-------|
| Source | TOOL-003 permission architecture section |
| Est | ~100 LOC |
| Priority | P3 |

**Scope**:
- Add `ToolNature` enum (Read/Write/Execute/Network) to `AgentTool` trait
- Implement `nature()` on all existing tools
- Refactor `PermissionEngine` to use nature-based decisions
- Remove `is_file_tool()` name matching

**Acceptance**:
- [ ] `ToolNature` enum defined in `talos-core/tool.rs`
- [ ] All tools override `nature()` correctly
- [ ] Permission decisions based on nature, not name substrings
- [ ] `is_file_tool()` removed
- [ ] All permission tests pass

## Exit Criteria

- [x] S1-S5 complete
- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] `cargo test -p talos-tools -p talos-permission -p talos-core -p talos-tui` passes
- [x] BOARD.md updated with final state
- [x] TOOL-002 and TOOL-003 acceptance criteria updated

## Retrospective

All 5 stories delivered in a single session:

| Story | Commit | Key Change |
|-------|--------|------------|
| S1 | `d3bc2e7` | Schema validation (`registry.validate_input()` before execute) |
| S2 | `38f6205` | diff + stat tools (similar crate + std::os::unix) |
| S3 | `700c4bc` | Fence info-string fix (backtick count tracking) |
| S4 | `3f0a1b0` | Mermaid rendering (mermaid-text v0.56, 1 dep) |
| S5 | `06b6b3f` | ToolNature enum replaces name-based permission matching |

Key architectural win: S5 eliminated all hardcoded tool-name substring matching in the
permission engine. Adding new tools no longer requires touching `is_file_tool()`,
`default_decision()`, or `add_default_rules()` — just override `nature()` on the tool.

## Out of Scope

- ARCH-002 God module decomposition
- TOOL-003 P2 (image reading, write/edit display block)
- I018 observability work
