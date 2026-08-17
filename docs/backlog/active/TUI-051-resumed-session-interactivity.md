# TUI-051: Resumed Session Interactivity Under Provider Delay

| Field | Value |
|---|---|
| Story ID | TUI-051 |
| Type | TUI / Runtime Reliability Story |
| Priority | P0 |
| Status | Ready / Planned / Unclaimed |
| Source | [GitHub Issue #272](https://github.com/wjhuang88/talos/issues/272) |
| Selected Iteration | I209 |
| Depends On | PROVIDER-005/#270/#271 Complete on main; existing structured-turn cancellation and history-projection contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #272 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Reproduce the resumed-turn cancellation loss at an exact main head, then establish an effective I209 claim before implementation. |

## Planning Checkpoint — 2026-08-17

- TUI-051 first reached main as `Intake / Unclaimed / Selected Iteration None` in PR #271 merge
  `89523dbc`; that registration did not select or authorize implementation.
- PROVIDER-005 is Complete at implementation commit `1d31847a`, its owner-first closeout merged in
  PR #274 as `c15da4cf`, Issue #270 is closed, and the open-Issue matrix synchronized through PR
  #275 merge `abf88657`.
- This planning slice refines TUI-051 to Ready, selects runnable iteration I209, and leaves both
  owner and iteration Unclaimed. No implementation branch or code change is authorized.

## Identity / Goal / Value

A user who resumes a large Session must be able to see bounded provider retry progress and cancel
the active turn promptly. A large transcript must not consume a full CPU core by reprojecting
unchanged history on every frame or starve keyboard input.

The incident evidence is Session `a07b39a9-d270-465a-8d80-49efdb530a4f`, turn
`turn_58412_1_1`: two provider dispatch cycles each took about 252 seconds under a 60-second
dispatch timeout with three retries, while the 342,988-byte transcript drove the TUI process to
about 100% CPU in `draw_frame -> history_projection::project_history`.

## Scope

- Make provider dispatch timeout/retry state visible through the existing turn-phase/status path,
  including the current attempt and a bounded wait indication.
- Preserve cancellation from TUI `Esc` through the generation- and turn-bound actor route during
  initial dispatch, retry backoff and first-packet wait, including after Session resume.
- Cache or invalidate history projection so an unchanged transcript is not fully reprojected on
  every frame; viewport, width, style or transcript changes must still invalidate correctly.
- Ensure supported process-termination paths restore terminal raw/alternate-screen modes, and
  document a bounded tty recovery command for an unavoidable hard termination.
- Add deterministic integration tests for a resumed structured turn, delayed response headers,
  retry/backoff cancellation and a large unchanged transcript.
- Capture CPU/input-latency and real-terminal evidence at the implementation exact head.

## Exclusions

- No UTF-8 transport decoding change from PROVIDER-005/#270/#271.
- No I200/#79 no-op scroll-position semantic change.
- No I206/TUI-048 steering activation behavior or steering insertion timing.
- No provider-specific timeout-default change, retry-policy redesign, transcript deletion,
  compaction-policy change, persistence migration or public API break.
- No global event bus or unrelated TUI rendering refactor.

## Dependencies And Constraints

- PROVIDER-005/#270/#271 are closed. Their emergency authorization does not extend to I209.
- The actor-owned cancellation token and generation-bound `InterruptTurn` identity remain the
  authority; the TUI must not bypass Session custody.
- NET-001 owns any broader retry/deadline policy redesign. I209 may expose existing bounded retry
  facts and make their waits cancellable, not redefine network policy.
- History caching must preserve visible-cell selection, Unicode display width, resize behavior,
  scroll anchors and transcript truth.

## Uncertainty And Validation Path

The provider retry duration and render hotspot are measured facts. The exact point where the
resumed-session `Esc` request was delayed or lost is not yet proven. Before implementation, add an
integration reproduction that observes `UserInput::Cancel`, `SessionOp::InterruptTurn`, actor token
cancellation and durable terminal outcome independently; do not infer the loss point from the
incident alone.

## State / Status Owners

- This Story owns readiness and requirement truth.
- `docs/iterations/I209-resumed-session-interactivity.md` owns execution state and completion
  evidence.
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md` and `docs/BOARD.md` are derived
  views only.
- Issue #272 remains open until the owner acceptance is delivered or explicitly cancelled.

## User-Facing Documentation

- Update TUI help/troubleshooting text if cancellation or retry status wording changes.
- Record the supported active-turn interrupt key and timeout-attempt semantics in the applicable
  README/reference surface selected during implementation.

## Required Reads

- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/backlog/active/NET-001-network-resilience-policy.md`
- `docs/backlog/active/TUI-048-steering-esc-activation.md`
- `docs/iterations/I166-interrupt-shortcut-reliability.md`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/openai_sse.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-agent/src/session.rs`
- `crates/talos-agent/src/session/turn.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/history_projection.rs`

## Acceptance For Behavior

- Given a resumed large Session and a provider that delays response headers across retries, when
  the user presses `Esc`, then the active turn becomes durably cancelled promptly without waiting
  for the provider timeout window.
- Given dispatch retry or first-packet wait, when the TUI is active, then status distinguishes the
  current bounded wait/retry from an indefinite frozen connection.
- Given an unchanged large transcript, when idle or waiting for provider output, then redraw does
  not repeatedly perform a full history projection and keyboard input remains responsive.
- Given transcript, viewport width, resize, selection or style changes, when the next frame draws,
  then the projection invalidates and renders the correct current content.
- Given Talos is terminated through a supported signal while the TUI owns raw mode, when control
  returns to the shell, then normal keyboard input and terminal rendering are restored.

## Acceptance For Technical Work

- [ ] Deterministic tests cover cancel during initial dispatch, retry backoff and first-packet wait.
- [ ] A resumed structured-turn integration test proves exact generation/turn cancellation and a
      durable `terminal_cancelled` outcome.
- [ ] Projection tests prove cache reuse and invalidation without changing scroll/selection truth.
- [ ] Exact-head CPU/input-latency evidence uses a large persisted transcript and a real terminal.
- [ ] A real-terminal termination test proves terminal mode restoration and records the bounded
      recovery path for an unavoidable hard kill.
- [ ] Focused and workspace locked validation, governance validators and `git diff --check` pass.
- [ ] Owner-first status, Issue #272 and user-facing documentation are synchronized.
