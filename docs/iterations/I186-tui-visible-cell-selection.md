# Iteration I186: TUI Visible-Cell Selection And Copy

> Document status: Review
> Published plan date: 2026-08-11
> Planned objective: implement the Accepted ADR-054 application-owned visible-cell selection policy so ordinary mouse drag selects and copies arbitrary rendered TUI text without Shift or unrelated Talos state mutation.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: on the exact implementation head, users can drag-select partial or multi-row visible cells, receive bounded edge autoscroll, retain a clamped selection through resize, and copy through the existing clipboard backend while keyboard history, `/copy`, terminal restoration, privacy and runtime behavior remain intact.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-11 |
| Work Slice | Implement only TUI-046-B / Issue #134 under the Accepted ADR-054 amendment: add bounded application-owned selection over the last rendered visible cells; ordinary primary-button down/drag/up without Shift; selection highlight; history-viewport edge autoscroll during drag; clamped, non-disappearing resize handling; copy completed selection with the existing OSC 52/macOS clipboard backend and truthful status; isolate pointer selection from history/composer/modal/approval/session/execution mutation; preserve keyboard history, existing wheel policy, `/copy`, Alternate Screen and exhaustive terminal restoration; add focused mixed-width/wrapped/panel/state-isolation/lifecycle tests, user docs and exact-head two-terminal acceptance evidence. No transcript/export/persistence/provider/permission/scheduler change, hidden-content access, rich persistent selection, dependency or unrelated TUI-042 work. |
| Claimed At | 2026-08-11 |
| Source Issue | #134 |
| Governance Claim PR | #192 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer explicitly directed that Issue #134 implementation proceed immediately and that real-terminal testing move to post-development acceptance. PR #192 requests exact-head governance review/authorization; this proposed claim has no ownership effect until merged to `main`. |
| Implementation PR | #193 (exact head recorded by PR review/CI evidence) |
| Last Updated | 2026-08-11 |
| Handoff / Release Condition | The exact code-head two-terminal matrix is recorded; obtain exact final PR-head review and green CI for PR #193, then perform merge-time CAS and record the real merge SHA before completion. |

## Closure Ledger

| Dimension | Record |
|---|---|
| Requested outcome | Finish the P0 Issue #134 implementation rather than treating I184 policy work or pre-implementation terminal observations as delivery. |
| Artifacts to create/update | TUI selection model/input/rendering/clipboard integration and focused tests; TUI-046/I186 owner evidence; user-facing TUI interaction documentation; exact-head terminal matrix. |
| Existing assets to preserve | ADR-054 Alternate Screen/full-frame ownership, `TerminalSession` transactional lifecycle, keyboard PageUp/PageDown and Ctrl+Home/End, current wheel semantics, `/copy`, hidden-data boundaries and all non-interactive modes. |
| State/status owners | TUI-046 and I186 first; backlog/iteration indexes, Board, manifest and Issue #134 second. |
| Validation required | Focused selection/unit/lifecycle tests, locked talos-tui/workspace tests, both governance validators, release preflight, exact-head Unix/Windows CI, independent review, Alacritty/macOS plus one materially different terminal matrix, merge-time CAS. |
| Evidence and uncertainty | I184 proves that captured mouse reporting plus no drag consumer blocks the required default and that disabling reporting is insufficient. Exact edge-drag cadence, wide-cell extraction and resize clamping remain implementation hypotheses to prove with deterministic tests and the post-implementation matrix. |
| Residual-work destination | TUI-042/#79 retains no-op wheel-scroll stability; broader clipboard/export, persistent semantic selection, hyperlink and drag-and-drop work need separate owners. |

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Preserve their TUI-037, sequential dependency, security and release gates. |
| I184/TUI-046-A | Closed | Accepted policy dependency satisfied at completion commit `f98488277803ee26180100089a48ef850939234b`. |
| I185/AG-12 | Claimed; implementation PR #191 open | Non-overlapping governance-validator work; may proceed independently and supplies no TUI authority. |
| TUI-042/#79 | Refinement / unclaimed | Related wheel no-op stability remains separate and does not block Issue #134. |
| Recovery PRs #120/#121 | Open archival evidence | Keep immutable, unmerged and non-overlapping. |

No open PR, remote branch or effective claim overlaps TUI-046-B at selection.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-046-B | TUI-046 / Issue #134 | Ready to claim | I184 Complete; ADR-054 I184 amendment Accepted; effective I186 claim | Ordinary application-owned visible-cell selection and clipboard copy with edge autoscroll, resize continuity and input-state isolation. |

### Scope

- Add a bounded selection model over terminal cell coordinates and the last complete rendered frame.
- Route primary-button down/drag/up into selection before any other mouse behavior; ordinary drag
  requires no modifier and cannot invoke composer, modal, approval, session or runtime actions.
- Highlight the selected cells, extract partial/multi-row visible text deterministically, trim only
  unselected row-end fill, preserve deliberate interior spacing/newlines, and keep UTF-8 valid for
  wide/combining/CJK/emoji content.
- While dragging at the history viewport's top or bottom edge, advance application history on frame
  ticks and extend the selection; keep keyboard history navigation and current non-selection wheel
  behavior unchanged.
- On resize, clamp active/completed endpoints to the new frame instead of clearing selection; keep
  the last valid copied payload available until a new selection replaces it.
- On primary-button release, copy the selected visible text through the existing clipboard backend
  and report success/failure without logging or persisting the text.
- Update user-facing TUI interaction docs and complete the exact-head real-terminal matrix after
  implementation, before acceptance.

### Non-Goals

- No transcript/session/export projection, `/copy`, clipboard backend or hidden-data expansion.
- No rich semantic/persistent selection model, selection handles, search, hyperlink, drag/drop or
  native-scrollback redesign.
- No dependency, provider, permission, scheduler, tool, storage or non-interactive behavior change.
- No TUI-042/#79 no-op wheel correction or guarantee that a terminal emulator preserves its own
  native selection across Talos redraws.

### Acceptance

- Ordinary primary-button drag selects partial lines, multiple rows and adjacent visible components
  without Shift; the rendered highlight and clipboard payload match selected visible cells.
- Dragging at a history edge autoscrolls in that direction within existing history bounds; release
  stops autoscroll and copies exactly once.
- Selection gestures do not mutate the composer/cursor, modal choice, approval state, input buffer,
  transcript/session, tail-follow state except the explicit edge-scroll history movement, or runtime
  execution.
- Resize clamps rather than silently clears the selection, active streaming/redraw reapplies the
  highlight safely, and mouse-up after either condition remains bounded and panic-free.
- ASCII, CJK, emoji, combining, wide, wrapped, fill-only and multiline cells produce valid UTF-8
  with deterministic row boundaries and no hidden/non-rendered content.
- Existing wheel behavior outside a drag, keyboard history, `/copy`, initialization/rollback/
  restoration, non-interactive modes and privacy boundaries remain unchanged.
- Exact-head Alacritty/macOS and one materially different terminal row pass the published matrix;
  unit tests alone cannot mark TUI-046-B Complete.

### Planned Validation

- Focused selection model, buffer extraction, mouse routing, edge tick, resize, streaming and modal/
  approval isolation tests under `talos-tui`.
- Existing exhaustive `TerminalSession` initialization/rollback/restoration tests.
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test -p talos-tui --locked`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `./scripts/release_preflight.sh`
- Exact-head CI, independent review, two-terminal matrix and merge-time CAS.

### Documentation To Update

- User-visible TUI interaction documentation for drag, auto-copy, edge behavior and resize.
- `docs/reference/TUI-NATIVE-SELECTION-MATRIX.md` with exact implementation SHA and observations.
- TUI-046/I186 owners, iteration/backlog indexes, Board, manifest and Issue #134 status.

### Risks And Rollback

- Risk: selection mouse events reach existing input routes. Rollback: consume primary down/drag/up
  at the selection boundary and test complete state snapshots around each event.
- Risk: cell extraction duplicates wide-cell continuation or exposes stale/hidden data. Rollback:
  read only the last completed frame buffer, respect cell continuation symbols, bound coordinates,
  and test mixed-width/padding cases.
- Risk: redraw/resize invalidates endpoints. Rollback: clamp coordinates every frame and retain the
  last valid completed payload independently of transient geometry.
- Risk: edge autoscroll runs after release. Rollback: gate tick movement on an explicit dragging
  state and clear it on mouse-up, cancel and restoration.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-11 | Selection | Maintainer priority keeps #134 at P0 and defers real-terminal acceptance until after development. I184 and ADR-054 gates are complete; I185/#191 is non-overlapping; no TUI-046-B claim, implementation PR or remote branch exists. Draft claim remains ineffective until target-branch merge. |
| 2026-08-11 | Claim merge | I186 claim PR #192 merged at `f4faa38e4815302db2ccf1f4888b2862e56493b`; implementation PR #193 contains existing commits `dabd31e2`, `6473d9f6`, and `cf2e06a3` for visible-frame selection, cross-screen history extension, resize clamping, clipboard copy, combining-cell preservation and focused tests. |
| 2026-08-11 | Implementation refinement | Code commit `39639c37` adds stable logical selection coordinates across resize/reflow, splash-row mapping, final mouse-up coordinates, no-history-area selection, and preservation of selected trailing spaces; later PR #193 changes only synchronize documentation status. `cargo test -p talos-tui --locked` passes 501 tests plus 2 integration tests and 2 doctests; terminal matrix remains pending. |
| 2026-08-11 | Manual-defect refinement | Exact-head terminal testing exposed stale screen-coordinate highlights after history movement, missing edge autoscroll/Logo traversal, wrapped-row over-highlighting and a bottom-component block. Commits `52c36ee1`, `86dce04b` and `f62a78a7` replace stale projection with logical-row reprojection, clamp wrapped-row ranges and retain history focus during edge drag. The attempted cross-component refinement `bfb5ae61` worsened the interaction and was explicitly rolled back by `fb2c4abe` at maintainer direction; the restored behavior passed the affected drag and offscreen-copy paths. |
| 2026-08-11 | Clipboard compatibility | Terminal.app 2.15 on `fb2c4abe` rendered and retained selection but left the previous system clipboard value in place. Code inspection proved OSC 52 write success was incorrectly treated as terminal acceptance, making the existing `pbcopy` fallback unreachable when Terminal.app silently ignored the sequence. Commit `70b51e28` makes the existing macOS backend `pbcopy`-first with OSC 52 fallback, preserves OSC 52-first behavior elsewhere and adds backend-order tests without adding a dependency or expanding clipboard data scope. |
| 2026-08-11 | Exact-code-head terminal acceptance | Code head `70b51e2835510c1a23f5bc5cd1521f41acc6805e` passed the published matrix on Alacritty 0.17.0 (`TERM=alacritty`) and macOS Terminal 2.15 (`TERM=xterm-256color`), both on macOS 26.5.2 (25F84), `TMUX=none`. Ordinary no-Shift drag, partial/wrapped/mixed-width copy, bidirectional edge autoscroll, Logo/offscreen traversal, wheel and PageUp/PageDown projection, resize, active streaming, normal exit and interrupted-output cleanup passed; exact observations live in the reference matrix. |

## Verification Evidence

- PR #193 code through `70b51e28`: 511 talos-tui tests plus 2 integration tests and 2 doctests pass; stable selection/reflow, Unicode buffer/selection, resize isolation, cross-screen history, bidirectional edge-tick, redraw preservation and macOS clipboard-backend ordering tests pass. `cargo fmt --all -- --check`, `cargo clippy -p talos-tui --locked --all-targets -- -D warnings`, `cargo build -p talos-cli --locked` and `git diff --check` pass. After the evidence update, `./scripts/release_preflight.sh` passes the site/installer checks, locked workspace check, workspace Clippy, full workspace tests/doctests and both governance validators with 0 warnings. The exact code head passed both terminal rows; exact final PR-head CI, independent review and merge-time CAS remain pending.

## Completion Evidence

- No completion evidence while Review. Completion requires an already-existing implementation merge SHA plus exact-head terminal evidence.

## Variance And Residuals

- `bfb5ae61` is superseded by the explicit UX rollback `fb2c4abe`; it is retained in history as
  truthful failed-refinement evidence and supplies no acceptance claim.
- TUI-042/#79 and all broader terminal interaction work remain separately owned.

## Retrospective

- Pending execution.
