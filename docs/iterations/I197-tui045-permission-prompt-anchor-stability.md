# Iteration I197: Permission Prompt Layout Anchor Stability

> Document status: Planned
> Published plan date: 2026-08-14
> Planned objective: keep the inline permission prompt adjacent to the current interaction while
> preserving the logical viewport/composer anchor and prior tail-follow state across prompt open,
> resolution, queued prompts and resize.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a maintainer can run the focused layout tests and a real-terminal matrix and
> observe one permission prompt open and close without an unconditional jump to the terminal bottom,
> progressive drift or any permission-policy change.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #125 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | After predecessors are dispositioned, establish an effective claim on `main`; create the implementation branch only from that claim merge or later current `main`. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| TUI-045 / Issue #125 | Ready | Current inline scrollback/composer ownership, permission panel state and ADR-054 renderer | One runnable terminal correction preserving layout anchors without changing authorization behavior |

### Scope

- Snapshot a logical pre-prompt viewport/composer anchor and prior tail-follow state when the first
  permission request opens.
- Place the transient permission panel adjacent to the current composer with only the minimum
  deterministic local reflow required to show every choice.
- Reuse the anchor across queued prompts and restore it after Allow, Deny, cancel, timeout or error.
- Recompute from logical state after resize; cover wrapped descriptions, multiline input and
  narrow/short terminals.
- Update user-facing TUI interaction documentation affected by the corrected behavior.

### Non-Goals

- No permission policy, default decision, request identity, timeout, tool execution, persistence or
  non-interactive approval change.
- No global composer-bottom policy, panel framework, broad renderer rewrite or unrelated TUI fix.
- No activation of I188, I189 or another Planned iteration.

### Acceptance

- With sufficient space, opening a permission prompt preserves the visible triggering context and
  logical composer position.
- With insufficient space, the viewport moves only enough to reveal every required choice.
- Resolution and repeated queued prompts restore the prior logical relationship and tail-follow
  state without blank-row growth, cursor artifacts or progressive drift.
- Resize and narrow/short terminal cases remain usable with wrapped descriptions and multiline input.
- Focused state/layout tests, relevant `talos-tui` tests and the Issue #125 real-terminal matrix pass.

### Planned Validation

- Focused permission-panel and viewport/layout state tests.
- Relevant `cargo test -p talos-tui --locked` targets.
- `cargo test --workspace --locked` and repository release preflight required by `AGENTS.md`.
- Both governance validators and `git diff --check`.
- Independent natural-person exact-head review plus maintainer real-terminal evidence; shared-account
  identity and role separation must be disclosed.

### Risks And Fallback

- Risk: restoring absolute coordinates after resize can corrupt the viewport or cursor.
- Risk: conflating focus transfer with viewport reset can hide the security choice or prior context.
- Fallback: retain the current permission behavior and keep I197 Review/Partial; do not weaken
  visibility, authorization or fail-closed behavior to satisfy layout acceptance.

## Actual Activation And Execution

No activation has occurred. This planned iteration remains Unclaimed and follows I196/I188
disposition in the mainline priority long task; that order is coordination, not implementation
authority.

## Verification Evidence

Pending implementation after an effective claim reaches `main`.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

General panel docking, configurable overlays and broad terminal-layout changes require separate
owners and iterations.

## Change Control — 2026-08-14

The maintainer added Issues #69, #79 and #111 to the coordinating long-running task after this
baseline was published. I197's objective, scope and acceptance remain unchanged. Its activation
order now follows I199/#69 and I200/#79 dispositions so permission-prompt anchor work consumes the
reviewed preview-capacity and scroll-normalization boundaries instead of duplicating them.

## Retrospective

Pending execution.
