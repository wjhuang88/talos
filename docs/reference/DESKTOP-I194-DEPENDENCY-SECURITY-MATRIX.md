# Desktop I194 Dependency And Motion Review Matrix

> Status: Dependency review input for accepted ADR-059; GPUI selected, implementation verification outstanding.

## Fixed Upstream Accessibility Evaluation (2026-09-16)

### License Review Checkpoint: Introduction Not Approved

Independent Agent-role review of revision `a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74`
returned REQUEST CHANGES. The macOS normal dependency tree contains unconditional paths
`gpui -> ztracing -> zlog / ztracing_macro` and `gpui -> sum_tree -> ztracing`.
All three tracing/logging packages declare GPL-3.0-or-later. `gpui_util` and
`gpui_shared_string` lack license/license-file metadata; the upstream
README describes GPL as the default with Apache components where marked. GPUI's own Apache
declaration is not evidence that its entire closure has that license. This is a dependency gate
finding, not a legal opinion. No supported feature disables these dependencies.

The first AccessKit revision `1d029c5ff5654fb1b1e8caf4462993c8ee13a133` also has the
`sum_tree -> ztracing` path. No reviewed replacement revision is currently approved. Upstream
clarification or another fully reviewed artifact is required before introducing this closure;
forking or changing licensing is not authorized by the existing renderer decision.

Direct dependency resolution and isolated macOS checks passed on Rust 1.97.0 without copying
upstream workspace patches. This confirms compile compatibility only. The macOS filtered metadata
contains 415 packages; full-platform offline metadata was incomplete due to unavailable cached
packages. Cross-platform license/build-script closure and runtime accessibility remain unverified.
The isolated test build was stopped after the license finding, not passed. No main-workspace
dependency switch or publication occurred.

### 2026-09-16 Upstream Relicensing Found And Independently Verified

Upstream PR #63573 merged as `ac5af8b9e1ea3f7922fbabefe409c05b8766135c` explicitly
relicenses zlog, ztracing and ztracing_macro under Apache-2.0 and adds the missing Apache-2.0
metadata to gpui_util and gpui_shared_string. Independent Agent-role API/tree inspection verified
all five manifest declarations, Apache license symlinks and removal of the three GPL symlinks.
Correction to the earlier inspection: the old gpui_util/shared_string directories already had
Apache symlinks; an rg file listing missed them. Their old gap was metadata, not absence of license
files. Old ztracing/macro also had both license symlinks alongside GPL manifest declarations.
The new upstream commit resolves these specific conflicts; no Talos-authored relicensing occurs.

The new exact revision is approved for isolated dependency evaluation, not final integration.
Its probe resolves 694 packages without imported workspace patches. Latest non-yanked crates.io
GPUI remains 0.2.2 according to the registry index; it does not supply this upstream capability.
The new tarball SHA-256 is
`a96d9e96ea9c55434c8f02fec95796b6fe701403c732e1217a28871e52b0fb60`.
Full graph/native/build review and Rust 1.97.0 compile compatibility are being rechecked rather than
inheriting the old revision's results. No main-workspace dependency switch has occurred.

Maintainer-authorized candidate: Zed `a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74`, not a
moving branch and not the crates.io artifact with the same 0.2.2 version string. Independent
Agent-role source review confirmed macOS SubclassingAdapter/tree events, Windows HWND adapter
and WM_GETOBJECT forwarding, and Wayland/X11 Unix adapters with activation/action/focus callbacks.
This establishes an implementation path, not native assistive-technology acceptance.

GPUI exposes role/name/value/state properties and accessible action callbacks. Only Click,
Focus and Blur have built-in window dispatch; editable value/selection actions need Talos
handlers. TextRun children and text-position mapping remain product work via the public
synthetic-children builder. Wayland does not expose global window position, so its bounds update
is a documented upstream no-op; check actual supported behavior rather than infer X11 parity.

The platform facade becomes a separate same-revision dependency. Upstream uses Rust 1.97.1;
compatibility with Talos's pinned 1.97.0 remains untested. Reachable transitive graph, build scripts,
Git pins and licenses still need review; do not import the upstream workspace's patches wholesale.
The isolated Git resolver made no completed graph after over thirteen minutes and was terminated;
a fixed-revision API source archive is being fetched for inspection. No new Talos dependency or
publication boundary is authorized by these source-review results alone.
> Evidence date: 2026-08-13

| Area | Current evidence | Risk / question | Required gate |
|---|---|---|---|
| GPUI direction | Zed/GPUI source snapshot `a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74`; `gpui` declares Apache-2.0, AccessKit, Wayland/X11 features, platform crates and macOS bindgen/cbindgen build dependencies. | Native Objective-C/Windows APIs, build-generated bindings and visible platform `unsafe` require security ownership; source snapshot is not a Talos lockfile. | Review the exact selected release, full locked graph, licenses/SBOM, build scripts, panic boundaries and platform tests before any dependency or crate change. |
| Iced comparison | Iced source snapshot `2b275718d19a5cf306537e1d2417a1f5e9d94ef4`; workspace MIT; winit path exposes IME state/cursor/purpose and X11/Wayland features; default renderer features include wgpu and tiny-skia. | Backend/native closure, accessibility behavior and Talos host integration are not established by repository inspection alone. | Keep as comparison only; perform the same exact-release, lockfile, platform, IME, accessibility and failure review if GPUI is reversed. |
| Host lifecycle | Desktop remains above `talos-runtime`; session truth remains in `talos-session`; ADR-059 assigns event-loop, teardown and failure handling to the host adapter. | Event-loop, shutdown or panic handling could duplicate runtime authority. | Implement only behind an explicit client projection with recoverable renderer failure and cancellation tests. |
| CJK / IME | Iced source exposes IME enable/cursor/purpose conversion; both candidates still require Talos controls tested with CJK/Latin composition. | Composition, caret and mixed CJK/Latin metrics may fail. | Validate editable controls on macOS, Windows and Linux with commit/cancel/reposition cases. |
| Accessibility | The inspected Zed source snapshot declares AccessKit; the selected crates.io GPUI 0.2.2 source and Talos lockfile contain no AccessKit integration. These are different dependency artifacts. | Keyboard focus works in the local mock, but no native name/role/state path has been established in the selected release. | Resolve the exact-release capability gap before claiming ADR-059 accessibility acceptance; keyboard walkthrough is insufficient. |
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

## Maintainer Acceptance Checkpoint

The maintainer subsequently explicitly accepted the GPUI renderer and execution-domain decision
and authorized updating ADR-059 and continuing I277. This supersedes the proposal-only wording
in the dated research above, not its missing verification evidence. Dependency/native review and
real visual acceptance remain outstanding. No real Runtime binding or bridge is authorized.

Independent Agent-role inspection of GPUI 0.2.2's build script found no network download or shell
command interpolation. macOS requires libclang and xcrun Metal/metallib; Windows release builds
require fxc, selected by GPUI_FXC_PATH, PATH or SDK fallback; Linux validates WGSL with naga and
has separate native font/display dependencies. These subprocesses lack internal timeouts, so
build execution needs an external bounded deadline. This finding does not approve the remaining
transitive build scripts. Top-level catch_unwind cannot contain ABI aborts or native faults.

The local host has xcrun's Metal compiler. A new isolated resolver-3 probe with package MSRV 1.95
and gpui = 0.2.2 defaults resolved 709 packages offline. It is being reviewed before integration;
no dependency build scripts have run. Presentation-model tests are independently runnable without
GPUI; they are not substitutes for actual keyboard, IME or visual tests.

### Independent Dependency Introduction Review

Independent Agent-role review of the isolated locked graph permits local GPUI 0.2.2 compilation
and window implementation, not final platform/security acceptance. It scanned all 110 custom-build
entrypoints for commands, environment, file and network operations and further inspected native,
generator and subprocess paths. No blocking malicious download or required arbitrary output was
identified; this is bounded source review, not a formal audit of every dependency line.

Required controls: unset RXCB_EXPORT and RING_PREGENERATE_ASM, keep xim-parser/bootstrap disabled,
use trusted compiler/SDK paths, exclude publication credentials and bound builds externally.
Current rav1e/av-scenechange features do not enable asm; registry ring uses pregenerated sources.
libfuzzer-sys is not reachable in the reviewed desktop target trees. Do not misreport these optional
paths as required build tools.

Before distribution, preserve FreeType FTL acknowledgement, rav1e PATENTS and ring license texts;
wrapper metadata alone is insufficient. This work does not authorize publication. The repository
integration may unify features differently from the probe and must be compared before final review.

The GUI is explicitly enabled by talos-desktop's desktop-ui feature; its binary requires that
feature and the package is publish=false. Existing CLI targets and runtime dependencies are not
changed. Local window integration and bilingual presentation tests are in progress, not complete.

### First Compilation Findings

The first locked/offline macOS check failed because xcrun's Metal entrypoint exists but the
separate Metal Toolchain is not installed. The mock now selects GPUI's reviewed runtime_shaders
path rather than downloading that toolchain: shader compilation moves to renderer startup, so
startup latency and errors still require validation. This does not establish release packaging.

The integrated macOS graph also reveals `gpui -> gpui_http_client -> zed-async-tar -> async-std`.
Earlier Linux-only descriptions of async-std reachability were incomplete. This is framework-internal
code, not Talos business execution; its coverage under the accepted bounded exception must be
explicitly checked in the final review. Default Desktop features still introduce no GPUI dependency.

## 2026-09-17 Current Integrated Artifact Inventory

The registry and earlier isolated-probe observations above are historical evidence. The current
I277 Cargo candidate pins both GPUI dependencies and all 23 resolved Zed packages to
`ac5af8b9e1ea3f7922fbabefe409c05b8766135c`; those Zed package manifests declare Apache-2.0.
The source/manifest license repairs at this revision, rather than locally relabeled older source,
underpin the local introduction evidence recorded in I277.

The command `cargo metadata --locked --offline --features talos-desktop/visual-test --format-version 1`
was resolved and its dependency IDs traversed from the talos-desktop node. This unfiltered
all-platform/all-dependency-kind metadata closure contains 713 packages, of which 97 declare a
custom-build target; none has an empty license declaration. These are inventory counts, not the
number of build scripts executed on macOS, a production-only closure or license compliance
approval. The full workspace metadata counts (1135 packages / 178 custom-build packages) must
not be reported as Desktop counts. Earlier probe counts use different graph scopes.

The visual-test closure includes proptest and proptest-macro at fixed Git revision
`3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b`, declaring MIT OR Apache-2.0. Final integrated
review must distinguish normal/build edges from dependency dev edges and account for test-support
features; neither the registry 110-script review nor the isolated 98-script review is sufficient
by itself. The independent current-graph review is still in progress.

The current lock and host/all-target reverse-tree queries contain no async-std, gpui_http_client
or zed-async-tar. The macOS async-std chain in First Compilation Findings belongs to the old
registry evaluation. Do not expand the ADR exception based on that superseded dependency fact.
Independent checks confirm default talos-desktop has no GPUI dependencies, and the CLI normal
tree has no GPUI/AccessKit/Desktop/smol/async-std entries. These isolation results do not establish
Windows/Linux native acceptance, final security approval or distribution readiness.

### Current-Graph Independent Local Introduction Review

The subsequent independent Agent-role review permits this fixed graph for I277 local mock
development and acceptance. It independently reproduced the 713/97 metadata inventory and
scanned the 97 custom-build entrypoints for process, network and file operations, expanding the
native/generator paths below. No blocking malicious download, moving Git source or shared-business
dependency pollution was identified. This is bounded review, not a formal audit of every line or
an exact-head merge/release approval.

The enabled all-target normal/build tree was separately enumerated with
`cargo tree -p talos-desktop --features visual-test --locked --offline --target all --edges normal,build --no-dedupe --prefix none --format '{p}'`.
It has 632 unique package lines; async-std, zed-scap, screencapturekit-sys and libfuzzer-sys are not
enabled. Merely filtering dev edges from metadata does not reproduce this feature-selected tree.

Fixed Git review confirmed proptest/proptest-macro have no build scripts and are introduced by
test-support; zed-font-kit only configures Fontconfig dlopen; xim-parser/bootstrap is not enabled;
wasm_thread has no build script and is target-conditional. GPUI Apple runtime_shaders generates
OUT_DIR shader/header material rather than invoking the offline Metal compiler. media uses trusted
xcrun/SDK and bindgen; Windows uses a resource compiler and release fxc. These subprocesses lack
internal deadlines: retain external build timeouts. RXCB_EXPORT and RING_PREGENERATE_ASM must
remain unset; RXCB_RUSTFMT, compiler and SDK selection are trusted build-environment inputs.
Current rav1e/av-scenechange do not enable asm. Native compilation/probing and runtime shader
failure/latency remain platform acceptance surfaces.

This review supersedes the preceding in-progress current-graph review statement and does not
reuse the old registry approval. Remaining gates include real Windows/Linux evidence, native
IME/AT/performance/failure acceptance, final stable-head CI/review and distribution-specific
advisory/notices work before any separately authorized release. No production permission or
runtime authority is added.

### 2026-09-17 Optional Profiler Delta Review

The visual-test feature now also enables GPUI's profiler, adding hdrhistogram 7.6.0. Direct
registry-index inspection found it to be the latest non-yanked version in the compatible major.
Independent Agent-role review verified MIT/Apache-2.0 licensing, MSRV 1.88, no build script and
forbid(unsafe_code). GPUI disables its default features: normal dependencies are byteorder and
num-traits; rug is development-only. No bench-support or additional tracing is enabled. The
ordinary desktop-ui graph excludes hdrhistogram; a reverse tree with visual-test confirms its
sole introduction through GPUI. Previous graph counts describe the pre-profiler snapshot.

Histograms allocate and snapshots clone data, so collection occurs only at interactive close.
The callback contains Rust panics and never blocks closure on metric failure. Sampling covers
inputs that invalidate a frame, coalesces events and excludes mid-draw inputs from statistics;
excluded samples do not mean lost application input. The measured endpoint is platform draw
return, not GPU completion or physical display. This approval is limited to local diagnostic
use, not native accessibility, performance acceptance, distribution or exact-head merge approval.
