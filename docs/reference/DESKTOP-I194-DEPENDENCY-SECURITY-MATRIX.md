# Desktop I194 Dependency And Motion Review Matrix

> Status: Review input for Proposed ADR-059; no renderer or localization dependency is selected.
> Evidence date: 2026-08-13

| Area | Current evidence | Risk / question | Required gate |
|---|---|---|---|
| GPUI direction | Design-only direction; no Cargo dependency. | Native/build/FFI/unsafe closure is unverified. | Review current upstream source and locked dependency closure before authorization. |
| Alternative renderer | None selected. | Stale ecosystem claims could produce a false choice. | Compare only current primary sources for credible alternatives. |
| Host lifecycle | Desktop remains above `talos-runtime`; session truth remains in `talos-session`. | Event loop, shutdown or panic handling could duplicate runtime authority. | Define host/client arrows and safe failure containment. |
| CJK / IME | I18N baseline requires CJK and Chinese IME. | Composition, caret and mixed CJK/Latin metrics may fail. | Validate editable controls on macOS, Windows and Linux. |
| Accessibility | Design baseline requires accessible localized controls. | Focus order, keyboard navigation and announcements may be incomplete. | Validate platform accessibility and keyboard-only flows. |
| Reduced motion | ADR-059 requires equivalent non-animated presentation. | Disabling motion may remove state comprehension. | Verify state/focus/order/completion parity. |
| Motion budget | No implementation timing evidence yet. | Queued or non-cancellable transitions can make the UI feel slow. | Measure input-to-visible latency, frame-time distribution and cancellation. |
| Supply chain | No Desktop dependency in Cargo files. | Native code, generated sources, downloads and licenses remain unknown. | Review locked graph, build scripts, licenses and provenance. |

## Motion acceptance shape

The later mock-only slice must test immediate input feedback, cancellable state transitions, one
semantic activity indicator without perpetual decoration, CJK/Latin and IME stability, reduced-motion
parity, and representative execution-view transitions under measured host budgets. Numeric budgets
remain deliberately unset until renderer/platform measurements exist.
