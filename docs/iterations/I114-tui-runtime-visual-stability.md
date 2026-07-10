# Iteration I114: TUI Runtime Visual Stability

> Document status: Active
> Published plan date: 2026-07-10
> Planned objective: Close the verified TUI-028 visual reliability gaps without changing session or provider semantics.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A real `talos` TUI run has stable time-based processing animation, a two-color three-segment thinking ripple, and compact display-width-safe model status rendering.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| #24 | TUI-028 | Open | Existing TUI render loop | Processing and ellipsis frames advance only on a fixed timer, not on redraws caused by input or stream traffic. |
| #25 | TUI-028 | Open | #24 timer | The transient `thinking` label uses exactly two colors in three contiguous segments whose center segment expands and contracts. |
| #31 | TUI-028 | Complete (2026-07-10) | Existing status renderer | Model switching uses display-width-aware truncation and a single-line redraw without a large padding gap. |
| #39 | TUI-028 | Open | Commit `2b0600e` | A focused regression and runtime check prove dashboard availability stays a transient tip and never enters scrollback. |

### Scope

- Decouple processing animation state advancement from `draw_frame` redraw frequency.
- Render the existing transient thinking preview with the requested two-color, three-segment center-out ripple.
- Make model-name truncation terminal-display-width-aware without padding the provider away from it.
- Add focused tests and PTY evidence through the real `talos` binary.

### Non-Goals

- No thinking history/archive work; that remains `TUI-029` under ADR-034 v4.
- No session schema, provider protocol, permission, sandbox, or dependency change.
- No general TUI refactor or alternate terminal backend.

### Acceptance

- Given repeated input, stream, or tool-output redraws while processing, when no animation timer tick occurs, then the processing frame and ellipsis frame remain unchanged; each 150ms timer tick advances exactly one frame.
- Given a live thinking preview, when it renders across successive animation frames, then `thinking` is represented by three contiguous segments using only the two designated colors, with the primary center segment expanding from the center and contracting cyclically.
- Given two model names with different ASCII or Unicode display widths at the same terminal width, when the active model changes, then the status row redraws as one bounded line, the provider remains adjacent to the model label, and the line does not exceed its width budget.
- Given the dashboard starts in TUI mode, when availability is reported, then the real binary displays a transient tip and no dashboard text is written to scrollback.

### Planned Validation

- `cargo fmt --check`
- `cargo test -p talos-tui`
- `cargo clippy -p talos-tui -- -D warnings`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- PTY capture of `target/debug/talos --mock --no-init` demonstrating timer-driven preview behavior and a clean exit; host `script` is a compatibility harness only, with no runtime dependency. If unavailable, record the platform limitation and replace it with a Talos-owned PTY harness before completion.

### Documentation To Update

- `README.md` only if user-visible TUI operation changes; otherwise the TUI owner, iteration index, product backlog, and board.
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
- `docs/iterations/README.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: frame advancement and rendering become out of sync around terminal status changes.
- Rollback: revert the isolated TUI commits; no persisted state or protocol data changes.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-10 | Activation | Inventory: no implementation iteration was Active; I106-I109 remain Review with non-qualifying self-bootstrap evidence and are not blockers for this independent TUI repair; I085 remains Paused on MC107; I018-I020, I028, I086-I089, and other Planned records remain deferred/blocked as recorded in `docs/iterations/README.md`. The maintainer explicitly selected TUI-028 over those planned items. The 2026-07-09 deferral of #24/#25/#31 is retained as history in the owner doc and superseded for execution by this iteration. |

## Verification Evidence

- `cargo test -p talos-tui`: 256 passed; includes timer-only frame advancement, two-color
  three-segment ripple, display-width model-slot, and tip-TTL regressions.
- `cargo test -p talos-cli mode_runners::tests::dashboard_notifications_are_transient_and_never_include_tokens`:
  1 passed; dashboard availability/failure is a `UiOutput::Tip`, and non-loopback TUI text does
  not include a token.
- `cargo build -p talos-cli`: passed.
- `scripts/validate_project_governance.sh .`: passed with 0 warnings.
- Runtime attempt: `HOME=/private/tmp/talos-i114-home target/debug/talos --mock --no-init` in the
  available PTY reached splash output and emitted the cursor-position query, but the host PTY did
  not respond to `ESC[6n`; Talos exited with `failed to initialize TUI` rather than producing a
  false visual result. Real visual evidence remains required before completion.
- Native runtime evidence: `/Users/GHuang/Downloads/录屏2026-07-10 10.35.37.mov` (57.5 seconds,
  Alacritty) shows the real TUI processing a request and handling cancellation. It also exposed the
  direct-`stderr` #39 viewport corruption documented below; it does not yet prove #24 or #31.
- Native visual confirmation: the maintainer confirmed in Alacritty that the live `thinking`
  preview uses the requested two-color, three-segment center-out ripple. #25 is accepted; no
  persistence or history behavior changed.
- Native visual confirmation: after the padding correction, the maintainer confirmed in Alacritty
  that model and provider names are compact and adjacent, with no visible layout fault. #31 is
  accepted.

## Variance And Residuals

- The prior claim that the animation path was independent of redraw workload was incorrect: `draw_frame` advanced `processing_tick` on every redraw. I114 corrects both the behavior and the missing evidence.
- Visual evidence recorded on 2026-07-10 exposed a #39 follow-up: direct `stderr` dashboard
  diagnostics were drawn into the inline viewport, leaving a stale line or a blank startup row.
  The next corrective commit routes those diagnostics to the terminal-UI log sink; the Tip remains
  the sole TUI notification surface.
- #24 still needs an observable processing-animation cadence capture. #39 still needs a native
  capture after its `stderr`-to-log-sink correction.

## Retrospective

- Outcome: active.
- Documentation: activation records updated before code changes.
- Lessons: pending.
