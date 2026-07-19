# Iteration I142: TUI-025 Composer Multiline Wrap

> Document status: Complete (2026-07-19) — full locked validation ladder green
> Published plan date: 2026-07-19
> Objective: make the TUI composer render multi-line and auto-wrap input correctly — width-aware line counting and cursor math (Unicode/CJK cell widths), `Shift+Enter` to insert `\n`, max composer height cap (~10 lines) with bottom-anchored scroll, and wrap-aware terminal cursor placement.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: paste a 200-char line into the composer at 80-col terminal and see it wrap to 3 visible lines with correct cursor; press Shift+Enter to insert explicit newlines; composer never exceeds 10 visual lines.

## Selection And Inventory

Pre-activation inventory on 2026-07-19: I141 (MODEL-007 + TUI-031) shipped as v0.3.9; main is clean and in sync with origin. TUI-014 was originally selected by Prometheus but turned out to be already implemented (`grep` is in `THRESHOLD_SUMMARIZE` at `tool_display.rs:137`); owner doc marked Complete. TUI-025 was re-selected after first-hand code verification confirmed the genuine gaps.

| Story | Prior state | Outcome target |
| --- | --- | --- |
| TUI-025 | Refinement (P1) | Composer auto-wrap + Shift+Enter + max-height cap + cursor placement |

## Plan Reference

The full TDD implementation plan was built by Prometheus and reviewed by Momus through 6 rounds. It lives at:

**`.sisyphus/plans/tui-025-composer-multiline-wrap.md`**

That plan is the source of truth for wave structure, TDD anchors, atomic commit strategy, and risk register. The Shared Wrap Convention section in the plan defines the critical invariants:
- `composer_content_width(terminal_width)` shared helper — all callers use this, never raw terminal width
- Glyph-aware per-char wrap (no splitting wide chars across rows)
- Content rows (`input_line_count_with_width`) vs cursor position (`cursor_line_col_with_width`) are separate concerns, unified via `height_hint = max(content_rows, cursor_row + 1)`

## Scope (summary — see plan file for full detail)

**In:** width-based auto-wrap; max-height cap (~10) with bottom-anchored scroll; Shift+Enter inserts `\n`; wrap-aware cursor row/col; terminal cursor placement updated at `app.rs:941-946`.

**Out:** rich text editing; changing bare Enter/Esc/Ctrl+C/Tab semantics; scrollback user-message re-render; persistence/format changes.

**ADR needed:** No (display-layer only; `unicode-width = "0.2"` already at `crates/talos-tui/Cargo.toml:18`).

## Acceptance (summary — see plan file for full test cases)

- [ ] `input_line_count_with_width` + `cursor_line_col_with_width` pass ASCII, CJK, newline, and edge (width=0/empty/exact-boundary) tests with content-vs-cursor separation.
- [ ] Shift+Enter inserts `\n`; bare Enter still submits.
- [ ] Composer height capped at 10 visual lines with bottom-anchored scroll keeping the cursor visible (including at exact wrap boundaries).
- [ ] `build_input_text` renders wrapped lines and correct cursor position via new helpers.
- [ ] Terminal cursor at `app.rs:941-946` follows wrap + scroll (uses width-aware helper + scroll offset).
- [ ] `cargo test --workspace --locked` exits 0.
- [ ] No new dependencies; no `unwrap()` in library paths; no public API/ADR change.

## Risks And Rollback

See "Risk Register" and "Rollback" sections of the plan file. Summary:
- CJK width miscalculation → mitigated by glyph-aware per-char accumulation + CJK tests.
- Cursor drift at exact boundary → mitigated by content-vs-cursor separation documented in Shared Wrap Convention.
- Width drift between helpers → mitigated by single shared `composer_content_width()`.
- Each wave maps to one atomic commit; revert per-commit if regression appears.

## Actual Activation And Execution

| Date | Type | Record |
| --- | --- | --- |
| 2026-07-19 | Plan | Prometheus produced TDD implementation plan; 6 Momus review rounds tightened CJK/cursor/width conventions. Plan saved at `.sisyphus/plans/tui-025-composer-multiline-wrap.md`. |
| 2026-07-19 | Wave 1A (S1) | `composer_content_width`, `input_line_count_with_width`, `cursor_line_col_with_width` landed in `scrollback_input.rs` with glyph-aware wrap and content-vs-cursor separation. 3 tests added. Deep-agent delegate. |
| 2026-07-19 | Wave 1B (S2) | Shift+Enter handler added to `app.rs:1091` composer Enter arm — `KeyModifiers::SHIFT` branch calls `input_append_char('\n')` instead of submitting. 2 state-level regression tests added. Done manually (the delegate attempt failed mid-task due to model-usage limits; trivial edit per "super simple" rule). |
| 2026-07-19 | Wave 2 (S3) | `composer_scroll_offset` helper + `MAX_COMPOSER_LINES = 10` constant + `height_hint` cap using `max(content_rows, cursor_row + 1).min(MAX_COMPOSER_LINES)`. 2 tests added. Deep-agent delegate. |
| 2026-07-19 | Wave 3 (S4) | `build_input_text` now takes a `width` param and uses `input_line_count_with_width` + `composer_scroll_offset` + `cursor_line_col_with_width`. Terminal cursor placement at `app.rs:941-946` updated to use width-aware helper + scroll offset subtraction + `COMPOSER_LEFT_PAD`. 4 integration tests added. Deep-agent delegate. |
| 2026-07-19 | Wave 4 (S5) | Validation ladder: fmt/check/clippy/test all green; 62 test suites pass (328 talos-tui tests, up from 316 baseline); release preflight passed; governance 0 warnings; `git diff --check` clean. |

## Closeout Evidence (2026-07-19)

- `cargo fmt --all -- --check`: clean.
- `cargo check --workspace --locked`: clean.
- `cargo clippy --workspace --locked -- -D warnings`: clean.
- `cargo test --workspace --locked`: 62 test suites pass; 0 failures.
- `./scripts/release_preflight.sh`: passed.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `git diff --check`: clean.
- Momus plan review: 6 rounds (5 REJECT, 1 OKAY) — caught CJK split, content-vs-cursor separation, exact-boundary convention, scroll offset bug, stale references. All resolved before implementation started.

## Files Touched (closeout summary)

- `crates/talos-tui/src/scrollback_input.rs` — new `composer_content_width`, `input_line_count_with_width`, `cursor_line_col_with_width`, `composer_scroll_offset` helpers; `COMPOSER_LEFT_PAD` + `MAX_COMPOSER_LINES` constants; `build_input_text` now takes width param and uses width-aware helpers + scroll offset.
- `crates/talos-tui/src/scrollback.rs` — composer `height_hint` updated to `max(content_rows, cursor_row + 1).min(MAX_COMPOSER_LINES)`.
- `crates/talos-tui/src/app.rs` — Shift+Enter branch at composer Enter arm (~line 1091); terminal cursor placement at `:941-946` uses `composer_content_width` + width-aware cursor + scroll offset.
- `crates/talos-tui/src/state_tests.rs` — 7 new tests (3 helper-level + 2 Shift+Enter state-level + 2 height/scroll).
- `crates/talos-tui/src/tests.rs` — 2 new build_input_text integration tests + 1 cursor placement test.
- `docs/backlog/active/TUI-025-composer-multiline-wrap.md` — status → Complete.
- `docs/backlog/active/TUI-014-grep-result-summary.md` — status → Complete (already-implemented discovery during I142 story selection).
- `docs/iterations/I142-composer-multiline-wrap.md` — iteration plan + execution records + closeout evidence (this file).
- `docs/iterations/README.md` — I141 row added then updated to Complete; I142 row added.

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
