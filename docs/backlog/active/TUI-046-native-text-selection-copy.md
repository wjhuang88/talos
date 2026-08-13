# TUI-046: Native Text Selection And Copy

| Field | Value |
|---|---|
| Story ID | TUI-046 |
| Type | Bug / TUI / Terminal Interaction Story |
| Priority | P0 |
| Status | Complete — TUI-046-A policy and TUI-046-B implementation delivered |
| Source | [GitHub Issue #134](https://github.com/wjhuang88/talos/issues/134) |
| Selected Iteration | I186 (Complete) |
| Depends On | ADR-054 alternate-screen renderer; existing `/copy` command |
| Coordinates With | TUI-042 / Issue #79 mouse-history scrolling |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-11 |
| Work Slice | Implement only I186/TUI-046-B under the Accepted ADR-054 amendment: bounded application-owned selection over last-rendered visible cells; ordinary primary-button drag without Shift; highlight, history-edge autoscroll, resize clamping and copy via the existing clipboard backend; strict input-state/privacy isolation; focused mixed-width/render/lifecycle tests, docs and exact-head two-terminal acceptance. Preserve Alternate Screen, keyboard history, current non-selection wheel policy, `/copy`, restoration and all non-TUI runtime behavior. Exclude TUI-042, hidden content, transcript/export/persistence, rich persistent selection, dependencies and unrelated product changes. |
| Claimed At | 2026-08-11 |
| Source Issue | #134 |
| Governance Claim PR | #192 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | TUI-046-A completed at `f98488277803ee26180100089a48ef850939234b`. TUI-046-B claim merge `f4faa38e4815302db2ccf1f4888b2862e56493b`; PR #193 final head `313e47e562125eaa8d18679295dfd0f95ef1e9e5` passed CI `31481069023`, independent review `4905391760`, merge-time CAS and merged as `a5115f5ce6484512ceb83867f72fa9b47ab8f5fc`. |
| Implementation PR | #193 |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | None - Issue #134 delivery is complete; TUI-042/#79 and broader terminal interaction work remain separate. |

Status: Complete

Completion Commits: `f98488277803ee26180100089a48ef850939234b`, `a5115f5ce6484512ceb83867f72fa9b47ab8f5fc`

### TUI-046-B Execution Evidence

- Effective claim merge: `f4faa38e4815302db2ccf1f4888b2862e56493b` (I186 claim PR #192).
- Implementation PR: #193; exact tested code head `70b51e2835510c1a23f5bc5cd1521f41acc6805e` includes the user-directed rollback of the worsened `bfb5ae61` interaction and the Terminal.app clipboard compatibility fix. Later evidence-only documentation commits do not replace that runtime evidence.
- Focused validation at the tested code head: `cargo test -p talos-tui --locked` (511 tests, 2 integration tests, and 2 doctests passed), `cargo clippy -p talos-tui --locked --all-targets -- -D warnings`, `cargo build -p talos-cli --locked`, and `git diff --check`. The evidence update also passes `./scripts/release_preflight.sh`, including locked workspace check, workspace Clippy, full workspace tests/doctests and both governance validators with 0 warnings.
- The exact-code-head Alacritty 0.17.0 and macOS Terminal 2.15 rows pass ordinary no-Shift drag, mixed-width/wrapped copy, logical wheel/keyboard projection, bidirectional edge autoscroll, resize/streaming continuity and normal/interrupted cleanup. Final PR head `313e47e5` passed CI `31481069023`, independent review `4905391760` and merge-time CAS before merge `a5115f5c`.

### TUI-046-A Completion Evidence

- Completion Commit: `f98488277803ee26180100089a48ef850939234b`
- This existing squash merge contains the accepted I184 decision evidence. The parent TUI-046
  Story remains Active because TUI-046-B implementation and acceptance are still outstanding.

## Identity / Goal / Value

Restore a documented, predictable way to select and copy arbitrary visible text in the interactive TUI without mutating Talos history, composer, modal, approval, session or runtime state.

## Scope

- Decide the product-level relationship between terminal-native selection and application-owned mouse-wheel history navigation.
- Preserve arbitrary partial-line and multi-row copy for visible transcript, tool, code, diagnostic, status and panel text.
- Preserve UTF-8 and terminal-cell behavior for ASCII, CJK, emoji, combining and wide characters.
- Keep mouse-reporting setup and restoration explicit in `TerminalSession`.
- Record real-terminal evidence for the selected default interaction and any documented override gesture.
- Amend ADR-054 or record a replacement decision if its current mouse-capture premise changes.

## Exclusions

- No I169 steering, provider, permission, scheduler, persistence or session behavior changes.
- No exposure of hidden, redacted or non-rendered content.
- No replacement of `/copy`, broad transcript/export redesign, rich editor selection model or abandonment of Alternate Screen without a separate accepted decision.

## Decision Links And Constraints

- ADR-054 remains the current renderer and terminal-lifecycle authority until explicitly amended.
- Terminal selection must not trigger Talos input actions or alter logical history/tail-follow state.
- Keyboard history navigation and terminal restoration contracts remain available.
- Native-selection causality must be proven across the selected terminal matrix; Alternate Screen alone is not assumed to be the root cause.
- Exact evidence rows live in `docs/reference/TUI-NATIVE-SELECTION-MATRIX.md`; Pending is not a pass.

## Uncertainty And Validation Path

Refine whether mouse capture is disabled by default, made explicit/configurable, combined with a cross-terminal gesture, or replaced by a bounded application-owned selection path. Validate the chosen contract on the maintainer's primary terminal and at least one materially different platform terminal before implementation acceptance.

## Current Implementation Baseline (2026-08-09)

- `TerminalSession` unconditionally enables mouse capture after entering
  Alternate Screen and tracks/restores it transactionally.
- The application consumes only mouse-wheel events; drag/down/up events do not
  implement an application-owned selection model.
- ADR-054 requires captured wheel events for application-owned history but does
  not define a native-selection gesture or opt-out. `/copy last` and `/copy all`
  are whole-message/transcript commands and do not satisfy arbitrary visible
  range selection.
- Therefore the verified causal gap is the missing product contract between
  default mouse capture and terminal-native drag selection. Alternate Screen by
  itself is not recorded as the cause.
- Latest baseline observation on `33cc8dab23a38c387063d1265c230dfa0f8922d9` (Alacritty 0.17.0 on
  macOS 26.5.2, no multiplexer): ordinary drag requires Shift, wheel scrolling does not carry the
  selection with projected content, edge-drag has no autoscroll, and resize clears selection.
  Native-only is therefore rejected as the complete default; TUI-046-B should implement bounded
  application-owned visible-cell selection.
- A second terminal observation on `c0fba2e92cace29fde4e2fc33fd26640058eddca`
  (Terminal.app 2.15 on the same macOS host) found that ordinary and Shift+drag both fail while
  reporting is enabled. Disabling reporting restores native selection but transfers scrolling to
  Terminal.app and repeated resize clears the selection. Cross-platform validation remains
  mandatory after implementation, but no longer blocks development start once A is Accepted and a
  B claim is effective.

## Executable Split

| ID | Deliverable | Status | Depends On |
|---|---|---|---|
| TUI-046-A | Native-selection versus mouse-capture contract and ADR-054 amendment | Complete in I184 | Completion Commit `f98488277803ee26180100089a48ef850939234b`; review `5237824299`; CI `31370219799` |
| TUI-046-B | Selected interaction implementation, restoration tests, docs and real-terminal matrix | Complete in I186 | Completion Commit `a5115f5ce6484512ceb83867f72fa9b47ab8f5fc`; final PR head `313e47e5`; CI `31481069023`; independent review `4905391760`; terminal matrix bound to code head `70b51e28` |

TUI-046-B must preserve keyboard history navigation and may not claim acceptance
from unit tests alone. The parent closes only after both terminal environments
prove the documented default gesture against the exact implementation head.
TUI-042/#79 retains ownership of no-op wheel-scroll layout shifts and is not a
completion prerequisite for this independent native-selection bug.

## State / Status Owners

- Story scope and acceptance: this file.
- Remote discussion: Issue #134.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## Acceptance For Future Implementation

- Arbitrary visible ranges can be selected and copied through the documented default interaction.
- Selection does not mutate history, composer, modal, approval, session or execution state.
- Mouse-wheel and keyboard history behavior match the documented policy.
- Mixed-width text, wrapped lines, streaming, resize and transient panels remain safe.
- Terminal modes restore correctly on normal exit and failure paths.
- Hidden/private data remains unavailable.
- Exact-head tests and a recorded real-terminal matrix pass.

## Residual Destination

Broader terminal interaction redesign, persistent application-owned selections, hyperlinking, drag-and-drop or transcript/export expansion require separate owners.

## Completion Evidence

- TUI-046-A Completion Commit `f98488277803ee26180100089a48ef850939234b` accepted the
  application-owned selection policy in ADR-054.
- TUI-046-B Completion Commit `a5115f5ce6484512ceb83867f72fa9b47ab8f5fc` delivers the runtime
  implementation. Final head `313e47e562125eaa8d18679295dfd0f95ef1e9e5` passed CI
  `31481069023` and independent review `4905391760`; the Alacritty and Terminal.app acceptance rows
  passed on exact runtime code head `70b51e2835510c1a23f5bc5cd1521f41acc6805e`.
