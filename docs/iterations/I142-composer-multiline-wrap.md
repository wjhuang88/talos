# Iteration I142: TUI-032 Composer Multiline Wrap

> Document status: Review (2026-07-20) — cross-terminal remediation pending runtime re-test
> Published plan date: 2026-07-19
> Objective: make the TUI composer render multi-line and auto-wrap input correctly — width-aware line counting and cursor math (Unicode/CJK cell widths), `Shift+Enter` to insert `\n`, max composer height cap (~10 lines) with bottom-anchored scroll, and wrap-aware terminal cursor placement.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: paste a 200-char line into the composer at 80-col terminal and see it wrap to 3 visible lines with correct cursor; press Shift+Enter to insert explicit newlines; composer never exceeds 10 visual lines.

## Selection And Inventory

Pre-activation inventory on 2026-07-19: I141 (MODEL-007 + TUI-031) shipped as v0.3.9; main is clean and in sync with origin. TUI-014 was originally selected by Prometheus but turned out to be already implemented (`grep` is in `THRESHOLD_SUMMARIZE` at `tool_display.rs:137`); owner doc marked Complete. The composer story was selected after first-hand code verification confirmed the genuine gaps. Its initially reused `TUI-025` ID was corrected to the unique `TUI-032` during acceptance remediation.

| Story | Prior state | Outcome target |
| --- | --- | --- |
| TUI-032 | Refinement (P1) | Composer auto-wrap + Shift+Enter + max-height cap + cursor placement |

## Plan Reference

The full TDD implementation plan was built by Prometheus and reviewed by Momus through 6 rounds. It lives at:

**`.sisyphus/plans/tui-032-composer-multiline-wrap.md`**

That plan is the source of truth for wave structure, TDD anchors, atomic commit strategy, and risk register. The Shared Wrap Convention section in the plan defines the critical invariants:
- `composer_content_width(terminal_width)` shared helper — all callers use this, never raw terminal width
- Glyph-aware per-char wrap (no splitting wide chars across rows)
- Content rows (`input_line_count_with_width`) vs cursor position (`cursor_line_col_with_width`) are separate concerns, unified via `height_hint = max(content_rows, cursor_row + 1)`

## Scope (summary — see plan file for full detail)

**In:** width-based auto-wrap; max-height cap (~10) with bottom-anchored scroll; Shift+Enter inserts `\n`; wrap-aware cursor row/col; terminal cursor placement updated at `app.rs:941-946`.

**Out:** rich text editing; changing bare Enter/Esc/Ctrl+C/Tab semantics; persistence/format changes.

The original plan incorrectly excluded scrollback user-message wrapping even though
the owner story required it. Runtime acceptance proved that omission invalid; explicit
scrollback wrapping is an in-scope correction under `CHANGE-CONTROL.md`.

**ADR needed:** No (display-layer only; `unicode-width = "0.2"` already at `crates/talos-tui/Cargo.toml:18`).

## Acceptance (summary — see plan file for full test cases)

- [x] `input_line_count_with_width` + `cursor_line_col_with_width` pass ASCII, CJK, newline, and edge (width=0/empty/exact-boundary) tests with content-vs-cursor separation.
- [ ] Shift+Enter inserts `\n`; bare Enter still submits — automated event test passes; rebuilt-binary terminal re-test pending.
- [x] Terminal capability is probed before enabling complete modified-key reporting; unsupported terminals degrade to normal input and support `Ctrl+J` as a portable newline fallback.
- [x] Composer height capped at 10 visual lines with bottom-anchored scroll keeping the cursor visible (including at exact wrap boundaries).
- [x] `build_input_text` renders wrapped lines and correct cursor position via new helpers.
- [x] Terminal cursor follows wrap + scroll (uses width-aware helper + scroll offset).
- [x] Finalized user history explicitly wraps into styled physical scrollback rows for ASCII and CJK content.
- [x] `cargo test --workspace --locked` exits 0 after acceptance remediation.
- [x] No new dependencies; no `unwrap()` in library paths; no public API/ADR change.

## Risks And Rollback

See "Risk Register" and "Rollback" sections of the plan file. Summary:
- CJK width miscalculation → mitigated by glyph-aware per-char accumulation + CJK tests.
- Cursor drift at exact boundary → mitigated by content-vs-cursor separation documented in Shared Wrap Convention.
- Width drift between helpers → mitigated by single shared `composer_content_width()`.
- Each wave maps to one atomic commit; revert per-commit if regression appears.

## Actual Activation And Execution

| Date | Type | Record |
| --- | --- | --- |
| 2026-07-19 | Plan | Prometheus produced TDD implementation plan; 6 Momus review rounds tightened CJK/cursor/width conventions. Plan saved at `.sisyphus/plans/tui-032-composer-multiline-wrap.md`. |
| 2026-07-19 | Wave 1A (S1) | `composer_content_width`, `input_line_count_with_width`, `cursor_line_col_with_width` landed in `scrollback_input.rs` with glyph-aware wrap and content-vs-cursor separation. 3 tests added. Deep-agent delegate. |
| 2026-07-19 | Wave 1B (S2) | Shift+Enter handler added to `app.rs:1091` composer Enter arm — `KeyModifiers::SHIFT` branch calls `input_append_char('\n')` instead of submitting. 2 state-level regression tests added. Done manually (the delegate attempt failed mid-task due to model-usage limits; trivial edit per "super simple" rule). |
| 2026-07-19 | Wave 2 (S3) | `composer_scroll_offset` helper + `MAX_COMPOSER_LINES = 10` constant + `height_hint` cap using `max(content_rows, cursor_row + 1).min(MAX_COMPOSER_LINES)`. 2 tests added. Deep-agent delegate. |
| 2026-07-19 | Wave 3 (S4) | `build_input_text` now takes a `width` param and uses `input_line_count_with_width` + `composer_scroll_offset` + `cursor_line_col_with_width`. Terminal cursor placement at `app.rs:941-946` updated to use width-aware helper + scroll offset subtraction + `COMPOSER_LEFT_PAD`. 4 integration tests added. Deep-agent delegate. |
| 2026-07-19 | Wave 4 (S5) | Validation ladder: fmt/check/clippy/test all green; 62 test suites pass (328 talos-tui tests, up from 316 baseline); release preflight passed; governance 0 warnings; `git diff --check` clean. |
| 2026-07-19 | Acceptance failure | Maintainer tested the built binary in Alacritty: Shift+Enter submitted instead of inserting a newline, and a width-wrapped composer message did not remain correctly wrapped after entering history. Independent acceptance also found a one-column effective-width drift caused by right block padding and a duplicate `TUI-025` story ID. I142 returned to Review. |
| 2026-07-19 | Change control | Classified all three runtime findings as in-scope corrections: terminal key disambiguation is required for the published Shift+Enter behavior; explicit finalized-history wrapping was already required by the owner story; right-padding accounting repairs the shared width convention. Story identity was corrected to `TUI-032` without changing the iteration objective. |
| 2026-07-19 | Remediation | Terminal initialization now pushes Crossterm modified-key disambiguation and restore pops it exactly once; finalized history lines are explicitly width-wrapped with styles/background and three-column continuation indentation preserved; composer width subtracts both left prefix and right block padding. Added event-path, ASCII/CJK history-wrap, keyboard-flag, and exact-boundary Buffer tests. |
| 2026-07-20 | Protocol correction | Review of the Kitty keyboard protocol showed that disambiguation alone intentionally leaves Enter and Shift+Enter identical. Terminal initialization now probes support, enables all-key escape reporting plus alternate keys on supported terminals, and leaves unsupported terminals untouched. `Ctrl+J` provides a protocol-independent newline fallback. |

## Closeout Evidence (2026-07-19)

- `cargo fmt --all -- --check`: clean.
- `cargo check --workspace --locked`: clean.
- `cargo clippy --workspace --locked -- -D warnings`: clean.
- `cargo test --workspace --locked`: 62 test suites pass; 0 failures.
- `./scripts/release_preflight.sh`: passed.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `scripts/assess_project_scale.sh .`: unchanged recommendation (`high-risk`, `release-managed`, `on-demand`).
- `cargo build --release --locked -p talos-cli`: passed; rebuilt `target/release/talos`.
- `target/release/talos --version`: `talos 0.3.9`.
- `git diff --check`: clean.
- Momus plan review: 6 rounds (5 REJECT, 1 OKAY) — caught CJK split, content-vs-cursor separation, exact-boundary convention, scroll offset bug, stale references. All resolved before implementation started.

Protocol-correction replay on 2026-07-20:

- `cargo test -p talos-tui --locked`: 335 passed, 0 failed.
- Locked workspace check, strict Clippy, full tests, and release preflight: passed.
- Governance validation: 0 warnings; `git diff --check`: clean.
- Release build and version check: passed; `target/release/talos` reports `talos 0.3.9`.

## Files Touched (closeout summary)

- `crates/talos-tui/src/scrollback_input.rs` — shared glyph-aware wrap/cursor/scroll helpers; composer width accounts for both the three-column prefix and one-column right padding.
- `crates/talos-tui/src/scrollback.rs` — composer height cap and right-padding convention.
- `crates/talos-tui/src/app.rs` — Shift+Enter branch, wrap-aware cursor placement, and explicit finalized-history wrapping before terminal insertion.
- `crates/talos-tui/src/app_stream.rs` — display-cell-aware styled scrollback row wrapping with continuation indentation.
- `crates/talos-tui/src/inline_terminal.rs` — support probe plus paired complete keyboard-enhancement push/pop for modified Enter reporting.
- `crates/talos-tui/src/state_tests.rs` and `crates/talos-tui/src/app/app_tests.rs` — helper, event-path, exact-boundary Buffer, and ASCII/CJK scrollback regressions.
- `docs/backlog/active/TUI-032-composer-multiline-wrap.md` — status → Review pending Alacritty re-test.
- `docs/backlog/active/TUI-014-grep-result-summary.md` — status → Complete (already-implemented discovery during I142 story selection).
- `docs/iterations/I142-composer-multiline-wrap.md` — iteration plan + execution records + closeout evidence (this file).
- `docs/iterations/README.md` — I141 row added then updated to Complete; I142 row added.
- `EVOLUTION.md` — records terminal-protocol and explicit-physical-scrollback-row lessons from runtime acceptance.

## Acceptance Remediation Verification

- Automated targeted tests: passing.
- `cargo test -p talos-tui --locked`: 335 passed, 0 failed.
- Full locked workspace validation, release preflight, governance validation, and release build: passing.
- Real rebuilt-binary tests: pending direct Alacritty Shift+Enter and unsupported-terminal/legacy-multiplexer Ctrl+J checks; I142 must remain Review until they pass.

## Retrospective

What worked:
- **Momus caught real design bugs before code** — 6 rounds of review identified CJK-split issue, content-vs-cursor semantic gap, exact-boundary inconsistency, scroll-offset bug, and effective-width drift. Each was a real bug that would have cost more to fix in code than in plan.
- **Pre-verified facts in plan** — Prometheus's second attempt with all file:line citations produced a tight, actionable plan in 2 minutes vs the first attempt that timed out at 14+ minutes investigating.
- **Deep-agent delegates on pure functions** — Wave 1A and Wave 2 both completed in under 3 minutes each with full TDD coverage.
- **Manual fallback for failed delegate** — When S2 delegate failed due to model-usage limits, the trivial Shift+Enter edit was done manually in 2 minutes per the "super simple" rule.

What didn't:
- **S2 delegate failed mid-task** — model-usage limits are a real constraint. Future iterations should plan for delegate failure on small tasks by having a manual fallback ready.
- **Initial Prometheus pick (TUI-014) was already shipped** — the owner doc was stale. Cost: one extra Prometheus round + manual doc update. Lesson: verify owner-doc claims against actual code before committing to a story.
- **Momus needed 6 rounds** — some of these were stale-reference cleanups that should have been caught in a single careful pass. The cost was ~6 minutes of review time; acceptable but a more careful single-pass editor could have closed it in 2.

Lessons for `EVOLUTION.md`:
- **Stale owner docs are a real selection risk.** Always grep the actual code for the story's claimed "missing" functionality before committing to implementation. TUI-014 had been shipped for weeks but the owner doc still said "Refinement".
- **Momus review is most valuable when the plan involves convention-setting** (shared helpers, semantic distinctions like content-vs-cursor). Iterations with simple additive features need only 1-2 rounds; iterations that establish cross-cutting invariants benefit from 4-6.
- **Deep-agent delegates work well on pure functions with TDD anchors.** Wave 1A and Wave 2 each completed in under 3 minutes when given concrete test cases + implementation hints. The pattern: "Write these tests first, then implement using this algorithm sketch, run these verification commands."
- **For trivial key-handler edits, manual implementation is faster than delegate** once model-usage limits become a concern.

## Planned Validation

- Pure-function unit tests for `input_line_count_with_width`, `cursor_line_col_with_width`, `composer_scroll_offset` covering ASCII / CJK / newline / exact-boundary / width=0 / empty cases.
- Integration tests in `app/app_tests.rs` for Shift+Enter + render pipeline + cursor placement.
- `cargo test --workspace --locked` exits 0; `./scripts/release_preflight.sh`; `scripts/validate_project_governance.sh .` 0 warnings; `git diff --check` clean.
