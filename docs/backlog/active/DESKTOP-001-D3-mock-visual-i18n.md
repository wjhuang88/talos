# DESKTOP-001-D3: Mock Desktop Visual and i18n Slice

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D3 |
| Parent | DESKTOP-001 / #29 |
| Status | Review / Claimed |
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
| Authorization Evidence | Claim+activation effective through #570 merge 5f9dcf053ba2b3b31f76693f6bc8c97df36a1df6. |
| Implementation PR | #571 merged as 707b538eea0421333544f22b3e3e4a42687c87a1; six exact-head checks and independent Agent-role reviews passed |
| Last Updated | 2026-09-18 |
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

## 2026-09-16 Accepted Renderer Boundary

The maintainer has explicitly accepted ADR-059's GPUI/native and execution-domain boundary.
This supersedes the decision-pending wording in the earlier clarification, not its acceptance
criteria. Exact dependency/security review remains outstanding. Keep Active / Claimed until real
visual implementation and its evidence justify Review; do not close out the stdout prototype.

## 2026-09-16 Manual Evidence and Release Capability Gap

The I277 owner records a passed macOS interaction walkthrough (bilingual state retention, IME,
focus, selection, clipboard, resize, scrolling and exit), with a reported IMK diagnostic whose
trigger remains unknown. This is not cross-platform or complete ADR acceptance.

Independent inspection found no custom-control accessibility name/role/state API in the selected
crates.io GPUI 0.2.2. The older Zed snapshot's AccessKit declaration is not evidence for this
release. Final acceptance requires a maintainer-approved resolution of that dependency capability
gap, plus the remaining performance/host/CI evidence recorded in I277. No scope reduction,
renderer replacement or completion is implied.

## 2026-09-17 Current Native Acceptance

The updated I277 owner records maintainer-confirmed Chinese multiline IME, cancellation,
scroll/selection, native folder selection/cancel, menu-to-input focus, locale draft retention
and resize behavior, plus close-time software timing distributions. The display-settings bar
must still be replaced with the maintainer-approved anchored dismissible language menu.
VoiceOver and reduced-motion checks were explicitly deferred by the maintainer as non-blocking
for this round; they remain unverified residuals owned here and in I277, not completed acceptance
or release/platform readiness. I277 remains Active / Claimed pending the local follow-up and
remaining delivery evidence.

Follow-up: the approved anchored language menu is now implemented and locally validated in I277,
including dismissal, locale selection, focus restoration and same-page navigation coverage.
The narrow human recheck of the changed interaction remains pending; existing manual input and
folder acceptance is preserved. The deferred accessibility/reduced-motion status is unchanged.

## 2026-09-18 Popover Acceptance and Review

The maintainer passed current-language highlighting, switching/reopening, Escape/outside dismissal,
resize anchoring and task-switch dismissal on the corrected binary recorded in I277. The local
mock surface is Review / Claimed; Completion Commit: Pending. Final local preflight and stable
candidate submission continue in #571, followed by fresh exact-head CI and independent review.
VoiceOver and reduced-motion remain explicitly deferred and unverified; cross-platform native
interaction and physical display latency are not claimed. The real Runtime/client remains out
of this mock-only slice.

## 2026-09-18 Implementation Merged

#571 merged as `707b538eea0421333544f22b3e3e4a42687c87a1` after six exact-head CI checks,
independent Agent-role review and merge-time CAS; the full evidence is in I277. The mock is
delivered on main. Review / Claimed and Completion Commit: Pending remain because VoiceOver
and reduced-motion are maintainer-deferred, unverified rows in #29. Prior candidate-pending
checkpoints are historical. No release or live Runtime binding is claimed.
