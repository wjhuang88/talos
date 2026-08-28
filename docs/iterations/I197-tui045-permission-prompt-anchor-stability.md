# Iteration I197: Permission Prompt Layout Anchor Stability

> Document status: Complete / Closed
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session |
| Work Slice | I197/TUI-045 only: implement the bounded permission-prompt logical anchor/layout correction, focused tests and real-terminal evidence while preserving all permission semantics and request identity. No policy, persistence, broad renderer or release work. |
| Claimed At | 2026-08-19 |
| Source Issue | #125 |
| Governance Claim PR | #304 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #304 merged to `main` as `0db92cf9` from the exact claim base. This effective claim authorizes only the bounded TUI-045 implementation; exact-head CI, independent Agent technical review, merge-time CAS and deferred human/manual rows remain required. Protected permission/security scope is non-deferrable. |
| Implementation PR | #305 |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | PR #305 merged as `d98f37e7` after exact-head CI/review/CAS. I211 terminal validation found docking/hierarchy defects and left part of the matrix incomplete; TUI-059/#330 is the separate Ready/Unclaimed corrective owner. Keep I197 Review with no permission-policy authority transfer. |

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

I197 activation is effective through claim PR #304 merge `0db92cf9`. Implementation PR #305 merged
to `main` as `d98f37e7` from final head `9fce4f13`, which contains implementation commit
`ff4141ca`; the bounded implementation contains no permission-policy or protected-crate changes.

## Verification Evidence

The implementation worktree passed focused TUI tests and the complete release preflight. Exact-head
CI run `32204974418` passed all five jobs, independent Agent technical review `5336592072` approved
final head `9fce4f13`, and merge-time CAS passed before merge `d98f37e7`. Natural-person and terminal
rows remain deferred to Issue #302 / I211.

## Completion Evidence

Completion Commit: `d98f37e7` (implementation merge for PR #305).

## Variance And Residuals

General panel docking, configurable overlays and broad terminal-layout changes require separate
owners and iterations.

## Change Control — 2026-08-14

The maintainer added Issues #69, #79 and #111 to the coordinating long-running task after this
baseline was published. I197's objective, scope and acceptance remain unchanged. Its activation
order now follows I199/#69 and I200/#79 dispositions so permission-prompt anchor work consumes the
reviewed preview-capacity and scroll-normalization boundaries instead of duplicating them.

## Change Control - 2026-08-18 Deferred Human Validation Timing

The maintainer selected the long-task Deferred Human Validation Mode because a natural-person
reviewer is unavailable during implementation. This changes scheduling only. I197 still requires
its exact-head CI, independent Agent technical review, locked checks, governance validation and CAS
before merge. If the diff remains TUI presentation/layout-only, the natural-person and terminal
matrix rows move to VALIDATION-002/I211/Issue #302 and I197 remains Review while I201 proceeds.

Any `talos-permission`, sandbox, process-hardening or permission-policy change is outside this
deferral and must stop for a separately authorized independent security review.

## Retrospective

Pending execution.

## 2026-08-19 Implementation Checkpoint

PR #305 contains the layout-only anchor correction. `cargo test -p talos-tui --locked` passed 535
unit tests, 2 integration tests and 2 doctests; `cargo fmt --all -- --check`, `git diff --check`
and `./scripts/release_preflight.sh` also passed. Completion remains pending exact-head CI,
independent review, merge-time CAS and the deferred human-validation rows.

## 2026-08-19 Implementation Merge Disposition

PR #305 final head `9fce4f13` merged to `main` as `d98f37e7` after exact-head CI `32204974418`
passed all five jobs, independent Agent technical review `5336592072` approved the exact head with
shared-account identity limits disclosed, and merge-time CAS passed. The merged change remains
TUI layout/anchor-only and contains no permission-policy, request-identity, protected-crate,
persistence or release change. I197 remains `Review` with `Completion Commit: Pending`; only the
deferred natural-person and terminal rows in Issue #302 / I211 remain open.

## 2026-08-20 I211 Human Validation Failure Disposition

Natural-person checkpoint `5341637918` on integrated `main@ec794515` found the permission selector
below the composer while running tool activity remained above it, and the required resize,
small-terminal and queued-prompt matrix was not completed. A separate new-session observation found
a non-bottom composer while the permission request rendered at the physical terminal bottom.

At this historical checkpoint I197 and TUI-045 remained Review with `Completion Commit: Pending`;
Corrective Story TUI-059 / Issue #330 subsequently owned composer-relative docking and the complete
terminal matrix. That corrective story is now complete.

## 2026-08-28 Natural-Person Validation Closure

The maintainer completed the executable Issue #125 walkthrough: in a new session with the composer
above the physical terminal bottom, the permission panel remained adjacent to the logical composer;
the narrow/short layout kept all choices visible; and after denial the panel closed while the
composer and history remained usable. The selector owns keyboard focus while open, so editing a
multiline draft inside it is not an executable acceptance path; that criterion is limited to
preservation of pre-existing composer/history state across open and close. I197 is now Complete /
Closed.
