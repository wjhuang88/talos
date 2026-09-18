# Talos Desktop Mock

I277 is a local, fixture-backed preview, not a live Talos Runtime client. The native window,
design-based pages, locale controls and multiline input are implemented. The updated macOS
input/folder walkthrough passed on 2026-09-17/18. The maintainer subsequently accepted replacing
the language popover with a Settings page containing language and preset controls. Delivery remains
in Review; VoiceOver and reduced-motion checks are explicitly deferred and unverified, and
Windows/Linux native interaction acceptance remains outstanding.

## Local Build

The renderer and platform facade are pinned together to Zed revision
`ac5af8b9e1ea3f7922fbabefe409c05b8766135c`, which includes upstream Apache license
clarifications and AccessKit support absent from the crates.io GPUI 0.2.2 artifact.
The initial build downloads this Git dependency. No moving branch or local upstream fork is used.

The GUI dependency is opt-in. Normal workspace builds do not enable it; this unpublished package
has no default binary target without the `desktop-ui` feature.

```sh
cargo run -p talos-desktop --features desktop-ui --locked -- zh-CN
cargo run -p talos-desktop --features desktop-ui --locked -- en-US
```

macOS requires Xcode SDK, libclang and a working Metal device/driver. This mock enables GPUI's
`runtime_shaders` feature: shaders compile during renderer startup instead of invoking the separate
offline Metal Toolchain. Startup compilation and its errors must be covered by visual acceptance.
Windows requires
MSVC and Windows SDK tools, including fxc for release compilation. Linux requires the native
development libraries for GPUI's X11/Wayland and font backends. A successful build on one platform
does not establish another platform's support.

Use trusted SDK/compiler paths, a bounded build timeout and no publication credentials. Leave
`RXCB_EXPORT` and `RING_PREGENERATE_ASM` unset, and do not enable `xim-parser/bootstrap`; the reviewed
default graph does not require their additional source/export writes.

The preview does not connect a provider, create a session, execute tools, persist edits or run a
Tokio bridge. Locale changes affect interface labels, not fixture identity or evidence.

## Acceptance Walkthrough

The New Task goal and preset instructions accept multiline text, with wrapping and a five-line
viewport. Enter inserts a newline; Up/Down and Shift-Up/Down navigate/select visual rows. Scroll
within the field to inspect longer text. Workspace remains single-line; model roles use selectors.
These are presentation-only edits; creating or saving a preview does not write workspace/config
files or call a model. Verify soft-wrap end clicks, cross-line dragging and IME candidate placement
after scrolling, in addition to the basic input checks below.

Start in each locale and at the minimum window size. Settings opens a full page containing
language selection and preset management; there is no separate Presets navigation entry or
language popover. The current language is selected independently of keyboard focus. Selecting
a language keeps the Settings page open and preserves draft and fixture state. The New Task
preset picker remains a separate control. Traverse English, Chinese and inputs with
Tab/Shift-Tab; activate locale controls with Enter/Space. Enter mixed Chinese/Latin/emoji text,
switch locale, and verify the same draft remains. With a real Chinese IME, verify composition,
candidate positioning, commit, Escape cancellation and focus changes. Drag a long selection beyond
the field and release; verify live selection and caret/scroll coordinates stay aligned.

The current unit tests cover text-state, catalog and display-configuration logic, not these
native interactions. On input blur, visible composition text is retained and composition rollback
state is cleared; test this policy with an actual system IME, including a subsequent Escape press.
Clipboard calls catch recoverable Rust panics and retain input on failure. Cut removes the selected
text only after the platform write returns normally; GPUI does not expose an OS write acknowledgement.
This guard does not contain native exceptions, aborts or indefinitely blocking host calls.
The UI intentionally has no animated transitions; visual review must still confirm equivalent
state/focus behavior under reduced-motion settings. Software timing distributions collected on
2026-09-17 are recorded in the I277 owner; they do not measure physical display latency. System
screen capture was unavailable for the original prototype; the optional native capture harness
below now provides scene screenshots. User confirmations establish the macOS walkthrough results,
not cross-platform or screen-reader acceptance.

On 2026-09-16 the maintainer verified bilingual switching with draft/fixture retention, IME
focus changes followed by Escape and new input, candidate placement, drag selection beyond the
field, keyboard traversal and activation, minimum-size bilingual layout and resize recovery,
whole-grapheme emoji deletion, clipboard round-trip, long-input scrolling and normal window exit.
These are manual macOS results, not automated or cross-platform evidence. A reported native
`error messaging the mach port for IMKCFRunLoopWakeUpReliable` diagnostic had no observed
interaction failure; its trigger and impact remain unconfirmed. VoiceOver, physical display latency
and reduced-motion evidence remain open in the I277 owner.

The CI workflow explicitly checks/tests desktop-ui on macOS, Windows and Linux for full-validation
changes. The #571 implementation passed those checks; this establishes compilation/test evidence,
not Windows/Linux native interaction or screen-reader acceptance of the subsequent Settings page.

The in-progress accessibility integration exposes language radio roles, names and checked state,
and a localized draft input with value, selection and text-edit actions. Stable element identities
survive locale changes. Real screen-reader validation remains required after this renderer migration;
the earlier macOS walkthrough does not establish the new renderer's native acceptance.

## Local Preset Preview

New preset opens an unsaved draft; Save preview adds it to the in-memory list, and Cancel discards
it. Editing or copying presets does not modify previously created task previews. Deletion requires
confirmation and keeps at least one preset. Drag the grip beside an entry to reorder it, or focus
the row and use Alt+Up/Alt+Down. Ordering does not change the default preset. All changes disappear
when the process exits; capability switches do not grant actual tool permissions.

## Native Capture Harness

The `visual-test` build also prints software timing histograms to the launching terminal when
the interactive window is closed with its native close button. Exercise text input, navigation,
menus and resize before closing. Output includes sample counts, p50/p95/p99/max in nanoseconds,
input coalescing and excluded mid-draw events. Zero samples mean unavailable, not zero latency.
The input measurement starts at GPUI dispatch and ends after the platform draw call returns;
it excludes OS input queueing and does not measure compositor completion or physical scanout.
Multiple input events in one frame share a single sample starting at the earliest event.
Animation intervals may be empty because this mock has no continuous animations. These are
diagnostics for native acceptance, not a replacement for visible responsiveness checks.
Ordinary `desktop-ui` builds do not enable the profiler.

Build the isolated test feature and choose an output directory that does not yet exist:

```sh
cargo build -p talos-desktop --features visual-test --locked
target/debug/talos-desktop-mock --capture /private/tmp/talos-desktop-capture-example
```

Run the capture process under an external timeout (90 seconds for the current full scenario batch).
For example, on a host with Perl: `perl -e 'alarm 90; exec @ARGV' target/debug/talos-desktop-mock --capture /private/tmp/talos-desktop-capture-example`.
This is a test watchdog, not a Desktop runtime dependency; use the host's timeout facility when
Perl is unavailable. A timeout is a failed/incomplete run, not a passing screenshot collection.
The upstream native renderer can block inside Metal readback; Rust panic containment cannot stop
that wait. This feature is optional and not enabled for normal Desktop or CLI builds.

The harness renders four reference pages, task navigation/list, empty and populated new-task forms,
plus preset/model picker and settings states in both locales at
1448×1086 and 640×480 logical sizes. Model controls are scrolled into view before capture. It also
dispatches GPUI keyboard events and asserts settings/picker focus and draft state, then exercises
cancel/save/reopen through command handlers. It also dispatches GPUI pointer drag/drop on the
separate preset handle and keyboard activation of the save button. Task navigation checks fixture
identity and tab focus through command handlers. An eleven-entry preset menu verifies scrolling
and pointer selection of its last item. Detail actions verify scrolling, pointer deletion request,
cancellation without deletion and explicit confirmation in both locales and sizes. It does not simulate OS IME or screen readers,
and does not establish visual similarity automatically. Inspect
the images against the archived design references. Output images are local evidence, not shipped
assets. I277 remains incomplete until the owner acceptance checklist is satisfied.

Preset-list descriptions and their trailing arrow share the edit hit area; the drag handle and
default selector remain separate. The harness clicks the description and checks editor focus.
It also exercises preset-trigger pointer toggling with a redraw between press/release, outside
blank-space dismissal and keyboard reopening from restored focus. In a small window the form is
scrolled to the trigger before that test, matching an actual reachable interaction.

The recent-task rail and compact task list use four independent in-memory fixtures, not stored
sessions. Switching tasks preserves goal/workspace text and the saved preview snapshot. Starting
a new form resets preset selection and task options; this is not full draft restoration. Built-in editable
preset descriptions are seeded in the startup language; changing display language does not
translate or overwrite editable preset content.
