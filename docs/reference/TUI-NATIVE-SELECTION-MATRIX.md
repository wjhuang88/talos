# TUI Native Selection And Copy Matrix

This matrix is the I184/TUI-046-A causal evidence and the later TUI-046-B implementation acceptance
schema. A blank or `Not run` cell is not a pass. Every result binds to one exact Talos commit and one
exact terminal environment.

## Required Observation Procedure

For each environment, run the interactive TUI with visible ASCII, CJK, emoji/combining text, wrapped
code or tool output, and at least one visible panel/status row. Record separately:

1. ordinary pointer drag across a partial line and multiple rows;
2. the terminal's documented mouse-reporting override modifier, if any;
3. standard copy shortcut or context-menu copy and the exact pasted text;
4. wheel behavior at the history boundary and while the composer/panel is focused;
5. selection during idle and active redraw/streaming;
6. resize while a selection exists;
7. normal exit and one tested failure/cleanup path, including whether mouse reporting leaks into the
   restored shell.

Do not infer clipboard correctness from a visible highlight. Paste into a non-Talos destination and
compare the observed text. Do not include credentials, hidden tool arguments or other private data
in the fixture.

## I184 Current-Baseline Causal Matrix

These rows exercise the current captured baseline before any TUI-046-B Rust change. They prove only
the policy cause and override behavior; they do not accept the future implementation.

| Talos SHA | Terminal / Version | OS / Platform | Multiplexer | Gesture / Modifier | Selection Result | Copied-Text Observation | Wheel Behavior | Content / Redraw / Resize | Restoration | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| `33cc8dab23a38c387063d1265c230dfa0f8922d9` | Alacritty 0.17.0 (94e7c88) | macOS 26.5.2 (25F84) | `none` | Ordinary drag | Highlight absent; Shift+drag required | `Command+C` then `pbpaste` matched the highlighted text | Wheel scrolls the app, but the selection does not track the projected content | Edge-drag stops at the viewport; no autoscroll; resize clears selection. Exact ASCII/CJK/emoji/combining fixture and active redraw/streaming were not recorded. | Normal exit and failure cleanup were not recorded | Incomplete baseline row; causal observations only, not an acceptance pass |
| `c0fba2e92cace29fde4e2fc33fd26640058eddca` | macOS Terminal 2.15 (`TERM=xterm-256color`) | macOS 26.5.2 (25F84) | `none` | Ordinary drag and Shift+drag with mouse reporting enabled; ordinary drag after disabling View > Allow Mouse Reporting | No highlight while reporting is enabled, including with Shift; disabling reporting allows cross-row native selection | `Command+C` paste matched exactly; status text appears only when included in the visible selection | Terminal-native scroll moves the highlight with its text, but Talos history does not move | Edge-drag invokes terminal-native scrolling rather than Talos history; selection initially survives resize but disappears after repeated resizing; TUI reflows normally. Exact ASCII/CJK/emoji/combining fixture and active redraw/streaming were not recorded. | Normal `/quit` restored the shell without mouse escape leakage; failure cleanup was not run | Incomplete baseline row; causal observations only, not an acceptance pass |

## TUI-046-B Implementation Matrix

Populate only after TUI-046-B has an effective claim and exact implementation head.

| Talos SHA | Terminal / Version | OS / Platform | Multiplexer | Default Drag | Copied Text | Keyboard History | Wheel Policy | Content / Streaming / Resize | Restoration | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| Exact implementation SHA required | Maintainer primary terminal | Exact version required | Exact version or `none` | Not run | Not run | Not run | Not run | Not run | Not run | Pending B |
| Exact implementation SHA required | Materially different platform terminal | Exact version required | Exact version or `none` | Not run | Not run | Not run | Not run | Not run | Not run | Pending B |

## Interpretation

- Ordinary drag blocked while a documented override succeeds under the same captured Talos SHA is
  evidence that mouse reporting, not Alternate Screen alone, owns the conflicting pointer path.
- On the captured Alacritty/macOS baseline, the override requires Shift, selection does not follow
  application scrolling, edge-drag does not autoscroll, and resize clears the selection. These are
  direct reasons native-only selection cannot be the complete default contract for Issue #134.
- On Terminal.app, Shift is not an override while mouse reporting is enabled. Disabling the
  terminal's Allow Mouse Reporting menu item restores native selection, but wheel and edge-drag
  then operate on terminal scrollback instead of Talos history, and repeated resize clears the
  selection. This independently rejects disabling reporting as the complete product policy.
- A terminal-specific override is diagnostic evidence only; it is not the default product contract.
- Passing one terminal does not generalize to another terminal, OS or multiplexer.
- Neither current-baseline row is a procedural pass: fixture coverage, active redraw and at least
  one failure-cleanup path remain unrecorded. The observed pointer/scroll/resize contrasts plus code
  inventory are causal design evidence only. Complete cross-platform execution is deferred to
  TUI-046-B implementation acceptance and does not block development start; TUI-046 cannot close
  until both implementation rows pass on the exact B head.
