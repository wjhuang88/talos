# Iteration I200: No-Op History Scroll State Stability

> Document status: Active
> Published plan date: 2026-08-14
> Planned objective: make mouse/touchpad history input mutate anchor state only when the visible
> frame-history start changes, and normalize anchors that become impossible after resize/reflow.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: in a rebuilt terminal client, repeated wheel events over fully visible history
> leave FollowTail state, composer position, cursor and rendered rows unchanged, while real overflow
> still anchors and returns to tail correctly.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I200/TUI-042 only: correct no-op and real-movement frame-history scroll transitions, normalize impossible anchors after current resize/reflow/projection metrics are known, and validate the published focused/full-frame/native-terminal matrix. Excludes kinetic/pixel scrolling, wheel-step changes, hit testing, renderer redesign, transcript/session mutation, TUI-045, TUI-043, provider, persistence and release work. |
| Claimed At | 2026-08-18 |
| Source Issue | #79 |
| Governance Claim PR | #300 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance PR #300 is ineffective until its finalized exact head passes CI, both validators, independent agent technical review with shared-identity limits disclosed, merge-time CAS and merge to `main`. The maintainer's unattended authorization permits this non-security claim path because no independent natural-person reviewer is available; it does not waive the published natural-person implementation review or terminal walkthrough acceptance. |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | After #300 merges, create the implementation branch from that merge or later current `main`; implementation may reach Review unattended, but Complete still requires the published independent natural-person exact-head review and maintainer mouse/touchpad walkthrough. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| TUI-042 / Issue #79 | Ready | TUI-039 completed layout contract; ADR-054; I199 disposition for overlap control | One runnable state-transition correction with full-frame and real-terminal evidence |

### Scope

- Compute scroll bounds from the same splash, projection and viewport metrics used by rendering.
- Centralize Noop/Anchored/FollowTail transitions and reject target-equals-current mutations.
- Normalize obsolete anchors when resize, reflow, content or preview-height changes make all rows fit.
- Cover short/exact-fit/overflow histories, boundary bursts, CJK wrapping, panels, multiline composer,
  buffer identity and native mouse/touchpad behavior.
- Record corrected wheel behavior in owner/Issue acceptance evidence.

### Non-Goals

- No kinetic/pixel scrolling, hit testing, wheel-step change, horizontal scroll, renderer change or
  transcript/session mutation.
- No I199 preview layout or I197 permission-panel implementation inside this slice.

### Acceptance And Planned Validation

- Issue #79's complete short/exact-fit/overflow, resize/reflow, preview-capacity, panel, composer,
  input-state, buffer and native-terminal matrix passes.
- Focused transition/full-frame tests and relevant `cargo test -p talos-tui --locked` targets pass.
- `cargo test --workspace --locked`, release preflight, both governance validators and
  `git diff --check` pass.
- Independent natural-person exact-head review and maintainer mouse/touchpad walkthrough are
  recorded with shared-account identity/role disclosure where applicable.

### Documentation Target

- TUI-042, I200 and Issue #79 evidence; add a focused terminal-acceptance reference if needed. No
  README feature claim is planned because this restores the completed TUI-039 interaction contract.

### Risks And Fallback

- Bounds reconstructed differently from rendering could create stale or oscillating anchors.
- Incorrect normalization could discard a genuine PageUp/keyboard anchor.
- Fallback: preserve current navigation semantics and leave I200 Review/Partial; do not force
  FollowTail without proving the viewport is fully visible.

## Actual Activation And Execution

No activation has occurred. I200 remains Unclaimed. I199 is ordered first to settle preview-driven
capacity, but a recorded I199 blocked disposition permits I200 to proceed with current behavior and
retain the preview interaction case as explicit residual evidence.

## Verification Evidence

Pending implementation after an effective claim reaches `main`.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

Kinetic scrolling, gesture accumulation and mouse hit testing require separate owners.

## Retrospective

Pending execution.

## 2026-08-18 Claim Preparation Checkpoint

Claim preparation starts from exact `main@4acb896e5a76253c50aa2075517edd8b0e53a7f9` after
I199/TUI-041 reached Complete/Closed and Issue #69 closed. No Active or Review iteration exists.
I189 remains Planned/Claimed and unactivated; I197, I198, I200, I201, I206, I207, I208 and I210
remain Planned/Unclaimed; I164 remains Paused. T3/#59 remains Blocked in the long-task ledger.
Archival Draft PRs #120/#121 do not overlap this slice, and no open claim or implementation PR owns
TUI-042/I200.

The proposed claim is limited to the published no-op history-scroll state transition and obsolete
anchor normalization scope. This governance branch contains no Rust/Cargo or implementation test
change. The target-branch Unclaimed record remains effective until the finalized Claimed record in
PR #300 reaches `main`; no implementation branch may be created before that merge.

## 2026-08-18 Finalized Claim Proposal

PR #300 now contains the actual Claimed record and proposes I200 as the sole Active iteration. The
claim remains ineffective before merge. Single-maintainer claim authorization is limited to this
non-security governance transition; an independent agent technical audit must still bind the exact
head, and the published natural-person implementation review plus maintainer terminal walkthrough
remain mandatory before Complete.
