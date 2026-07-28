# Iteration I166: Interrupt Shortcut Reliability

> Document status: Complete
> Published plan date: 2026-07-28
> Planned objective: Give Talos one deterministic shortcut model in which Ctrl+C owns local clear/close and idle exit, while Esc owns active-turn interruption after higher-priority UI consumes its local cancel action.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: In the rebuilt interactive binary, Esc reliably interrupts every normal active turn without restart, while Ctrl+C never interrupts or exits an active turn and all modal/approval priorities remain deterministic.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-036` | None | Ready | TUI-009 Complete; TUI-035/I156 Complete; existing `UserInput::Cancel` bridge | Deterministic Ctrl+C local-clear/idle-exit and Esc active-turn cancellation with modal priority and real-terminal evidence. |

### Baseline And Ownership

- Starting HEAD: `5c1acb78435be20af5d8412139bda2e618761338`
- Branch: `main`
- Execution mode: direct main-branch implementation; no worktree while no parallel task exists.
- TUI key dispatch and visible feedback owner: `crates/talos-tui`.
- Cancellation transport remains the existing
  `UserInput::Cancel -> SessionOp::Interrupt -> engine.cancel_turn()` path.
- No public protocol or session-persistence change is authorized.

### Authorized Shortcut Semantics

#### Ctrl+C

- Normal composer with content: clear the composer locally, regardless of
  whether a turn is idle or processing; do not emit `UserInput::Cancel`.
- Empty composer while processing: do not cancel and do not arm/complete the
  idle exit gesture; show bounded guidance that Esc interrupts the turn.
- Empty composer while idle: retain the explicit double-Ctrl+C exit gesture.
- Slash menu, credential input, provider wizard, or approval: consume Ctrl+C
  locally as close/clear/cancel; never append a literal `c`, interrupt the turn,
  or exit the process.

#### Esc

- Approval visible: resolve the local cancel as `ApprovalChoice::Deny`; do not
  also interrupt the turn.
- Credential input or provider wizard: close/cancel that modal and clear its
  local input; do not also interrupt the turn.
- Slash menu or other dismissible bottom panel: close it; do not also
  interrupt the turn.
- No higher-priority UI and an active turn: emit exactly one
  `UserInput::Cancel` for that key press and show bounded cancellation feedback.
- No higher-priority UI and no active turn: no-op; preserve composer content.

### Scope

- Reorder/refine TUI key dispatch so modifier-aware shortcuts are never treated
  as ordinary modal text input.
- Remove active-turn cancellation from Ctrl+C without changing its idle clear
  and idle exit behavior.
- Route normal active-turn Esc through the existing cancellation transport.
- Preserve approval/modal/slash priority so one key event performs only one
  action.
- Keep cancellation stateless across turns: do not add a long-lived latch that
  can leave later turns permanently non-interruptible.
- Update the composer hint and EN/zh-CN keyboard documentation.
- Add entry-point tests through `Tui::handle_input_event`; helper-only tests are
  insufficient.

### Non-Goals

- No change to tool execution, permission decisions, provider behavior,
  session persistence, transcript facts, queue draining, or cancellation
  protocol.
- No new global event bus, cancellation channel, dependency, `unsafe`, or
  renderer.
- No change to Alternate-Screen ownership, layout, history projection, mouse
  scrolling, Shift+Enter, or terminal restoration.
- No implementation of I157, I158-I162, OBS-002, or unrelated shortcut work.
- No tag, publish, release, or workspace-version change.

### Required Failing Tests Before Production Changes

- `entry_point_esc_cancels_active_turn_once`
- `entry_point_esc_can_cancel_a_later_turn_after_cancelled_failed_and_timed_out_states`
- `entry_point_esc_idle_preserves_composer`
- `entry_point_esc_slash_menu_closes_without_turn_cancel`
- `entry_point_esc_credential_closes_without_turn_cancel`
- `entry_point_esc_provider_wizard_closes_without_turn_cancel`
- `entry_point_esc_approval_denies_without_turn_cancel`
- `entry_point_ctrl_c_active_draft_clears_without_cancel_or_exit`
- `entry_point_ctrl_c_active_empty_does_not_cancel_or_exit`
- `entry_point_ctrl_c_idle_empty_retains_double_press_exit`
- `modified_ctrl_c_is_never_inserted_into_modal_input`
- `repeated_esc_does_not_corrupt_cancellation_state`

Tests must assert the actual `UserInput` channel contents, exit return value,
composer/modal state, approval response, and visible Tip where applicable.

### Acceptance

- Given any normal active turn, when Esc is pressed, then exactly one existing
  cancellation request is emitted for that key event and Talos remains running.
- Given a later active turn after cancellation, failure, timeout, or successful
  completion, when Esc is pressed, then that turn is still interruptible
  without restarting Talos.
- Given approval, credential/provider, or slash UI, when Esc is pressed, then
  only its local cancel action occurs.
- Given an active turn, when Ctrl+C is pressed, then it never emits
  `UserInput::Cancel` and never exits; a non-empty composer is cleared locally.
- Given an idle empty composer, when the documented double-Ctrl+C gesture is
  used, then Talos exits cleanly.
- Given modal input, modified Ctrl+C is never accepted as literal credential or
  provider text.
- Shift+Enter/Ctrl+J, app-owned history, layout, terminal lifecycle, and
  exhaustive restore regressions remain green.

### Planned Validation

```bash
cargo test --locked -p talos-tui --lib entry_point_esc
cargo test --locked -p talos-tui --lib ctrl_c
cargo test --locked -p talos-tui --lib approval
cargo test --locked -p talos-tui --lib credential
cargo test --locked -p talos-tui --lib wizard
cargo test --locked -p talos-tui
cargo test --locked -p talos-conversation -p talos-cli
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
cargo build --locked -p talos-cli
```

### Runtime Evidence

Use the binary built from the implementation commit in Alacritty or an
equivalent keyboard-capable terminal:

1. start a turn and press Esc during streaming;
2. start another turn and press Esc during or after a tool call;
3. queue a steering message, cancel the current turn, let the queued message
   start, and press Esc again;
4. verify Ctrl+C clears an active draft without cancelling the active turn;
5. verify Ctrl+C with an empty active composer neither cancels nor exits;
6. verify idle double-Ctrl+C still exits and restores Primary Screen;
7. verify Esc/Ctrl+C priority in slash menu, approval, credential input, and
   provider wizard;
8. verify the hint says Esc interrupts and no longer says Ctrl+C interrupts.

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/active/TUI-036-interrupt-shortcut-reliability.md`
- `docs/backlog/active/TUI-009-input-and-session-exit-polish.md` change-control note
- this iteration
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: Esc may both close a modal and cancel its underlying turn.
  Prevention: return immediately after every higher-priority UI action and
  assert the input channel remains empty.
- Risk: modified Ctrl+C may be treated as `KeyCode::Char('c')` in credential or
  wizard input. Prevention: shortcut/modifier handling precedes generic
  character insertion.
- Risk: a new cancel-pending latch can reproduce the permanent stuck state.
  Prevention: use authoritative processing state and the existing idempotent
  cancellation path; do not add a persistent latch.
- Rollback: revert the I166 implementation commit; `f77a6f0` remains the safe
  current Ctrl+C behavior.

### Stop And Escalate

- The existing `UserInput::Cancel` path cannot support Esc without public
  protocol changes.
- Approval Esc semantics require a permission-policy change rather than local
  `Deny`.
- Reliable cancellation requires a new dependency, `unsafe`, global event bus,
  or session-persistence change.
- Baseline tests reveal an unexplained cancellation or terminal lifecycle
  failure.

Record the exact evidence under Variance And Residuals and keep I166/TUI-036
Blocked or Active as appropriate; do not broaden scope.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-28 | Activation | Maintainer explicitly prioritized the shortcut refactor. Fresh inventory found no Active or Review iteration; I164 remains Paused, I157 remains Planned and is deferred until I166 disposition, I158-I162 remain Blocked, and TUI-036 dependencies are Complete. TUI-036 moved from Refinement to Ready and was selected into I166, then I166/TUI-036 became the sole Active implementation authority. Baseline `5c1acb78`; direct `main`, no parallel worktree. |
| 2026-07-28 | Implementation | `d1a8759` adds 12 entry-point tests locking the target Esc/Ctrl+C dispatch semantics. `d85514e` refactors `handle_input_event` so Ctrl+C is checked before all modal blocks (preventing literal 'c' insertion), active-turn cancellation moves from Ctrl+C to Esc, approval gets Esc→Deny, and the hint/README text is updated. 480 TUI tests and 2542 workspace tests pass. |
| 2026-07-28 | Acceptance correction | `264ba8c` corrects the post-clear Ctrl+C guidance, proves the full idle double-press state transition, adds slash-menu and approval Ctrl+C entry-point coverage, and reconciles the TUI-036 owner evidence. 483 TUI tests and 2545 workspace tests pass. |
| 2026-07-28 | Maintainer acceptance | The maintainer rebuilt and exercised binary commit `264ba8c` in Alacritty. Streaming and consecutive queued turns remained Esc-interruptible; active draft and empty-composer Ctrl+C behavior, slash/approval/credential/provider priority, Shift+Enter/Ctrl+J regression, idle double-Ctrl+C exit, and Alternate-Screen restoration all passed. |

## Verification Evidence

- Focused tests: 15 entry-point tests through `Tui::handle_input_event` +
  renamed legacy consecutive-cancel test. All pass.
  `cargo test --locked -p talos-tui --lib entry_point_esc` = 7 passed.
  `cargo test --locked -p talos-tui --lib entry_point_ctrl_c` = 6 passed.
  `cargo test --locked -p talos-tui --lib modified_ctrl_c` = 1 passed.
  `cargo test --locked -p talos-tui --lib repeated_esc` = 1 passed.
  `cargo test --locked -p talos-tui --lib test_ctrl_c` = 3 passed.
  `cargo test --locked -p talos-tui --lib esc_cancels_each` = 1 passed.
  `cargo test --locked -p talos-tui --lib` = 483 passed, 0 failed.
- Full locked validation: `cargo fmt --all -- --check` clean;
  `cargo check --workspace --locked` exit 0;
  `cargo clippy --workspace --locked -- -D warnings` exit 0;
  `cargo test --workspace --locked` = 2545 passed, 0 failed;
  `scripts/validate_project_governance.sh .` = 0 warnings;
  `git diff --check` clean;
  `cargo build --locked -p talos-cli` exit 0.
- Rebuilt-binary runtime evidence: passed in Alacritty using
  `target/debug/talos` built from `264ba8c`. All guided shortcut, queued-turn,
  modal-priority, multiline-input, idle-exit, and terminal-restore cases passed.
- Governance activation validation: `scripts/validate_project_governance.sh .` passed with
  0 warnings; `git diff --check` passed.

## Completion Evidence

- Completion Commit: `d1a8759e`, `d85514ef`, `264ba8c0`.
- I166 and TUI-036 completed after locked validation and maintainer
  rebuilt-binary Alacritty acceptance.

## Variance And Residuals

- I157/MODEL-010 remains Planned/Ready and is deferred, not superseded.
- The already-landed `f77a6f0` fixes the stale Ctrl+C exit-state defect but is
  not evidence for the new Esc shortcut deliverable.

## Retrospective

- Outcome: Ctrl+C now owns local clear/close and idle exit, while Esc reliably
  interrupts normal active turns without contaminating later queued turns.
- Documentation: README keyboard guidance, TUI-036, iteration index, Board, and
  v0.6 program state are synchronized.
- Lessons: shortcut dispatch must resolve modifier-aware and modal-local actions
  before generic character input, and exit gestures must not share state with
  turn cancellation.
