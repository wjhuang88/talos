# TUI-046: Native Text Selection And Copy

| Field | Value |
|---|---|
| Story ID | TUI-046 |
| Type | Bug / TUI / Terminal Interaction Story |
| Priority | P1 |
| Status | Refinement — causal baseline confirmed; decision and implementation children unselected |
| Source | [GitHub Issue #134](https://github.com/wjhuang88/talos/issues/134) |
| Selected Iteration | None |
| Depends On | ADR-054 alternate-screen renderer; existing `/copy` command |
| Coordinates With | TUI-042 / Issue #79 mouse-history scrolling |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #134 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-03 |
| Handoff / Release Condition | Refine the selection/mouse-capture contract, select an iteration and establish a claim before implementation. |

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

## Executable Split

| ID | Deliverable | Status | Depends On |
|---|---|---|---|
| TUI-046-A | Native-selection versus mouse-capture contract and ADR-054 amendment | Ready, not selected | Current lifecycle/input inventory; coordinate with TUI-042 without absorbing it |
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
