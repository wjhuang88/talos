# DESKTOP-001-D3: Mock Desktop Visual and i18n Slice

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D3 |
| Parent | DESKTOP-001 / #29 |
| Status | Active / Claimed |
| Selected Iteration | I277 |
| Work Slice | Mock-only Desktop visual surface with bilingual localization fixtures; no production runtime binding. |
| Depends On | DESKTOP-001-D0 / I194; WORK-001 P0-P4; I276 browser boundary |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | Mock-only Desktop visual surface and bilingual localization fixtures; no production runtime binding. |
| Claimed At | 2026-09-16 |
| Source Issue | #29 |
| Governance Claim PR | #570 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Proposed atomic claim+activation in #570; ineffective until merged to main. |
| Implementation PR | Not started |
| Last Updated | 2026-09-16 |
| Handoff / Release Condition | Requires D0 boundary confirmation, GPUI dependency decision, and overlap check with shared crates. |

## Scope

- Establish a runnable mock-only Desktop window/surface and `zh-CN`/`en-US` localization fixtures.
- Validate layout, keyboard/focus, Chinese IME and deterministic locale fallback.
- Keep state presentation-local and fixture-backed.

## Non-Goals

No real Mission, Work Graph, Evaluation, permissions, persistence, session authority, browser
automation, Dashboard changes, release packaging, or shared runtime/Cargo dependency movement.

## Acceptance

- Given mock execution fixtures, when the surface is rendered in either locale, then every visible
  string comes from a stable catalog and fallback is deterministic.
- Given locale switching, then fixture identity and evidence remain unchanged.
- Given unsupported host/display conditions, then the mock surface reports an explicit unavailable
  state without mutating shared runtime state.

## Planned Validation

Visual/manual host walkthrough, bilingual snapshot tests, IME/focus checks, dependency inventory,
governance validators, exact-head CI and independent review.
