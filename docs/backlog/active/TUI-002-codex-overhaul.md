# TUI-002: TUI inline-by-default refactor (Codex-style)

## Outcome

The `talos-tui` crate adopts Codex TUI's **inline-by-default** architecture
(verified 2026-06-06 in `docs/reference/codex-tui-architecture.md`):

- The terminal's native scrollback **is the transcript**. Chat turns are
  appended to the scrollback above the viewport via escape-sequence
  operations (Codex: `insert_history.rs` with `SetScrollRegion(1..area.top())`).
- `EnterAlternateScreen` is removed from the default code path. The viewport
  sits at the user's current cursor y on TUI entry; on TUI exit the cursor
  returns to that anchor, and the scrollback already contains every
  finalized turn.
- Alt-screen is **opt-in** for full-screen sub-views only (model picker,
  theme picker, keymap remapper, plugin browser, onboarding). Gated by
  `alt_screen_enabled: bool` (Codex pattern, `codex-rs/tui/src/tui.rs`).
- The module split (`history_cell/`, `keymap.rs`, `bottom_pane/`,
  `slash_command.rs`, `tui/event_stream.rs`, `tui/frame_requester.rs`,
  `tui/frame_rate_limiter.rs`, `tui/job_control.rs`, etc.) is a **consequence
  of the architecture**, not a goal. A pure structural refactor that
  preserves `EnterAlternateScreen` would ship the wrong thing.

This supersedes the 2026-06-06 "modular overhaul" framing and absorbs
TUI-003 (TUI exit transcript into terminal scrollback) — the inline model
makes a "transcript dump on exit" step unnecessary because the scrollback
already is the transcript.

## Status

Planned. **Sub-slice A is unblocked and can land first**; sub-slices B-E
remain blocked behind I015-I017 (R6-R8) so the new history cells can render
the new provider/tool/git output formats from day one. Natural iteration
slot for sub-slice A: **I022 or a dedicated small iteration**. Sub-slices
B-E: I023+.

## Priority

P1 (sub-slice A) → P2 (sub-slices B-E). The current "alt-screen + transcript
discarded on exit" behavior is the single biggest day-to-day UX gap
(`crates/talos-tui/src/app.rs:50-71, 614-625`); sub-slice A fixes it
without requiring I015-I017.

## Required Reads

- `docs/proposals/tui-codex-overhaul.md` — full proposal, sub-slices A-E,
  alternatives, open questions, scheduling rationale.
- `docs/reference/codex-tui-architecture.md` — **authoritative technical
  reference** for what Codex TUI actually does (verified by 2026-06-06
  source read of `codex-rs/tui/src/`). 11 sections, file:line evidence.
- `docs/iterations/I010-polished-agent.md` — R2/R3 Codex-like baseline
  (AppServerSession convergence, inline mode flag, Nord theme, markdown,
  diff, steering, slash).
- `docs/iterations/I014-tui-completion.md` — provenance markers,
  `/plugins`, `/copy` + `/export` (most recent TUI work; transcript
  serializers at `crates/talos-tui/src/state.rs:477-503` are reused).
- `docs/reference/REFERENCE-PROJECTS.md` §687-741 — Codex TUI PRIMARY
  reference (module layout, file-by-file mapping). Note: §689 "Full-screen
  ratatui TUI" is a misnomer; correct read is "inline-by-default, alt-screen
  opt-in for sub-views".
- `docs/decisions/003-tui-progressive-evolution.md` — TUI evolution
  anchor.
- `docs/decisions/005-tui-event-architecture.md` — TUI event architecture
  boundary (single-mpsc `AppEvent` bus + `AppServerSession` seam).
- `docs/decisions/006-event-architecture-boundary.md` — single-consumer
  loop rule (refactor must not introduce a global pub/sub bus).
- `docs/decisions/007-process-hardening-unsafe.md` — `unsafe` boundary
  (`tui/job_control.rs` SIGTSTP handler may need `libc::raise`; ADR-gated
  per AGENTS.md rule #2).
- `docs/iterations/I015-provider-schema.md` (R6) — provider output format
  precondition for B/C/D/E.
- `docs/iterations/I016-portable-file-search.md` (R7) — tool cell
  rendering precondition for bottom_pane file search.
- `docs/iterations/I017-embedded-git-tools.md` (R8) — diff cell rendering
  precondition for `history_cell/diff.rs`.

## Acceptance Criteria

### Architecture (sub-slice A — the foundation)

- [ ] `Tui::new` (`crates/talos-tui/src/app.rs:50-71`) does **not** call
      `EnterAlternateScreen`; the viewport sits at the user's current
      cursor y.
- [ ] `impl Drop for Tui` + `restore_terminal` (`crates/talos-tui/src/app.rs:614-625`)
      does **not** call `LeaveAlternateScreen`; on exit the cursor
      returns to the TUI-entry anchor, and the scrollback already contains
      every finalized chat turn.
- [ ] A new `crates/talos-tui/src/tui/` subdir contains: `mod.rs`,
      `custom_terminal.rs` (inline-viewport Terminal fork; MIT-attributed
      per Codex file header), `insert_history.rs` (SetScrollRegion
      algorithm), `event_stream.rs` (EventBroker stdin pause/resume),
      `frame_requester.rs` (actor pattern), `frame_rate_limiter.rs`
      (120 FPS cap), `job_control.rs` (SIGTSTP with alt-screen
      awareness; ADR-007-gated if `unsafe` is used), `keyboard_modes.rs`.
- [ ] A new `crates/talos-tui/src/history_cell/` subdir contains: `mod.rs`
      (HistoryCell trait, `HistoryRenderMode::{Rich, Raw}`), `base.rs`
      (`PlainHistoryCell`, `PrefixedWrappedHistoryCell`,
      `CompositeHistoryCell`, `WebHyperlinkHistoryCell`).
- [ ] `crates/talos-tui/src/app.rs` is slimmed from a god module (625 lines
      in I014) to a thin `tokio::select!` event loop; cell rendering
      and lifecycle live in the new modules.
- [ ] When a chat turn finalizes, the resulting `Vec<Line<'static>>` is
      pushed to the scrollback via `insert_history_lines`, **not** drawn
      into a ratatui frame. `schedule_frame()` is called so the viewport
      redraws below the new history rows.
- [ ] All I014 functionality still works: provenance markers in tool call
      cells, `/plugins`, `/copy last`, `/copy all`, `/export <path>`.

### Cross-cutting (applies to every sub-slice)

- [ ] I008 hook-based learning still observes the same `HookEvent`
      ordering (verified by `crates/talos-cli/tests/hooks_e2e.rs` and
      `mcp_client_e2e.rs` at `RUST_LOG=debug`).
- [ ] Public API of `talos-tui` is unchanged (or carries a semver bump +
      ADR per AGENTS.md rule #6). Public types at
      `crates/talos-tui/src/lib.rs:15-19` remain stable: `Tui`,
      `SkillInfo`, `SkillSidebar`, `ApprovalState`, `nord`.
- [ ] `TuiState` private methods (`transcript_plain_text`,
      `transcript_markdown`, `last_assistant_text`) stay `pub(crate)`;
      the cell refactor must not need to expose them more widely.
- [ ] `cargo test --workspace` passes with no regressions (baseline
      652+ tests at the time the refactor starts).
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui`
      are either unchanged or reduced; no new warnings.
- [ ] An iteration record (e.g. `docs/iterations/I022-tui-inline-default.md`)
      documents sub-slice outcomes and runtime evidence.

## Sub-Slices

The proposal (`docs/proposals/tui-codex-overhaul.md`) defines 5 sub-slices
that an iteration may pick independently. **Sub-slice A is the
architectural foundation and must land first.**

- **A (P1, unblocked)** — Inline-by-default Tui (architectural foundation).
  Removes `EnterAlternateScreen`; adds `tui/` subdir plumbing (custom
  terminal, insert_history, frame_requester, frame_rate_limiter,
  event_stream, job_control, keyboard_modes) + `history_cell/` base
  cells + slims `app.rs`. Absorbs TUI-003. ~1 week.
- **B (P2, blocked behind I015-I017)** — `tui/` subdir plumbing
  refinements (e.g. better SynchronizedUpdate handling, 60 FPS
  fallback for low-end terminals).
- **C (P2, blocked behind I016)** — `bottom_pane/` / `chat_composer.rs`
  (multi-line composer, `@`-mention file search, popup stack,
  `ApprovalOverlay` modal, file search popup).
- **D (P2, blocked behind I015-I017)** — Slash command framework
  (`slash_command.rs` with `strum` enum, kebab-case, `is_visible` filter,
  `supports_inline_args`); replaces `state::handle_slash_command`'s
  match-on-`&str` at `crates/talos-tui/src/state.rs:316-372`.
- **E (P2, blocked behind I015-I017)** — Keymap system (`keymap.rs` with
  4 contexts: `App`, `Chat`, `Composer`, `Approval`; Codex has 8, Talos
  starts with 4 to avoid scope creep); replaces inline `KeyCode` match
  at `crates/talos-tui/src/app.rs:280-362`.

A single iteration may pick sub-slice A alone; sub-slices B-E can be
picked up in successor iterations.

## Residual Work Destination

Per-cell UX refinements (e.g. syntax highlighting in tool output cells,
animation polish, per-provider theming) stay in this story's notes until
the modular restructure lands. The architectural flip — "scrollback IS
the transcript" — is the DoD for sub-slice A.
