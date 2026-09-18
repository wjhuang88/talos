# ADR-059: Desktop Renderer, Host, And Motion Quality Boundary

> Status: Accepted
> Accepted: 2026-09-16, explicit maintainer approval in the I277 conversation
> Date: 2026-08-13 (evidence refresh)
> Owner: DESKTOP-001-D0 / I194

## Context

Talos Desktop is a consumer-facing product and must feel responsive, calm, and visually
intentional. The existing design baseline calls for restrained native motion, but the renderer and
host boundary has not yet established testable motion, input, accessibility, or failure constraints.
GPUI was initially a design input only. The accepted decision below now selects the bounded
Desktop renderer; dependency/security verification remains mandatory.

Desktop also remains a host/client above `talos-runtime`. Durable session/transcript truth stays in
`talos-session` under ADR-042, and `talos-runtime` remains the supported composition facade under
ADR-052. Motion must therefore present state; it must not create a second work authority or imply
that a visual transition is a runtime event.

## Decision

### Renderer and host boundary

The future Desktop host remains above the supported runtime facade. Renderer and platform-host code
must stay outside `talos-core` and `talos-runtime`; it may consume approved runtime/session
projections through an explicit host/client boundary. TUI and Desktop remain independent renderers.
GPUI `0.2.2` is authorized only as a dependency of the isolated `talos-desktop` crate, subject to
the exact locked-graph, native/build-script, license and platform gates below. No renderer,
localization, native, build-script, FFI, or `unsafe` dependency is authorized in shared runtime
crates.

The current evidence refresh selects GPUI `0.2.2` for the isolated mock Desktop boundary. The
direction is supported by the GPUI source snapshot listed in the companion matrix:
it exposes an AccessKit dependency, explicit Wayland/X11 features, platform crates for macOS,
Windows and Linux, and IME/text handling in the host paths. The same snapshot also contains native
Objective-C/Windows API integration, macOS `bindgen`/`cbindgen` build steps and visible `unsafe`
platform code. Those facts require a separate dependency/security approval before any Cargo change.
Iced is retained as the minimum comparison candidate: its source snapshot exposes winit-backed IME
state and X11/Wayland feature paths, but its wgpu/tiny-skia/rendering and winit dependency closure
also remains unreviewed for Talos. No other renderer is authorized by this ADR.

The host responsibilities below are binding requirements for a later implementation decision:

| Host area | Required boundary before implementation |
|---|---|
| macOS | Renderer owns only window/event presentation; Objective-C/CoreText/Metal or equivalent native calls are isolated in the renderer adapter, audited for `unsafe`, and cannot own runtime/session state. |
| Windows | Win32/D3D or equivalent handles, message dispatch, device-loss and shutdown paths stay in the host adapter; failures become an explicit UI error and do not terminate or fabricate domain completion. |
| Linux | Wayland/X11 selection, clipboard, text input and compositor shutdown are host concerns; unsupported display backends fail with a recoverable diagnostic. |
| All hosts | A single host lifecycle owns event-loop startup, cancellation, renderer teardown and process exit. Runtime/session work remains behind an explicit client projection and is never reconstructed in the renderer. |
| Input and IME | Editable controls must preserve composition text, caret rectangle, commit/cancel ordering and mixed CJK/Latin layout; IME state is presentation state and never durable Mission/session state. |
| Accessibility | Every interactive state has keyboard focus/order and an accessibility name/role/state path; motion and colour are supplementary, never the only status signal. |
| Reduced motion | The same state, focus, ordering and completion semantics are reached without non-essential interpolation or looping effects. |

Localization selection is likewise a mechanism criterion, not a dependency authorization. A later
choice must support stable message keys, named interpolation, plural/count formatting, locale-aware
date/number formatting, deterministic `en-US` fallback, missing-key diagnostics, and runtime
`zh-CN`/`en-US` switching without allowing locale into canonical domain identity. User-authored
Mission text, commands, paths, raw evidence and artifacts remain untranslated facts.

### Motion quality policy

Motion is a product-quality constraint, not a decorative layer:

1. Input response and state clarity take priority over animation. A user action must remain
   interruptible and must not wait for a visual transition to complete.
2. Motion must be semantic: use it for focus, insertion/removal, progress, status change, or
   spatial continuity. Do not use perpetual decorative loops, glow, bounce, or attention-stealing
   effects in the primary execution view.
3. Transitions must be short, bounded, cancellable, and replaceable by an immediate final state.
   A newer state supersedes an older transition; queued visual work must not accumulate.
4. Motion must preserve the existing information hierarchy: current Goal and Work state remain the
   visual center, and activity indicators must not compete with errors, approvals, or user input.
5. Reduced-motion mode is an equivalent presentation path, not a hidden disable switch. It removes
   non-essential interpolation and looping motion while preserving state, focus, ordering, and
   completion feedback through immediate changes, opacity/colour cues, or static markers.
6. Motion must respect accessibility and localization: no meaning may depend only on movement,
   timing assumptions, or language-specific geometry; CJK/Latin layout and IME composition cannot be
   animated in a way that obscures text or caret state.

### Performance evidence gate

The first mock-only visual/i18n slice must record, on each supported host class, input-to-visible
feedback latency, frame-time distribution during representative state transitions, cancellation
behavior, and reduced-motion equivalence. The acceptance target is stable interaction under the
renderer-selected host budget, with no long-task or unbounded-animation regression; exact numeric
budgets are to be selected from measured renderer/platform evidence rather than guessed in D0.

## Explicit exclusions

- The D0 decision itself contains no implementation. I277 separately owns the authorized mock UI.
- No real Mission, Work Graph, Evaluation, Approval, Artifact, Delivery, reconnect, or durable
  Desktop state.
- No change to ADR-042, ADR-052, SESSION-009, I188, I189, SESSION-008, RUNTIME-005, or
  ARCH-034-R04.
- Renderer selection is not evidence of shipped Desktop behavior.

## Security and failure inputs

The later implementation decision must trace material native/FFI/build/`unsafe` ownership, catch
panic-prone integration boundaries, define shutdown and process-lifetime containment, and preserve
permission/session/runtime authority. Renderer failure must degrade to a safe error state; it must
not silently terminate the host or fabricate a completed work state.

## Reversal triggers

Reopen this proposal if primary-source evidence shows that the candidate renderer cannot provide
acceptable CJK/IME/accessibility behavior, reduced-motion parity, bounded input latency, stable
frame timing, or safe native/panic containment on a supported host. Reopen it if motion materially
obscures state hierarchy, creates unbounded visual work, or requires domain state duplication.

## Next gate

This accepted ADR governs the mock-only visual/i18n child, subject to independent exact-head
review, current renderer/dependency evidence, the security matrix, and merge-time CAS. That child
requires its own owner, iteration, claim, and worktree. Real Mission/runtime/work/evaluation binding
remains gated by the DESKTOP-001 P0-P4 chain.

## Evidence status

The local repository and governance boundaries are confirmed. Current primary-source snapshots were
retrieved on 2026-08-13 and are recorded by immutable commit in the companion matrix. They establish
candidate capability and risk facts, not Talos compatibility. Renderer selection is accepted;
locked dependency closure, license/SBOM review, platform test evidence, panic containment and motion
benchmarks remain open. Acceptance does not claim that these verification gates have passed.

## Execution-Domain Boundary

This accepted boundary uses the
[release capability inventory](../reference/DESKTOP-I194-DEPENDENCY-SECURITY-MATRIX.md).
The maintainer explicitly accepted this boundary after reviewing the proposal. The exception
is limited to Desktop framework internals, not Talos-authored business async.

### Reuse Framework Scheduling, Preserve Business Authority

Do not replace GPUI's scheduler merely to remove non-Tokio dependency names. Its foreground
execution domain exists to preserve UI-thread affinity; background scheduling and task primitives
are presentation infrastructure, not a replacement for Talos Runtime.

| Domain | Owner | Boundary |
|---|---|---|
| Window events, layout, drawing, focus and IME | GPUI foreground / native event loop | App/Entity/Window stay UI-local; no synchronous wait for business work. |
| Presentation-only background work | GPUI background executor | No Agent, Session, permission, durable-work or provider authority. |
| Agent, Session, provider/tool I/O and business deadlines | Existing Talos Tokio runtime | Retain existing admission, cancellation, persistence and shutdown contracts. |
| Future Desktop/runtime adapter | Thin adapter to the existing runtime owner | Transfer bounded messages/results; never create a runtime per request or duplicate business authority. |

The last two rows describe the later integration boundary, not I277 implementation scope.
I277 remains fixture-backed and needs no Tokio bridge, business runtime or live network client.

### Lifecycle And Failure Rules

- Keep UI event processing nonblocking. Tokio-bound futures must run with their required Tokio
  context; GPUI polling alone does not establish its reactor or timer driver.
- UI-owned presentation tasks have retained handles and explicit cancellation lifetimes. Do not
  detach work as a workaround for ownership errors.
- Dropping a view may cancel its local observation or presentation task; it must not implicitly
  cancel durable Talos work. Business cancellation remains an explicit runtime operation.
- Any later bridge must distinguish observer cancellation, task abort request and confirmed
  business shutdown. Blocking work and external effects are not undone by dropping a handle.
- Catch recoverable Rust panics and report host errors at integration boundaries. Do not claim
  that `catch_unwind` contains native faults, aborts or device failures; review these separately.

### Accepted Exception And Remaining Gates

The accepted decision is a bounded renderer exception, not global relaxation: permit the
reviewed framework-native integration and framework-internal scheduling/primitives only within the
Desktop dependency boundary. Talos-authored business async remains Tokio-owned; application code
must not introduce direct smol/async-std usage merely because GPUI depends on them.

Dependency introduction still requires exact dependency/native/build-script review. The exception
permits GPUI-internal native bindings, async-task, smol APIs and Linux async-std adapters only;
it does not authorize new Talos-authored unsafe code. Latest stable GPUI must be rechecked at
introduction; older constrained
transitive versions require documented upstream reasons, not silent overrides. Preserve default
CLI Cargo run/build behavior. Do not add `gpui_tokio`, fork GPUI, or copy Zed-main APIs without
an actual authorized requirement and compatible-version evidence.

## 2026-09-16 Fixed Upstream Evaluation Authorized

After the exact crates.io 0.2.2 accessibility gap was reported, the maintainer accepted assessment
of a fixed upstream revision rather than silently deferring accessibility. This authorizes source
and isolated dependency evaluation; it does not claim migration or platform validation complete.
The candidate is Zed revision `a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74` (2026-08-13),
already identified by the original research. Its package version still reads 0.2.2, but that
does not make it equivalent to the crates.io artifact. Pin the full revision, never a moving branch.

Verified via GitHub contents API at that revision: GPUI exposes stable semantic element IDs,
`role`, `aria_label`, value/state properties and `on_a11y_action`; its example demonstrates these
APIs. Platform manifests declare AccessKit adapters for macOS, Windows and Linux. These source
facts justify evaluation, not a claim that Talos screen-reader behavior has passed.

Migration must use `gpui_platform::application()` and move `runtime_shaders` to that platform
dependency; focus calls now take the app context. GPUI and its platform facade must use the same
revision. The mock's language buttons and editable draft need real semantic roles, localized
names, current values/states and accessible actions, with stable identity across locale changes.
Use the framework's input-latency/frame-duration histogram facilities where suitable, after
checking what their measurements actually include; do not equate CPU timing with visible latency.

The upstream toolchain file specifies Rust 1.97.1; Talos remains pinned to 1.97.0 until compatibility
is tested or a separate toolchain change is justified. Resolve the full isolated graph before
changing Talos Cargo files. Inspect reachable Git dependencies and native/build scripts; upstream
workspace patch declarations do not automatically apply to a downstream consumer. No wholesale
Zed workspace patch import, Talos fork, new unsafe shim, Tokio bridge or scope expansion is allowed.
Preserve the existing local mock and manual evidence; repeat affected checks after migration.

## 2026-09-17 Current Artifact Clarification

The I277 working candidate now pins both `gpui` and `gpui_platform` to
`ac5af8b9e1ea3f7922fbabefe409c05b8766135c`, the upstream license-clarification revision
recorded in the I277 execution owner. The earlier a8fafdd7 candidate and registry 0.2.2
observations are historical evaluations, not the current artifact or current platform evidence.

The current Cargo.lock contains neither `async-std`, `gpui_http_client` nor `zed-async-tar`.
Locked/offline reverse-tree queries for async-std on both the host and all targets report no
matching package. In particular, the old registry macOS async-std chain in the dependency matrix
must not be read as a current dependency or used to expand the accepted exception. No direct
Talos smol/async-std use, additional runtime, native binding or unsafe code is authorized here.

This records the implemented artifact and evidence boundary, not a replacement architecture
decision or final dependency approval. The current integrated production/test graph review,
native accessibility/IME/performance acceptance and platform validation remain I277 gates.
