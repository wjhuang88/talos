# Iteration I130: TUI-030 Composer Input History

> Document status: Complete (2026-07-16) — architecture re-review accepted five `handle_input_event` entry-point tests
> Published plan date: 2026-07-15
> Planned objective: Let a user navigate previously submitted composer input with Up and Down without losing the draft they were editing.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Up/Down navigates in-memory submitted-input history with exact draft restoration, proven by runtime evidence and state tests.

## Pre-Activation Code Inspection (2026-07-15)

### Key Dispatch Priority Chain (`crates/talos-tui/src/app.rs:924-1075`)

1. **Approval panel** (`!ApprovalState::Hidden`) → `handle_pending_approval_input()` → returns early (line 930-932). History navigation must NOT fire here.
2. **Credential input** (`slash_menu.is_credential_input()`) → custom handling → returns early (line 934-962). History navigation must NOT fire here.
3. **Main match block** (line 964):
   - Ctrl+C/A/E/G → special handling
   - `Up if slash_menu.is_open` → slash menu prev (line 1000)
   - `Down if slash_menu.is_open` → slash menu next (line 1004)
   - Tab/Enter/Esc with slash menu → panel actions
   - `Char('/')` → open slash menu
   - `Char(c)` → append to input buffer
   - `Backspace/Left/Right` → cursor/input operations
   - `Enter` → `submit_input_message()` (line 1054)
   - `Esc` → reset ctrl_c_state

### Up/Down Availability

Up/Down are **unused** when the slash menu is closed. Adding history navigation here is safe — no existing behavior conflicts.

### Input Submission Flow

`submit_input_message()` (line 1082) → `state.input_submit()` (line 1087) → returns content, clears buffer. History recording should happen inside `input_submit()` before clearing.

### State Model (`crates/talos-tui/src/state.rs`)

- `input_buffer: String` — current input text
- `cursor_pos: usize` — char-index cursor position
- `input_submit()` (line 386) — clones buffer, clears, returns content
- `input_clear()` (line 143) — clears buffer and cursor

### Findings

- No existing history fields or Up/Down navigation outside slash menu.
- Approval and credential input intercept keys before the main match block — history navigation is automatically excluded.
- Slash menu Up/Down has explicit `if slash_menu.is_open` guards — history Up/Down uses `if !slash_menu.is_open`.
- Multiline input uses Ctrl+A/Ctrl+E for line start/end; no line-up/line-down cursor movement exists, so Up/Down history navigation does not conflict with multiline editing.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-030` | none | Refinement | TUI-004, TUI-010 (both Complete) | In-memory Up/Down history with draft restoration and priority regressions. |

### Scope

- Add `input_history: Vec<String>`, `history_cursor: Option<usize>`, `draft_input: String` to `TuiState`.
- Add `history_prev()`, `history_next()`, `record_history()` methods.
- Record non-empty submitted input on submit; dedup consecutive duplicates.
- Up navigates toward older entries; Down toward newer; past newest restores exact draft.
- Add `Up if !slash_menu.is_open` and `Down if !slash_menu.is_open` handlers in `app.rs`.
- State tests: navigation, boundaries, duplicate policy, draft restoration, priority (slash menu / approval / credential don't trigger history).

### Non-Goals

- No cross-session or on-disk persistence.
- No search, filtering, or new UI panel.
- No transcript/session format change.
- No change to approval, credential, or slash-command dispatch priority.
- No new dependencies.

### Acceptance

- Given the user has submitted "hello" then "world"
  When the user presses Up once
  Then the composer shows "world" (most recent entry).
- Given the user presses Up again
  Then the composer shows "hello" (oldest entry).
- Given the user presses Up again at the oldest entry
  Then the composer stays on "hello" (no wrapping).
- Given the user has navigated to history and presses Down past the newest entry
  Then the exact unsubmitted draft is restored.
- Given the slash menu is open
  When the user presses Up/Down
  Then slash menu navigation occurs (not history navigation).
- Given an approval dialog is visible
  When the user presses Up/Down
  Then approval input handling occurs (not history navigation).
- Given the user submits the same text twice consecutively
  Then only one entry is recorded (consecutive duplicate dedup).

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `git diff --check`
- Runtime evidence: TUI semantic-buffer or manual PTY test proving Up/Down navigation and draft restoration.

### Documentation To Update

- `docs/backlog/active/TUI-030-composer-input-history.md`
- `docs/BOARD.md`
- `docs/iterations/README.md`
- Execution package checkpoint

### Risks And Rollback

- **Risk**: Up/Down history conflicts with future multiline cursor movement.
  **Mitigation**: Up/Down history is the only current use of these keys outside slash menu; if line-up/line-down is needed later, a guard on cursor position can be added.
- **Risk**: History recording changes `input_submit()` semantics.
  **Mitigation**: Recording happens before clear; the returned content and side effects are unchanged.
- **Rollback**: Remove history fields, methods, and Up/Down handlers. Input submission and all other key dispatch revert to existing behavior.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-15 | Code inspection | Composer, approval, and slash-command key dispatch inspected. Findings appended above. |
| 2026-07-15 | Implementation | History state fields, methods, Up/Down handlers added. 9 state tests + 5 entry-point tests pass (312 TUI total). |
| 2026-07-15 | Commit | Pushed to origin/main. Working tree clean. |

## Verification Evidence

- `cargo fmt --all -- --check`: clean.
- `cargo check --workspace --locked`: clean.
- `cargo clippy --workspace --locked -- -D warnings`: clean.
- `cargo test --workspace --locked`: all pass (312 TUI tests, including 9 state and 5 entry-point history tests).
- `./scripts/release_preflight.sh`: passed.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `git diff --check`: clean.
- **Semantic-buffer evidence**: 9 state tests cover navigation (newest→oldest→stays), draft restoration (exact multiline), boundaries (empty history, at-oldest, at-draft), dedup (consecutive and non-consecutive), cursor-to-end on load, and submit-resets-cursor.
- **Priority evidence**: Code structure proves approval/credential/slash-menu handlers return early before history Up/Down arms. No regression in existing 298 TUI tests.
- **No persistence change**: `input_submit()` records in-memory only; no file, session, or transcript write.
- **Entry-point evidence (review v2 supplement)**: 5 tests in `app_tests.rs` call `Tui::handle_input_event` with actual `Event::Key` events:
  - `entry_point_up_down_history_navigation`: Up→newest→oldest→stays, Down→newer→draft restored.
  - `entry_point_slash_menu_open_does_not_trigger_history`: slash menu open, Up via handler, history cursor stays None.
  - `entry_point_approval_active_does_not_trigger_history`: approval active, Up via handler, history untouched.
  - `entry_point_credential_input_does_not_trigger_history`: credential input active, Up via handler, history untouched.
  - `entry_point_full_roundtrip_multiline_draft`: 3 entries, multiline draft, full roundtrip through handle_input_event.
  Uses `Tui::for_test()` and `InlineTerminal::test_instance()` (both `#[cfg(test)]`) to construct a real `Tui` through test-only helpers.

## Variance And Residuals

- No variance from baseline. All acceptance criteria met.
- No residuals.

## Retrospective

- Outcome: met. All acceptance criteria closed with state tests and real `handle_input_event` entry-point evidence.
- Documentation: I130, TUI-030, Board, iterations README, execution package updated.
- Lessons: The existing key-dispatch priority chain made adding history navigation safe.
