# TUI-012: Session Resume History Rendering

| Field | Value |
|---|---|
| ID | TUI-012 |
| Title | Session Resume History Rendering — Full Render Replay |
| Type | Technical Story |
| Priority | P0 (urgent) |
| Status | Refinement |
| Depends on | Session history loading (I024 ✅), Stream rendering pipeline (I023 ✅) |
| Source | User report 2026-06-21 |
| Blocks | — |

## Problem

When resuming a session with `--continue` (`-c`) or `--session <id>`, the
previous conversation history is mechanically dumped as raw text into the
scrollback. Code blocks show raw ``` markers, markdown formatting is lost,
and tool calls appear as raw JSON instead of styled summaries.

The hydrate pipeline calls `render_history_messages()` → `render_history_message()`
which uses `StreamRenderState::push_chunk()`, but the content is fed as a single
blob rather than streamed line-by-line as in live sessions. This causes:

1. **Code blocks** show raw fence markers instead of `[lang] ───` header + line numbers
2. **Markdown** formatting (bold, italic, headings, links) is inconsistently rendered
3. **Tool calls** appear as raw JSON `{"tool_use_id": "...", ...}` instead of ` → tool_name`
4. **Tool results** are not summarized — full content shown instead of human-readable summaries

## Scope

Fix the resume history hydration path to use the same rendering pipeline as
live streaming sessions:

- Push history messages through the markdown renderer chunk-by-chunk
- Apply `render_history_message()` styling consistently
- Render tool calls with `build_tool_call_scrollback_line()`
- Summarize tool results with `should_suppress_tool_result_content()`

## Acceptance

- Given a session resumed with `--continue`
  When history is hydrated into scrollback
  Then code blocks show `[lang] ───` header with line numbers (no ``` markers)

- Given a resumed session containing tool calls
  When history is hydrated
  Then tool calls appear as ` → tool_name, args_summary` (not raw JSON)

- Given a resumed session containing markdown formatting
  When history is hydrated
  Then bold, italic, headings, and links render with proper colors and styles

- Given a resumed session with tool results
  When history is hydrated
  Then `read` results are summarized (not full content), other results follow suppression rules

- `cargo test -p talos-tui` passes, `cargo test --workspace` passes

## Required Reads

- `crates/talos-tui/src/app.rs` — `hydrate_history()` (line 480)
- `crates/talos-tui/src/scrollback.rs` — `render_history_messages()` (line 449)
- `crates/talos-tui/src/stream_markdown.rs` — `StreamRenderState`
- `crates/talos-tui/src/tool_display.rs` — `build_tool_call_scrollback_line()`, output suppression
- `crates/talos-conversation/src/types.rs` — `ToolCallDisplay`, `ToolResultDisplay`
- `crates/talos-conversation/src/engine.rs` — How tool calls are emitted as `UiOutput`

## Non-Goals

- Do not change the streaming render pipeline for live sessions
- Do not add new UI components or change the scrollback layout
- Do not modify session storage format (JSONL)
