# TUI-003: TUI exit transcript into terminal scrollback

> **Status: SUPERSEDED by TUI-002 sub-slice A (inline-by-default refactor).**
>
> This story is retained for historical reference. The behavior it
> intended — "transcript visible in the host scrollback on TUI exit" — is
> achieved **by construction** once `Tui::new` no longer calls
> `EnterAlternateScreen` (TUI-002 sub-slice A). The terminal's native
> scrollback already contains every finalized chat turn; no "dump on
> exit" step is needed.
>
> **Do not implement this story.** Land TUI-002 sub-slice A instead.
> See `docs/backlog/active/TUI-002-codex-overhaul.md` and
> `docs/proposals/tui-codex-overhaul.md`.

---

## Original Outcome (historical)

When the user exits the TUI, the session transcript remains visible in the
host terminal's scrollback — matching the Codex TUI behavior. The TUI no
longer "swallows" the conversation on exit; the user can scroll up in
their shell to see what happened.

## Why it was wrong

The 2026-06-06 framing proposed achieving this by **keeping
`EnterAlternateScreen`** and adding a transcript-dump step in
`impl Drop for Tui` after `LeaveAlternateScreen`. After reading the
Codex TUI source end-to-end on 2026-06-06, this is not how Codex works:

- Codex TUI never enters alt-screen in the first place. The viewport is
  inline; the scrollback already contains the transcript.
- A dump step would print the transcript **after** leaving alt-screen,
  which is fragile (timing, panic exits, TTY detection, character
  truncation, sensitive-arg redaction, stdout-vs-stderr, transcript
  length caps — all of which this story's `Open Questions` section
  enumerated but could not cleanly resolve).
- It also conflicted with the user's design principle:

  > "我们整体的ui设计必须是信息流驱动的,不是框架分割的而是行追加的风格"
  >
  > "codex就是流式的呀,整体上一直是当前终端追加内容,只是追加的内容可以是复杂格式或者组件块"

  The UI is an information flow, line-appended into the terminal's
  scrollback. Cells are not widgets in a frame-segmented layout; they
  are line blocks pushed to the host terminal as the conversation
  grows.

## What survives in TUI-002 sub-slice A

| Original TUI-003 requirement | How TUI-002 sub-slice A satisfies it |
|---|---|
| Transcript visible in scrollback on clean exit | Scrollback **is** the transcript by construction; every finalized turn is appended via `insert_history_lines` (Codex: `codex-rs/tui/src/insert_history.rs`) with `SetScrollRegion(1..viewport.top())`. |
| No transcript on panic | The scrollback simply contains the turns that were finalized before the panic; the partial streaming state lives in the viewport (which is discarded on panic) and is not pushed to scrollback. No explicit gating needed. |
| Transcript not dumped in `--print` / `-p` mode | Print mode never enters the TUI; stdout already contains the conversation. |
| Transcript not dumped in inline mode | Inline mode never enters alt-screen; the conversation is in scrollback by definition. (After TUI-002 sub-slice A, the TUI **is** inline by default; the `--inline` flag becomes the default behavior and a future `--alt-screen` flag opts in to full-screen sub-views only.) |
| `--no-transcript-on-exit` opt-out | No longer needed. Users who want a clean scrollback can use `--alt-screen` mode for sub-views only, or `clear` their scrollback manually. |
| Transcript markers (`--- Talos session transcript (N turns) ---` / `--- end of session ---`) | Not needed; the scrollback contains the conversation directly, line by line, with no separator. |
| `/export <path>` still works | Preserved unchanged. I014's `crates/talos-tui/src/export.rs` (thin permission wrapper around `TuiState::transcript_plain_text`/`transcript_markdown`) is the canonical save path; remains the right way to capture a transcript to disk. |

## Original Acceptance Criteria (for audit trail)

The original acceptance criteria from the 2026-06-06 version of this
file were:

- [ ] When the TUI exits cleanly (double Ctrl+C, `/quit`, `/exit`),
      the conversation transcript is appended to the host terminal's
      scrollback in plain text form.
- [ ] When the TUI exits via panic or signal, no transcript is dumped.
- [ ] Transcript is **not** dumped when `--print` / `-p` is used.
- [ ] Transcript is **not** dumped when `--inline` is used.
- [ ] Transcript dump respects a `--no-transcript-on-exit` opt-out flag.
- [ ] The transcript dump shows clear markers.
- [ ] The transcript dump goes to **stdout** (not stderr).
- [ ] `cargo test -p talos-tui` passes; new code path covered by
      at least 3 unit tests.
- [ ] `cargo test --workspace` passes with no regressions.
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui`
      are unchanged or reduced.
- [ ] I008 hook-based learning still observes the same `HookEvent`
      ordering.

**Status of these criteria**: all are subsumed by TUI-002 sub-slice A's
acceptance criteria. See `docs/backlog/active/TUI-002-codex-overhaul.md`.

## Required Reads (for historical context)

- `docs/proposals/tui-codex-overhaul.md` — full proposal including
  sub-slice A's architectural foundation.
- `docs/reference/codex-tui-architecture.md` — authoritative record of
  Codex TUI's verified implementation as of the 2026-06-06 source read.
- `docs/iterations/I014-tui-completion.md` — provides the
  `TuiState::transcript_plain_text()` and `transcript_markdown()` methods
  that `/copy` and `/export` reuse (unchanged in the refactor).
- `crates/talos-tui/src/app.rs:50-71` — `Tui::new` (the
  `EnterAlternateScreen` call that TUI-002 sub-slice A removes).
- `crates/talos-tui/src/app.rs:614-625` — `impl Drop for Tui` +
  `restore_terminal` (the `LeaveAlternateScreen` call that TUI-002
  sub-slice A removes).
- `crates/talos-tui/src/state.rs:477-503` — `transcript_plain_text` and
  `transcript_markdown` serializers (preserved unchanged; reused by
  `/copy all`/`/export` and by `/copy last` for
  `last_assistant_text`).
- `crates/talos-tui/src/export.rs` — thin permission wrapper around
  `transcript_plain_text`/`transcript_markdown` (preserved unchanged;
  `/export <path>` is the canonical save path).
- `docs/reference/REFERENCE-PROJECTS.md` §687-741 — Codex TUI PRIMARY
  reference. Verified 2026-06-06 to confirm the inline-by-default model
  (alt-screen is opt-in for sub-views only; `EnterAlternateScreen` is
  not called in the default code path).

## Original Open Questions (resolved by TUI-002 sub-slice A)

- **Exact Codex mechanism**: Resolved. Codex uses
  "no-alt-screen at all" with `SetScrollRegion(1..viewport.top())`
  push-append for finalized turns. The terminal's scrollback is the
  transcript; no dump step exists.
- **Transcript length**: N/A. The scrollback is the terminal's own
  scrollback (typically 10k+ rows); the user's terminal handles
  retention. The `/export <path>` path is the way to capture a full
  transcript to disk.
- **Redaction**: Out of scope for sub-slice A. Tool call cells can
  redact sensitive arguments at the cell-render layer (future work;
  tracked as residual work in TUI-002).
- **Timing**: N/A. There is no dump step; the scrollback is updated
  continuously as turns finalize.
- **TTY check**: N/A. There is no dump step; the scrollback is the
  terminal's own scrollback, so TTY semantics are inherited from the
  host terminal.

## Original Non-Goals (re-asserted for TUI-002 sub-slice A)

- No new TUI features beyond the architectural flip.
- No changes to TUI internals (state, widgets, key handling) beyond what
  sub-slice A requires.
- No changes to `/export <path>` (still the canonical save path).
- No interaction with the I008 hook layer.

## Scheduling

- **Do not schedule independently.** Sub-slice A is the replacement;
  land it via TUI-002 (I022 or a dedicated small iteration).
- Estimated effort for sub-slice A: ~1 week (1 new `tui/` subdir with
  7 files, 1 new `history_cell/` subdir with 2 files, slim `app.rs`,
  10+ unit tests, 1 docs commit, 1 atomic code commit).
- Should ship as a single atomic commit on `main`.
