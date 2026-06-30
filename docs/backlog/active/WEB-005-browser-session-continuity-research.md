# WEB-005: Browser Session Continuity And Page Access Research

| Field | Value |
|-------|-------|
| Story ID | WEB-005 |
| Priority | P2 |
| Status | Research |
| Depends On | WEBFETCH-001; TOOL-012; TOOL-013 |
| Relates To | WEB-001; REMOTE-001; PLUGIN-001 |
| Blocks | Browser session continuity decision; authenticated-page access model; page access record design |
| Origin | User request 2026-06-30; evaluate Tencent BrowserSkill for logged-in browser state and page access record patterns |

## Problem

Talos can fetch public URLs and extract local documents, but authenticated web workflows raise a
different product and security question: whether an agent should be able to work with pages that
the user can already access in their normal browser, without copying cookies, exporting secrets, or
turning browser automation into an implicit credential channel.

Tencent BrowserSkill is a relevant reference because it explicitly centers on the user's existing
browser context: logged-in sessions, cookies, extensions, isolated agent windows that share login
state, explicit existing-tab handoff, and remembered page/tab state. Talos needs a source-grounded
research pass before deciding whether any of those patterns belong in WEBFETCH, WEB-001, plugin
runtime work, or a separate browser-integration track.

## Scope

Research and document borrowable patterns from `github.com/Tencent/BrowserSkill`:

- Use existing browser login state without exporting or persisting cookies in Talos.
- Isolated agent window/session that shares browser auth state while avoiding disruption of the
  user's active windows.
- Explicit user-mediated handoff for an existing tab/page and take-over of a user-opened page.
- Page access records: URL, tab identity, navigation state, title, timestamp, and revisit behavior.
- Human-in-loop handling for login, captcha, 2FA, checkout, destructive actions, and confirmation
  pages.
- Permission gates and auditability for browser read, click, form-fill, download, and file-upload
  actions.

## Non-goals

- No implementation in this story.
- No dependency on BrowserSkill, MCP browser automation, Playwright, Puppeteer, or a browser
  extension before a follow-up ADR/story.
- No captcha bypass, credential extraction, cookie import/export, or hidden browser profile access.
- No change to WEBFETCH-001's current non-browser fetch/document extraction boundary.
- No remote browser control or public network control surface.

## Acceptance

- [ ] Produce a research note comparing BrowserSkill's browser-session model with Talos'
      WEBFETCH-001, WEB-001, TOOL-012, and TOOL-013 boundaries.
- [ ] Identify which ideas are reusable, which should be rejected, and which need an ADR before
      implementation.
- [ ] Define a candidate security model for authenticated browser access, including session-state
      ownership, page access records, audit logs, and user confirmation points.
- [ ] Decide whether follow-up work should be rejected, monitor-only, prototype-only, or promoted
      to an implementation story.
- [ ] If accepted as an ongoing reference, update `docs/reference/REFERENCE-PROJECTS.md`.

## Initial Source Notes

Source: <https://github.com/Tencent/BrowserSkill>

- BrowserSkill positions itself as a browser-use skill that talks directly to the user's existing
  browser rather than a separate browser process or extra API server.
- It advertises use of the real browser context: logged-in sessions, cookies, extensions, and
  normal browser state.
- It describes an isolated agent window that avoids disturbing normal browsing while sharing login
  state.
- It supports connecting to an existing tab and taking over a page the user opened manually.
- It records opened webpages and their exact state so later requests can revisit the right tab.
- It expects human help for login, captcha, 2FA, and sensitive confirmation flows.

## Required Reads

- <https://github.com/Tencent/BrowserSkill>
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/reference/REFERENCE-PROJECTS.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/010-rust-first-dependency-policy.md`
