# WEB-005 Browser Session Continuity — Design Proposal

Created: 2026-06-30 (T18+T19 of the four-month plan)
Status: Design (pre-ADR); no implementation without browser-connector ADR

## Scope

This proposal formalizes the WEB-005 browser-session continuity permission model (T18), the
`browser_page_read` permission facet (T19), the page record schema, and the no-cookie-leak
security boundary. It is a design artifact: implementation requires a separate browser-connector
ADR before any extension, daemon, or browser automation dependency is added.

Source backlog: `docs/backlog/active/WEB-005-browser-session-continuity-research.md`.

## Permission Model

Browser-page access is a **conditional backend** of `fetch_url`, not a separate tool. The
permission engine sees the real backend risk through facets:

| Facet | Applies to | Risk surface | Default policy |
|---|---|---|---|
| `network_read` | `fetch_url` HTTP backend | Public URL fetch, SSRF-guarded | Allow (read-only) |
| `browser_page_read` | `fetch_url` browser-page backend | Authenticated page text/title/links | Ask (first read per origin) |
| `browser_page_revisit` | `fetch_url` using a prior `BrowserPageRecord` | Reuse of stored page snapshot | Ask (scoped by TTL/origin) |

### Composition with `fetch_url` continuation disclosure (T19)

The TOOL-014 continuation contract governs how the browser-page backend is disclosed:

1. Model calls `fetch_url(url)`.
2. HTTP backend fetches the URL. If the response is a login page, mostly client-rendered, or
   requires authenticated browser context, the HTTP backend returns a `ToolContinuation` that
   discloses the `browser_page` backend.
3. Model calls `fetch_url(url, backend="browser_page")` — this is the point where
   `browser_page_read` permission is evaluated.
4. If permission is `Ask`, the approval handler receives the tool name, arguments (URL), and
   summary fields (origin, title preview). The user approves or denies.
5. On approval, the browser connector reads visible text, title, URL, and selected links.
   A `BrowserPageRecord` is created and stored.
6. The result enters model context as a normal `fetch_url` tool result.

**Key rule**: a continuation disclosure is NOT a permission grant. It only tells the model the
backend exists. Permission is evaluated separately at the point of actual browser-page access.

### Permission composition with multi-facet tools

`fetch_url` is a TOOL-013 multi-resource tool. Its permission profile is evaluated per invocation:

- HTTP fetch → `network_read` facet only.
- Browser-page fetch → `browser_page_read` facet (most restrictive wins if both apply).
- Browser-page revisit → `browser_page_revisit` facet (TTL/origin-scoped).

## BrowserPageRecord Schema

```json
{
  "record_id": "uuid-v4",
  "url": "https://example.com/dashboard",
  "final_url": "https://example.com/dashboard#overview",
  "origin": "example.com",
  "title": "Dashboard — Example",
  "visible_text_excerpt": "Welcome to your dashboard...",
  "selected_links": [
    { "text": "Settings", "url": "https://example.com/settings" }
  ],
  "timestamp": "2026-06-30T12:00:00Z",
  "session_id": "talos-session-uuid",
  "connector_kind": "manual_handoff | extension | mcp_browser",
  "access_mode": "current_tab | revisit",
  "ttl_seconds": 3600,
  "compression_metadata": {
    "original_size_bytes": 45000,
    "stored_size_bytes": 8000,
    "strategy": "visible_text_only"
  }
}
```

### What records MUST NOT store

- Cookies, cookie jars, or session identifiers.
- `localStorage` / `sessionStorage` contents.
- Passwords, tokens, API keys, or hidden form values.
- Full DOM dump (by default).
- Screenshots (by default).
- Browser profile paths or extension identifiers.
- Authentication headers or bearer tokens.

This is a hard security boundary. Any connector that cannot guarantee these exclusions is rejected.

## No-Cookie-Leak Boundary

### Threat model

The primary threat is implicit credential channel: if the agent can access authenticated pages
without explicit per-origin approval, any compromised prompt or tool result could direct the agent
to exfiltrate authenticated content (e.g., "fetch this URL" where the URL is an attacker-controlled
page that reflects session data).

### Mitigations

1. **Per-origin explicit approval.** First browser-page read of each origin requires user
   confirmation. The approval handler sees the origin and page title.
2. **No credential pass-through.** The connector reads visible text only. Cookies, storage, and
   auth headers never enter Talos's process memory or the model context.
3. **Visible-text-only extraction.** The `BrowserPageRecord` stores only what a human could see
   by looking at the rendered page: text, links, title, URL. Hidden elements, data attributes,
   and network requests are excluded.
4. **Revisit scope.** `browser_page_revisit` is scoped by TTL (default 1 hour), origin, and
   session. Records expire automatically.
5. **Audit trail.** Every browser-page access creates a log entry with timestamp, origin, URL,
   access mode, and approval decision.
6. **Connector isolation.** The connector runs in the user's browser context (extension or
   manual handoff), not in Talos's process. Talos receives only the extracted text record.

### What this does NOT prevent

- If the user approves a malicious origin, the agent sees whatever is on that page. This is the
  same risk as any `fetch_url` of a public URL.
- If the browser connector is compromised at the browser level, Talos cannot detect it. This is
  outside Talos's trust boundary.

## Connector Paths (future, ADR-gated)

| Path | Status | ADR required? |
|---|---|---|
| Manual handoff (user copies URL, connector reads tab) | Prototype-ready | No (no new dependency) |
| Talos-owned browser extension | Research | Yes (extension security review) |
| BrowserSkill-compatible external connector | Research | Yes (external dependency review) |
| MCP browser connector | Research | Yes (transport + dependency review) |

No connector is implemented in this plan. T47 (mock backend) may prototype the record flow without
a real connector.

## Implementation Phases (reference only)

Phase 0 (this design): permission model, record schema, security boundary defined.
Phase 1 (future): read-only browser-page backend behind `fetch_url`, manual-handoff connector.
Phase 2 (future): page-record revisit with TTL/origin scoping.
Phase 3 (future): connector prototype after ADR gate.
