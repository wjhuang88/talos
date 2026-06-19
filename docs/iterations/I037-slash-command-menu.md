# I037: Slash Command Menu

> Document status: Complete
> Published plan date: 2026-06-20
> Planned objective: User types `/` and sees a Codex-style command menu below the composer,
>   with real-time filtering, keyboard navigation, and shared metadata from CMD-001's registry.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `/` opens a command menu rendered below the input area; typing filters the
>   list; Up/Down/Enter/Tab/Esc navigate; close does not clear composer text.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-010 | TUI-002 | Ready | CMD-001 ✅, TUI-009 ✅ | `/` slash command menu below composer |

### Scope

- Typing `/` at composer start opens the slash command menu.
- Menu renders **below** the input area where terminal geometry permits; falls back to
  above-input when space is insufficient.
- Real-time filtering as user types after `/`.
- Keyboard: `Up`/`Down` move selection, `Enter` accepts, `Tab` completes, `Esc` closes.
- `Esc` closes menu without clearing normal composer text (per TUI-009).
- Menu content driven by CMD-001's `CommandRegistry` (same source as help/completion).
- Unavailable commands hidden or disabled per `AvailabilityPredicate`.
- Commands show label + description + argument hint where applicable.
- Menu does NOT write transient UI into scrollback history.

### Non-Goals

- Do not implement command-specific full-screen views (model picker, session picker).
- Do not migrate approval rendering into the popup layer (TUI-008 follow-up).
- Do not introduce a global event bus (ADR-006).
- Do not enumerate every model tool as a user command.

### Acceptance

- Given an idle TUI session
  When user types `/` as the first character
  Then a command menu appears below the input area

- Given the menu is open
  When user types additional characters
  Then the menu filters to matching commands in real time

- Given the menu is open
  When user presses `Esc`
  Then the menu closes and composer text is preserved

- Given the menu is open with a selection
  When user presses `Enter`
  Then the selected command is inserted into the composer

- Given the menu is open
  When user presses `Tab`
  Then the selected command name is completed

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo test -p talos-tui` (existing 105 tests + new menu tests)
- Manual: `cargo run -p talos-cli -- "test"`, type `/` in TUI, verify menu opens/filters/navigates

### Documentation To Update

- `docs/backlog/active/TUI-010-slash-command-menu.md` — AC status
- `docs/BOARD.md` — move TUI-010 from Next to Now
- `README.md` — if user-facing command behavior changes
- `docs/iterations/I037-slash-command-menu.md` — this file

### Risks And Rollback

- Risk: Popup rendering interferes with inline terminal viewport/scrolling.
  Rollback: isolate menu rendering behind a feature flag in `TuiState`, revert if viewport
  corruption is observed.
- Risk: Menu steals keyboard focus from approval overlay (future TUI-008).
  Rollback: menu activation gated on `approval_state == Hidden`; Esc+Ctrl+C semantics
  follow TUI-009.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-20 | Activation | Non-terminal inventory clean; TUI-010 dependencies met (CMD-001, TUI-009); activated as I037 |

## Verification Evidence

- `cargo test -p talos-tui` — 114 tests pass (105 existing + 9 new slash menu tests)
- `cargo test -p talos-conversation` — 58 tests pass
- `cargo clippy -p talos-tui -p talos-conversation -- -D warnings` — clean
- `cargo fmt --all` — clean
- `cargo check --workspace` — clean
- Runtime evidence: typing `/` in TUI composer opens menu; filtering, navigation, selection all work

## Variance And Residuals

- None. All TUI-010 acceptance criteria met.

## Retrospective

- Outcome: met
- Documentation: TUI-010 ACs updated, board synced, iteration README updated
