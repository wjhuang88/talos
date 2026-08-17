# Iteration I209: Resumed Session Interactivity Under Provider Delay

> Document status: Review / Claimed
> Planned date: 2026-08-17
> Objective: deliver TUI-051 so a resumed large Session remains responsive, exposes bounded
> provider retry progress and can cancel an active turn promptly.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline session (GPT-5) |
| Work Slice | I209 only: cache and invalidate unchanged TUI history projection; prove and repair resumed structured-turn Esc cancellation across TUI/bridge/actor/durable boundaries; project existing bounded provider retry facts; verify terminal restoration and update directly affected user documentation. Excludes retry-policy redesign, I200 scrolling, I206 steering, persistence migration, public API, dependency, release and broad renderer work. |
| Claimed At | 2026-08-17 |
| Source Issue | #272 |
| Governance Claim PR | #276 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer directed continued mainline execution. No separate natural-person reviewer is available in the unattended flow; claim merge requires exact-head CI, both governance validators, merge-time dependency/overlap CAS and no unresolved blocking feedback. Executing, technical-audit and merge roles may be separated, but the shared GitHub identity limitation is explicit and no distinct natural person is fabricated. |
| Implementation PR | #279 |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | PR #279 requires exact-head CI, real-terminal CPU/input and terminal-restoration evidence, independent review and merge-time CAS before closeout. |

## Selected Story

- `TUI-051` - `docs/backlog/active/TUI-051-resumed-session-interactivity.md`

## Activation Gate

- PROVIDER-005/#270/#271 are Complete/Closed through implementation `1d31847a`, owner closeout
  `c15da4cf` and remote synchronization `abf88657`; their emergency authority is not reused.
- The exact loss point for resumed-turn `Esc` is reproduced across the TUI, bridge, actor and
  cancellation-token boundaries.
- Current Active, Review, Planned, Blocked and Paused work is inventoried and dispositioned.
- An effective target-branch Collaboration Claim exists before an implementation branch is made.

## Runnable Deliverable

A rebuilt `talos -c` demonstrates that a persisted large Session no longer consumes a full CPU
core while unchanged, shows bounded provider retry progress, and durably cancels a delayed resumed
turn through `Esc`, with deterministic regression tests, terminal-mode restoration and a
real-terminal trace.

## Scope

- Existing OpenAI-compatible dispatch/retry status projection and cancellation responsiveness.
- Generation-bound resumed structured-turn interruption.
- TUI history-projection caching/invalidation for large unchanged transcripts.
- Focused, workspace and real-terminal verification plus user-facing status/help documentation.

## Exclusions

- #271 UTF-8 transport decoding, timeout-default or retry-policy redesign.
- I200/#79 scroll semantics and I206/TUI-048 steering activation.
- Transcript deletion, persistence migration, compaction policy, public API changes or a broad TUI
  rendering refactor.

## Acceptance

- [ ] Delayed response headers and retry backoff expose bounded, changing status instead of a
      static indefinite `connecting...` label.
- [ ] `Esc` promptly cancels initial dispatch, retry backoff and first-packet wait after resume,
      with durable terminal-cancelled evidence.
- [ ] Unchanged large history reuses its projection; transcript/viewport/style/selection changes
      invalidate it correctly.
- [ ] A real persisted transcript comparable to the incident remains input-responsive without
      sustained full-core redraw work.
- [ ] Supported termination signals restore raw/alternate-screen terminal state before returning
      control to the shell.
- [ ] Locked focused/workspace checks, governance validators and `git diff --check` pass at the
      reviewed exact head.
- [ ] TUI help/troubleshooting, Issue #272, owner status and derived views are synchronized.

## Planned Validation

```bash
cargo test -p talos-provider --locked
cargo test -p talos-agent --locked
cargo test -p talos-cli --locked
cargo test -p talos-tui --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .
git diff --check
```

## Current Disposition

I209 is Planned / Unclaimed in the upcoming task pool. Its provider prerequisite is now closed, and
it precedes I205 plus the existing long-running mainline sequence because the observed interaction
failure can block reliable execution and review in large Sessions. This ordering grants no
implementation authority.

## Exact-Main Non-Terminal Inventory — 2026-08-17

Baseline: `main@abf88657b046379cc5216ee211a24568495d2a52`.

| State | Iterations | Disposition |
|---|---|---|
| Active | None | Do not activate I209 in this planning PR. |
| Review | I188 | Retain Review pending its owner-first decision-only closeout; its background-job contract does not authorize or overlap I209 TUI implementation. |
| Planned / Claimed | I189, I195, I196 | Keep unactivated. I189 is protected permission work, I195 is Dashboard-owned, and I196 remains later in the published mainline sequence. |
| Planned / Unclaimed | I197, I198, I199, I200, I201, I205, I206, I207, I208 | Keep unactivated in their published order and require separate claims. |
| Planned / Unclaimed | I209 | Select only the runnable plan; reproduce the loss point and establish an effective claim before activation or implementation. |
| Blocked | None | No current iteration owner is classified Blocked. Blocked backlog parents remain governed by their own owners. |
| Paused | I164 | Preserve the superseded target; do not resume. |

Open PRs #120/#121 remain archival Draft recovery records, #233 remains Dashboard-owned, and this
planning PR #273 is the only open mainline PR touching TUI-051/I209. None grants implementation
authority.

## Exact-Main Reproduction Checkpoint — 2026-08-17

Baseline: `main@e885d368bd6a29f1ab06b878a9afab4bb536944f`.

- `cargo test --locked -p talos-tui --lib entry_point_esc`: 7/7 passed, including the active-turn,
  later-turn, modal-priority and repeated-cancel entry-point cases.
- `cargo test --locked -p talos-cli conversation_loop_cancel_emits_terminal_cancelled_status`: passed;
  the bridge test observed `UserInput::Cancel` reaching its legacy `SessionOp::Interrupt` route and
  a terminal cancelled status.
- `cargo test --locked -p talos-agent --test i169_targeted_interrupt`: passed; exact generation and
  turn identity cancellation remains correct at the actor boundary.
- Source inspection confirms the resumed structured path can reach `StructuredRunning`, but the TUI
  calls `project_history` twice synchronously per frame before re-entering the input/event select.
  The projection has no frame-level cache keyed by transcript/viewport/style/selection inputs.

The incident's most supported loss point is therefore before `UserInput::Cancel` is serviced: a
large unchanged transcript can monopolize the TUI task and starve keyboard polling. The bridge and
actor cancellation boundaries are not shown to lose the message by these tests. A claim-backed
implementation must add a resumed structured-turn reproduction that observes the four boundaries
independently and records CPU/input-latency evidence; this checkpoint grants no implementation
authorization.

## Activation Checkpoint — 2026-08-17

- Effective claim: PR #276 merge `33b11433bd28f70d1e15e80d902e51ca0ee33e1b`.
- Activation PR: #277; this branch has no target-branch effect until merged.
- Non-terminal inventory: I188 remains Review; I189/I195/I196 remain Planned/Claimed and
  unactivated; I197-I201/I205-I208 remain Planned/Unclaimed; I164 remains Paused; no iteration is
  Blocked and no other iteration is Active.
- I209 is the only proposed Active iteration. No Rust, Cargo, dependency, persistence, release or
  product behavior changes are part of this activation record.
- Implementation starts only from the activation merge point or later current main and remains
  bounded by the effective Work Slice and the reproduction checkpoint above.

## Change-Control Checkpoint — 2026-08-17

The maintainer authorized splitting truthful provider retry-progress projection from I209 after
implementation-time source inspection established that retry attempt and backoff facts exist only
inside `talos-provider` tracing. `TurnPhase::Retrying` has no production producer, and the current
`LanguageModel` contract cannot report dispatch or backoff progress before `stream()` returns.

Adding that contract would change a semver-bound public API and require an ADR, while the effective
I209 claim explicitly excludes public API changes. The original Published Baseline remains intact;
its retry-progress acceptance is transferred to PROVIDER-006 / I210 / Issue #278. I209 remains
responsible for the urgent independently runnable subset:

- reuse unchanged large-history projections so redraw does not monopolize the TUI task;
- prove cancellation at `UserInput::Cancel`, generation/turn-bound `SessionOp::InterruptTurn`,
  provider-future drop and durable terminal-cancelled boundaries;
- preserve terminal restoration and directly affected user documentation.

No retry policy, public API, dependency, persistence, Desktop, Dashboard or release change is
authorized by this checkpoint.

## Implementation Review Checkpoint — 2026-08-17

- Implementation commits `7b82fea6` and `7d90def8` were created from the I209 activation merge
  `c7380332` and published in PR #279 for independent exact-head review.
- The projection cache is keyed by transcript revision and terminal width; height, scrolling,
  selection and ordinary redraw reuse shared projected rows and logical lines.
- A real bridge/actor/durable-session test reopens 2,000 persisted messages (approximately 320 KB)
  and observes all four cancellation boundaries plus terminal-cancelled status returning to TUI.
- Focused locked tests passed for `talos-tui`, `talos-provider`, the resumed CLI integration and
  targeted agent interruption; locked CLI/agent/TUI check, formatting and `git diff --check`
  passed.
- I209 is in Review, not Complete. Exact-head CI, real-terminal CPU/input-latency and terminal
  restoration evidence, independent review and merge-time CAS remain required.
