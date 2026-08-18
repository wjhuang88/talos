# Iteration I199: Thinking Preview Wrap And Bounded Height

> Document status: Review
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | TUI-041 / I199 only: bounded transient preview layout correction with shared display-width planning, six-row cap, constrained compression, anchor preservation and cleanup. No TUI-042/TUI-045, persistence, provider protocol or release work. |
| Claimed At | 2026-08-18 |
| Source Issue | #69 |
| Governance Claim PR | #295 |
| Authorization Mode | Independent review |
| Authorization Evidence | Activation PR #295 exact head `883d5cc1`, CI `32114936912`, independent approval `5325926209`, and claim merge `8127fa57`. |
| Implementation PR | #297 |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Close only after PR #297 exact-head CI, maintainer native-terminal acceptance, independent review, merge-time CAS and owner-first closeout. |

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

PR #295 merged as `8127fa57` after exact-head CI `32114936912` and independent approval
`5325926209`, making the bounded claim effective. The implementation branch started from later
current `main@df2e6ed6`; commit `938c9edb9b3336e81a3b90232a69e0993574bc69` is submitted as PR
#297. I199 remains ordered before I200 so the latter can include preview-driven viewport capacity
in its scroll-bound matrix.

## Verification Evidence

- `cargo test -p talos-tui --locked`: 536 unit tests, 2 integration tests and 2 doctests passed.
- `cargo clippy -p talos-tui --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed before the implementation commit.
- Isolated local PTY fixture at 80x24 and 40x15 exercised multiline ASCII/CJK wrapping, 1-to-6
  row growth, clipping marker and newest-tail retention, completion shrink/clear and usable
  composer/status regions. The fixture used no real provider credentials and was cleaned up.
- Pending: PR #297 exact-head CI completion, maintainer native-terminal acceptance and independent
  exact-head review. Agent-run PTY evidence does not substitute for the maintainer gate.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

User-configurable, scrollable or persistent preview behavior requires separate owners.

## Retrospective

The shared plan removed measurement/render divergence while keeping the correction bounded to the
existing transient preview. Completion remains pending the external terminal and review gates.

## 2026-08-18 Maintainer Terminal-Acceptance Correction

Maintainer terminal inspection of PR #297 rejected the thinking presentation at head `f00c6e4d`:
the semantic `thinking` label and its content shared the first row, while clipping consumed a
standalone marker row that appeared as a persistent separator before the rolling tail. This is an
in-scope correction of the published semantic-prefix, bounded-tail and clipping requirements, not
authorization for a full, persistent or keyboard-scrollable thinking panel.

The corrected layout reserves row zero for the animated `thinking` title and uses the remaining
preview capacity for automatically rolling newest content. When older content is hidden, the
ellipsis is embedded in the first visible content row instead of consuming a separate row. Generic
stream previews retain their semantic first row and newest tail under the same embedded-marker
rule. PR #297 must obtain new exact-head CI, independent review and maintainer terminal acceptance;
the earlier `f00c6e4d` evidence does not carry forward after this rework.

The first rework build exposed one additional presentation inconsistency during maintainer
inspection: embedding the clipping marker caused the entire first visible content row to inherit
the marker's dim color. The follow-up moves `…` into the semantic prefix slot and confines dim
styling to that marker; content across the rolling boundary keeps one aligned content column and
the same preview foreground color.

The next terminal run showed a separate visibility defect in the inherited model-title extractor:
token usage continued growing while the preview remained frozen on the latest standalone Markdown
heading, then the complete reasoning appeared at once in history. The fixed Talos-owned `thinking`
title now replaces that extractor in the live preview, and the bounded body always follows the raw
displayable thinking tail. This changes no reasoning archive, provider or persistence boundary.

## 2026-08-18 Maintainer-Approved Height Variance

After native-terminal inspection, the maintainer increased the bounded preview maximum from the
published six rows to ten rows so the rolling thinking body retains more immediately useful
context. This is an explicit, size-S variance within the same bounded-preview objective. Composer,
required-panel and constrained-terminal compression priorities remain unchanged; the preview still
shrinks below ten rows when those higher-priority controls need space.
