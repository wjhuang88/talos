# I022: TUI Inline-by-Default (Codex-style)

**User can**: Launch the TUI without clearing the host terminal's scrollback, see chat turns
append into scrollback as the conversation grows (information-flow driven, line-appending),
and resume the shell's previous content after the TUI exits — matching the Codex TUI
experience.

## Status: PLANNED (2026-06-07)

## Decision Gate

Follow `docs/reference/codex-tui-architecture.md` (verified 2026-06-06) and the architectural
principle verified there: **Codex TUI is inline-by-default; alt-screen is opt-in for sub-views
only.** Any deviation (e.g. preserving the unconditional `EnterAlternateScreen` call, or
re-introducing a transcript-dump-on-exit step) requires updating ADR-018 and is out of scope.

Required reading before implementation:

- `docs/reference/codex-tui-architecture.md` — authoritative technical reference.
- `docs/proposals/tui-codex-overhaul.md` — sub-slice A scope.
- [ADR-018](../decisions/018-tui-job-control-unsafe.md) — TUI job control unsafe boundary
  (the single authorized `libc::raise(SIGTSTP)` site in `tui/job_control.rs`).
- [ADR-007](../decisions/007-process-hardening-unsafe.md) — parent ADR for the `libc` FFI
  pattern in a different module.
- [ADR-006](../decisions/006-event-architecture-boundary.md) — single-consumer event loop rule.
- [ADR-003](../decisions/003-tui-progressive-evolution.md) — TUI evolution anchor.
- `crates/talos-tui/src/app.rs:50-71, 614-625` — the two sites this iteration removes.
- `crates/talos-tui/src/state.rs:477-503` — `transcript_plain_text` / `transcript_markdown`
  / `last_assistant_text` (preserved unchanged; reused by `/copy` and `/export`).
- `docs/iterations/I014-tui-completion.md` — provenance markers, `/plugins`, `/copy`,
  `/export` (the I014 functionality that must survive the refactor).

## Selected Stories

- [ ] #I022-S1: New `tui/` subdir with custom terminal, `insert_history`, `EventBroker`,
      `FrameRequester`, `FrameRateLimiter`, `JobControl` (per ADR-018), and
      `KeyboardModes`. `Tui::new` and `impl Drop for Tui` no longer call
      `EnterAlternateScreen` / `LeaveAlternateScreen`; the viewport sits at the user's
      current cursor y.
- [ ] #I022-S2: New `history_cell/` subdir with the `HistoryCell` trait, `HistoryRenderMode`,
      and base cells (`PlainHistoryCell`, `PrefixedWrappedHistoryCell`,
      `CompositeHistoryCell`, `WebHyperlinkHistoryCell`).
- [ ] #I022-S3: A finalized chat turn produces a `Vec<Line<'static>>` and is **appended to
      the scrollback** via `insert_history_lines` (`SetScrollRegion(1..viewport.top())` +
      `MoveTo` + `Print("\r\n")` per line + `ResetScrollRegion`), **not** drawn into a
      ratatui frame. `schedule_frame()` is called so the viewport redraws below the new
      history rows.
- [ ] #I022-S4: Existing widgets (`ToolCallBubble`, `ApprovalOverlay`, etc.) become
      history cells; the render loop becomes a cell-stream consumer. `state::handle_slash_command`
      match-on-`&str` (`crates/talos-tui/src/state.rs:316-372`) becomes a cell push.
- [ ] #I022-S5: Verification, documentation sync, and runtime evidence recorded in this
      file. README updated. `EVOLUTION.md` lessons recorded if any non-obvious problems
      were encountered.

## Scope

- **Architectural flip**: scrollback **is** the transcript by construction. No
  transcript-dump-on-exit step (the original TUI-003 is superseded; see
  `docs/backlog/active/TUI-003-tui-exit-transcript.md`).
- **New modules** in `crates/talos-tui/src/`:
  - `tui/mod.rs` — re-exports the new `Tui` API
  - `tui/custom_terminal.rs` — inline-viewport `Terminal` fork, MIT-attributed to
    Florian Dehau + Ratatui Developers (per Codex file header)
  - `tui/insert_history.rs` — `SetScrollRegion(1..viewport.top())` push-append
  - `tui/event_stream.rs` — `EventBroker` with stdin pause/resume for `$EDITOR` handoff
  - `tui/frame_requester.rs` — actor-style rate-limited redraw
  - `tui/frame_rate_limiter.rs` — 120 FPS clamp
  - `tui/job_control.rs` — SIGTSTP via `libc::raise` (per ADR-018)
  - `tui/keyboard_modes.rs` — keyboard enhancement flag stack
  - `history_cell/mod.rs` — `HistoryCell` trait, `HistoryRenderMode`
  - `history_cell/base.rs` — `PlainHistoryCell`, `PrefixedWrappedHistoryCell`,
    `CompositeHistoryCell`, `WebHyperlinkHistoryCell`
- **Slimmed** `crates/talos-tui/src/app.rs` (625 lines → thin `tokio::select!` event loop).
- **Preserved unchanged**: `crates/talos-tui/src/lib.rs:15-19` public API (`Tui`,
  `SkillInfo`, `SkillSidebar`, `ApprovalState`, `nord`); `state.rs:477-503` transcript
  serializers (reused by `/copy` and `/export`); `state.rs:316-372` slash command dispatch
  (rewritten in shape, not in semantics); `widgets/` and `tests.rs`.
- **No new top-level dependencies.** `libc` is already transitive via `crossterm` and
  `ratatui`; reference it directly in `tui/job_control.rs` and (if needed) add `libc` to
  `crates/talos-tui/Cargo.toml` for clarity.

## Non-Goals

- No new TUI features beyond the architectural flip.
- No removal of `/copy` or `/export` (both still work via `TuiState::transcript_plain_text`).
- No changes to the I008 hook layer (hooks fire during the turn, not on exit; the
  transcript serializer does not invoke hooks).
- No new permissions for the TUI lifecycle.
- No sub-slices B/C/D/E (deferred to I023+; require I015-I017 foundations).
- No drop-in of Codex's full `bottom_pane/` composer, `markdown_render.rs`, or
  `diff_render.rs` (out of I022 scope; `bottom_pane/` lands in sub-slice C).
- No `nix` or `signal-hook` crate adoption (per ADR-018 §Rejected alternatives).

## Acceptance Criteria

### Architecture (sub-slice A — the foundation)

- [ ] `Tui::new` does **not** call `EnterAlternateScreen`; the viewport sits at the
      user's current cursor y.
- [ ] `impl Drop for Tui` does **not** call `LeaveAlternateScreen`; on exit the cursor
      returns to the TUI-entry anchor, and the scrollback already contains every
      finalized chat turn.
- [ ] A new `crates/talos-tui/src/tui/` subdir contains the 7 files listed in Scope.
- [ ] A new `crates/talos-tui/src/history_cell/` subdir contains `mod.rs` and `base.rs`.
- [ ] `crates/talos-tui/src/app.rs` is slimmed to a thin `tokio::select!` event loop;
      cell rendering and lifecycle live in the new modules.
- [ ] A finalized chat turn pushes `Vec<Line<'static>>` to the scrollback via
      `insert_history_lines` (per `codex-rs/tui/src/insert_history.rs`).
- [ ] `custom_terminal.rs` carries the MIT attribution header from the Codex source.

### Cross-cutting (applies to every story)

- [ ] I014 functionality still works: provenance markers in tool call cells, `/plugins`,
      `/copy last`, `/copy all`, `/export <path>`.
- [ ] I008 hook-based learning still observes the same `HookEvent` ordering (verified
      by `crates/talos-cli/tests/hooks_e2e.rs` and `mcp_client_e2e.rs` at `RUST_LOG=debug`).
- [ ] Public API of `talos-tui` is unchanged (5 re-exports at `lib.rs:15-19`).
- [ ] `TuiState` private methods (`transcript_plain_text`, `transcript_markdown`,
      `last_assistant_text`) stay `pub(crate)`; the cell refactor must not need to expose
      them more widely.
- [ ] `cargo test --workspace` passes with no regressions (baseline 652 tests at
      iteration start).
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui` are unchanged or
      reduced; no new warnings.
- [ ] `cargo clippy --workspace` is clean (no new warnings).
- [ ] All new public items have `///` doc comments (AGENTS.md Rust-Specific Rules).
- [ ] No `unwrap()` in library code (AGENTS.md Rust-Specific Rules).
- [ ] No `unsafe` outside `tui/job_control.rs`'s single ADR-018 site.

### End-to-End Runtime Acceptance Gate (mandatory per ITERATION-WORKFLOW §3a)

- [ ] The feature is reachable from a real run path: launching `cargo run -p talos-cli`
      (TUI mode) does not clear the host terminal; chat turns append into the host
      scrollback above the viewport; exiting returns the cursor to the entry anchor.
- [ ] There is a recorded manual transcript or integration test that drives the inline-by-default
      behavior through the binary and asserts the user-visible result.
- [ ] The MIT attribution in `custom_terminal.rs` is visible in the source file.
- [ ] The `// SAFETY:` comment in `tui/job_control.rs` references ADR-018.
- [ ] Runtime evidence (command + observed output) is pasted into this file's
      Verification section.

## Estimated Effort

~1 week (1 small iteration):

| Story | Day | Notes |
|---|---|---|
| #I022-S1 (tui/ subdir) | 1-3 | Most code; copied from Codex reference under MIT |
| #I022-S2 (history_cell/ base) | 3-4 | Trait + 4 base cells; `Box<dyn HistoryCell>` dispatch |
| #I022-S3 (inline-by-default) | 4-5 | Remove `EnterAlternateScreen`/`LeaveAlternateScreen`; wire `insert_history_lines` |
| #I022-S4 (cell-driven render loop) | 5-6 | Migrate `ToolCallBubble` → cell; migrate `ApprovalOverlay` → cell + bottom_pane stub |
| #I022-S5 (verification + sync) | 6-7 | 652/652 tests; I008 hook regression; README sync; this file's runtime evidence |

1 atomic commit on `main` (per AGENTS.md "One logical change per commit"). If the diff is
too large for a single commit, split by module boundary: commit 1 = `tui/` subdir +
`custom_terminal.rs`; commit 2 = `history_cell/` base; commit 3 = `app.rs` slim + inline-by-default
flip. Each commit must keep `cargo test --workspace` green and I008 hook ordering intact.

## Dependencies

**No external dependencies** (sub-slice A is unblocked):

- I015-I017 (R6-R8) are required only for sub-slices B-E (provider schema, file search, Git
  diffs), not for sub-slice A.
- I014 (TUI completion) is **complete**; provenance markers and `/copy`/`/export` are
  the regression baseline.

**Pre-existing transitive `libc`**: must be referenced directly in `tui/job_control.rs`. If
`libc` is not in `crates/talos-tui/Cargo.toml`'s direct deps, add it (no new top-level
dependency, just making the transitive dep explicit).

## Risks

1. **Ratatui 0.30 `Terminal` API drift.** Codex forks `ratatui::Terminal` from a slightly
   older API surface; the custom terminal may need small adaptations to compile against
   ratatui 0.30. Mitigation: keep the fork minimal; only add inline-viewport support; do
   not attempt to backport unrelated Codex changes.
2. **Terminal emulator variance.** `SetScrollRegion` is well-supported on xterm-like
   emulators (iTerm2, gnome-terminal, Windows Terminal, VSCode integrated terminal) but
   has historically had quirks on some rare emulators. Mitigation: the inline-by-default
   viewport falls back to a simpler "draw normally into the bottom line" path on
   terminals that do not honor `SetScrollRegion`; document the fallback in
   `tui/insert_history.rs` module docs.
3. **Cell dispatch performance.** `Box<dyn HistoryCell>` dispatch can be slow if cells are
   rendered every frame. Mitigation: cells are rendered once when appended to scrollback;
   the viewport only redraws below the last history row (Codex pattern). Profile
   `cargo bench` after the refactor lands.
4. **I008 hook regression.** The new `ChatWidget` / cell-stream orchestrator must not
   reorder `HookEvent`s. Mitigation: the existing `hooks_e2e` and `mcp_client_e2e` tests
   must continue to pass; run with `RUST_LOG=debug` and compare hook event sequences
   before/after the refactor.
5. **Public API surface.** AGENTS.md #6 binds `talos-tui`'s public API. The refactor must
   preserve the 5 re-exports at `lib.rs:15-19`. Mitigation: the public API diff is part
   of the PR review checklist; any breakage is a semver bump + ADR.
6. **The MIT attribution.** Forking `ratatui::Terminal` requires carrying the MIT
   copyright header (Florian Dehau + Ratatui Developers). Mitigation: copy the header
   verbatim from the Codex file; verify in the PR review.

## Residual Work Destination (out of I022 scope)

- **Sub-slice B** (tui/ refinements, 60 FPS fallback for low-end terminals, better
  SynchronizedUpdate handling) — I023+.
- **Sub-slice C** (`bottom_pane/` / `chat_composer.rs` with multi-line composer,
  `@`-mention file search, popup stack, full `ApprovalOverlay` modal) — I023+, blocked
  behind I016.
- **Sub-slice D** (`slash_command.rs` with `strum` enum, kebab-case, `is_visible` filter)
  — I023+, blocked behind I015-I017.
- **Sub-slice E** (`keymap.rs` with 4 contexts: `App`, `Chat`, `Composer`, `Approval`;
  Codex has 8, Talos starts with 4) — I023+, blocked behind I015-I017.
- **Per-cell UX refinements** (syntax highlighting in tool output cells, animation
  polish, per-provider theming) — TUI-002 notes.
- **Cell-level redaction** of sensitive tool call arguments (paths, env vars, file
  contents) — TUI-002 notes.

## Verification (template; filled in at completion)

- `cargo test --workspace` exits 0: ___ tests pass (baseline 652).
- `cargo clippy --workspace` is clean: no new warnings.
- `cargo run -p talos-cli` (TUI mode) does not clear the host terminal.
- Chat turn appends into scrollback above the viewport; visible in manual transcript.
- `Ctrl+Z` suspends the TUI; `fg %1` resumes it; no scrollback corruption.
- I008 hook ordering unchanged: `cargo test -p talos-cli --test hooks_e2e` passes at
  `RUST_LOG=debug`.
- `/copy last`, `/copy all`, `/export <path>`, `/plugins` all work in the new architecture.
- Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui` are unchanged or
  reduced.
- `README.md` updated to reflect the new TUI behavior.
- This file updated with execution results and runtime evidence.
