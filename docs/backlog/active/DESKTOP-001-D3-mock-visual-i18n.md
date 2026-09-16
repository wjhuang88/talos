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

## 2026-09-16 Implementation Plan Clarification

This is an in-scope correction, not reduced acceptance or renderer authorization. Follow the
[ADR-059 execution-domain proposal](../../decisions/059-desktop-renderer-host-motion-boundary.md)
and [release capability inventory](../../reference/DESKTOP-I194-DEPENDENCY-SECURITY-MATRIX.md).
The existing dependency decision gate remains open; a stdout fixture renderer does not deliver
the planned Desktop visual surface.

1. Resolve the native dependency and renderer-internal scheduling decision before adding GPUI.
   Verify the latest stable release and its exact dependency graph; preserve CLI default build/run
   scope. Do not assume APIs from Zed main exist in the published crate.
2. Reuse GPUI Application/window/layout, foreground/background executors, Task handles, focus,
   keyboard and IME interfaces. Do not build a custom scheduler, Tokio bridge or live runtime
   integration for this fixture-only slice.
3. Implement the actual mock Execution surface with stable fixture identity, presentation-local
   edits, catalog-backed interface strings and interactive zh-CN/en-US switching. User-authored
   fixture text and evidence remain unchanged; unsupported locales fall back deterministically.
4. Implement editable input against the framework IME contract, explicit focus order and keyboard
   actions. Use GPUI-controlled timers for deterministic presentation tests. Keep host failures
   explicit and free of shared-runtime mutation.
5. Converge locally with bilingual visual snapshots, locale/identity assertions, keyboard-only
   traversal, CJK composition/commit/cancel/caret checks, reduced-motion equivalence and measured
   responsiveness. Record host coverage and unavailable cases honestly; framework API presence
   does not replace product-level acceptance evidence. Only then submit a stable candidate for
   exact-head CI and independent review.

The Tokio bridge, runtime subscription lifecycle and business cancellation adapter belong to the
later authorized runtime-binding slice. They are not new deliverables of I277.
