# TUI-046: Native Text Selection And Copy

| Field | Value |
|---|---|
| Story ID | TUI-046 |
| Type | Bug / TUI / Terminal Interaction Story |
| Priority | P0 |
| Status | Active — TUI-046-A decision execution started from effective claim merge `66d0f932370f679d491cb78f64dff9d84878479d` |
| Source | [GitHub Issue #134](https://github.com/wjhuang88/talos/issues/134) |
| Selected Iteration | I184 (Active) |
| Depends On | ADR-054 alternate-screen renderer; existing `/copy` command |
| Coordinates With | TUI-042 / Issue #79 mouse-history scrolling |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 architecture session 2026-08-10 |
| Work Slice | Implement only I184/TUI-046-A: establish the native-selection versus mouse-capture contract, validate the causal interaction on the selected terminal matrix, and amend or replace ADR-054 with the explicit TUI-046-B gate; no Rust implementation or TUI-046-B authority. |
| Claimed At | 2026-08-10 |
| Source Issue | #134 |
| Governance Claim PR | #186 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent review `5236470750` approved exact claim head `00fc49376715fc1fc4e3bfe9e82465aea676b3bf` with no blockers and disclosed that a distinct natural-person reviewer used the shared `@wjhuang88` account. Exact-head CI `31358815361` passed all four jobs; merge-time CAS passed against `main@a403fdbae61372db4f830f2bf0c9adf2173a85ba`; PR #186 merged at `66d0f932370f679d491cb78f64dff9d84878479d`. |
| Implementation PR | #187 (decision/docs only) |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | Execute only TUI-046-A from claim merge `66d0f932370f679d491cb78f64dff9d84878479d` or later `main`; TUI-046-B remains blocked until the decision is Accepted. |

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
  Terminal.app and repeated resize clears the selection. The published cross-platform environment
  remains pending and TUI-046-B remains Blocked.

## Executable Split

| ID | Deliverable | Status | Depends On |
|---|---|---|---|
| TUI-046-A | Native-selection versus mouse-capture contract and ADR-054 amendment | Active in I184 | Effective claim `66d0f932`; coordinate with TUI-042 without absorbing it |
| TUI-046-B | Selected interaction implementation, restoration tests, docs and real-terminal matrix | Blocked | TUI-046-A decision Accepted |

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
