# TUI-037: Dashboard Link In Logo Region

| Field | Value |
| --- | --- |
| Story ID | TUI-037 |
| Type | Product / rendering story |
| Priority | P1 |
| Status | Ready — selected for I202; proposed claim PR #229 pending target-branch merge |
| Source | Maintainer request 2026-07-27; reprioritized 2026-07-31; Issue #104 |
| Parent Epic | None |
| Depends On | TUI-005, TUI-028, TUI-035, ADR-031, ADR-054; I158 Complete |
| Blocks | I159 activation until I202 reaches a terminal disposition |

## Identity / Goal / Value

When the local Dashboard becomes available, show one concise Dashboard line in the display-only Logo region instead of the generic tips row. In ordinary loopback mode, the token-free URL should be clickable where terminal hyperlink support is safe and remain fully copyable everywhere else.

## Priority And Sequencing

The maintainer raised this Story from P2 to P1 on 2026-07-31 and explicitly selected it for complete
implementation and closeout before the mainline long task on 2026-08-14.

- I158 is Complete; TUI-037 is selected into the dedicated I202 iteration.
- I202 is governance-only until its proposed Collaboration Claim reaches `main`; no implementation
  branch or production edit is authorized before then.
- I188 remains independently in Review through PR #228, awaiting its mandatory security review.
  I202 does not modify or complete TOOL-024 scope.
- I159-I162 retain their published baselines.

## Scope

- Replace the successful Dashboard-ready info-tip path with structured, non-secret availability state projected by the Logo-prefix renderer.
- Add exactly one Dashboard line to the display-only Logo prefix.
- In ordinary loopback mode, show the complete token-free URL as copyable plain text.
- Remove the existing token-bearing Dashboard startup log field.
- Keep the Logo line out of transcript, session persistence, export, logs, and primary-screen scrollback.
- Preserve normal tips for other information, warnings, errors, queue feedback, and approval results.
- Test wide and narrow layouts, wrapping, Alternate Screen lifecycle, escape safety, and plain-text behavior.

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
- I202 emits no OSC 8 sequence. The current ratatui/crossterm full-frame buffer has no safe,
  width-correct hyperlink-cell representation; injecting control sequences into cell symbols would
  corrupt width/diff/selection semantics.
- Show the complete URL as plain text and emit no escape sequence. A later clickable-link proposal
  requires a separate owner after the renderer gains a safe hyperlink primitive.

### Token-required loopback configuration

- Show the complete token-free base URL plus `authentication required`.
- Keep the row non-clickable.
- Do not treat terminal output as a verified token-delivery boundary.
- Never place the token in visible text, OSC 8, logs, transcript, session state, or export.
- The non-clickable token-free base URL is the complete supported I202 behavior.

### Failure behavior

Dashboard startup failure remains a normal error tip and creates no Dashboard Logo line.

## Resolved Design Gates And Validation Path

- Ratatui 0.30's buffer/backend path does not model hyperlink metadata. The known raw OSC 8
  cell-symbol workaround mixes control bytes with display cells and is rejected for I202.
- Both ordinary and token-required configurations therefore render complete copyable plain text;
  token-required adds `authentication required` and remains non-clickable.
- Manual validation covers rebuilt Talos in Alacritty and macOS Terminal, plus a tmux pass when tmux
  is available. The matrix checks wrapping, scrolling, selection/copy, lifecycle restoration and the
  absence of raw escape sequences or token disclosure.

## State / Status Owners

- Dashboard startup and safe availability state: `talos-cli` / Dashboard boundary.
- Logo-prefix projection and hyperlink rendering: `talos-tui`.
- Remote tracking: Issue #104.
- Priority, scope, acceptance, and status: this document.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex — mainline session 2026-08-14 |
| Work Slice | Implement only TUI-037 / I202: replace successful Dashboard-ready Tips with one display-only Logo-prefix line; render complete token-free plain-text URLs for ordinary and token-required loopback modes, add `authentication required` for the latter, remove token-bearing startup logging, preserve failure Tips, and prove no transcript/session/export/log/primary-screen persistence. No OSC 8, Dashboard route/auth/bind, persistence, conversation protocol, Desktop or I159-I162 behavior change. |
| Claimed At | 2026-08-14 |
| Source Issue | #104 |
| Governance Claim PR | #229 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent natural-person exact-head security review is mandatory before claim merge and implementation merge because the slice removes an existing bearer-token logging path and changes terminal rendering. This proposed ownership remains ineffective until the finalized claim reaches `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Pass both governance validators and exact-head CI/independent review/CAS on PR #229, then merge it to `main` before creating an implementation branch. |

The `Claimed` record above is proposed by PR #229 and remains ineffective until that exact record
reaches `main`. No I202 implementation branch or production edit is authorized before then.

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
- Ordinary loopback renders the complete copyable token-free URL and emits no OSC 8 or other raw
  terminal escape sequence under the I202 baseline.
- Token-required loopback renders the complete token-free base URL plus `authentication required`, remains non-clickable under the current baseline, and exposes no token.
- Narrow layouts do not corrupt Logo, history, composer, or status rows.
- Scrolling keeps the Dashboard row with the Logo prefix and never persists it.
- Startup failure remains an error tip and creates no misleading link.
- Focused CLI/TUI and full-frame tests, the real-terminal matrix, and
  `cargo test --workspace --locked` pass.

## Residuals

- Clickable OSC 8 rendering is excluded from I202 because the current buffer has no safe hyperlink
  primitive. Any later attempt needs a separate owner, strict target validation and terminal matrix.
- No token delivery/navigation mechanism is introduced; token-required mode intentionally remains a
  non-clickable discovery notice.
