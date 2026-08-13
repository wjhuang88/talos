# Desktop I194 Dependency And Motion Review Matrix

> Status: Review input for Proposed ADR-059; no renderer or localization dependency is selected.
> Evidence date: 2026-08-13

| Area | Current evidence | Risk / question | Required gate |
|---|---|---|---|
| GPUI direction | Zed/GPUI source snapshot `a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74`; `gpui` declares Apache-2.0, AccessKit, Wayland/X11 features, platform crates and macOS bindgen/cbindgen build dependencies. | Native Objective-C/Windows APIs, build-generated bindings and visible platform `unsafe` require security ownership; source snapshot is not a Talos lockfile. | Review the exact selected release, full locked graph, licenses/SBOM, build scripts, panic boundaries and platform tests before any dependency or crate change. |
| Iced comparison | Iced source snapshot `2b275718d19a5cf306537e1d2417a1f5e9d94ef4`; workspace MIT; winit path exposes IME state/cursor/purpose and X11/Wayland features; default renderer features include wgpu and tiny-skia. | Backend/native closure, accessibility behavior and Talos host integration are not established by repository inspection alone. | Keep as comparison only; perform the same exact-release, lockfile, platform, IME, accessibility and failure review if GPUI is reversed. |
| Host lifecycle | Desktop remains above `talos-runtime`; session truth remains in `talos-session`; ADR-059 assigns event-loop, teardown and failure handling to the host adapter. | Event-loop, shutdown or panic handling could duplicate runtime authority. | Implement only behind an explicit client projection with recoverable renderer failure and cancellation tests. |
| CJK / IME | Iced source exposes IME enable/cursor/purpose conversion; both candidates still require Talos controls tested with CJK/Latin composition. | Composition, caret and mixed CJK/Latin metrics may fail. | Validate editable controls on macOS, Windows and Linux with commit/cancel/reposition cases. |
| Accessibility | GPUI declares AccessKit; declaration is not proof of Talos focus/name/state behavior. | Focus order, keyboard navigation and announcements may be incomplete. | Validate platform accessibility and keyboard-only flows against every mock state. |
| Reduced motion | ADR-059 requires equivalent non-animated presentation. | Disabling motion may remove state comprehension. | Verify state/focus/order/completion parity. |
| Motion budget | No implementation timing evidence yet. | Queued or non-cancellable transitions can make the UI feel slow. | Measure input-to-visible latency, frame-time distribution and cancellation. |
| Supply chain | No Desktop dependency in Cargo files. | Native code, generated sources, downloads and licenses remain unknown. | Review locked graph, build scripts, licenses and provenance. |

## Source snapshot provenance

The snapshots were obtained with shallow Git clones on 2026-08-13. They are audit inputs only and
are not vendored, copied into Talos, or treated as a dependency lock:

| Candidate | Repository | Commit | License/package facts inspected |
|---|---|---|---|
| GPUI | `zed-industries/zed` | `a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74` | `crates/gpui/Cargo.toml`: package `gpui` 0.2.2, Apache-2.0; AccessKit; Wayland/X11; platform and macOS bindgen/cbindgen entries. |
| Iced | `iced-rs/iced` | `2b275718d19a5cf306537e1d2417a1f5e9d94ef4` | workspace `Cargo.toml`: MIT; wgpu/tiny-skia defaults; X11/Wayland features; `winit/src/window.rs` and conversion paths for IME state and cursor/purpose. |

The source review does not prove release stability, transitive license closure, platform coverage,
or runtime panic behavior. Those remain required gates.

## Motion acceptance shape

The later mock-only slice must test immediate input feedback, cancellable state transitions, one
semantic activity indicator without perpetual decoration, CJK/Latin and IME stability, reduced-motion
parity, and representative execution-view transitions under measured host budgets. Numeric budgets
remain deliberately unset until renderer/platform measurements exist.
