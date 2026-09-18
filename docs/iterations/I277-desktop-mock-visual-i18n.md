# Iteration I277: Desktop Mock Visual and Localization Slice

> Document status: Review
> Published plan date: 2026-09-16
> Planned objective: Deliver a fixture-backed, mock-only Desktop visual surface with bilingual localization.
> Baseline rule: preserve this scope; changed objectives use a new iteration ID.
> MVP deliverable: a runnable mock Desktop surface that renders the same fixture in `zh-CN` and `en-US`.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | DESKTOP-001-D3 mock-only visual/i18n slice |
| Claimed At | 2026-09-16 |
| Source Issue | #29 |
| Governance Claim PR | #570 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim+activation effective through #570 merge 5f9dcf053ba2b3b31f76693f6bc8c97df36a1df6. |
| Implementation PR | #571 merged as 707b538eea0421333544f22b3e3e4a42687c87a1; settings-page follow-up #573 merged as 8caf2b3a7b1701080cf7bf7e7ea1dd0fdd86f035 |
| Last Updated | 2026-09-18 |
| Handoff / Release Condition | Atomic claim+activation required before code or Cargo changes. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| DESKTOP-001-D3 | DESKTOP-001 / #29 | Planned / Unclaimed | D0/I194 and WORK-001 P0-P4 | Mock-only bilingual visual surface |

### Scope

- Fixture-backed visual state, localization catalogs, fallback, keyboard/focus and IME validation.

### Non-Goals

- No real runtime/session/permission/evaluation binding, browser automation, or release packaging.

### Acceptance

- Given the same fixture, when locale changes, then visible labels change but fixture identity and state do not.
- Given an unavailable locale, then deterministic fallback is shown without network access.

### Planned Validation

- Bilingual snapshot/manual visual walkthrough and IME/focus checks.
- `./scripts/validate_project_governance.sh .` and `bash scripts/validate_collaboration_claims.sh .`.
- Exact-head CI and independent review after implementation.

### Documentation To Update

- Desktop owner, Board, backlog and user-facing setup documentation for the selected mock surface.

### Risks And Rollback

- Risk: renderer or localization dependency leaks into shared runtime crates.
- Rollback: remove the isolated Desktop slice while preserving all shared runtime contracts.

## Activation Gate

### 2026-09-16 Design Reference Correction

The maintainer reiterated that Desktop development must implement the design images in Issue #29
comment 5340359578. All four images were retrieved and inspected: task overview, new task, preset
list and preset detail/edit. The current minimal window demonstrates framework/input feasibility,
not design completion. Use the confirmed mapping in `docs/design/talos-desktop/DESIGN.md` for the
visual implementation and acceptance; do not close I277 on the basis of the input test window.
Preserve this iteration's published mock-only boundary while explicitly identifying the subsequent
real runtime/session/preset bindings needed to deliver the requested functioning Desktop. Mock
interaction and unit tests cannot be claimed as evidence of those real capabilities.

### 2026-09-16 Fixed Upstream License Gate

The fixed upstream candidate `a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74` resolved
successfully by direct connection, with proxies disabled. Its isolated locked macOS check passed
on Rust 1.97.0. A separate copy of Desktop sources also passed check after adapting focus,
window-close and text-paint APIs and adding initial AccessKit button roles/states and draft
value/selection actions. This is compile evidence, not native accessibility acceptance. The test
build was deliberately stopped after the dependency review found blockers; no test pass is claimed.

Independent Agent-role dependency review returned REQUEST CHANGES: reachable `ztracing`,
`zlog` and `ztracing_macro` declare GPL-3.0-or-later; `gpui_util` and `gpui_shared_string`
lack explicit package license evidence. Supported features cannot disable these paths. The first
AccessKit revision also depends on `sum_tree -> ztracing`, so that older revision does not solve it.

I277 remains Active/Claimed and incomplete; fixed-upstream introduction is blocked by the license
gate. Talos dependencies were not switched and no candidate was pushed. Preserve the accepted
renderer and Published Baseline. The isolated prototype is currently at
`/private/tmp/talos-i277-upstream.NgcJ13` (temporary recovery context, not completion evidence).
Resume with upstream license clarification or another reviewed artifact; do not silently fork,
relabel licenses, drop accessibility acceptance or equate compilation with dependency approval.

Follow-up: upstream PR #63573's exact merge `ac5af8b9e1ea3f7922fbabefe409c05b8766135c`
explicitly fixes these license declarations. Independent review verified the five Apache manifests
and license symlinks and permits isolated evaluation of this newer fixed artifact. It also corrected
the earlier missing-license-file claim: old gpui_util/shared_string already had Apache symlinks,
but omitted manifest metadata. The new probe resolved 694 packages; fresh compilation and complete
dependency review are underway. The old revision remains unapproved and Talos dependencies remain
unchanged. This follow-up supersedes the instruction to wait for upstream clarification: evaluate
the already-published upstream fix, without copying that license change onto the old snapshot.

The new exact-revision independent dependency review subsequently permitted Desktop-only local
introduction: full metadata resolved 695 packages including the probe, 98 build-script entrypoints,
and 23 Apache-licensed Zed packages. No GPL-only dependency was identified. Rust 1.97.0 check and
13 isolated input/localization tests passed. Upstream macOS deprecated APIs and block 0.1.6 future
compatibility warnings remain recorded risks, not suppressed findings. Talos now pins gpui and
gpui_platform to this same revision and includes the migrated input/window code; main-workspace
lock resolution, feature-union validation and native acceptance are still in progress. No push,
merge or completion is claimed.

Claim and activation became effective through #570. The dependency/security gate remains required
before GPUI Cargo changes; approval of the architecture alone is not passing dependency evidence.

## 2026-09-16 Renderer Decision Acceptance

The maintainer explicitly accepted the GPUI renderer and execution-domain proposal and authorized
updating ADR-059 and continuing I277. GPUI 0.2.2 is the selected release subject to rechecking latest
stable at introduction. Framework-native integration and internal scheduling are isolated to
Desktop; business async stays Tokio-owned. No real Runtime binding, bridge, release or new
Talos-authored unsafe code is authorized. The published baseline above is unchanged.

Next: complete independent dependency/native/build-script review, then implement and validate the
actual bilingual mock window locally. PR #571's stdout prototype is insufficient for completion.

## Local Implementation Checkpoint

This checkpoint supersedes the preceding next-action instruction, not the Published Baseline.
Independent Agent-role dependency inspection permits local GPUI compilation under the controls in
the dependency/security matrix; it is not final release or platform approval.

The local candidate now implements a GPUI window, locale controls, keyboard focus traversal and
presentation-local editable input with UTF-16/IME state, grapheme navigation and selection.
Follow-up review identified and corrected off-hitbox drag updates and stale composition on blur.
The blur policy retains visible text, clears rollback state and terminates dragging. It still
requires native IME verification. The mock remains isolated and unpublished with an explicit
desktop-ui feature; no real runtime, permission or persistence integration was added.

Evidence collected locally:

- Locked/offline Desktop build and check passed on macOS with runtime_shaders.
- Eleven feature-enabled unit tests passed; these test models, not real native interactions.
- Strict Desktop Clippy passed before the latest blur patch; rerun required for final candidate.
- Governance and collaboration validators passed before the latest checkpoint; rerun required.
- The initial window process stayed running without immediate startup error, but screencapture
  failed with `could not create image from display`. No visual or IME pass is claimed.
- Explicit macOS, Windows and Linux GUI CI checks are configured locally, not yet executed remotely.

Remaining work: native visual/IME/focus/drag walkthrough, timing and reduced-motion evidence,
host-specific validation, final independent implementation review, complete local preflight and
stable candidate submission. Do not merge PR #571's old head or mark this iteration Complete.
Resume from the current impl/i277-desktop-d3 worktree; preserve its uncommitted implementation.
The runnable command after rebuilding is `target/debug/talos-desktop-mock zh-CN`.

## 2026-09-16 macOS Manual Acceptance Checkpoint

Requested outcome: record the completed local interaction walkthrough without claiming full
iteration acceptance. Artifacts: this owner and Desktop README; Published Baseline and prior
checkpoints remain unchanged. Remaining validation and residuals stay owned by I277.

The maintainer supplied screenshots and explicit confirmations for bilingual locale switching
with fixture/draft retention, Chinese IME blur then Escape and subsequent committed input,
candidate placement, off-field drag selection, Tab/Shift-Tab traversal and Enter/Space activation,
minimum-size bilingual layout and resize recovery, whole-grapheme deletion of a family emoji,
clipboard copy/cut/paste, long-input horizontal scrolling and normal window-close exit.
These macOS interactions passed. Direct cancellation of an active IME composition is distinct
from the verified Escape-after-blur path and is not inferred from it.

The maintainer also reported this native diagnostic at 14:49:20:
`error messaging the mach port for IMKCFRunLoopWakeUpReliable`.
No corresponding interaction failure was observed; exact trigger, recurrence and root cause
are unknown. Do not suppress the log or describe it as proven harmless.

Incremental independent Agent-role code review found no new deterministic defect, but is not
final exact-head approval. Remaining gates: accessibility name/role/state integration,
input-to-visible/frame-time measurements, reduced-motion equivalence, native failure-path and
other-host evidence, full local preflight, stable-candidate CI and final independent review.
I277 remains Active / Claimed; no Completion Commit is claimed.

### Exact-release capability finding

Follow-up independent source inspection confirmed that the selected crates.io GPUI 0.2.2
does not expose a native accessibility name/role/state path for custom controls. Its Element
and interactive div contracts provide layout, paint, focus and input, not semantic nodes.
The earlier Zed snapshot's AccessKit declaration does not establish published-release capability.
The dependency matrix now makes that distinction explicit. ADR-059's accessibility requirement
remains binding; this blocks final acceptance, not the truth of the passed macOS interactions.

Resolution requires an explicit dependency/boundary decision (for example, assessment of a fixed
upstream revision) or an explicitly approved acceptance change. No GPUI fork, new native shim,
dependency switch or silent accessibility deferral is authorized by this checkpoint. Full
preflight and remote candidate submission remain pending; do not consume full CI merely to
reconfirm an already-known capability gap. Both local governance validators passed with zero
warnings after the manual checkpoint; rerun after final record edits. Next: present this exact
capability mismatch to the maintainer and agree the resolution before changing the renderer.

### Fixed-upstream evaluation in progress

The maintainer accepted that resolution direction. ADR-059 now records the bounded evaluation of
`a8fafdd7ee36fb3fb98ebbfe5d3be983301d9e74`, retaining the accessibility requirement and existing
implementation. GitHub API source inspection confirms semantic APIs and three host adapter
declarations. Raw-source HTTP requests timed out; GitHub contents API succeeded. The disposable
probe `/private/tmp/talos-i277-upstream.NgcJ13` uses that exact revision for both gpui and
gpui_platform; metadata without dependency resolution passed. Full lock resolution is pending,
not evidence of a compatible graph. No Talos Cargo dependency switch has occurred.

Next: finish isolated lock/native dependency review, check pinned-toolchain compatibility and
host action wiring, then decide the concrete migration from evidence. The prior review agent
failed at the service layer; no independent approval of the new revision is claimed.

The subsequent independent source review completed: actual AccessKit adapters and native
activation/action/focus paths were confirmed on all three host implementations. This is bounded
source approval of feasibility, not dependency/build or screen-reader acceptance. Editable
text/value/selection handlers still require Talos implementation.

The fixed-revision source archive downloaded successfully (SHA-256
`cb607af4cb520f3bdb4ce7b7e23c4a848f83de5f29dca295a1019c708571805b`). The disposable probe now uses
that extracted source by path, excluded from its own workspace to preserve upstream manifest
inheritance. Offline resolution reached upstream's Git dependency `zed-scap` and stopped because
it was not cached; this is a source-resolution requirement, not proof screen capture is enabled.
Online resolution is being retried with Git CLI / HTTP 1.1. Project Cargo files remain unchanged
by this evaluation. Both governance validators passed with zero warnings before this addendum.

## 2026-09-17 Current Local Recovery Checkpoint

This checkpoint supersedes earlier next-action instructions, not the Published Baseline.
Work remains on `impl/i277-desktop-d3`, unsubmitted local candidate, Active/Claimed.
GPUI and gpui_platform now use the independently inspected upstream Apache clarification commit
`ac5af8b9e1ea3f7922fbabefe409c05b8766135c`; the main workspace lockfile is updated.
Locked Desktop compilation, 17 tests and all-target strict Clippy pass on macOS Rust 1.97.0.
Default CLI tree inspection found no GPUI/AccessKit dependency. The `block 0.1.6` future-incompatibility
warning remains. No cross-platform compile or final independent approval is claimed.

Current local behavior: task overview fixture, five task-local tabs, recent-task navigation,
new-task form with required goal/workspace and preset selection, memory-only preview creation,
preset list and explicit save/cancel editing of instructions and Default/Quick/Deep model roles.
Created previews copy template contents; later template edits and locale changes do not mutate
them. All data is fixture or presentation memory, with no provider, runtime, config or workspace
write. This is not a working production task launcher or persistent preset manager.

The four original Issue #29 design images are archived under `docs/design/talos-desktop/`, with
source URLs and hashes in REFERENCES.md. Current implementation is only partial design coverage:
multiline goal/instruction editing, complete preset management/capabilities, visual polish and
image-by-image native verification remain. Do not count the basic input window's prior acceptance
as acceptance of these new surfaces. The screen-capture probe still fails with
`could not create image from display`; no new screenshot-based pass is claimed.

Accessibility now exposes roles, localized names, scalar text-selection actions and same-frame
shaped/scrolled text-run geometry. The selection mismatch identified by independent review was
fixed and tested. The geometry follow-up review agent failed with a service quota error before
returning a verdict. Native assistive-technology action/geometry validation, performance and
reduced-motion evidence, full preflight, stable exact-head CI/review, merge and owner-first closeout
remain required. Resume from this worktree, preserve all uncommitted changes, and continue local
design/function convergence before updating PR #571.

### 2026-09-17 Multiline Input Local Checkpoint

The goal and preset instructions now enable multiline editing, hard newlines, width-dependent
wrapping, a five-line viewport, vertical scrolling and visual-row keyboard/pointer navigation.
Painting, selection, IME range coordinates and AccessKit text runs share the same row layout.
Accessible runs retain hard newline bytes and scalar selection offsets; soft-wrap caret affinity
distinguishes the previous row end from the next row start at the same byte position.

Independent Agent-role input review found a missing soft-wrap affinity and insufficient local
panic containment around native-backed shaping/painting. Both have local corrections and focused
tests; this is not final approval. Shaping failure preserves editable text and accessible value
without re-entering the native shaper and shows an error edge. Full native-failure and assistive
technology behavior still need validation, including other GPUI-rendered elements.

Evidence: locked/offline Desktop tests pass (23/23); all-target Desktop Clippy with `-D warnings`
passes; `git diff --check` passes. The upstream `block 0.1.6` future-incompatibility warning remains.
Tests cover deterministic text/layout models, not actual platform font rendering or native IME.
No new screenshot, platform acceptance, remote push or completion is claimed. Existing residuals
above remain owned by I277. Next: review these corrections, finish remaining design-based controls,
then exercise the actual window and complete stable-candidate gates. Stay on this worktree.

### 2026-09-17 Design Rework and Native Harness Checkpoint

Requested outcome: converge the four reference surfaces and preserve truthful acceptance evidence.
Artifacts: this owner, Desktop README and local Desktop implementation. Preserve all earlier
checkpoints and Published Baseline. Status remains Active/Claimed; all residual work below belongs
to I277. No new claim, publication, runtime binding or completion is asserted.

The maintainer rejected the initial design fidelity. Functional form controls and passing unit
tests were insufficient: page proportions, navigation, grouped rows and interaction hierarchy
diverged from the four archived images. Subsequent local edits bound the new-task form width,
introduced preset selection, compacted detail rows, separated the default preset in the list,
added a task path/activity structure, moved language selection to local display settings, and
introduced compact top navigation below 900 logical pixels. These are partial corrections,
not a visual APPROVE. Screenshot count alone is not acceptance.

Presentation-only preset management now includes name/description/instructions, three model
roles, capability toggles, default selection, independent copies, confirmed deletion, unsaved
creation and reordering. New drafts do not allocate a list entry before valid save; cancellation
leaves the list unchanged. Ordering moves stable IDs without changing defaults or preview
snapshots. Independent review identified parent-row drag triggering child clicks; the local
correction uses a separate drag handle. Actual pointer drag/drop remains unverified.

The optional `visual-test` feature enables GPUI's native offscreen scene readback; unlike the
earlier system screencapture probe, it works on this macOS host. Run the capture subprocess with
a watchdog because upstream Metal readback waits synchronously. Its test-support dependency
expansion received bounded local Agent-role review, not final dependency approval. It introduces
pinned proptest revision `3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b`; final resolved graph/advisory
review remains required. No ScreenCaptureKit feature is enabled.

Latest native harness run used local build `cargo build -p talos-desktop --features visual-test
--locked --offline` and `talos-desktop-mock --capture` under a 45-second process watchdog. It
exited zero and generated 28 nonblank PNGs (seven scenarios, two locales, two logical sizes).
The temporary output `/private/tmp/talos-i277-render-20260917-m` is inspection context, not durable
completion evidence. Reproduce it from the README commands when needed.

The harness dispatches actual GPUI key events for Settings Enter/Escape/Space and checks focus
restoration. Model-picker checks scroll the editor to the model group before opening, dispatch
Enter/Escape/Space, focus a candidate and press Enter, then assert draft-only mutation and focus
restoration. Cancel/save/reopen are subsequently exercised by command dispatch, not button clicks.
These checks passed in both locales and sizes; they are not OS keyboard, IME or screen-reader
acceptance. The latest unit run passed 33 tests; strict all-target Desktop Clippy and whitespace
checks passed after the model harness extension. Upstream `block 0.1.6` future incompatibility
remains unsuppressed.

Remaining closure requirements:

- Finish reference correspondence: icons, navigation/back affordances, grouped visual treatment,
  remaining new-task controls and final four-page comparison. Do not infer acceptance from builds.
- Verify actual pointer drag/drop, save/cancel button activation, resize with focus/popovers and
  all relevant scroll states; extend the harness instead of recording invisible targets as tested.
- Fresh native multiline IME and assistive-technology action/geometry acceptance after migration;
  measured responsiveness, reduced-motion and native failure-path evidence.
- Final dependency/feature isolation review, complete local preflight, cross-platform exact-head
  CI, independent implementation review, merge-time CAS and owner-first closeout.

Resume on `impl/i277-desktop-d3` with the existing uncommitted work. PR #571 remains an older
incomplete candidate; do not merge it or open per-edit PRs. Preserve local convergence and push
only the complete stable stage. Global unrelated manifest drift is outside this checkpoint.

### 2026-09-17 Reference-Fidelity Correction Checkpoint

Outcome remains partial, Active/Claimed. This checkpoint records local implementation and
verification only; Published Baseline, prior evidence and completion requirements remain intact.
No new branch, remote candidate, runtime authority or release action was introduced.

The latest reference comparison again confirmed a material visual gap. The overview now selects
structured English or Chinese fixture copy instead of displaying both languages concatenated.
It restores goal/work descriptions, heading hierarchy, connected stage nodes, timeline dots and
current-activity highlighting. Its change summary derives file/addition/deletion totals from the
fixture and navigates to the Changes tab, including focus transfer. Pinned local Lucide assets
now render with explicit colors; the primary new-task button uses a white icon for contrast.

Local verification: `cargo test -p talos-desktop --features visual-test --locked --offline`
passed 37 tests, including single-language execution copy and immutable fixture identity.
`cargo clippy -p talos-desktop --features visual-test --all-targets --locked --offline -- -D warnings`
and the matching locked offline build passed. The existing upstream `block 0.1.6` future
incompatibility warning remains visible.

The native capture process exited zero under a 45-second watchdog and produced 38 PNGs in
`/private/tmp/talos-i277-render-20260917-t`: two locales, two sizes, nine common scenarios and
desktop-only preset dragging. This path is disposable inspection evidence, not a durable
completion artifact. The Changes scenario checks command dispatch and tab focus, not a physical
pointer click. The harness also exercises actual GPUI pointer events for separate-handle preset
reordering and keyboard events for task options; these supersede the earlier unverified harness
coverage statements, but not OS-level manual acceptance requirements.

Residuals remain owned by I277: sidebar task/user fidelity, remaining preset/new-task copy and
layout correspondence, four-page visual acceptance, native directory-picker/IME/AT acceptance,
resize/scroll/focus coverage, dependency isolation and final stable-candidate delivery gates.
Do not infer design approval from these tests or screenshots. Resume on the existing worktree,
continue local convergence and do not merge the older PR #571 candidate.

### 2026-09-17 Preset and Fixture Navigation Checkpoint

Requested outcome: continue four-page reference convergence. Changed artifacts are isolated
Desktop implementation, its README and this owner. Preserve Published Baseline and all earlier
checkpoints. State remains Active/Claimed; remaining visual/native/final delivery acceptance is
owned by I277. No external execution, persistence, permission or session authority was added.

Preset defaults now seed a single-language editable description using the startup locale.
Switching locale never rewrites those fields or saved preview snapshots. Coding instructions
match the reference's five-line example. Basic info, models and capabilities use grouped rows;
the default preset has its own emphasis. Compact list rows retain drag handle, icon and name on
one row. Workspace input and directory selection share a row. A populated-form capture scene
supports like-for-like reference inspection without changing empty new-task behavior.

The sidebar now provides four independently identified, localized task fixtures with separate
goals, work, stages, activities and files. Selection opens the corresponding overview and moves
focus to its tab. View-all/compact recent-task navigation opens a scrollable task list. This is
mock navigation only, not real session switching. Tests check invalid IDs, locale preservation,
fixture identities and preview retention. Existing unsaved input entities are not replaced.

Locked/offline Desktop tests passed 39 cases; strict all-target Clippy and the visual-test build
passed. The expanded native harness completed with exit zero and 50 nonblank scenes in both
locales and sizes, including repeated task-switch/focus assertions. Latest disposable inspection
output is `/private/tmp/talos-i277-render-20260917-w`. Earlier output directories are not durable
evidence and are removed once superseded, per the maintainer's cleanup request; original design
assets stay in the repository.

The earlier 42-scene batch hit its 45-second aggregate watchdog after 36 images (exit 142).
It was not recorded as passing. The same binary completed under a 90-second aggregate watchdog;
the current 50-scene batch also completed under that limit. The timeout protection remains
mandatory because upstream native readback can block. README records the revised batch budget.

Independent Agent-role visual inspection identified remaining user-area treatment, model display
names, detail action grouping and activity source-column correspondence; these stay local I277
work, not new remote issues. No final visual approval, native IME/AT acceptance, full preflight,
cross-platform CI, implementation merge or completion is claimed.

Follow-up independent navigation review found that cross-page entry overwrote Overview focus
with root focus. Adding Tasks-to-Overview to the native harness reproduced the failure with
exit 1 (`Fixture 1 navigation or focus failed`). The common page-transition focus route now
selects the Overview tab for Fixture pages. The harness additionally covers NewTask and Presets
origins, same-page switching and goal/workspace text retention. The repaired full 50-scene batch
exited zero in `/private/tmp/talos-i277-render-20260917-y`; 39 tests and strict Clippy passed again.
Earlier v/w/regression images were deleted after replacement. Preserve original reference assets.

Clarification: retained goal/workspace text and saved preview snapshots are verified, not complete
form restoration. `begin_task` resets preset selection and options. README makes that boundary
explicit. The independent review is scoped to navigation correctness, not final visual acceptance.

### 2026-09-17 Preset Menu and Bottom Action Verification

Requested outcome: continue reference fidelity and prove scrollable actions remain reachable.
Changed artifacts: isolated Desktop view/capture harness, README and this execution checkpoint.
Published Baseline and prior dated evidence are preserved; status remains Active/Claimed.

The local view now uses localized model display names without changing stored fixture IDs,
explicit OTHER grouping, task activity source labels and a local user icon. Preset emphasis and
menu spacing are compacted. The preset menu is bounded at 280 logical pixels with independent
scrolling. Its eleven-entry scenario selects the last item through real GPUI pointer events and
checks selected identity, dismissal and restored focus.

The new bottom-action scenario scrolls the detail view, clicks Delete, cancels and verifies that
the original list is unchanged, then requests and confirms deletion and checks the list/page
transition. All four locale/size combinations passed. These are dispatched GPUI events, not an
OS-level human walkthrough. The final screenshot reopens the remaining General preset at its
bottom actions; it is not a screenshot of the confirmation prompt.

Validation: locked/offline visual-test build passed, 40 Desktop tests passed, strict all-target
Clippy passed. The native batch completed with exit zero under the 90-second watchdog and
produced 58 nonblank images in `/private/tmp/talos-i277-render-20260917-ac`. The superseded ab
batch is disposable and is removed after replacement; original repository design images remain.
The upstream block 0.1.6 future incompatibility warning remains visible.

Independent Agent-role comparison of the preceding ab batch found no new blocking layout
regression, but recommends a clearer preset edit hit area and checkbox sizing. Those refinements,
native multiline IME/accessibility acceptance, responsiveness/reduced-motion evidence, final
dependency review, preflight and exact-head remote delivery gates remain owned by I277. Neither
this scoped review nor the new interaction test establishes final visual acceptance. Continue
local convergence on the existing branch; do not merge the old PR #571 candidate.

### 2026-09-17 Edit Hit Area and Menu Event Ordering

The preset list now exposes one keyboard/pointer edit button for name, description and trailing
chevron, independently of drag/default controls. Task-option checkboxes use fixed 16-pixel boxes
with embedded Lucide check icons rather than font-dependent checkbox glyphs. Native capture
clicks the description and asserts the correct editor and name-field focus.

Independent review identified preset-menu outside-down/click ordering that reopened the menu
when its trigger was clicked twice, and focus loss when dismissing to blank space. The outside
handler now excludes the measured trigger bounds. Root bubble handling restores trigger focus
only when a child has not consumed default focus, preventing root autofocus from overriding it.
The ae batch failed the focus assertion; it is not passing evidence. The af batch then exposed an
unreachable compact test position: the trigger was below the viewport and the overlay intercepted
the test click. The test now scrolls the form and clicks the visible trigger arrow, with a redraw
between pointer press/release, followed by outside dismissal and Enter reopening.

The ag batch exited zero under the 90-second watchdog, producing 62 nonblank images across both
locales and sizes. Latest disposable output: `/private/tmp/talos-i277-render-20260917-ag`.
Superseded ac/ad and failed ae/af images are removed after replacement; original designs remain.
Forty feature-enabled tests, locked/offline build, strict Clippy and package formatting passed.
Independent follow-up found no new reproducible event-order defect in the root/child focus fix;
clicking a real input while the menu is open remains an additional native acceptance check.
I277 stays Active/Claimed. No final native accessibility/IME/performance acceptance, full
preflight, remote candidate push or merge is claimed. Continue from this branch without creating
another implementation branch or PR.

### 2026-09-17 Workspace Preflight and Native Acceptance Handoff

The standard `./scripts/release_preflight.sh` was run with the pinned Rust 1.97.0 toolchain.
Public-site/installers, governance, collaboration, text-boundary and CI-classifier checks passed.
The process ultimately exited 101 during workspace test compilation/linking with explicit
`No space left on device` diagnostics, including talos-cli and an agent integration test.
This is a failed preflight, not a workspace test pass or an intermittent exception. Native
linker diagnostics also reported tree-sitter objects targeting macOS 26.5 while linking for
11.0; review deployment-target consistency before claiming broader host compatibility.

The stopped, obsolete `/private/tmp/talos-i277-upstream.NgcJ13/target` build directory was removed,
reclaiming about 3.9 GiB. Its source, locks and upstream review inputs remain; the current Desktop
binary and repository source were not removed. The failed workspace run grew the main target:
deps about 8.6 GiB, incremental 2.2 GiB, examples 1.1 GiB. With only about 3.9 GiB free, establish
adequate build space before retrying; do not repeat the same full build blindly.

The maintainer is available for fresh native acceptance. The first requested step is launching
`target/debug/talos-desktop-mock zh-CN`, entering New Task and focusing Goal. No result has yet
been supplied for that step. Keep this binary unchanged during the walkthrough; do not inherit
the earlier renderer's IME/accessibility acceptance.

Independent read-only dependency review confirmed default CLI/Desktop feature isolation, the
common exact GPUI revision and the absence of new Talos unsafe/shared-runtime integration. It
also found stale artifact-review wording in ADR-059 and the dependency matrix, including the
macOS framework-internal async-std path and visual-test closure. Those records and current-graph
review remain I277 residuals; this is not final dependency/platform approval. I277 remains
Active/Claimed, with no stable candidate pushed or merge authorized by this checkpoint.

Follow-up independent current-graph review permits the fixed ac5af8b9 dependency graph for local
mock development/acceptance. Its reproducible inventory, enabled normal/build tree and bounded
build-script/Git review are appended in the Desktop dependency/security matrix. The reviewer
corrected the earlier interpretation of the historical async-std chain; no ADR expansion is
required. This resolves the stale current-artifact record and local-introduction review residual,
not final platform/security acceptance, failed workspace preflight or exact-head delivery gates.
The manual walkthrough remains at the first requested step without a maintainer result yet.

### 2026-09-17 Low-Concurrency Preflight Follow-up

After confirming no active compiler, about 3.2 GiB of regenerable main-target incremental caches
were removed. `env CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 ./scripts/release_preflight.sh` then
passed workspace check and Clippy and reached workspace tests without another disk exhaustion.
It exited 101 at Runtime's `tests::finalizers_share_the_original_global_deadline_without_resetting_it`:
42 tests passed and the cancellation-marker assertion failed. Do not report complete preflight.

The Runtime source and shutdown implementation are unchanged against HEAD. The test budgets
200ms of wall time and creates its cancellation marker only after its second finalizer starts;
budget exhaustion before that start is a plausible failure path, not yet proof of the original
schedule. The already-built exact Runtime test binary passed the isolated test and all 43 tests
with `--test-threads=1`. These narrower reruns do not repair or supersede the full-run failure.
A focused Cargo invocation was stopped when it began building another feature graph; reuse the
existing test artifact for further diagnosis rather than spending remaining disk on duplicates.
The existing timing assumption needs investigation before declaring the final candidate green;
do not silently relax assertions or modify shared Runtime authority under Desktop scope.

Independent lock comparison also found an unjustified core-foundation 0.10.1-to-0.10.0 change.
All current 0.10 consumers permit 0.10.1, which is not yanked and satisfies the pinned toolchain.
Restore the pre-existing version and validate the affected graph before stable delivery. The
current Desktop binary remains unchanged for manual acceptance, with SHA-256
`5bb37a6d87871a6551efc9ca8fcec646e4f1f1ef402dd7e5411602b4e161d3c0`.

Follow-up: `cargo update -p core-foundation@0.10.0 --precise 0.10.1 --offline` restored the
original version without other reported updates. Structured comparison with HEAD's lock now
finds no removed original package identities and 418 added identities. The locked/offline
Desktop visual-test check passed with CARGO_BUILD_JOBS=2 and CARGO_INCREMENTAL=0. This is check
evidence only: the existing manual binary retains the exact SHA-256 above and has not been
relinked. Final candidate build/native acceptance must bind the restored lock, not assume the
old binary contains the updated library. Runtime test diagnosis remains open.

### 2026-09-17 Restored-Lock Tests And Measurement Boundary

With the restored core-foundation 0.10.1 lock, `env CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
cargo test -p talos-desktop --features visual-test --bin talos-desktop-mock --locked --offline`
passed all 40 tests. An initial `--lib` invocation exited 101 because Desktop has no library
target; the corrected binary-test invocation above is the passing evidence. The upstream
block 0.1.6 future-incompatibility warning remains. The interactive binary hash was rechecked
and still equals the previous checkpoint, so these tests did not replace the manual executable.
Both governance validators completed successfully with zero warnings before this appended record.

Native manual acceptance is awaiting the maintainer's first input-focus confirmation; no new
manual pass is recorded. I277 remains Active/Claimed, and the failed full Runtime preflight is
not superseded by the Desktop tests.

Inspection of the pinned GPUI source establishes the measurement boundary: Window::present
calls platform_window.draw, then records Instant::now as present_end. The profiler records
input latency from first_input_at to that software timestamp, draw duration separately, and
presentation intervals only under its active-animation conditions. These histograms require
the optional profiler feature, currently disabled. They are useful software-path diagnostics,
not proof of compositor/scanout completion or physical input-to-visible latency. Do not label
capture time, an empty animation histogram, or this software timestamp as measured visible
latency. Measurement implementation and native responsiveness acceptance remain owned here.

### 2026-09-17 Automated Convergence Before Manual Acceptance

The maintainer requested finishing automated work before resuming the guided manual walkthrough.
No new human acceptance is claimed. The binary was rebuilt against the restored lock and optional
profiling feature; its SHA-256 is
`5f15007ebc2bf3819ffe84853bbaa352708f8764af731afac1738205ea393380`.

Independent visual review found and the local candidate corrected an unfocused text field
scrolling to its end on first display. Caret reveal now requires focus, and gaining focus resets
the reveal state. The native harness asserts that the initial unfocused preset description stays
at offset zero. Preset menus use a viewport-bounded height instead of a fixed 280px limit.
Separate pre-action captures now show the deletion confirmation and scrolled long-menu expansion;
the earlier post-action screenshots were not evidence of those expanded states.

The bounded native batch at `/private/tmp/talos-i277-render-20260917-ah` exited zero with the
updated binary in both locales and viewport sizes. Locked/offline Desktop tests passed 40/40;
strict feature-enabled Clippy passed. Upstream block 0.1.6 future compatibility remains recorded.
Only regenerable, unused `target/debug/examples` artifacts were removed to recover about 1.6 GiB.

The preflight timing-test failure was handled as bounded test-fixture maintenance within this
local candidate, not a new product requirement or Runtime production change. Runtime's original
wall-clock test was replaced with a private finalizer-stage virtual-clock test with explicit
start/release handshakes. It checks the original stage deadline, aborted-future destruction,
and no later finalizer start. Only cfg(test) code and a dev-only Tokio test-util feature changed;
existing integration ordering/report tests remain. An independent Agent-role review accepted
this limited replacement. The workspace filtered test invocation passed; the current Runtime
test artifact explicitly ran the replacement test (1 passed). This does not alone establish
end-to-end deadline propagation or supersede the outstanding complete preflight run.

Performance diagnostics now use GPUI's profiler only under visual-test and print on native
window-close. The additional hdrhistogram 7.6.0 artifact received independent dependency review.
The ordinary desktop-ui graph does not contain it. Reports state software-only timing and
include sample counts, quantiles, coalescing and excluded mid-draw samples. Real IME, directory
picker, VoiceOver, visible responsiveness and reduced-motion acceptance remain outstanding.

Follow-up validation: the first complete preflight reached talos-skill and failed because its
existing shared-skill fixture writes to ~/.agents/skills/dedup-test outside the sandbox. The
target did not exist before the approved unsandboxed rerun. That rerun of
`env CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 ./scripts/release_preflight.sh` exited zero, including
workspace tests and doctests, and the fixture directory was absent afterward. This supersedes
the earlier failed preflight result for the current local source, without concealing its cause.

Independent visual review inspected the 11 changed ah scenes and accepted the description,
confirmation and menu corrections. The full batch contains 70 nonblank screenshots. Superseded
ag generated images were removed; ah and the four archived source designs remain. Independent
code review requested more precise performance labels; those now say platform-submit-return,
not draw-return. The subsequent locked/offline build passed; the fixed manual-acceptance binary
SHA-256 is `adb844475602e0c96a2090a4e0f3e335ace4e7eed321fe04b9ef8c8b0e5544b9`.
The ah screenshots predate only this diagnostic-label adjustment, not a visual change.
No push, merge, final exact-head approval or new human acceptance is claimed. Resume with the
guided native walkthrough on this binary before final delivery gates.

### 2026-09-17 Native Maintainer Acceptance Results

After rebuilding the same acceptance binary (SHA-256 above), the maintainer explicitly confirmed:
Chinese IME composition and two-line entry with Enter inserting a newline rather than submitting;
Escape cancellation followed by fresh composition without stale text; long multiline scrolling
and aligned live/released selection; native folder selection and cancellation preserving the path
and draft; preset-menu dismissal by clicking the goal field followed by immediate input; locale
switching with draft retention; minimum-size layout, scrolling and resize recovery.

The maintainer found the display-settings interaction unintuitive despite its functional result.
Approved local follow-up: replace the inserted settings bar with a menu anchored near the settings
button; selecting a language closes it, as do outside click and Escape. Preserve draft state and
keyboard focus. This change is still pending and requires focused verification, not a replay of
unaffected acceptance items.

The maintainer explicitly deferred VoiceOver and reduced-motion checks and instructed that neither
block this round's progress. Both remain unverified residuals owned by I277/DESKTOP-001-D3, not
passes or removed requirements. This scheduling exception does not establish accessibility,
reduced-motion or cross-platform readiness or authorize a release.

The supplied close-time software timing report was:

| Metric | Samples | p50 ns | p95 ns | p99 ns | Max ns |
|---|---:|---:|---:|---:|---:|
| Coalesced input to platform submit return | 823 | 11526143 | 33816575 | 48136191 | 81199103 |
| Dirty to platform submit return | 1067 | 9756671 | 23314431 | 33046527 | 82116607 |
| CPU draw | 1200 | 8384511 | 21266431 | 29179903 | 47841279 |

Active-animation intervals had zero samples (unavailable, not zero duration). Input coverage was
823 frames, at most 3 coalesced events per frame, with zero mid-draw excluded samples. These are
observed software distributions, not physical input-to-visible measurements or evidence that a
particular frame budget is met. The report proves the close callback ran; no process exit code
was supplied. The recurring macOS IMKCFRunLoopWakeUpReliable mach-port diagnostic appeared again,
without a reported corresponding functional failure; its cause remains unconfirmed. The pasted
command used zh-C, which falls back to English; it is not evidence of zh-CN startup selection.

### 2026-09-17 Display Settings Popover Follow-Up

The approved local interaction change is implemented: display settings now opens an anchored
language popover beside the sidebar trigger, or below the compact navigation trigger. It does
not insert a bar into the task layout. Language selection closes the menu and returns focus;
Escape and outside clicks dismiss it, preserving clicked child-control focus. Navigation closes
it even when switching task fixtures without changing the page kind. Trigger-bound changes
request a settling frame while open so resize does not retain stale coordinates.

Independent Agent-role review identified the resize and same-page navigation edge cases; both
were corrected and the final incremental static review found no remaining blocking issue.
Locked/offline Desktop build, 40 tests and strict all-target Clippy passed. The 90-second bounded
native capture run at `/private/tmp/talos-i277-render-20260917-aj` exited zero, including language
selection/value, close/focus, outside dismissal and same-page navigation assertions in both
locales and viewport sizes. Screenshot inspection confirms anchored placement. Actual live
resize and pointer language selection still need the narrow maintainer recheck; previous manual
input/folder results are retained, not relabelled as proof of this new interaction.

The rebuilt binary SHA-256 is
`03166865379eaf0d78ba0c8f79428470c189970ba78ef3e40c8edb75e7c8753f`.
No remote push or merge occurred. VoiceOver and reduced-motion remain explicitly deferred,
non-blocking for this round and unverified. Full-workspace preflight evidence predates this
Desktop-only change; the commands above are its focused local validation, not a new full run.

### 2026-09-18 Current-Language Menu Highlight Correction

The maintainer found that reopening settings always highlighted English. The open command
unconditionally focused English and the background represented focus rather than selection;
the earlier native harness also incorrectly expected English focus in both locales. Opening
now focuses the current locale. Selected-language background follows locale state, while a
separate border indicates keyboard focus without implying a selection change.

The native harness now checks current-locale focus on initial opening and after switching and
reopening. The bounded capture batch at `/private/tmp/talos-i277-render-20260918-locale` exited
zero in both locales and sizes; the Chinese compact screenshot visibly selects Chinese.
Desktop tests passed 40/40, visual-test all-target strict Clippy and locked build passed.
Acceptance binary SHA-256: `cf61a681d57522f3768e062ef76a3bb459a4b18de0dbea4602dc069b605ec8e2`.
Live maintainer recheck remains pending. Earlier input acceptance and explicitly deferred
VoiceOver/reduced-motion checks retain their recorded scope; no remote candidate was pushed.

### 2026-09-18 Maintainer Popover Acceptance and Candidate Preparation

The maintainer explicitly confirmed all three guided checks on the rebuilt binary above:
current-language highlight and switching/reopening; Escape and outside-click dismissal without
page movement; resize anchoring and task navigation closing the popover. This completes the
narrow recheck without replaying or broadening earlier input/folder acceptance.

Delivery is now Review / Claimed. Completion Commit: Pending. VoiceOver and reduced-motion
remain maintainer-deferred, non-blocking for this round, unverified residuals owned by I277 and
DESKTOP-001-D3. Windows/Linux native interaction and physical display latency are not established.
The existing PR #571 will receive the stable local candidate after final preflight and diff review;
its old head is not evidence for the new implementation. Updated exact-head CI, independent
Agent-role implementation/dependency review and merge-time CAS remain required. No release,
real Runtime binding, or Complete state is authorized by this checkpoint.

Independent local review found a missing clipboard panic boundary. Copy/cut/paste now catch
recoverable Rust panics locally; a failed read/write preserves text, selection and composition.
Cut removes text only after GPUI's write call returns normally. Its `()` API cannot acknowledge
OS clipboard success, and this does not contain native exceptions, aborts or blocking host calls.
Two fault-injection/normal-path tests passed within the updated 42-test Desktop suite. This
post-walkthrough defensive correction does not relabel the earlier manual binary as its evidence.

#### Stable Candidate Changed-File Inventory

The following inventory covers this local convergence stage. Desktop source/assets and design
references implement the mock surface; Cargo/CI carry its isolated renderer validation; ADR/rules
record the accepted boundary; owner/index files carry acceptance and Review state. The three
Runtime files are bounded test-only deadline-fixture maintenance, independently reviewed above.
No Dashboard, permission production, live Runtime implementation, version or release files change.

- `.github/workflows/ci.yml`
- `AGENTS.md`
- `Cargo.lock`
- `EVOLUTION.md` (current-language selection/focus regression lesson)
- `crates/talos-desktop/Cargo.toml`
- `crates/talos-desktop/README.md`
- `crates/talos-desktop/THIRD-PARTY-NOTICES.md`
- `crates/talos-desktop/assets/icons/LICENSE`
- `crates/talos-desktop/assets/icons/README.md`
- `crates/talos-desktop/assets/icons/arrow-left.svg`
- `crates/talos-desktop/assets/icons/bookmark.svg`
- `crates/talos-desktop/assets/icons/check.svg`
- `crates/talos-desktop/assets/icons/chevron-right.svg`
- `crates/talos-desktop/assets/icons/code.svg`
- `crates/talos-desktop/assets/icons/copy.svg`
- `crates/talos-desktop/assets/icons/database.svg`
- `crates/talos-desktop/assets/icons/folder.svg`
- `crates/talos-desktop/assets/icons/globe.svg`
- `crates/talos-desktop/assets/icons/grip-vertical.svg`
- `crates/talos-desktop/assets/icons/message-square.svg`
- `crates/talos-desktop/assets/icons/plus.svg`
- `crates/talos-desktop/assets/icons/search.svg`
- `crates/talos-desktop/assets/icons/settings.svg`
- `crates/talos-desktop/assets/icons/star.svg`
- `crates/talos-desktop/assets/icons/trash.svg`
- `crates/talos-desktop/assets/icons/user-round.svg`
- `crates/talos-desktop/assets/icons/wrench.svg`
- `crates/talos-desktop/src/input.rs`
- `crates/talos-desktop/src/main.rs`
- `crates/talos-desktop/src/presentation.rs`
- `crates/talos-desktop/src/window.rs`
- `crates/talos-runtime/Cargo.toml`
- `crates/talos-runtime/src/lib.rs`
- `crates/talos-runtime/src/shutdown.rs`
- `docs/BOARD.md`
- `docs/backlog/active/DESKTOP-001-D3-mock-visual-i18n.md`
- `docs/decisions/059-desktop-renderer-host-motion-boundary.md`
- `docs/decisions/README.md`
- `docs/design/talos-desktop/DESIGN.md`
- `docs/design/talos-desktop/REFERENCES.md`
- `docs/design/talos-desktop/reference-new-task.png`
- `docs/design/talos-desktop/reference-preset-detail.png`
- `docs/design/talos-desktop/reference-preset-list.png`
- `docs/design/talos-desktop/reference-task-overview.png`
- `docs/iterations/I277-desktop-mock-visual-i18n.md`
- `docs/iterations/README.md`
- `docs/reference/DESKTOP-I194-DEPENDENCY-SECURITY-MATRIX.md`
- `scripts/test_dependency_windows.ps1` (offline cache prerequisite comment)

Final local candidate validation: `env CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
./scripts/release_preflight.sh` exited zero, including workspace check, strict Clippy, tests and
doctests. The explicit Desktop suite passed 42/42, visual-test all-target strict Clippy and locked
build passed. Both governance validators reported zero warnings after Review synchronization;
staged whitespace and Published Baseline byte comparison passed. Independent local implementation
and dependency reviews found no remaining blocker after the clipboard correction. Dependency
review traced all 419 added lock identities to Desktop and found no old identity removed/replaced.
These are local-stage results, not a claim of new remote exact-head CI or approval. Upstream
`block 0.1.6` future incompatibility remains visible. Final binary identity is recorded in the
candidate PR; the earlier maintainer walkthrough remains bound to its recorded pre-clipboard hash.

### 2026-09-18 First Stable Remote Candidate and Offline Cache Repair

Candidate `08fc27c29a6e16d501b461c224f53e164506e988` was pushed to #571 after local convergence;
two independent Agent-role reviews approved that exact head/base with shared-account limits
disclosed in comment 5723492190. Issue #29 comment 5723500147 records the state and deferred rows.
Linux Desktop CI passed. macOS release_preflight also passed, then its offline dependency parity
step failed: all-feature metadata needed uncached accesskit 0.24.1, which default workspace
validation intentionally does not fetch. This was a workflow prerequisite defect, not a failed
Rust test or evidence of a network outage. The subsequent Desktop step was skipped, not passed.

The local correction adds bounded `cargo fetch --locked` before offline all-feature auditing,
preserving the audit's offline contract and locked versions. The next stable head requires fresh
CI and incremental exact-head review; earlier approvals do not silently transfer across heads.

Local `cargo fetch --locked` and the complete `bash scripts/test_dependency_audit.sh` suite
passed (76 complete Bash/PowerShell records). Both governance validators passed. The same fetch
prerequisite covers Windows' offline audit; its misleading workspace-test-cache comment is
corrected. No Rust source or dependency version changed in this CI follow-up.

### 2026-09-18 Implementation Delivery and Deferred Acceptance

Implementation merge: `707b538eea0421333544f22b3e3e4a42687c87a1` (#571).
Exact head `ccf1bbd1d5f4bcf1a813f850bcb8f1806803eaa1` and base
`5f9dcf053ba2b3b31f76693f6bc8c97df36a1df6` remained unchanged at merge-time CAS.
All six checks in CI 35295546744 succeeded, including macOS, Windows and Linux Desktop feature
validation. Independent Agent-role implementation and dependency/security/API reviews approved
the exact head in comment 5723642620, with shared-account identity limits. MERGEABLE/CLEAN,
no overlapping open PR and no blocking feedback were rechecked before expected-head squash merge.
Issue #29 comment 5724493588 and PR comment 5724493593 retain the delivery evidence.

This supersedes candidate-pending descriptions in earlier dated checkpoints. The implementation
is delivered on main; Review / Claimed and Completion Commit: Pending remain truthful until
deferred acceptance is resolved. VoiceOver and reduced-motion remain explicitly unverified and
non-blocking for this round by maintainer instruction, tracked in the existing #29 queue rather
than new per-subtask issues. Do not repeat passed macOS input/popover acceptance. Cross-platform
compilation/tests do not establish Windows/Linux native interaction or physical display latency.
No real Runtime integration, release or next-iteration activation is implied.

## 2026-09-18 Settings Page Decision

The language popover is replaced by a dedicated settings-page path in the mock surface. The
existing preset-management page is reused as the settings destination and now exposes the
language choice as an in-page radio group. Preset creation, editing, duplication, deletion, and
default selection remain on that page; the new-task preset picker remains unchanged. This
removes deferred-popover accessibility and paint-order coupling without adding persistence,
runtime binding, or new configuration authority. Acceptance now covers settings navigation,
in-page locale selection, preset management, keyboard focus, and VoiceOver traversal.
The navigation label is `Settings` / `设置`; there is no separate top-level Presets navigation item.

The maintainer accepted the settings-page interaction in manual validation; the former popover
path is no longer part of the active acceptance flow.

### 2026-09-18 Settings Local Regression Checkpoint

The local follow-up removes the disabled language-popover implementation and obsolete focus/open
state. Settings now has a matching page heading and selected navigation entry. Language radios
expose checked state and support Enter/Space and directional selection; preset management and
the New Task preset picker remain available. The native capture harness now exercises the actual
settings page instead of asserting the removed popover lifecycle.

Forty-two Desktop tests, locked visual-test build, strict visual-test Clippy and formatting passed.
The complete native capture run exited zero with 70 images covering both locales and both sizes
at `/private/tmp/talos-i277-settings-native-final-20260918` (temporary diagnostic evidence).
It dispatched real GPUI keyboard events for Settings navigation and radio selection, checked draft
and preset retention, and exercised preset editing through pointer input. The first local run
exposed an offscreen click assumption at narrow size after adding language controls; the corrected
test scrolls the Settings content until the target is visible before clicking. No test is skipped.
English/Chinese Settings captures were visually inspected. This is not OS screen-reader evidence.

The existing six green #573 checks bind only `4b6f13eabe3485ceb88ace45504c980b30d5760c`;
they do not validate this substantive local correction. Fresh stable-head CI/review are required.
Review / Claimed and Completion Commit: Pending remain. VoiceOver and reduced-motion rows remain
explicitly deferred/unverified in #29; Windows/Linux compilation is not native interaction evidence.
I278's separate Plugin-source plan and the Auto-review incident intake are not Desktop changes.

### 2026-09-18 Settings Delivery And Remaining Acceptance

PR #573 merged as `8caf2b3a7b1701080cf7bf7e7ea1dd0fdd86f035`. Its exact head
`89417002159fce6752852747aa7ec7d774d66c54` and base
`b86db68d30100d4f56745c597847d6bdf872ed5f` remained unchanged through merge-time CAS.
CI 35347139940 passed all six jobs, including Windows Rust workspace and explicit Desktop
feature validation on macOS/Linux/Windows. Local full locked release_preflight also exited zero
after an authorized unsandboxed rerun: the initial sandbox run could not create the existing
talos-skill fixture under the user's shared skill directory. No failing test was skipped.

Independent Agent-role APPROVE was bound to that exact head/base. The reviewer independently
ran 42 Desktop tests and inspected code, GPUI accessibility action wiring, bilingual captures
and owner evidence. Shared-account/workspace role separation is disclosed; this was not an
independent natural-person review. The PR description retains the approval and validation scope.
Immediately before expected-head squash merge, all checks were green, merge state was CLEAN,
no other PR was open, no blocking review existed and remote main/head matched the reviewed pair.

This completes the settings-page implementation delivery, not all I277 acceptance. Keep
Review / Claimed and Completion Commit: Pending until the explicitly deferred VoiceOver and
reduced-motion rows are resolved. Cross-platform native interaction and physical display latency
also remain unproven; compilation and software timestamps are not substitutes. The next human
walkthrough must use the Settings page, never the removed language popover. Preserve earlier
passed macOS IME/input/locale/preset confirmations. I278 may proceed separately after its own
effective claim under the recorded deferred-validation scheduling rule.
