# Native terminal acceptance procedure

Owner: [TUI-PERF-001](../../2026-11-10-tui-render-performance.md).
Status: **Partially executed** in Linux Wayland + niri + Ghostty.
Maintainer-reported passes and deferred operations are recorded in the owner
under “Acceptance evidence reconciliation — 2026-11-12”. The procedure itself
is not evidence; missing exact binary/environment metadata remains open.
Windows: no native environment available, as confirmed by the user.

Run the same candidate build on Linux Wayland + niri + Ghostty, another supported
terminal, and a native Windows terminal when available. Record each platform
separately; do not extrapolate a pass from one terminal to another.

## Record before testing

- Exact source commit and binary SHA-256; debug/release profile and Rust version.
- OS, terminal name/version, compositor where applicable, font and display scale.
- Initial terminal dimensions and resize dimensions.
- Test workspace fixture, history size, diagnostic switch state.
- Start/end time, whether another workload was running, and evidence paths.

Build a CLI executable from the same source as the replay candidate using the
repository-pinned toolchain and `cargo build --locked -p talos-cli` (or the
corresponding locked release build). A replay binary's digest is not the CLI
binary's digest. Start with an isolated test session/workspace and synthetic
content, preserving existing sessions and real approval decisions.

## Operations and required observations

| Operation | Required observation |
|---|---|
| Leave empty/settled session idle for 60 seconds | No repeated full redraw, background churn or cursor relocation; record CPU separately |
| Wait for a model response, including connecting/reconnecting | Animation cadence stays consistent; transient notices change without a key press |
| Reduce window until preview disappears, then restore | Hidden animation stops causing visible work; preview resumes without stale rows |
| Stream text while typing and cancelling | Input remains responsive, cancellation is delivered, final text is retained once |
| Show an approval; scroll detail pages and choose a decision on a harmless fixture | Single cursor policy, paging and decision keys work; no bypass or changed authorization scope |
| Close/cancel an approval from its producer | Expired panel disappears without another key press; no decision is submitted |
| Drag-select at top/bottom edge, then release | Auto-scroll advances while dragging and stops on release; copied selection matches text |
| Hold a navigation key, then release | No prolonged replay after release; distinct key presses remain preserved |
| Resize long history repeatedly and return to tail | Anchors, grouping, ellipsis, explicit newlines and background fills remain correct |
| Enter Chinese, emoji, combining marks and multiline text; move cursor and use IME | No half-wide glyph residue, double cursor, misplaced caret or changed commit behavior |
| Switch workspaces during Git status activity | Old branch/status cannot overwrite the new workspace; input stays responsive |
| Modify a fixture externally | Git status reflects change within the configured refresh/staleness behavior; errors show unknown, not clean |
| Repeat open/close/resize/stream cycles | No growing backlog, accumulating workers or persistent layout residue |

For each row record Pass/Fail/Unavailable, observed result, evidence and exact
reproduction steps for failures. Do not record Pass based only on exit code,
source inspection, unit tests, xwin or PTY replay. Physical display latency and
terminal emulator CPU/PSS must be identified separately from software
input-to-submit latency and Talos process usage.

## Evidence template

```text
Platform:
Source commit:
Binary SHA-256 / profile:
Terminal version / OS / compositor:
Font / scale / dimensions:
Workspace / history / diagnostics:
Started / ended:
Operation:
Observed result:
Pass / Fail / Unavailable:
Evidence path:
Residual issue:
```

Append results to the owner or an explicitly linked evidence report. Missing
platforms remain Review/待验收; they do not become passed when other checks pass.
