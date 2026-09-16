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

Crates.io metadata queried on 2026-08-13 confirmed `gpui 0.2.2` (Apache-2.0) and `iced 0.14.0`
(MIT, Rust 1.88) with the default feature families summarized above. A disposable external
`cargo metadata` probe did not resolve the complete graph because the registry proxy failed while
fetching transitive index data. No partial graph is treated as an SBOM or authorization input, and
no probe file or dependency entered the Talos repository.

## Motion acceptance shape

The later mock-only slice must test immediate input feedback, cancellable state transitions, one
semantic activity indicator without perpetual decoration, CJK/Latin and IME stability, reduced-motion
parity, and representative execution-view transitions under measured host budgets. Numeric budgets
remain deliberately unset until renderer/platform measurements exist.

## 2026-09-16 I277 Registry And Delivery Recheck

This checkpoint supplements the historical evidence; it does not accept ADR-059 or authorize
native dependencies. I277 remains incomplete. Do not merge PR #571 as a completed visual slice.

- Source: `https://index.crates.io/gp/ui/gpui`, queried on 2026-09-16. The highest published
  non-prerelease, non-yanked version returned was `0.2.2`, published 2025-10-22. This is a fresh
  registry observation, not an assumption from the August snapshot. The HTTP API returned 403;
  the official sparse index was accessible.
- No `rust_version` is declared. Compatibility with repository-pinned Rust 1.97.0 still requires
  compiling the selected full graph.
- The manifest declares macOS build dependencies `bindgen` and `cbindgen`, and native Cocoa/CoreText
  bindings. An explicit Desktop-only native integration decision is required.
- Linux/FreeBSD `ashpd` enables `async-std`; GPUI defaults include Wayland and X11. The Tokio-only
  rule requires an explicit dependency-policy decision or a verified dependency alternative.
  Disabling Linux backends would not establish Linux support.
- Metadata does not prove the locked transitive graph, licenses, panic containment or host support.

### Required Decision And Evidence

The proposed direction remains GPUI isolated to `talos-desktop`, without shared runtime authority.
Resolve and review the exact graph, native/build-script/license ownership, then obtain explicit
acceptance of bounded native integration and any transitive executor exception before implementing.
Do not silently change the Tokio rule or existing default CLI build/run behavior. Real runtime,
permissions, persistence and release packaging remain outside I277.

PR #571 at `b93efeaceed04003092a1d3bba0bb52e2d2bebb4` has passing CI but only prints fixture
text. Independent Agent-role review found missing window/layout, interactive locale switching,
keyboard/focus, Chinese IME, host-unavailable handling and visual evidence. Two unit tests do not
prove those requirements. Preserve the original I277 acceptance; stdout is not its substitute.

### Isolated Default-Feature Resolution

An external, unpublished probe with only `gpui = "=0.2.2"` resolved successfully using Rust
1.97.0. `cargo generate-lockfile` selected 709 dependencies; locked offline `cargo metadata`
reported 710 packages including the probe. This is an all-target resolution, not a per-host
compiled dependency count. The lockfile SHA-256 is
`ecf3e5386a11597feaac86f4f3b3345b8e2aa2f696757dfaadbf4f72d391ca65`.
The disposable probe is `/private/tmp/talos-i277-dependency-audit.GF9TdS`; this path is a local
working artifact, not durable release evidence. No repository Cargo file was changed and no
dependency build script was executed.

The source manifest confirms unconditional `gpui -> smol` (resolved `2.0.2`), in addition to
Linux portal `async-std` (`1.13.2`). Source usage shows that GPUI uses `smol` APIs for timers,
channels and platform operations, while its executor schedules work through its own platform
dispatcher. A dependency name alone is therefore insufficient evidence that a second full
application runtime is started. Nevertheless, these APIs and the GPUI scheduler remain renderer
implementation details that require boundary review. Talos application/runtime async work must
remain Tokio-owned.

The intended Desktop model is explicitly layered:

1. GPUI foreground/background scheduling owns UI-thread affinity, rendering and presentation-local
   work.
2. Talos Runtime, Session and provider/tool I/O remain on Tokio and retain their existing
   cancellation and shutdown semantics.
3. A narrow bridge is used only when a Tokio-bound library must be called from a GPUI task; the
   bridge owns cancellation and must not duplicate Runtime or Session authority.

This is a renderer boundary decision, not a request to make GPUI's internal scheduler Tokio-based.
It also does not authorize importing GPUI's internal `smol` APIs into Talos production code.

The graph includes 110 packages with custom-build targets. Metadata declares license expressions
for all registry packages, but this is not source/license compliance approval. Native integration
includes Clang, FreeType, Fontconfig and ring; `links` fields alone do not enumerate all native
code. Host-specific reachability, build-script actions and failure containment still need review.

Latest direct GPUI also constrains older transitive versions: Cargo reported cocoa 0.26.0 versus
available 0.26.1, cocoa-foundation 0.2.0 versus 0.2.1, core-foundation 0.10.0 versus 0.10.1,
generic-array 0.14.7 versus 0.14.9, and taffy 0.9.0 versus 0.9.2. These observations do not
authorize patches, dependency forks or silent overrides of upstream version constraints.

## 2026-09-16 Capability Reuse And Evidence Boundary

The maintainer supplied a survey of Zed `main`, including `gpui_tokio` and the extracted scheduler.
Those moving-branch references are architecture input, not release compatibility evidence. The
following capability inventory was checked in the downloaded `gpui 0.2.2` registry source; its
crate checksum is `979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707`.

| Capability | Published GPUI evidence | Talos responsibility |
|---|---|---|
| UI-thread scheduling | GPUI's executor module: `ForegroundExecutor::spawn` accepts non-Send futures and dispatches to the main thread. | Keep App/Entity/Window access on the UI thread; do not implement another UI executor. |
| Background presentation work | `BackgroundExecutor::spawn` accepts Send futures; async-task runnables use the platform dispatcher. | Use for presentation work only, not Agent/Session execution. |
| Task ownership | `Task` documents drop cancellation and explicit `detach`. | Retain handles for the intended lifetime; cancellation is not proof of external side-effect rollback or durable task termination. |
| Deterministic scheduling tests | Executor timer and test-support methods include `advance_clock` and `run_until_parked`. | Use scheduler-controlled time for UI tests; do not substitute independent smol/Tokio timers or real sleeps for those assertions. |
| Window and focus | GPUI's application module: Application, window lifecycle APIs, focus handles and key binding registration. | Build the product layout, focus order, shortcuts and unavailable-host behavior. |
| IME integration | GPUI's input module and input example: `EntityInputHandler`, marked-text replacement, unmarking and range bounds. | Implement editable control state and caret geometry; validate composition/commit/cancel with real IMEs. An example is not a ready-made production input widget. |
| Tokio bridge | Zed survey references a separate `gpui_tokio` component, not a confirmed GPUI 0.2.2 API. | Defer until real runtime binding is authorized; verify exact-version compatibility before reuse. |
| Dedicated executor / extracted scheduler / priority APIs | Reported against Zed main, not established by this release audit. | Do not assume these APIs exist in the selected release or add substitutes speculatively. |

The survey's central distinction is adopted as a proposal: async primitives, GUI execution domains
and I/O runtimes are different concepts. Multiple executor names do not establish multiple full
application runtimes; conversely, polling a Future on GPUI does not provide a Tokio reactor for
Tokio-bound resources. `Task::drop` and a bridge abort request do not prove that blocking work,
subprocesses or external effects have stopped.

Reference inputs (moving branches, not pinned release evidence):

- https://github.com/zed-industries/zed/blob/main/crates/gpui/src/executor.rs
- https://github.com/zed-industries/zed/blob/main/crates/gpui/src/platform_scheduler.rs
- https://github.com/zed-industries/zed/blob/main/crates/gpui_tokio/src/gpui_tokio.rs
- https://zed.dev/blog/zed-decoded-async-rust

Stable release source entry: https://docs.rs/crate/gpui/0.2.2/source/
