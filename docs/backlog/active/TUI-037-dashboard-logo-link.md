# TUI-037: Dashboard Link In Logo Region

| Field | Value |
| --- | --- |
| Story ID | TUI-037 |
| Type | Product / rendering story |
| Priority | P2 |
| Status | Refinement — deferred behind I156 / TUI-035 real-terminal acceptance |
| Source | Maintainer request 2026-07-27 |
| Parent Epic | None |
| Depends On | TUI-005, TUI-028, TUI-035, ADR-031, ADR-054 |
| Blocks | None |

## Identity / Goal / Value

When the local Dashboard becomes available, show one concise Dashboard line in
the display-only Logo region instead of the generic tips row. Its URL should be
clickable in terminals that support hyperlinks, making the Dashboard reachable
without copying an address manually.

## Scope

- Replace the current `Dashboard ready: …` generic `TipKind::Info` path with a
  structured, non-secret dashboard-availability presentation owned by the
  Logo-prefix renderer.
- Add exactly one Dashboard line to the Logo prefix, with a concise label and
  visible URL/fallback text.
- Encode the URL as an OSC 8 terminal hyperlink when the renderer/backend can
  do so safely; terminals without hyperlink support must still show a complete,
  copyable URL.
- Keep the line display-only: it scrolls with the Logo prefix, never enters
  `TranscriptStore`, session persistence, export, or primary-screen
  scrollback.
- Preserve ordinary tips for non-dashboard information, warnings, errors,
  queue feedback, and approval results.
- Test wide/narrow Logo layouts, URL clipping/wrapping, Alternate Screen
  lifecycle, hyperlink escape safety, and click/fallback behavior in at least
  the real-terminal matrix named by the story.

## Exclusions

- No Dashboard route, data model, authentication, bind-address, or remote
  access change.
- No browser automation or OS-level URL opener; the terminal owns hyperlink
  activation.
- No bearer token, credential, session secret, or query token in visible text
  or OSC 8 hyperlink targets.
- No change to normal TipsComponent behavior other than removing the dashboard
  availability notice from it.

## Decision Links And Constraints

- ADR-031: Dashboard remains loopback-first; its existing token/loopback
  boundary is unchanged.
- ADR-054: Logo is a display-only virtual history prefix in the Alternate
  Screen renderer. Dashboard presentation must share that prefix rather than
  becoming a second renderer or a transcript fact.
- If a structured dashboard notice requires changing a public conversation UI
  enum, create the required compatibility/decision record before implementation
  rather than extending the protocol implicitly.

## Uncertainty And Validation Path

Confirm the exact `crossterm`/`ratatui` hyperlink capability and terminal
compatibility before marking Ready. The fallback URL is mandatory; OSC 8 click
activation is validated manually in Alacritty, Kitty or WezTerm, macOS Terminal
or iTerm2, and tmux. For authenticated non-loopback Dashboard configurations,
the story must establish a usable non-secret navigation URL or explicitly omit
the click target rather than exposing the token.

## State / Status Owners

- Dashboard startup and safe availability metadata: `talos-cli` / Dashboard
  boundary.
- Logo-prefix projection and hyperlink rendering: `talos-tui`.
- Story status: this document; iteration selection is a future owner.

## User-Facing Documentation

- Update Dashboard configuration/usage documentation and the TUI keyboard/UI
  guide to state where the Dashboard URL appears and how terminal hyperlink
  fallback works.
- Document that the URL contains no Dashboard token and may need manual copy in
  terminals that do not activate OSC 8 links.

## Required Reads

- `docs/backlog/active/TUI-005-logo-splash.md`
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
- `docs/backlog/active/TUI-035-narrow-viewport-and-resize-rendering-robustness.md`
- `docs/decisions/031-web-loopback-dashboard-boundary.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/splash.rs`
- `crates/talos-tui/src/inline_terminal.rs`

## Acceptance

- Given a loopback Dashboard starts successfully, when the first or later TUI
  frame is rendered, then the Logo prefix contains one Dashboard line and the
  generic tips row does not contain the dashboard-ready message.
- Given a hyperlink-capable terminal, when the Dashboard URL is activated, then
  the terminal opens the exact loopback Dashboard URL without a token.
- Given a terminal without OSC 8 support, when the Logo line is rendered, then
  a complete copyable URL remains visible and no malformed escape sequence is
  emitted.
- Given a narrow Logo layout, when the Dashboard line is projected, then it is
  width-bounded and does not corrupt Logo, history, composer, or status rows.
- Given history grows or the user scrolls, when Logo rows leave or re-enter the
  history rectangle, then the Dashboard line follows the Logo prefix and never
  becomes a transcript/session/export fact.
- Given Dashboard startup fails, when the error is rendered, then it remains a
  normal error tip and no misleading clickable Logo link appears.
- Given an authenticated non-loopback Dashboard configuration, when no
  non-secret usable URL is available, then no token is displayed or embedded in
  a hyperlink target and the chosen fallback is documented.
- Unit/full-frame tests and the specified real-terminal hyperlink matrix pass;
  `cargo test --workspace --locked` passes.

## Residuals

- OSC 8 support and the safe authenticated-Dashboard navigation behavior must
  be resolved before this story becomes Ready.
- Selection requires a new iteration after I156/TUI-035 reaches its documented
  completion gate.
