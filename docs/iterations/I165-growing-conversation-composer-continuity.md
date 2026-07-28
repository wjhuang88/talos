# Iteration I165: Growing Conversation Composer Continuity

> Document status: Active
> Published plan date: 2026-07-28
> Planned objective: Keep the Alternate-Screen composer directly below the Logo/history flow until projected content exhausts the usable frame, then use the bounded bottom-composer history viewport.
> Baseline rule: this iteration replaces only the paused I164 post-first-submit acceptance target; all ADR-054 renderer ownership invariants remain unchanged.
> MVP deliverable: A rebuilt `talos` keeps a short first conversation visually contiguous with the Logo and moves to bottom-fixed composer only at overflow.

## Iteration Inventory And Disposition At Selection

| Iteration | State | Disposition |
| --- | --- | --- |
| I164 / TUI-038 | Paused after manual verification | Preserve published evidence; changed target moved here. |
| I157 / MODEL-010 | Planned | Preserved and deferred until I165 disposition. |
| I158-I162 | Blocked | Unchanged. |
| I165 / TUI-039 | Active | Sole implementation authority. |

## Authorized Scope

- calculate a FollowTail conversation history cap from current-width projected
  history rows plus the display-only Logo prefix;
- retain inline placement while that flow fits; fall back to the existing
  bounded bottom-composer layout on overflow or anchored-history navigation;
- add tests before changing production layout behavior;
- update story, Board, program, and this record with validation evidence.

## Non-Goals

- No native terminal scrollback, primary-screen rendering, second renderer,
  transcript persistence change, or modal redesign.
- No I157 or I158-I162 work, release/version/publish/tag, or ADR change.

## Acceptance And Validation

The implementation must prove first-submit continuity, progressive growth,
overflow fallback, resize/CJK/multiline safety, wheel/anchored-history behavior,
and bounded full-frame rendering. Before completion, run the locked TUI and
workspace validation ladders and perform rebuilt-binary terminal acceptance.

## Execution Record

| Date | Type | Record |
| --- | --- | --- |
| 2026-07-28 | Activation | Maintainer rejected I164's post-first-submit bottom-composer UX during manual verification and authorized the replacement layout target. I164/TUI-038 is paused without completion; I165/TUI-039 is the sole Active implementation authority. |
| 2026-07-28 | Implementation | `8e6ffe9` caps FollowTail history to the Logo plus current-width projected content rows. Composer/status therefore follow a short conversation, while normal allocation returns to bounded bottom placement at overflow or during anchored history navigation. |
| 2026-07-28 | Automated validation | `cargo test --locked -p talos-tui --lib` = 464 passed; `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, `cargo test --workspace --locked`, `scripts/validate_project_governance.sh .`, `git diff --check`, and `cargo build --locked -p talos-cli` all exit 0. The build script emitted only its informational `models.toml compressed` warning. |
| 2026-07-28 | Approved scope refinement | Maintainer reduced the initial display-only Logo-to-composer gap from two rows to one row. This preserves the renderer/transcript model and is covered by the startup layout assertion. |
| 2026-07-28 | Startup tips correction | Maintainer confirmed that the normal tips surface must remain visible in a fresh session. Startup now keeps its one-row tips surface, so the existing loopback Dashboard address tip is visible and copyable before first submit. `cargo test --locked -p talos-tui --lib` = 465 passed. |

## Completion Evidence

- Implementation Commit: `8e6ffe9`.
- Completion Commit: pending.
- Human rebuilt-binary acceptance: pending.
