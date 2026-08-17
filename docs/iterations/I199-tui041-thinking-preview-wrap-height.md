# Iteration I199: Thinking Preview Wrap And Bounded Height

> Document status: Planned
> Published plan date: 2026-08-14
> Planned objective: render live thinking/stream previews from one display-width-aware plan so
> multiline content wraps, grows to a bounded cap and shrinks without destabilizing composer,
> required panels, history anchors or terminal cells.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a maintainer can stream multiline ASCII/CJK preview text in a rebuilt terminal
> client and observe deterministic 1–6 row growth, clipped-tail indication and clean shrink/clear.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #69 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | After predecessor disposition, establish an effective I199 claim on `main` and branch only from that claim merge or later current `main`. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| TUI-041 / Issue #69 | Ready | TUI-039 layout continuity; ADR-034; ADR-054 | One runnable generic preview-layout correction with buffer/layout and native-terminal evidence |

### Scope

- Build one display-cell-aware preview plan shared by height measurement and rendering, preserving
  explicit newlines, CJK/wide text, safe UTF-8 boundaries, semantic prefix and indentation.
- Bound live preview growth to six rows, retain newest content after the cap and indicate clipping.
- Make optional preview expansion compressible below composer and required interactive panels.
- Preserve FollowTail/anchored history behavior, stable panel placement and buffer-driven cleanup.
- Record the corrected observable behavior and native-terminal matrix in the owner/Issue evidence.

### Non-Goals

- No thinking persistence, provider/reasoning protocol, title extraction, full/collapsible thinking
  panel, keyboard preview scrolling, configurable height or renderer redesign.
- No TUI-042/#79 scroll-transition fix or TUI-045/#125 permission-panel anchor implementation.

### Acceptance And Planned Validation

- Issue #69's ASCII, CJK, newline/CRLF, cap, resize, constrained-layout, history-anchor, buffer
  cleanup and native-terminal matrix passes.
- Focused planner/layout/buffer tests and relevant `cargo test -p talos-tui --locked` targets pass.
- `cargo test --workspace --locked`, release preflight, both governance validators and
  `git diff --check` pass.
- Independent natural-person exact-head review and maintainer native-terminal evidence are recorded;
  shared-account identity and role separation are disclosed.

### Documentation Target

- TUI-041, I199 and Issue #69 evidence; add a focused terminal-acceptance reference if the matrix
  cannot be recorded compactly. No README feature claim is planned because this restores intended
  transient-preview behavior.

### Risks And Fallback

- Separate measurement/render paths could reintroduce stale cells or neighboring-region corruption.
- Preview allocation could steal rows from security-sensitive panels or oscillate their placement.
- Fallback: preserve the current bounded one-row behavior and leave I199 Review/Partial; do not
  weaken required-panel visibility or ADR-034/054 boundaries to obtain visual acceptance.

## Actual Activation And Execution

No activation has occurred. I199 remains Unclaimed and is ordered before I200 to establish the
preview-driven viewport-capacity behavior that I200 must include in its scroll-bound matrix.

## Verification Evidence

Pending implementation after an effective claim reaches `main`.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

User-configurable, scrollable or persistent preview behavior requires separate owners.

## Retrospective

Pending execution.
