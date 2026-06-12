# TUI-006: Code Block Rendering Overhaul

| Field | Value |
|-------|-------|
| ID | TUI-006 |
| Title | Code Block Rendering — Rounded Border & Syntax Highlighting |
| Priority | P2 |
| Status | Planned |
| Depends on | I023 (TUI state model — stable code fence hold/render pipeline); CODE-001 for syntax highlighting |
| Blocks | None |
| Owner | `crates/talos-tui/src/` |

## Outcome

Code blocks in assistant streams render without visible Markdown fence markers (``` / ~~~).
Instead, a rounded Unicode border frame (`╭╮╰╯│`) visually separates the code block from
surrounding content, with the language tag displayed as a dim label on the top border.
Syntax highlighting via tree-sitter (or equivalent) applies Nord-themed colors to code
content, making multi-line blocks significantly easier to scan.

The border-only improvement can ship independently; syntax highlighting may be deferred to
align with a future tree-sitter integration for code analysis tooling.

## Motivation

Current code fence rendering (shipped in I023) preserves the Markdown fence markers as
visible dim rows. Two problems:

1. **Fence markers are visual noise** — the user wrote ``` to mark a code block, not to
   see ``` on screen. The rendered output should convey "this is a code block" through
   framing, not through raw Markdown syntax.
2. **No syntax highlighting** — all code content renders in a single flat color
   (`#E5C07B` warm amber). For longer code blocks this is hard to scan.

## Design

### Sub-slice A: Rounded Border (independent, no new deps)

Replace the current code fence rendering:

```
Current:
 ● ```rust
 ● fn main() {
 ●     println!("hello");
 ● }
 ● ```

Proposed:
 ● ╭─rust─────────────────────────────╮
 ● │ fn main() {                      │
 ● │     println!("hello");           │
 ● │ }                                │
 ● ╰──────────────────────────────────╯
```

- Strip opening and closing fence lines from rendered output.
- Top border: `╭─` + language tag + `─` repeats + `╮`, all on one line.
- Content lines: left `│` + code content + right-padding + `│`, right-aligned to
  the max content width (matching table rendering style).
- Bottom border: `╰` + `─` repeats + `╯`, same width as top border.
- Language tag derived from fence info string (e.g. `rust` from ` ```rust `).
- Fallback for blocks without language tag: `╭─────────────────────────╮` (no label).
- Narrow-terminal fallback: if terminal width < border + min content width, drop
  to left-only `│` without right border and rounded corners.

Implementation touches `render_code_block_line()` and adds a new
`render_code_block()` function in `app.rs`, following the same pattern as
`render_table_block()`.

### Sub-slice B: Syntax Highlighting (blocked on CODE-001 + tree-sitter ADR decision)

Integrate a syntax highlighting backend to colorize code content inside the
rounded border. Tree-sitter is the leading candidate, but it must go through
CODE-001 research before any dependency lands:

- **Phase A (standalone):** `tree-sitter` + `tree-sitter-highlight` tokenize
  code content and apply Nord-themed highlight queries. No other crate changes.
- **Phase B (after tree-sitter tooling integration):** If Talos later adds
  tree-sitter for code analysis tools (AST-aware search, refactoring), the
  highlighting path reuses the same infrastructure.

Alternative: `syntect` (TextMate grammar-based) is simpler to integrate but
does not align with a future tree-sitter tooling path.

### Nord-themed Highlight Palette (Sub-slice B)

| Category | Nord Color | Hex |
|---|---|---|
| Keyword | NORD9 (Frost blue) | `#81A1C1` |
| Function / Method | NORD8 (Frost cyan) | `#88C0D0` |
| String | NORD14 (Aurora green) | `#A3BE8C` |
| Comment | NORD3 (Polar Night muted) | `#4C566A` |
| Type / Constructor | NORD15 (Aurora purple) | `#B48EAD` |
| Number / Constant | NORD13 (Aurora yellow) | `#EBCB8B` |
| Operator / Punctuation | NORD4 (Snow Storm primary) | `#D8DEE9` |
| Variable | NORD5 (Snow Storm bright) | `#E5E9F0` |

## Acceptance Criteria

### Sub-slice A: Rounded Border

- [ ] Opening fence line (``` or ~~~ with optional language tag) is **not** rendered
      as a visible row in scrollback.
- [ ] Closing fence line is **not** rendered as a visible row in scrollback.
- [ ] Code block is framed with rounded border characters (`╭╮╰╯│`).
- [ ] Language tag (e.g. `rust`) appears as a dim label on the top border row.
- [ ] Blocks without a language tag render a plain rounded border (no label).
- [ ] Content lines preserve current code color (`#E5C07B`) as default.
- [ ] Block boundary detection (via `StreamBlockClassifier`) and hold behavior
      remain unchanged — only the final rendering of completed code blocks changes.
- [ ] Fallback path for oversized blocks (`FallbackImmediate`) still emits raw
      content without data loss.
- [ ] Existing code fence tests pass or are updated to match new rendering.
- [ ] `cargo test -p talos-tui` passes.
- [ ] Runtime verification: 3-line code block with ` ```rust ` renders as 5 rows
      (top border + 3 content + bottom border) with no visible fence markers.

### Sub-slice B: Syntax Highlighting

- [ ] ADR recorded for adding `tree-sitter` or `syntect` as a dependency.
- [ ] Code content inside rounded border is syntax-colored using the Nord palette.
- [ ] Highlighting works for at least Rust, Python, and JavaScript.
- [ ] Unknown language tags fall back to current flat code color (`#E5C07B`).
- [ ] Highlight query files are embedded at compile time (no runtime file I/O).
- [ ] `cargo test -p talos-tui` passes.
- [ ] Runtime verification: a Rust code block shows distinct colors for keywords,
      strings, comments, and function names.

## Dependencies

| Dependency | Type | Notes |
|-----------|------|-------|
| I023 (TUI state model) | Hard | Code fence hold/render pipeline is stable |
| `render_code_block_line()` | Hard | Existing rendering function to be replaced |
| `StreamBlockClassifier` | Soft | Boundary detection unchanged; only FinishHold rendering changes |
| tree-sitter / syntect (Sub-slice B only) | Hard | Requires ADR before integration |

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Rounded border characters unsupported in some terminals | Layout breakage | Fallback to plain `│` on narrow terminals; test on xterm/kitty/Windows Terminal |
| tree-sitter crate size significantly increases binary | Distribution | Measure before/after; consider `syntect` as lighter alternative |
| Highlight query files bloat embedded assets | Build size | Start with top-5 languages; add more incrementally |
| Code block width exceeds terminal width | Visual overflow | Truncate or wrap content lines inside border, matching existing table behavior |

## Required Reads

- `docs/proposals/tui-stream-markdown-rendering.md` — full proposal with Future Work section
- `docs/iterations/I023-tui-state-model.md` — I023 iteration plan
- `crates/talos-tui/src/app.rs` — `render_code_block_line()`, `render_table_block()` pattern
- `crates/talos-tui/src/stream_markdown.rs` — `StreamBlockClassifier` boundary logic
- `crates/talos-tui/src/theme.rs` — Nord palette constants

## Scope Boundary

**In scope:**
- Rounded border rendering for completed code blocks
- Language tag extraction and display
- Narrow-terminal fallback
- Syntax highlighting via tree-sitter (Sub-slice B, may be deferred)

**Out of scope:**
- Line numbers inside code blocks (future enhancement)
- Git diff gutter inside code blocks (future enhancement)
- Copy-on-select for code block content (separate UX story)
- Code analysis / AST tooling via tree-sitter (separate backlog)
- Inline code (` `code` `) rendering changes (already styled)
