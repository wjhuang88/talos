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
| 2026-06-20 | Review reopened | Post-completion audit found hidden filter state, incorrect Backspace behavior, menu priority over a newly arrived Approval, missing placement fallback, and stale governance status. |
| 2026-06-20 | Remediation | Replaced duplicate query state with composer-backed filtering; made Approval input exclusive; implemented bounded above-input fallback; added interaction, approval, placement, and height tests; synchronized user and governance docs. |

## Remediation Closure Ledger

- Requested outcome: repair every finding from the I037 development review.
- Artifacts updated: TUI state/input handling, menu rendering/layout, tests, README, story, iteration,
  product backlog, and board.
- Existing assets preserved: CMD-001 command registry, ADR-006 single event loop, inline terminal
  scrollback boundary, and the published I037 baseline.
- State owners synchronized: TUI-010 owner, I037, product backlog, and derived board.
- Validation required: formatting, TUI tests, workspace check/clippy/test, governance validators,
  diff checks, and interactive smoke evidence.
- Residual destination: none within I037 after all required validation passes.

## Verification Evidence

- `cargo test -p talos-tui` — 120 tests pass, including composer query, Backspace, argument
  completion, Approval preemption, placement fallback, and bounded-height coverage
- `cargo test -p talos-conversation` — 58 tests pass
- `cargo clippy -p talos-tui -p talos-conversation -- -D warnings` — clean
- `cargo fmt --all` — clean
- `cargo check --workspace` — clean
- The original runtime-evidence statement was not accompanied by reproducible geometry or input
  evidence and was superseded by the post-completion remediation validation below.
- Reproducible terminal smoke: ran `target/debug/talos --mock --tui` inside detached GNU Screen;
  `/he` remained visible in the composer and filtered to `/help`, `Backspace` restored the match
  after a no-match query, and `Esc` closed the menu while preserving `/he`.
- Small-terminal smoke: resized GNU Screen to 80x12; `/` rendered a five-row capped menu above the
  composer while keeping the composer and status row visible.
- `cargo check --workspace` — passed after remediation.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed after remediation.
- `cargo test --workspace` — passed after remediation; no failures, one pre-existing
  timing-sensitive ignored Agent test.
- `cargo fmt --all -- --check`, `git diff --check`, POSIX governance validation, and PowerShell
  governance validation — passed.

## Variance And Residuals

- Initial implementation variance: filtering used hidden menu state instead of the composer;
  Backspace closed the menu; Approval did not preempt an open menu; above-input fallback was not
  implemented; user and governance documentation was stale.
- Resolution: all identified variances were repaired in the 2026-06-20 remediation pass.

## Retrospective

- Outcome: met
- Documentation: English and Chinese README files, TUI-010 owner, product backlog, board, and this
  iteration are synchronized.
- Lesson: state-only menu tests cannot substantiate input-routing or terminal-geometry acceptance;
  future popup work must test the owning input state and deterministic placement calculation.
