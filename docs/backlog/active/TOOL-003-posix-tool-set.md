# TOOL-003: POSIX Tool Set — Embedded Rust Utilities**Status**: Planned  
**Priority**: P2  
**Depends on**: TOOL-001 (portable file/search baseline), TOOL-002 (tool calling remediation)  
**Related ADRs**: ADR-010 (host dependency discipline), ADR-020 (tree-sitter)

## Problem

Talos currently has 4 file/shell tools (bash, read, write, edit) and 4 AST symbol tools. Agents fall back to `bash` for common operations (grep, find, diff, ls, rm, cp, image read) that could be first-class tools with structured JSON output, permission granularity, and sandbox awareness. Each `bash` invocation is a security surface and permission prompt; dedicated tools reduce that surface.

## Scope

### In Scope

Embed Rust-native implementations of high-value POSIX utilities as built-in tools, reducing host dependency on shell commands. Include multimodal image reading support.

### Out of Scope

- Process management (ps/kill)
- Network utilities (curl/dig)
- Text processing pipelines (sed/awk/tr)
- Archive operations (tar/zip) — deferred to a later slice

## Current Tool Inventory

| Tool | Name | Read-Only | POSIX Equivalent |
|------|------|-----------|------------------|
| BashTool | `bash` | No | `sh` (covers everything indirectly) |
| ReadTool | `read` | Yes | `cat`, `head`, `tail` |
| WriteTool | `write` | No | `touch`, `echo >` |
| EditTool | `edit` | No | `sed -i`, `patch` |
| FindSymbolTool | `find_symbol` | Yes | N/A (AST-level) |
| FindReferencesTool | `find_references` | Yes | N/A (AST-level) |
| ListSymbolsTool | `list_symbols` | Yes | N/A (AST-level) |
| ListImportsTool | `list_imports` | Yes | N/A (AST-level) |

## Proposed New Tools

### P0 — High Agent Utility

#### 1. GrepTool (`grep`)

Search file contents by regex across the workspace.

| | |
|---|---|
| **Tool name** | `grep` |
| **Read-only** | Yes |
| **Parameters** | `pattern` (string, required), `path` (string, optional, default "."), `include` (glob, optional), `max_results` (int, optional, default 50) |
| **Output** | JSON array: `[{file, line_number, line, match_start, match_end}]` |
| **Crate** | `regex` (direct usage, not ripgrep library facade — simpler API, fewer deps) |
| **Crate version** | `regex` 1.x |
| **License** | MIT/Apache-2.0 |
| **Rationale** | Highest-value gap. Agents search codebases constantly. `regex` + `walkdir` (already a dep) covers this without ripgrep's internal crate complexity. |

#### 2. GlobTool (`glob`)

Find files by name pattern.

| | |
|---|---|
| **Tool name** | `glob` |
| **Read-only** | Yes |
| **Parameters** | `pattern` (string, required, e.g. `**/*.rs`), `path` (string, optional, default ".") |
| **Output** | JSON array of file paths |
| **Crate** | `glob` |
| **Crate version** | 0.3.x |
| **License** | MIT/Apache-2.0 |
| **Rationale** | File discovery by pattern is the second most common agent operation. `glob` is zero-dep, battle-tested, 200M+ downloads. |

#### 3. DeleteTool (`delete`)

Delete files or directories.

| | |
|---|---|
| **Tool name** | `delete` |
| **Read-only** | No |
| **Parameters** | `path` (string, required) |
| **Output** | `deleted: {path}` |
| **Crate** | `std::fs` (no external dep) |
| **Rationale** | No native delete capability exists. Currently requires `bash rm`. WriteTool refuses overwrite but can't remove files. |

### P1 — Medium Agent Utility

#### 4. LsTool (`ls`)

List directory contents with metadata.

| | |
|---|---|
| **Tool name** | `ls` |
| **Read-only** | Yes |
| **Parameters** | `path` (string, optional, default "."), `all` (bool, optional, show hidden), `recursive` (bool, optional, default false) |
| **Output** | JSON array: `[{name, type, size, modified}]` |
| **Crate** | `std::fs` (no external dep) |
| **Rationale** | Structured directory listing without shell parsing. More reliable than parsing `ls -la` output from bash. |

#### 5. DiffTool (`diff`)

Compare two files or strings, show unified diff.

| | |
|---|---|
| **Tool name** | `diff` |
| **Read-only** | Yes |
| **Parameters** | `old_path` (string, required), `new_path` (string, required) |
| **Output** | Unified diff text |
| **Crate** | `similar` |
| **Crate version** | 3.x |
| **License** | Apache-2.0 |
| **Rationale** | Code review workflows. `similar` is the standard Rust diff library (142M downloads, used by `insta`). |

#### 6. StatTool (`stat`)

Get file metadata.

| | |
|---|---|
| **Tool name** | `stat` |
| **Read-only** | Yes |
| **Parameters** | `path` (string, required) |
| **Output** | JSON: `{size, is_file, is_dir, is_symlink, modified, created, permissions, mime_type}` |
| **Crate** | `std::fs` + `infer` (for MIME detection) |
| **Crate version** | `infer` 0.19.x |
| **License** | MIT |
| **Rationale** | Agents need file metadata for decision-making. MIME detection helps identify binary/image files. |

### P2 — Multimodal Support

#### 7. ReadTool Enhancement: Image Reading

Extend the existing `read` tool to detect and return image data for multimodal LLMs.

| | |
|---|---|
| **Tool name** | `read` (enhanced, no new tool) |
| **Behavior change** | When path points to an image file (png/jpeg/gif/webp), return base64-encoded image data instead of text error |
| **Output for images** | Text placeholder `[Image: {filename} ({width}x{height} {mime_type})]` + `ImageData` attached to ToolResult |
| **Image formats** | image/png, image/jpeg, image/gif, image/webp (all Anthropic-supported formats) |
| **Crates** | `infer` 0.19.x (type detection), `imagesize` 0.14.x (dimensions), `base64` 0.22.x (encoding) |
| **Licenses** | All MIT |

**Message type changes required**:

- `MessageToolResult` gains optional `images: Vec<ImageData>` field
- `ImageData` struct: `{ media_type: String, data: String }` (base64, no prefix)
- Anthropic provider: serialize as `content: [{type: "image", source: {type: "base64", media_type, data}}, {type: "text", text: placeholder}]`
- OpenAI provider: serialize as `content: [{type: "image_url", image_url: {url: "data:{mime};base64,{data}"}}, {type: "text", text: placeholder}]`
- TUI: render text placeholder only, never base64 data
- Agent: propagate `images` from `ToolExecutionResult` to `MessageToolResult`

### P3 — Nice to Have

#### 8. TreeTool (`tree`)

Visualize directory structure.

| | |
|---|---|
| **Tool name** | `tree` |
| **Read-only** | Yes |
| **Parameters** | `path` (string, optional, default "."), `max_depth` (int, optional, default 3) |
| **Output** | ASCII tree with box-drawing characters |
| **Crate** | `termtree` |
| **Crate version** | 0.5.x |
| **License** | MIT |
| **Rationale** | Quick project overview. `termtree` is maintained by rust-cli org. |

## ReadTool Enhancement: Offset/Limit Partial Read

**Status**: To implement alongside P0 batch  
**Current state**: `ReadInput` has `start_line` and `end_line` (line-number based, both optional)

### Problem

Current read tool uses `start_line` / `end_line` which:
- Requires knowing line numbers in advance
- Reads the entire file into memory before slicing
- Doesn't support byte-level offset for binary/large files
- Doesn't support "read next N lines from where I left off" pagination pattern

### Proposed Change

Add `offset` and `limit` parameters for partial reading:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `offset` | `Option<u32>` | `0` | Starting line number (0-based or 1-based, see decision below) |
| `limit` | `Option<u32>` | `2000` | Maximum number of lines to return |

Behavior:
- `offset` + `limit` replaces `start_line` / `end_line` (keep old params for backward compat)
- If `limit` not specified, default cap at 2000 lines to prevent context blowup
- Response includes `total_lines` so the LLM knows if there's more to read
- If file exceeds `limit`, append `\n... ({remaining} more lines, use offset={next_offset} to continue)` hint

### Migration

- `start_line` / `end_line` → deprecated but still functional (translated to offset/limit internally)
- New `offset` / `limit` preferred in system prompt tool description
- LLM naturally adopts the pagination pattern from the hint text

### Open Questions

1. **0-based vs 1-based offset**: `offset=0` means first line, or `offset=1` means first line? Recommend 0-based (matches most APIs, simpler math).
2. **Default limit**: 2000 lines? Or token-based (e.g., 10K tokens)?
3. **Byte offset mode**: Should there be a `byte_offset` for binary reads? Or keep line-only?

## Crate Dependency Summary
| Crate | Version | License | Pure Rust | New Dep? | Purpose |
|-------|---------|---------|-----------|----------|---------|
| `regex` | 1.x | MIT/Apache-2.0 | Yes | Yes | GrepTool |
| `glob` | 0.3.x | MIT/Apache-2.0 | Yes | Yes | GlobTool |
| `similar` | 3.x | Apache-2.0 | Yes | Yes | DiffTool |
| `infer` | 0.19.x | MIT | Yes | Yes | StatTool + ReadTool image detection |
| `imagesize` | 0.14.x | MIT | Yes | Yes | ReadTool image dimensions |
| `base64` | 0.22.x | MIT/Apache-2.0 | Yes | Yes | ReadTool image encoding |
| `termtree` | 0.5.x | MIT | Yes | Yes | TreeTool |
| `std::fs` | stdlib | N/A | Yes | No | DeleteTool, LsTool |

All crates are pure Rust, actively maintained, and widely used. No C dependencies. Compliant with AGENTS.md HC #1 (Rust first).

## Registration

All new tools must be registered in:
- `build_print_tool_registry()` — print mode
- `build_tui_tool_registry()` — TUI mode
- `build_mcp_tool_registry()` — MCP server mode
- `run_interactive_mode()` — interactive REPL (currently missing symbol tools too)

## Permission Rules

| Tool | Default Permission |
|------|-------------------|
| `grep` | Allow (read-only) |
| `glob` | Allow (read-only) |
| `ls` | Allow (read-only) |
| `stat` | Allow (read-only) |
| `diff` | Allow (read-only) |
| `tree` | Allow (read-only) |
| `delete` | Ask (destructive) |
| `read` (image) | Allow (read-only, already allowed) |

Workspace-root auto-approval (from TOOL-002 permission fix) applies to all read-only tools.

## Acceptance Criteria

### P0
- [x] `grep` tool searches file contents by regex, returns structured output
- [x] `glob` tool finds files by pattern, returns path list
- [x] `delete` tool removes files/directories with workspace path validation
- [x] ReadTool `offset`/`limit` pagination with hint text
- [x] `ls` tool lists directory contents with metadata (bonus — pulled forward from P1)
- [x] All tools registered in all 4 registry builders (print, TUI, MCP, interactive)
- [x] Permission rules configured (grep/glob/ls auto-allow, delete requires approval)
- [x] Unit tests for each tool (42 new tests across 5 tools)
- [x] TUI tool call summaries for all new tools

### P1
- [ ] `diff` tool compares files using `similar`, outputs unified diff
- [ ] `stat` tool returns file metadata including MIME type

### P2
- [ ] `read` tool detects image files and returns base64 data
- [ ] `ImageData` type added to `MessageToolResult`
- [ ] Anthropic provider serializes image blocks in tool_result
- [ ] OpenAI provider serializes image_url in tool result
- [ ] TUI shows text placeholder, never base64
- [ ] Agent propagates images from tool execution to LLM context

### P3
- [ ] `tree` tool renders directory structure
- [ ] Mermaid code blocks (` ```mermaid `) render as Unicode diagrams via `mermaid-text`

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| New crate deps increase build time | Low | Low | All are small, pure Rust crates |
| `regex` crate compilation is slow | Medium | Low | Use `regex-lite` if build time is a concern |
| Image base64 inflates context tokens | Medium | Medium | Cap image size (e.g., 5MB), warn in tool description |
| `delete` tool is destructive | Medium | High | Permission Ask + workspace path validation + no recursive without explicit flag |

## Implementation Order

1. **P0 batch**: grep + glob + delete (3 tools, highest value)
2. **P1 batch**: ls + diff + stat (3 tools, medium value)
3. **P2 slice**: read image enhancement (cross-layer change: tools → core → provider → agent → TUI)
4. **P3 optional**: tree (nice to have)

## Open Design Discussion: Write/Edit Content Display Block

**Status**: Needs design decision before implementation  
**Reference**: OpenCode's write/edit tool rendering

### Problem

When the agent uses `write` or `edit`, the current TUI only shows a one-line summary:

```
 → write, path: poem.txt (921 bytes)
   ✓ wrote 921 bytes to poem.txt
```

The user cannot see WHAT was written or WHAT changed without reading the file separately. OpenCode shows a content preview/diff block inline in the conversation — similar to how code blocks are rendered, but specifically for file mutations.

### Desired Behavior

The `write` and `edit` tools should produce a **structured content display** that the TUI renders as a visual block (not just a one-liner). The format and rendering need discussion:

#### Open Questions

1. **Write tool**: Show full file content? Or just first N lines? Or a collapsed/expandable block?
   - Full content could be very long (1000+ lines)
   - Need to balance "transparency" with "noise"

2. **Edit tool**: Show unified diff? Or just the changed region with context?
   - Diff format (unified vs side-by-side) affects rendering complexity
   - `similar` crate (already proposed for DiffTool) can produce unified diffs

3. **Rendering format**:
   - Option A: Markdown code block with syntax highlighting (reuse existing TUI-006 code block renderer)
   - Option B: Custom "file diff" block with green/red coloring for added/removed lines
   - Option C: Collapsed summary line with "expand" to see full content

4. **Scrollback vs viewport**:
   - Full content blocks in scrollback could be very tall
   - Should large content be truncated with a "show more" indicator?

5. **Event protocol**:
   - Need a new `UiOutput` variant (e.g., `UiOutput::FileChange { tool, path, content/diff }`) or extend `ToolCallDisplay`?
   - Or should the tool result content carry structured metadata that the TUI interprets?

6. **Permission integration**:
   - If the user can see the content before approval, they can make better decisions
   - Should the content block appear in the approval prompt?

### Reference: OpenCode Approach

OpenCode renders write/edit operations as inline content blocks in the conversation:
- `write`: Shows the full file content in a code block
- `edit`: Shows a diff-style view with green/red highlighting
- Both are rendered as first-class visual elements, not plain text

### Proposed Direction (Subject to Discussion)

- Extend `ToolCallDisplay` with optional `content_preview: Option<String>` for write and optional `diff: Option<String>` for edit
- TUI renders these as styled blocks (code block for write, diff block for edit)
- Truncate at N lines (configurable, default 50) with "..." indicator
- Reuse TUI-006 syntax highlighting for content preview

**Decision needed from user before implementation proceeds.**

## Permission Architecture: Tool Nature Attribute

**Status**: Planned — Requirement documented, not yet implemented

### Problem

Current permission engine (`PermissionEngine`) determines default permissions by matching tool **names** against hardcoded substring patterns:

```
name_lower.contains("read") → Allow
name_lower.contains("write") → Ask
name_lower.starts_with("find") → Allow
...
```

This is fragile: adding a new read-only tool requires updating `is_file_tool()`, `add_default_rules()`, and `default_decision()` in three separate places. A tool's permission behavior is implicit in its name, not explicit in its definition.

### Proposed Solution

Add a `ToolNature` enum to categorize every tool by its operational nature, then base default permissions on nature instead of name:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolNature {
    /// Read-only: inspects files/code without side effects.
    Read,
    /// Writes or modifies files.
    Write,
    /// Executes external processes or commands.
    Execute,
    /// Makes network requests (HTTP, API calls).
    Network,
}
```

### Changes Required

#### 1. `talos-core/tool.rs` — Add `ToolNature` + default method on `AgentTool`

```rust
pub trait AgentTool: Send + Sync {
    // ... existing methods ...

    fn nature(&self) -> ToolNature {
        ToolNature::Read  // conservative default
    }
}
```

#### 2. Tool implementations — Override `nature()` per tool

| Tool | Nature | Reasoning |
|------|--------|-----------|
| read, grep, glob, ls | `Read` | File inspection only |
| find_symbol, find_references, list_symbols, list_imports | `Read` | AST inspection only |
| stat (future), diff (future), tree (future) | `Read` | Metadata/comparison only |
| write, edit | `Write` | Modify files |
| delete | `Write` | Remove files |
| bash | `Execute` | Run external commands |
| (future: curl, web_fetch) | `Network` | HTTP requests |

#### 3. `talos-permission` — Replace name-based matching with nature-based

```rust
fn default_decision(tool_nature: ToolNature) -> PermissionDecision {
    match tool_nature {
        ToolNature::Read => PermissionDecision::Allow,
        ToolNature::Write => PermissionDecision::Ask,
        ToolNature::Execute => PermissionDecision::Ask,
        ToolNature::Network => PermissionDecision::Ask,
    }
}
```

Remove `is_file_tool()` entirely — workspace auto-allow applies to ALL `ToolNature::Read` tools regardless of name.

#### 4. User-configurable overrides (future)

```
[permissions]
# Override: require approval for specific read tool
read = "ask"
# Override: allow specific execute tool
bash = "allow"
```

### Benefits

- **Single source of truth**: tool behavior is declared at the tool definition, not inferred from names
- **New tools don't need permission plumbing**: just override `nature()`
- **Harder to misclassify**: explicit enum vs fragile substring matching
- **Extensible**: add `Network` nature later without changing permission logic

### Implementation Order

1. Add `ToolNature` enum and `nature()` method to `AgentTool` trait
2. Implement `nature()` on all existing tools
3. Refactor `PermissionEngine` to use nature-based decisions
4. Remove name-based matching code (`is_file_tool`, hardcoded rules in `add_default_rules`)
5. Update tests

## Mermaid Code Block Rendering

**Status**: Planned — Library selected (`mermaid-text` v0.56.0, MIT), integration plan documented

### Library Selection

Survey of 8+ Rust crates for Mermaid-to-terminal-text rendering:

| Crate | License | Version | Deps | Diagram Types | Verdict |
|-------|---------|---------|------|---------------|---------|
| **mermaid-text** | MIT | 0.56.0 | 1 (unicode-width) | 18+ | **Selected** |
| graphs-tui | AGPL-3.0 | 0.4.0 | 0 | 4 | Rejected (license) |
| mermaid-ascii | MIT | α | 3 | 5 | Good but less mature |

**Selection rationale**:
- MIT license — fully compatible with Talos's Apache-2.0
- v0.56.0 — battle-tested in production (`leboiko/markdown-reader` TUI app, 28 stars)
- 18+ diagram types (flowchart, sequence, class, ER, state, pie, gantt, gitGraph, timeline, mindmap, sankey, architecture, quadrantChart, requirementDiagram, xychart, block, packet, journey)
- 1 dependency (`unicode-width`) — truly lightweight
- Explicitly designed for LLM agent consumption
- API: `mermaid_text::render(src) -> Result<String, Error>` — dead simple

### Integration Plan

#### Current Code Block Pipeline (reference)

The existing Markdown block rendering flow in `crates/talos-tui/src/`:

```
StreamBlockClassifier (stream_markdown.rs)
  │  classifies incoming lines into MarkdownBlockKind
  │
  ▼
app.rs::render_block_lines()  [line 249]
  │  if kind == CodeFence:
  │    ├── try_highlight_code_block()  [line 273]
  │    │     extracts lang from opening line (e.g. ```rust → "rust")
  │    │     → HighlightEngine → build_code_block()
  │    │
  │    └── fallback: render_code_block()  [line 1284]
  │          → build_code_block()  [line 1304]
  │            renders with: ─── [lang] ─── border + line numbers + content
  │
  ▼
ScrollbackLine[] → TUI display
```

#### Insertion Point

In `app.rs::render_block_lines()`, add a **mermaid check before syntax highlighting**:

```rust
// app.rs, in render_block_lines()
if kind == &MarkdownBlockKind::CodeFence {
    let opening = &block_lines[0];
    let lang = opening.trim_start().trim_start_matches(['`', '~']).trim();

    // NEW: Mermaid rendering path
    if lang == "mermaid" {
        let code_lines = &block_lines[1..block_lines.len() - 1];
        let mermaid_src = code_lines.join("\n");
        return Self::render_mermaid_block(&mermaid_src, bg);
    }

    // Existing: syntax highlighting path (unchanged)
    // ...
}
```

#### New Method: `render_mermaid_block()`

```rust
fn render_mermaid_block(mermaid_src: &str, bg: Option<CColor>) -> Vec<ScrollbackLine> {
    match mermaid_text::render(mermaid_src) {
        Ok(rendered) => {
            let header = ScrollbackLine::styled(
                vec![HistorySegment::styled(
                    format!("   [mermaid] {}", "─".repeat(40)),
                    to_crossterm_color(semantic::DIM_TEXT),
                    HistoryAttrs::default(),
                )],
                bg,
            );
            let mut lines = vec![header];
            for text_line in rendered.lines() {
                lines.push(ScrollbackLine::styled(
                    vec![HistorySegment::styled(
                        format!("   {text_line}"),
                        to_crossterm_color(semantic::CODE_BLOCK_TEXT),
                        HistoryAttrs::default(),
                    )],
                    bg,
                ));
            }
            lines
        }
        Err(_) => {
            // Fallback: show Mermaid source as plain code block
            let plain_lines: Vec<Vec<(String, Option<CColor>)>> = mermaid_src
                .lines()
                .map(|l| vec![(l.to_string(), None)])
                .collect();
            build_code_block(&plain_lines, "mermaid", bg)
        }
    }
}
```

#### Dependency

Add to `crates/talos-tui/Cargo.toml`:

```toml
mermaid-text = "0.56"
```

This pulls in only `unicode-width` (already a transitive dep via `tui-markdown`).

### Files Changed

| File | Change |
|------|--------|
| `crates/talos-tui/Cargo.toml` | Add `mermaid-text = "0.56"` |
| `crates/talos-tui/src/app.rs` | Add `render_mermaid_block()` method + mermaid check in `render_block_lines()` |
| (no new files needed) | |

### Error Handling

- Mermaid parse failure → fall back to plain code block rendering (existing `build_code_block`)
- Empty diagram → show "empty mermaid diagram" placeholder
- Overly large output (>200 lines) → truncate with `... (truncated)` indicator

### Acceptance Criteria

- [ ] ` ```mermaid ` code blocks render as Unicode box-drawing diagrams
- [ ] Invalid Mermaid source falls back to plain code block display
- [ ] Non-mermaid code blocks unaffected (regression test)
- [ ] Width-constrained rendering respects terminal width
- [ ] Existing highlight/table/list/quote rendering unaffected
- [ ] Tests: `test_mermaid_block_renders_diagram`, `test_mermaid_fallback_on_invalid_syntax`, `test_non_mermaid_code_block_unchanged`

### Prerequisite Bug Fix: Fence Info-String Misdetection

**Status**: Planned — Root cause identified, fix documented, not yet implemented

#### Problem

`stream_markdown.rs::is_matching_fence_close()` incorrectly treats ANY line starting with backticks as a closing fence, including opening fences with info strings:

```rust
// Current (line 401-403)
fn is_matching_fence_close(line: &str, marker: &str) -> bool {
    line.trim_start().starts_with(marker)  // "```rust" matches "```" → false close
}
```

| Input | Current | Correct |
|-------|---------|---------|
| ```` ```rust ```` | 当成闭合 ❌ | 不应闭合（后面有 info string） |
| ```` ``` ```` | 当成闭合 ✅ | 应闭合（后面只有空白） |
| 不同级反引号（` ``` ` vs ` ```` `） | 外层被内层提前闭合 ❌ | 不闭合（数量不够） |

#### Proposed Fix

Replace `Option<String>` with `(String, usize)` to track actual backtick count, and add info-string awareness:

```rust
// fence_marker → return (marker, count) instead of Option<String>
fn fence_marker(line: &str) -> Option<(String, usize)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let count = 3 + rest.chars().take_while(|c| *c == '`').count();
        Some(("```".to_string(), count))
    } else if let Some(rest) = trimmed.strip_prefix("~~~") {
        let count = 3 + rest.chars().take_while(|c| *c == '~').count();
        Some(("~~~".to_string(), count))
    } else {
        None
    }
}

// is_matching_fence_close → check: count >= open_count + rest is whitespace-only
fn is_matching_fence_close(line: &str, marker: &str, open_count: usize) -> bool {
    let trimmed = line.trim_start();
    let ch = marker.chars().next().unwrap_or('`');
    let close_count = trimmed.chars().take_while(|c| *c == ch).count();
    close_count >= open_count && trimmed[close_count..].trim().is_empty()
}
```

#### Files Changed

| File | Change | Effort |
|------|--------|--------|
| `crates/talos-tui/src/stream_markdown.rs` | `ClassifierState::Holding.fence_marker` type, `fence_marker()` return type, `is_matching_fence_close()` signature + logic | ~30 lines |

#### Acceptance Criteria

- [ ] ` ```rust ` opening line NOT treated as closing fence
- [ ] ` ```` ` (4 backticks) NOT closed by inner ` ``` ` (3 backticks)  
- [ ] Existing code block detection not regressed (` ```text ... ``` ` still works)
- [ ] Tests: `test_fence_info_string_not_closed`, `test_fence_nested_backtick_count`

Each batch is independently shippable.
