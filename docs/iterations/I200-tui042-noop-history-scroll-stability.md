# Iteration I200: No-Op History Scroll State Stability

> Document status: Planned
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
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #79 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | After I199 disposition, establish an effective I200 claim on `main` and branch only from that claim merge or later current `main`. |

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
