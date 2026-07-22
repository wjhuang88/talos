# Iteration I145: Queued Steering Message Display

> Document status: Review
> Published plan date: 2026-07-20
> Planned objective: implement TUI-026's bounded, engine-owned display of queued steering message
> content without changing steering delivery semantics.
> Baseline rule: this iteration selects TUI-026 only. Editing, cancelling, reordering, persisting,
> or cross-session handling of queued messages requires a separate story.
> MVP deliverable: a runnable TUI that shows a bounded FIFO queue preview above the composer while
> a turn is processing, with correct width, cursor and terminal-height behavior.

## Published Baseline

- Selected Ready story: TUI-026, under ADR-049.
- Dependencies satisfied: TUI-004 state ownership, TUI-032 multiline composer, ADR-035 scrollback
  boundary and ADR-039 ordered UI flow.
- `ConversationEngine` remains the single steering-queue owner. The only new projection is bounded
  `UiOutput::SteeringQueueSnapshot` in the existing ordered stream.
- The snapshot limits are immutable for this slice: first 8 FIFO entries, 4 KiB UTF-8 per entry,
  exact total/omitted counts. The TUI render budget is 6 terminal rows; a height-constrained
  terminal collapses the preview before reducing the composer below one row.
- New public `UiOutput` variant is a pre-1.0 semver break for exhaustive downstream matches.
  Implementation must add migration notes and the later release must be a minor bump, not a patch.

## Scope

1. Add typed bounded queue snapshot data and one `UiOutput` projection; emit it on every
   authoritative queue mutation in FIFO order with the related status update.
2. Render a transient queue preview above the composer. Reuse TUI-032 display-width/CJK wrapping,
   cursor and scroll calculations; do not render the queue in terminal history.
3. Reconcile preview state across enqueue, dequeue after full turn completion, terminal error or
   cancellation, new/resume session and TUI exit.
4. Add unit/integration/layout regressions and update user-facing help/documentation and public API
   migration notes.

## Explicit Non-Goals

- No TUI-local queue mirror, second channel, global event bus, concurrent turns or changed drain
  timing.
- No queued-message editing, cancellation, deletion, reordering, durable persistence or resume.
- No change to permission, sandbox, Session/TLOG, RPC input, tool behavior or finalized scrollback.
- No release tag in this iteration; release selection remains a follow-up after acceptance.

## Acceptance

- Given A/B/C submitted during one active turn, When the TUI receives projections, Then it shows
  FIFO previews and exact count 3 while keeping the composer ready for another input.
- Given a tool-use intermediate event, When no authoritative turn completion has occurred, Then no
  queued item is removed or sent.
- Given completion, error, cancellation or session replacement, When the canonical queue changes,
  Then the preview changes in the same ordered UI flow with no stale rows.
- Given long, multiline or CJK queued content at wide and narrow terminal sizes, When the preview
  is rendered, Then glyph width, 6-row preview cap, composer minimum row, cursor location and
  composer scroll offset remain correct.
- Given more than 8 queued items or a message over 4 KiB, When projected, Then total and omitted
  counts are exact and truncation is explicit without unbounded UI payloads.
- Given a downstream exhaustive `UiOutput` match, When the public API is upgraded, Then migration
  documentation identifies the new variant and release notes require handling or wildcard fallback.

## Planned Validation

- Engine: FIFO order, snapshot bounds/truncation, full-turn-only dequeue, terminal clear paths.
- CLI bridge: ordered snapshot/status projection and session lifecycle reconciliation.
- TUI: rendering at 60/80/100+ widths, CJK/multiline text, six-row cap, narrow height collapse,
  composer cursor/scroll, slash/credential/approval priority.
- Real terminal acceptance: enqueue multiple messages during a tool-running turn, inspect FIFO
  display, then verify one-at-a-time dispatch after completion.
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Required Reads

- `docs/backlog/active/TUI-026-queued-input-display.md`
- `docs/decisions/035-tui-history-scrollback-boundary.md`
- `docs/decisions/039-runtime-event-semantic-single-flow.md`
- `docs/decisions/049-steering-queue-projection-boundary.md`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/scrollback.rs`
- `crates/talos-tui/src/scrollback_input.rs`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Planning | Created after I144 completed. No implementation, release, tag, or production-code change has started. |
| 2026-07-20 | Implementation | Core types + engine snapshot + TUI rendering + bridge lifecycle + tests. |
| 2026-07-20 | Rework 1 | Fixed 6 blockers: 6-row cap via `plan()`, hidden count = total - rendered, multiline normalization (`\n` → `⏎`), 4 KiB truncation reserves ellipsis bytes, lifecycle snapshots (cancel/error/success), TUI regression tests. |
| 2026-07-20 | Rework 2 | Session boundary cleanup: `/new`, `/resume`, `/fork` success paths emit empty `SteeringQueueSnapshot`. Truncation marker `⚠` width reserved in text budget. Buffer+InlineFrame render tests added (8-entry header/4-entry/+4 summary, truncated+narrow, CJK, multiline, empty, narrow-height compress). |
| 2026-07-20 | Rework 3 | Added lifecycle and render coverage for queue projection, and updated README/README.zh-CN with user-facing queued-steering documentation. The initial layout coverage duplicated application arithmetic; Rework 4 replaced it with tests of the production allocator. Status changed to Review. |
| 2026-07-20 | Rework 4 | Corrected narrow-height layout through the production `compress_layout()` helper: `draw_frame` first reserves fixed rows, then allocates the remaining budget in modal → composer (minimum one row when possible) → queue order. The bottom panel is counted once for placement. Composer text rendering and cursor scrolling use the same allocated height. `/new`, `/resume`, and `/fork` now share a tested ordered boundary helper that emits queue-clear before `SessionIdentity`. |
| 2026-07-22 | Rework 5 — terminal-growth repaint and terminal-preview cleanup | Maintainer terminal acceptance proved queue ownership, count, and FIFO logic correct but exposed stale Tip text overlaid behind newly added queue rows. `InlineTerminal::draw` had cleared only shrinking viewports even though any viewport resize/reposition resets both diff buffers. It now forces a full clear for every viewport-area change before repainting. Separately, `TurnPhase::Cancelled` intentionally persists in the status snapshot, but the preview had rendered terminal phases even when inactive; it now clears after cancellation while the status bar retains the result. Growth and terminal-phase regressions protect both invariants. I145 remains Review pending a clean real-terminal rerun of the queue preview and drain. |

## Actual Validation Results (2026-07-20)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ clean |
| `cargo test --workspace --locked` | ✅ workspace unit, integration, and doctests pass (including dashboard loopback tests) |
| `scripts/validate_project_governance.sh .` | ✅ 0 warnings |
| `git diff --check` | ✅ clean |

### Rework 5 Validation (2026-07-22)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo check --workspace --locked` | ✅ exit 0 |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ clean |
| `cargo test -p talos-tui --locked` | ✅ 388 tests + 2 doctests pass |
| `git diff --check` | ✅ clean |

## Remaining: Real Terminal Acceptance

The following acceptance items require real terminal verification and are deferred for manual acceptance:

- Enqueue multiple messages during tool-running turn, verify FIFO display.
- Verify `+N more` summary appears for queues longer than 6 lines.
- Verify queue preview clears when queue empties.
