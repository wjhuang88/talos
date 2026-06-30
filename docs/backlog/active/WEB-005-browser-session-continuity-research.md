# WEB-005: Browser Session Continuity Through Unified Web Fetch

| Field | Value |
|-------|-------|
| Story ID | WEB-005 |
| Priority | P2 |
| Status | Planned |
| Depends On | WEBFETCH-001; TOOL-012; TOOL-013; TOOL-014 |
| Relates To | WEB-001; REMOTE-001; PLUGIN-001 |
| Blocks | Authenticated-page context ingestion; browser page access record design; browser connector ADR |
| Origin | User request 2026-06-30; evaluate Tencent BrowserSkill for logged-in browser state and page access record patterns; refined 2026-06-30 into unified `fetch_url` backend design |

## Outcome

Talos can read user-approved authenticated browser page context through the existing web/document
tool family without exposing a separate browser subsystem to the model or user.

The model-facing surface should stay small:

- `fetch_url` reads URL/API/page/document context, including conditionally authorized browser-page
  context.
- `save_url` explicitly downloads remote content to a file and remains separate because it writes.
- `http_request` remains an advanced conditional tool for low-level HTTP/API work.

Browser-page access is a conditional backend of `fetch_url`, not a default sibling tool such as
`capture_page` or `read_browser_page`.

## Problem

Talos can fetch public URLs and extract local documents, but authenticated web workflows raise a
different product and security question: whether an agent should be able to work with pages that
the user can already access in their normal browser without copying cookies, exporting secrets, or
turning browser automation into an implicit credential channel.

Tencent BrowserSkill is a relevant reference because it centers on the user's existing browser
context: logged-in sessions, cookies, extensions, isolated agent windows that share login state,
explicit existing-tab handoff, and remembered page/tab state. The borrowable idea is not a new
top-level browser command surface; it is a safe browser-page backend behind the existing context
ingestion workflow.

## Product Model

User intent should remain natural:

```text
Summarize this page.
Look at the current page in my browser.
Continue from the dashboard page I showed you earlier.
Fetch this URL; if it needs my logged-in browser, use that path.
```

The agent should normally start with `fetch_url`. If static HTTP is enough, no browser capability is
presented. If the page needs login state, is mostly client-rendered, or the user explicitly refers to
the current browser page, `fetch_url` returns a structured continuation that asks the runtime to
disclose the browser-page backend for the next step.

## Architecture

```text
fetch_url (model-visible unified entry)
  |
  +-- http backend
  |     public pages, APIs, redirects, HTML extraction
  |
  +-- document backend
  |     local/downloaded text-like extraction; heavy formats remain gated
  |
  +-- browser_page backend (conditional)
        user-approved browser context, visible text, title, URL, selected links,
        and BrowserPageRecord creation
```

The browser-page backend should be implemented behind connector traits, so the core workflow is not
tied to BrowserSkill, MCP, Playwright, Puppeteer, or a specific browser extension.

Candidate connector paths:

- Talos-owned browser extension connector.
- BrowserSkill-compatible external connector.
- MCP/browser connector.
- Manual handoff fallback for early prototypes.

Any connector that introduces an extension, external daemon, or browser automation dependency needs
an ADR before implementation.

## Browser Page Records

`fetch_url` with the browser-page backend may create a `BrowserPageRecord`.

Records may store:

- record id;
- URL and final URL;
- origin;
- title;
- visible text/link snapshot approved for model context;
- timestamp and session id;
- connector kind;
- whether the page came from explicit current-tab handoff or revisit.

Records must not store:

- cookies;
- localStorage/sessionStorage;
- passwords, tokens, or hidden form values;
- full DOM by default;
- screenshots by default;
- browser profile paths.

## Permission Boundary

The model-facing tool can stay `fetch_url`, but the permission engine must see the real backend
risk.

Suggested facets:

- `network_read` for ordinary HTTP fetches.
- `browser_page_read` for visible browser page text/link/title/URL reads.
- `browser_page_revisit` for using a prior `BrowserPageRecord`.

Future facets such as browser click, form fill, upload, download, checkout, or destructive action
are out of scope and require ADR-gated stories.

Rules:

- Static `fetch_url` permission does not authorize browser-page reads.
- A browser-page continuation is not a permission grant.
- First read of an origin/tab/page record requires explicit user or browser-side confirmation.
- Revisit may be allowed only within a bounded TTL/origin/scope policy decided by the implementation
  story.
- Browser-page access must be auditable.

## Implementation Phases

### Phase 0: Design and ADR Gate

- Finalize `TOOL-014` continuation/backend-disclosure contract.
- Write a browser connector ADR if implementation will use an extension, external daemon, or
  BrowserSkill-compatible bridge.
- Decide record storage location and retention policy.

### Phase 1: Read-Only Browser Page Backend

- Add core browser-page types and connector traits in the appropriate crate boundary.
- Add `fetch_url` target/access schema for browser-page reads behind conditional disclosure.
- Return title, URL, visible text excerpt, selected links, and record id.
- Add permission profile for `browser_page_read`.

### Phase 2: Page Record Revisit

- Add page-record lookup and revisit flow through `fetch_url`.
- Enforce TTL/origin/scope restrictions.
- Add audit entries and tests for denied/expired records.

### Phase 3: Connector Prototype

- Implement one connector path selected by ADR.
- Keep automation read-only: no click, fill, upload, download, or submit.
- Document fallback behavior when no browser connector is available.

## Non-goals

- No standalone `/browser` user workflow as the primary path.
- No default top-level `capture_page` or `read_browser_page` tool.
- No dependency on BrowserSkill, MCP browser automation, Playwright, Puppeteer, or a browser
  extension before ADR approval.
- No captcha bypass, credential extraction, cookie import/export, hidden browser profile access, or
  localStorage export.
- No browser clicks, form filling, uploads, downloads, checkout, or destructive actions.
- No change to `save_url` as the explicit write-capable remote download tool.

## Acceptance

- [ ] `WEB-005` is implemented through `fetch_url` as a conditional backend, not a separate browser
      tool group.
- [ ] Browser-page access is disclosed only through TOOL-014 continuation/backend policy or strong
      user intent.
- [ ] Static HTTP fetch and browser-page read permissions remain separate.
- [ ] BrowserPageRecord stores only approved metadata/text snapshots and never cookies, storage, or
      credentials.
- [ ] BrowserSkill patterns are documented as reference input, with explicit accept/reject/defer
      decisions.
- [ ] A follow-up ADR exists before any extension, external daemon, or browser automation connector
      is implemented.
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
- `docs/backlog/active/TOOL-014-conditional-tool-backends.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/reference/REFERENCE-PROJECTS.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/010-rust-first-dependency-policy.md`
