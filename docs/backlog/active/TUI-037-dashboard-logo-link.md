# TUI-037: Dashboard Link In Logo Region

| Field | Value |
| --- | --- |
| Story ID | TUI-037 |
| Type | Product / rendering story |
| Priority | P1 |
| Status | Refinement — first post-I158 disposition; design gates unresolved |
| Source | Maintainer request 2026-07-27; reprioritized 2026-07-31; Issue #104 |
| Parent Epic | None |
| Depends On | TUI-005, TUI-028, TUI-035, ADR-031, ADR-054; I158 disposition |
| Blocks | I159 activation until TUI-037 receives an explicit disposition |

## Identity / Goal / Value

When the local Dashboard becomes available, show one concise Dashboard line in the display-only Logo region instead of the generic tips row. In ordinary loopback mode, the token-free URL should be clickable where terminal hyperlink support is safe and remain fully copyable everywhere else.

## Priority And Sequencing

The maintainer raised this Story from P2 to P1 on 2026-07-31.

- I158 remains the sole Active implementation iteration; do not stack this UI change onto I158 branches or PRs.
- After I158 reaches Complete or Paused, TUI-037 is the first product item to disposition before I159 may activate.
- TUI-037 remains Refinement while the hyperlink and token-required loopback navigation gates below are unresolved; this document does not authorize implementation.
- The post-I158 inventory must either resolve the gates and select a dedicated iteration, or explicitly record TUI-037 as Blocked or Deferred.
- I159-I162 retain their published baselines.

## Scope

- Replace the successful Dashboard-ready info-tip path with structured, non-secret availability state projected by the Logo-prefix renderer.
- Add exactly one Dashboard line to the display-only Logo prefix.
- In ordinary loopback mode, show the complete token-free URL and optionally encode it as OSC 8 after validation.
- Keep the Logo line out of transcript, session persistence, export, logs, and primary-screen scrollback.
- Preserve normal tips for other information, warnings, errors, queue feedback, and approval results.
- Test wide and narrow layouts, wrapping, Alternate Screen lifecycle, escape safety, and click/plain-text behavior.

## Exclusions

- No Dashboard route, data model, authentication, bind-address, or remote-access change.
- No browser automation or OS-level URL opener.
- No secret value in visible text, hyperlink targets, logs, sessions, or export.
- No I159-I162 implementation, release, tag, publish, or version action.

## Decision Links And Constraints

- ADR-031 keeps Dashboard loopback-bound. When `dashboard.loopback_only = false`, Dashboard is still loopback-bound but requires an additional token; this Story calls that the token-required loopback configuration.
- ADR-054 defines Logo as a display-only virtual history prefix. Dashboard state must share that prefix rather than become a second renderer or transcript fact.
- A public conversation UI enum change requires the normal compatibility and decision record before implementation.

## Display And Hyperlink Policy

### Ordinary loopback configuration

- Show the complete token-free Dashboard URL.
- Emit OSC 8 only after validating that the target contains no userinfo or secret value.
- Without safe OSC 8 support, show the same complete URL as plain text and emit no malformed escape sequence.

### Token-required loopback configuration

- Show the complete token-free base URL plus `authentication required`.
- Keep the row non-clickable under the current unresolved design baseline.
- Do not treat terminal output as a verified token-delivery boundary.
- Never place the token in visible text, OSC 8, logs, transcript, session state, or export.
- Before Ready, document and validate a safe navigation/authentication boundary, or explicitly accept the non-clickable token-free base URL as the complete supported behavior.

### Failure behavior

Dashboard startup failure remains a normal error tip and creates no Logo link.

## Uncertainty And Validation Path

Before Ready:

- confirm the exact crossterm/ratatui OSC 8 capability and sanitization boundary;
- resolve the token-required loopback navigation decision;
- define manual validation for Alacritty, Kitty or WezTerm, macOS Terminal or iTerm2, and tmux.

## State / Status Owners

- Dashboard startup and safe availability state: `talos-cli` / Dashboard boundary.
- Logo-prefix projection and hyperlink rendering: `talos-tui`.
- Remote tracking: Issue #104.
- Priority, scope, acceptance, and status: this document.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #104 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-07-31 |
| Handoff / Release Condition | None |

## User-Facing Documentation

- Update Dashboard and TUI documentation with the final display location and hyperlink fallback.
- Document ordinary and token-required loopback behavior without implying an unsafe token-delivery path.

## Required Reads

- `docs/backlog/active/TUI-005-logo-splash.md`
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
- `docs/backlog/active/TUI-035-narrow-viewport-and-resize-rendering-robustness.md`
- `docs/decisions/031-web-loopback-dashboard-boundary.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- `docs/iterations/I158-tool-registration-composition.md`
- `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
- `docs/tasks/2026-07-28-four-month-v06-execution-package.md`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-tui/src/`

## Acceptance

- Ordinary loopback success renders exactly one Dashboard Logo line and no dashboard-ready generic tip.
- Safe OSC 8 activation opens the exact token-free loopback URL.
- Without safe OSC 8 support, the complete copyable URL remains visible and no malformed escape sequence is emitted.
- Token-required loopback renders the complete token-free base URL plus `authentication required`, remains non-clickable under the current baseline, and exposes no token.
- Narrow layouts do not corrupt Logo, history, composer, or status rows.
- Scrolling keeps the Dashboard row with the Logo prefix and never persists it.
- Startup failure remains an error tip and creates no misleading link.
- Focused/full-frame tests, the real-terminal matrix, and `cargo test --workspace --locked` pass.

## Residuals

- OSC 8 capability, sanitization, and terminal compatibility remain design gates.
- Token-required loopback navigation remains a design gate.
- TUI-037 stays Refinement until both gates are resolved.
- After I158 disposition, TUI-037 must be selected into a dedicated iteration or explicitly recorded Blocked/Deferred before I159 activation.
