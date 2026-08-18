# Iteration I200: No-Op History Scroll State Stability

> Document status: Review
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
| Authorization Evidence | Claim PR #300 exact head `c70dcfa7` passed CI `32144285868`, independent agent review `5329269096`, merge-time CAS `5329300644` and merged as `356dc3c5`. The shared-identity agent review is not represented as a distinct natural person and does not waive the published implementation review or terminal walkthrough acceptance. |
| Implementation PR | #301 |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | PR #301 merged after exact-head CI, independent Agent technical review and merge-time CAS. PR #303 proposes that, on merge, I200 stays Review while VALIDATION-002/I211/Issue #302 owns the deferred natural-person exact-head review and maintainer mouse/touchpad walkthrough. |

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

Claim PR #300 merged as `356dc3c5321d4cb0b7fec2f9533947dcc3acdfd8`, making I200 the sole
Active/Claimed iteration. The implementation branch started exactly from that merge. Commit
`3afeeb2859a441ef7e1b7628ff4b5b83b974210d` is submitted as PR #301 and moves I200 to
Review without claiming the remaining human acceptance gates.

## Verification Evidence

- `cargo test -p talos-tui --locked`: 533 unit tests, 2 integration tests and 2 doctests passed.
- `cargo clippy -p talos-tui --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed before the implementation commit.
- `./scripts/release_preflight.sh`: passed, including locked workspace check, Clippy, tests,
  doctests, site/installer validation, classifier tests and both governance validators.
- Full-frame regressions cover non-scrollable alternating wheel bursts, exact fit, one-row
  overflow, top/tail idempotency, multiline input/cursor/history preservation, height growth,
  CJK width reflow and I199 preview shrink.
- PR #301 exact head `8a58cb2d` passed CI `32149762367`, received independent Agent technical
  approval `5330234992` with its identity limit disclosed, passed merge-time CAS and merged as
  `9628e183`.
- Pending in VALIDATION-002/I211/Issue #302: maintainer mouse-wheel and touchpad walkthrough plus
  the published independent natural-person exact-head review. Agent or shared-account review
  evidence cannot be represented as a distinct natural person.

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

## 2026-08-18 Implementation Review Checkpoint

Implementation commit `3afeeb2859a441ef7e1b7628ff4b5b83b974210d` centralizes rendering-derived
frame-history scroll outcomes as Noop, Anchored or FollowTail. A target equal to the current start
does not mutate anchors; reaching the real tail from an earlier viewport returns to FollowTail;
and a post-layout projection that fully fits clears an obsolete anchor and replans the same frame
once under the natural inline history cap.

PR #301 is open from the exact claim merge. TUI tests, Clippy and full release preflight pass
locally, but CI, independent technical review, merge-time CAS, independent natural-person
exact-head review and maintainer mouse/touchpad acceptance remain open. Completion Commit remains
Pending, so I200 is Review rather than Complete.

## 2026-08-18 Deferred Human Validation Change Control

PR #301 subsequently passed exact-head CI `32149762367`, independent Agent technical approval
`5330234992` and merge-time CAS, then merged as `9628e183f410ddc0ae22067107d286cefa37d016`.
The approval explicitly does not claim a distinct natural person.

The maintainer changed only validation timing: unavailable natural-person review and real-terminal
mouse/touchpad acceptance move to VALIDATION-002/I211/Issue #302 near long-task closeout. The
published baseline and acceptance remain unchanged, those rows remain unpassed, and Completion
Commit remains Pending. PR #303 proposes this residual disposition; only after it reaches `main`
may the non-overlapping I197 claim be prepared.
