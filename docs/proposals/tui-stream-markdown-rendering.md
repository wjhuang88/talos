# TUI Stream Markdown Rendering

## Status

Proposal and implementation guide for the TUI stream renderer. The first
classifier and renderer slices landed during I023: block detection, hold
status, plain-text fallbacks, box-drawing table rendering, styled history
spans, prefix colors, horizontal rules, and conservative inline Markdown
rendering are implemented in `talos-tui`. Full CommonMark support remains
future work.

## Goals

- Keep the live preview exactly one row high.
- Preserve immediate visibility for plain streaming text.
- Support simple single-line Markdown without buffering a whole message.
- Support block Markdown by holding only the structured block, not the entire
  assistant response.
- Expose enough classifier state for the preview animation to explain what is
  happening.
- Keep `InlineTerminal::insert_history` as a single-row history writer.

## Non-Goals

- No dynamic-height streaming preview.
- No full CommonMark implementation in the first slice.
- No terminal-history rewrite or reflow after rows are inserted.
- No markdown parsing in `talos-conversation`; rendering stays in `talos-tui`.

## Pipeline

```text
StreamMessage { source, stream }
  -> StreamRenderState
  -> StreamBlockClassifier
  -> MarkdownLineRenderer / MarkdownBlockRenderer
  -> Vec<ScrollbackLine>
  -> InlineTerminal::insert_history / insert_styled_history
```

`talos-conversation` emits semantic streams only. `talos-tui` owns display
classification, buffering, prefixes, backgrounds, preview text, and rendered
history rows.

## Proposed Approach

Introduce a TUI-local `StreamBlockClassifier` and keep it separate from
`StreamRenderState`.

- `StreamBlockClassifier` owns Markdown boundary recognition and emits
  `BlockDecision` values.
- `StreamRenderState` owns stream-local prefix counters, raw buffering,
  preview state, source background rows, and conversion from decisions to
  `ScrollbackLine`s.
- Markdown line/block renderers own display formatting and always provide a
  plain-text fallback.

This separation makes boundary bugs testable without terminal rendering and
keeps terminal layout bugs separate from Markdown parsing bugs.

## Preview Contract

The preview component remains one row.

For content that can be represented as one streaming line, preview displays the
latest incomplete rendered text. For content that cannot be represented safely
as one row, preview hides the raw content and shows an animation/status message
derived from classifier state.

Examples:

| Classifier state | Preview text |
|---|---|
| Plain text incomplete line | ` ● generating answer` |
| Holding table block | ` ● rendering table...` |
| Holding code fence | ` ● receiving code block...` |
| Holding unknown structured block | ` ● formatting block...` |

The spinner or processing marker is preview-only. It is displayed on the single
preview row and is never inserted into history.

## Prefix Contract

Every logical stream block uses the existing three-column prefix policy:

| Source | First rendered row | Continuation rows |
|---|---|---|
| User | ` > ` | `   ` |
| Assistant | ` ● ` | `   ` |
| Tool | ` ● ` | `   ` |
| System | ` # ` | `   ` |
| Error | ` ! ` | `   ` |

Rendered Markdown rows are still stream-local rows. The first row produced for
the stream receives the source prefix; every later row receives the blank
alignment prefix. A held block must not reset the prefix counter unless it
starts a new `StreamMessage`.

## Single-Line Markdown

Single-line Markdown is rendered in immediate mode. Complete newline-terminated
lines are pushed to history as soon as they arrive. The incomplete trailing line
stays in the one-row preview.

Initial supported inline forms:

| Markdown form | Recognition | Rendering |
|---|---|---|
| Plain text | Default | Unstyled text |
| Inline code | Balanced backticks on one line, e.g. `` `cmd` `` | Strip delimiters and render code span with code color |
| Strong | Balanced `**text**` or `__text__` on one line | Strip delimiters and render bold |
| Emphasis | Balanced `*text*` or `_text_` on one line | Strip delimiters and render italic/dim |
| Link | `[label](url)` on one line | Render label as underlined link text and append dim ` (url)` |
| Heading | `# ` through `###### ` on one line | Strip heading marker, render emphasized heading row, no extra vertical spacing |

Recognition is conservative. If delimiters are unbalanced or the line is
ambiguous, render it as plain text rather than entering a hold state.

Inline detection never delays a completed line. If a line cannot be rendered
confidently as inline Markdown, it is emitted as plain text immediately.
User-authored streams are rendered literal: pasted input keeps Markdown markers
visible and never enters the Markdown block classifier.

## Block Markdown

Block Markdown may require complete block context before deciding terminal rows.
Only the active structured block is held; surrounding plain lines still stream
immediately.

Initial supported block forms:

| Block kind | Start condition | End condition | Preview status | First-slice rendering |
|---|---|---|---|---|
| Fenced code block | Line starts with triple backticks or tildes | Matching closing fence line, or stream finish fallback | `receiving code block...` | Preserve fences, style fence rows dim and code rows with code color |
| Markdown table | Header row with pipes followed by separator row | Blank line, non-table line, or stream finish | `rendering table...` | Render a box-drawing table with display-width alignment; render inline Markdown inside cells; fallback to visible rows if structured rendering fails |
| List block | Consecutive `- `, `* `, `+ `, or ordered `1. ` lines | Blank line or non-list line | `formatting list...` | Preserve marker and indentation, style marker, render inline Markdown in item body |
| Block quote | Consecutive `> ` lines | Blank line or non-quote line | `formatting quote...` | Preserve quote marker, style marker, render quote body dim |

Fenced code blocks suppress other block recognizers until the closing fence is
seen. For example, a pipe-delimited line inside a code fence must not start a
table.

### Boundary Algorithm

The classifier consumes complete logical lines plus a stream-finish signal. It
may keep a small pending candidate line when a block cannot be identified from
one line alone.

```text
Plain state
  code fence opener        -> StartHold(CodeFence)
  possible table header    -> PendingTableHeader(line)
  list marker              -> StartHold(List)
  quote marker             -> StartHold(Quote)
  otherwise                -> ImmediateLine

PendingTableHeader
  separator row            -> StartHold(Table) with header + separator
  otherwise                -> ImmediateLine(header), then reprocess current line

Holding(CodeFence)
  matching closing fence   -> FinishHold(CodeFence)
  stream finish            -> FallbackImmediate(CodeFence)
  otherwise                -> ContinueHold(CodeFence)

Holding(Table)
  table row                -> ContinueHold(Table)
  blank or non-table row   -> FinishHold(Table), then reprocess current line
  stream finish            -> FinishHold(Table)

Holding(List / Quote)
  compatible line          -> ContinueHold
  blank or incompatible    -> FinishHold, then reprocess current line
  stream finish            -> FinishHold
```

The `reprocess current line` step is important: it preserves content following
a completed block without requiring the upstream stream to resend anything.

## Classifier State

The classifier is a deterministic state machine independent of terminal
rendering.

```rust
enum MarkdownBlockKind {
    PlainText,
    InlineMarkdown,
    CodeFence,
    Table,
    List,
    Quote,
    UnknownStructured,
}

enum BlockDecision {
    ImmediateLine,
    StartHold { status: HoldStatus },
    ContinueHold { status: HoldStatus },
    FinishHold { status: HoldStatus },
    FallbackImmediate { status: HoldStatus, reason: FallbackReason },
}

struct HoldStatus {
    kind: MarkdownBlockKind,
    lines: usize,
    bytes: usize,
    boundary_hint: BoundaryHint,
}

enum BoundaryHint {
    CodeFenceClose,
    TableSeparator,
    TableEnd,
    NonListLine,
    NonQuoteLine,
}
```

`StreamRenderState` consumes `BlockDecision`; it does not duplicate block
boundary rules. The UI preview derives its animation text from `HoldStatus`.

The classifier must expose `HoldStatus` on start, continuation, finish, and
fallback decisions so the preview can update animation text as the block grows
instead of showing a generic spinner forever.

Future classifiers may add more boundary hints, but the first implementation
keeps the public state small: code fence close, table separator/end, non-list
line, and non-quote line.

## Fallback Rules

The renderer must prefer visible output over perfect formatting.

- If a held block exceeds the configured byte or line threshold, emit
  `FallbackImmediate` and flush the raw held lines as plain rows.
- If stream finish occurs while a code fence is still open, flush the held block
  as plain rows or as an unterminated code block with no data loss.
- If table rendering fails because of malformed rows, flush original rows.
- If inline Markdown is ambiguous, render the original line immediately.

No fallback path may drop buffered text.

## Implementation Slices

1. Add `StreamBlockClassifier` with decisions, hold status, and exhaustive unit
   tests. Rendering output remains plain text.
2. Wire classifier decisions into `StreamRenderState` while preserving the
   current immediate-line default for plain text.
3. Add preview status mapping from `HoldStatus` to one-row animation text.
4. Add first-slice block renderers: box-drawing table rendering and code-fence
   preservation. Every renderer keeps a plain fallback.
5. Add richer styled row support only after the plain row path is stable.

I023 implementation status:

- Slices 1-5 have landed for the conservative first renderer.
- `ScrollbackLine` now carries visible `text`, styled `HistorySegment`s, and
  optional background color.
- `InlineTerminal` remains a single-row writer; styled rows use
  `insert_styled_history`, while plain rows can still use `insert_history`.
- User streams intentionally bypass Markdown parsing and stay literal.

## History Rendering Contract

`InlineTerminal` receives already-rendered rows and remains ignorant of
Markdown. A history row carries both its visible fallback text and the styled
segments used for terminal output. The terminal writer still inserts finalized
rows above the viewport one line at a time.

```rust
struct ScrollbackLine {
    text: String,
    segments: Vec<HistorySegment>,
    bg: Option<CColor>,
}
```

Markdown renderers must keep `text` as a stable visible fallback so tests,
copy/export paths, and terminal-width padding are not coupled to specific ANSI
style decisions.

## Test Matrix

Classifier tests:

- Plain text never enters hold.
- Inline code, strong, emphasis, links, and headings render immediately.
- Unbalanced inline markers render as plain text.
- Code fence starts hold and finishes only on a matching fence.
- Pipes inside a code fence do not start a table.
- Table starts only after header and separator rows are both seen.
- Table ends on blank line, non-table line, or stream finish.
- Lists and quotes hold only consecutive compatible lines.
- Chunk boundaries split across newline, pipe, backtick, and delimiter tokens.
- Multiple blocks in one stream finish in order.
- Held block threshold triggers `FallbackImmediate`.
- Stream finish with an unterminated block flushes all raw content.

Renderer tests:

- First rendered stream row gets the source prefix.
- Continuation rows get the blank prefix.
- Held block rows do not reset the stream-local line counter.
- User background rows still wrap user streams and only user streams.
- Preview text is single-row for immediate mode.
- Preview status hides held raw content and reflects `HoldStatus`.
- Spinner/processing marker is never emitted to history.
- Plain fallback preserves all input bytes as visible text.

Integration tests:

- Assistant paragraph streams complete lines to history immediately.
- Assistant table holds only the table, shows rendering status in preview, then
  flushes aligned rows to history.
- Text before and after a held block remains visible in order.
- Pasted multiline user input remains one user block and does not run Markdown
  block detection.

## Alternatives Considered

| Alternative | Reason rejected for first slice |
|---|---|
| Dynamic-height streaming preview | Makes bottom anchoring and stale viewport cleanup harder; can hide or jump content under small terminal heights |
| Hold the entire assistant response until completion | Preserves formatting context but destroys streaming visibility for long responses |
| Render all Markdown line-by-line without holding | Keeps streaming simple but cannot reliably format tables or fenced code blocks |
| Put Markdown parsing in `talos-conversation` | Mixes semantic conversation state with terminal display policy |

## Open Questions

- Exact byte/line thresholds for held blocks before fallback.
- Whether table alignment should consider East Asian wide characters from the
  start or initially reuse the existing display-width utilities only.

## Dependencies

- I023 stream render state extraction and hold-buffer boundary.
- Existing `InlineTerminal::insert_history` single-line history insertion.
- Existing Unicode width handling used by TUI preview/input rendering.
- Requirement intake before implementation; this proposal alone is not an
  executable backlog story.
