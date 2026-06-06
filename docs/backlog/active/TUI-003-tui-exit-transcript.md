# TUI-003: TUI exit transcript into terminal scrollback

## Outcome

When the user exits the TUI, the session transcript remains visible in the
host terminal's scrollback — matching the Codex TUI behavior. The TUI no
longer "swallows" the conversation on exit; the user can scroll up in their
shell to see what happened.

## Status

Planned. **No blockers.** This is a TUI lifecycle fix, not a structural
refactor: it touches `Tui::Drop` and the TUI entry/exit sequence, not
`state.rs` / `widgets.rs` / `app.rs` event loop. Can land as the next
iteration (I015 or a dedicated small iteration) ahead of the I015-I017
foundations and ahead of TUI-002.

## Priority

P1. The current "transcript discarded on exit" behavior is the single
biggest day-to-day UX gap: every TUI session loses its context on quit,
forcing users to `/export <path>` manually before exit if they want a
record.

## Required Reads

- `docs/proposals/tui-codex-overhaul.md` — adjacent structural work (TUI-002)
  that this story does **not** depend on. TUI-003 is lifecycle-layer,
  TUI-002 is structure-layer.
- `docs/iterations/I014-tui-completion.md` — provides the
  `TuiState::transcript_plain_text()` and `transcript_markdown()` methods
  that the fix reuses.
- `crates/talos-tui/src/app.rs:50-71` — `Tui::new` (terminal init: alt-screen
  + raw mode + mouse capture).
- `crates/talos-tui/src/app.rs:614-625` — `impl Drop for Tui` +
  `restore_terminal` (terminal teardown: leave alt-screen, disable raw mode,
  disable mouse capture). **This is where the fix lands.**
- `crates/talos-tui/src/state.rs:477-503` — `transcript_plain_text` and
  `transcript_markdown` serializers (deterministic source text, not
  rendered buffers).
- `docs/reference/REFERENCE-PROJECTS.md` §687-741 — Codex TUI PRIMARY
  reference. TBD: confirm the exact Codex mechanism (alt-screen + transcript
  dump on exit, or no-alt-screen, or custom terminal wrapper) before
  implementation.

## Acceptance Criteria

- [ ] When the TUI exits cleanly (double Ctrl+C, `/quit`, `/exit`),
      the conversation transcript is appended to the host terminal's
      scrollback in plain text form.
- [ ] When the TUI exits via panic or signal, no transcript is dumped
      (avoid spamming scrollback with partial data on crash).
- [ ] Transcript is **not** dumped when `--print` / `-p` is used (the
      user is in print mode, not TUI; transcript goes to stdout already).
- [ ] Transcript is **not** dumped when `--inline` is used (inline mode
      does not use alt-screen; the conversation is already in scrollback).
- [ ] Transcript dump respects a `--no-transcript-on-exit` opt-out flag
      for users who want a clean scrollback.
- [ ] The transcript dump shows a clear marker (e.g. `--- Talos session
      transcript (12 turns) ---` and `--- end of session ---`) so users
      can grep / scroll past it.
- [ ] The transcript dump goes to **stdout** (not stderr) so it is
      captured by shell redirects and tmux/screen scrollback.
- [ ] `cargo test -p talos-tui` passes; the new code path is covered by
      at least 3 unit tests (clean exit dumps, panic exit does not dump,
      flag opt-out works).
- [ ] `cargo test --workspace` passes with no regressions.
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui`
      are unchanged or reduced; no new warnings.
- [ ] I008 hook-based learning still observes the same `HookEvent`
      ordering (verified by `crates/talos-cli/tests/hooks_e2e.rs` at
      `RUST_LOG=debug`).

## Proposed Approach (Reference for the Iteration)

The minimal fix reuses I014's transcript serializers. In
`crates/talos-tui/src/app.rs`, the `impl Drop for Tui` block (lines 614-618)
currently calls `restore_terminal()` and returns. The fix:

```rust
impl Drop for Tui {
    fn drop(&mut self) {
        let _ = restore_terminal();
        // New: dump transcript to scrollback unless opted out.
        // Guard: only dump on clean exit (state.should_exit == true).
        if self.state.should_exit && transcript_on_exit_enabled() {
            let transcript = self.state.transcript_plain_text();
            if !transcript.is_empty() {
                println!(
                    "\n--- Talos session transcript ({} chars) ---\n{}\
                     \n--- end of session ---\n",
                    transcript.chars().count(),
                    transcript,
                );
                let _ = io::stdout().flush();
            }
        }
    }
}
```

The `transcript_on_exit_enabled()` helper consults:
1. CLI flag `--no-transcript-on-exit` (highest priority).
2. Config field `tui.transcript_on_exit: bool` (default `true`).
3. **The `should_exit` flag** (skipped on panic — panics do not set
   `should_exit`, so the dump is automatically skipped).

Why this works:
- `TuiState::transcript_plain_text()` already produces deterministic
  source text (excludes `current_turn_text` streaming, includes all
  `ChatLine::{Text, Assistant, ToolCall}` entries).
- The transcript is written to stdout, after `LeaveAlternateScreen`, so
  it lands in the host shell's scrollback.
- The `should_exit` check naturally filters out panic and signal-induced
  exits.
- The transcript contains no terminal escape sequences (it's plain text),
  so it does not corrupt the scrollback.

## Open Questions

- **Exact Codex mechanism**: I should confirm whether Codex uses
  "alt-screen + transcript dump" (our proposed approach) or
  "no-alt-screen at all" (which is a much larger change). Look at
  `codex-rs/tui/src/tui.rs` and `custom_terminal.rs` before implementation.
- **Transcript length**: long sessions may produce MB-scale transcripts.
  Should we cap at N characters with a `(...truncated, use /export for
  full transcript)` notice?
- **Redaction**: tool calls can include sensitive arguments (paths, env
  vars, file contents). Codex appears to dump raw. We should at minimum
  document the behavior; `--redact-on-exit` is a follow-up.
- **Timing**: the dump happens in `Drop`. If `Drop` is called during
  stack unwinding from a panic, the dump will be skipped via the
  `should_exit` check. Verify this is sufficient.
- **TTY check**: should we dump only when stdout is a TTY? Or always?
  Dumping to a piped stdout (e.g. `talos --tui | tee session.log`)
  would write the transcript to the pipe, which is probably what the
  user wants.

## Non-Goals

- No new TUI features (only exit behavior change).
- No changes to TUI internals (state, widgets, key handling) — pure
  lifecycle fix.
- No removal of the alt-screen mode (we keep it; we add a transcript
  dump on exit).
- No changes to `/export <path>` (still the canonical way to save a
  transcript to disk).
- No interaction with the I008 hook layer (hooks fire during the
  turn, not on exit; the transcript serializer does not invoke hooks).

## Scheduling

- **Next iteration slot** (I015-equivalent or a small dedicated iteration).
- Not blocked by I015-I017 (foundations). Not blocked by TUI-002
  (structural refactor). Can land independently.
- Estimated effort: 0.5-1 day (1 file in `crates/talos-tui/src/app.rs`,
  3-4 unit tests, 1 docs commit, 1 atomic code commit).
- Should ship as a single atomic commit on `main`.

## Residual Work Destination

Per-user preferences for transcript formatting (e.g. colored markdown vs
plain text, character-level truncation, sensitive-arg redaction) stay in
this story's notes. The behavior flip — "transcript visible in scrollback
on exit" — is the DoD.
