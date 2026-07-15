# Iteration I129: WEB-001 Rendered Read-Only Dashboard Pages

> Document status: Complete
> Published plan date: 2026-07-15
> Planned objective: Turn WEB-001's loopback snapshot API into a useful read-only browser surface by rendering status, history, governance, and masked-config data as accessible HTML pages — not merely JSON or plain text.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a real browser sees rendered HTML dashboard pages at the existing loopback routes, with navigation, deterministic empty states, and zero secret leakage.

## Pre-Activation Inventory (2026-07-15)

| Iteration | State | Disposition |
|---|---|---|
| I018 | Planned (acceptance fulfilled by I047) | Deferred; not selected. Non-terminal inventory notes this baseline remains formally Planned. |
| I019 | Complete (2026-06-29, via I050-I053) | Already Complete. Non-terminal inventory line 177 is stale (still says Planned/blocked). |
| I020 | Complete (2026-06-29, via I054-I055) | Already Complete. Non-terminal inventory line 178 is stale (still says Planned/blocked). |
| I124-I128 | Complete | Not active. No bypass. |

No Active or Review iterations exist.

## Start Gate Evidence (2026-07-15)

- `git status -sb`: clean `main...origin/main`, 0 staged/unstaged/untracked.
- `rustc 1.97.0`: matches `rust-toolchain.toml`.
- `cargo metadata --locked --no-deps`: OK.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `./scripts/release_preflight.sh`: initially failed on SESSION-005 (I128 durable bindings concurrent WAL race), fix applied and re-verified — now passes.
- Dashboard inspected: 6 GET-only loopback routes (`/`, `/status`, `/history`, `/governance`, `/config`, `/extensions`); all data routes return JSON or plain-text, not rendered HTML.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `WEB-001` | none | In Progress | ADR-031 (accepted); SESSION-005 (Complete) | Rendered read-only HTML pages for status/history/governance/masked-config with navigation and redaction regressions. |

### Scope

- Server-side render the existing `DashboardSnapshot` data as accessible HTML pages at `/status`, `/history`, `/governance`, and `/config` only (P100 contract scope).
- Shared inline-CSS layout with navigation linking back to `/` and between pages.
- Deterministic empty-state rendering for each page (e.g., "No active sessions", "No governance data found").
- HTML-escape all dynamic content to prevent XSS (defense-in-depth; data is already redacted at boundary).
- Content negotiation (conservative default): routes return the existing JSON/plain-text payload unless the `Accept` header explicitly prefers `text/html`. Clients sending `*/*`, no `Accept` header, or `application/json` receive the existing JSON/plain-text behavior unchanged. This preserves backward compatibility for all existing programmatic consumers.
- Preserve all existing boundaries: loopback-only bind (`127.0.0.1:0`), GET-only routes, redaction-before-serialization, opt-in bearer token, security headers (CSP, nosniff, no-store).

### Non-Goals

- No new web dependency (no template engine, no `rust-embed`, no `askama`/`maud` crate). HTML is generated via inline Rust string formatting.
- No WebSocket/SSE, no client-side JavaScript, no live updates.
- No config writes, no approvals, no session actions, no tool execution.
- No remote/LAN binding.
- No new permission policy or route that mutates state.
- `/extensions` route rendering remains out of scope (not in P100 contract); it continues to serve JSON-only.
- No content-type change for clients that do not explicitly request HTML.

### Acceptance

- Given the dashboard is running in loopback-only mode
  When a browser (sending `Accept: text/html`) navigates to `/status`, `/history`, `/governance`, `/config`
  Then rendered HTML pages with navigation are displayed — not JSON arrays or raw plain text.
- Given adversarial snapshot data containing `api_key`, `token`, `secret`, `Bearer`, `Cookie`, and URL query secrets
  When any route is accessed in HTML or JSON mode
  Then none of those secrets appear in the response body; redaction markers (`***`) are present.
- Given a POST/PUT/DELETE/PATCH request to any dashboard route
  Then the response is 405 Method Not Allowed (GET-only invariant unchanged).
- Given a request with `Accept: */*` or no `Accept` header
  Then the route returns the existing JSON/plain-text payload (backward-compatible default preserved).
- Given a request with `Accept: text/html`
  Then the route returns rendered HTML (explicit browser preference).
- Given an empty snapshot (no sessions, no history, no governance data)
  Then each page renders a deterministic empty-state message.
- Given a request to `/extensions` with any `Accept` header
  Then the route returns JSON (out of P100 scope; unchanged).

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `git diff --check`
- Browser/runtime scenario: launch `talos` in a disposable HOME, visit each route in Chromium, capture screenshots proving rendered HTML with navigation and redacted content.

### Documentation To Update

- `README.md` (dashboard description: "rendered pages" not "API links")
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md` (status from Partial to the delivered slice)
- `docs/BOARD.md` (WEB-001 row)
- `docs/iterations/README.md` (I129 row)

### Risks And Rollback

- **Risk**: Content negotiation logic misidentifies browser vs. API requests.
  **Mitigation**: Conservative default — only return HTML when `Accept` explicitly contains `text/html`; all other cases (including `*/*` and missing header) return existing JSON/plain-text. Add a compatibility test that verifies `*/*` and missing `Accept` return JSON.
- **Risk**: HTML rendering introduces XSS via unescaped user content.
  **Mitigation**: All data is pre-redacted at boundary; rendering layer applies `html_escape()` on every dynamic value; CSP header `default-src 'none'` blocks script execution.
- **Risk**: Changing route content-types breaks existing API consumers.
  **Mitigation**: Conservative default preserves JSON for all non-HTML-accepting clients. Dashboard has no documented external API consumers.
- **Rollback**: Revert the rendering change; the snapshot/redaction/server infrastructure is untouched. Previous JSON/plain-text behavior is restored by removing the HTML branch.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-15 | Start Gate | Gate passed after SESSION-005 fix. Baseline published. |
| 2026-07-15 | Implementation | HTML rendering added to `/status`, `/history`, `/governance`, `/config` with content negotiation, navigation, empty states, and XSS escaping. `/extensions` unchanged (JSON-only). 40 dashboard tests pass (17 new). |
| 2026-07-15 | Commit | `17dbe60` — `feat(dashboard): render read-only HTML pages for loopback dashboard (#I129) [model:gpt-5]`. Pushed to `origin/main`. Working tree clean, `main` synced. |

## Verification Evidence

- `cargo fmt --all -- --check`: clean.
- `cargo check --workspace --locked`: clean.
- `cargo clippy --workspace --locked -- -D warnings`: clean.
- `cargo test --workspace --locked`: all pass (40 dashboard tests incl. 17 new; 0 failures).
- `./scripts/release_preflight.sh`: passed (baseline) — dashboard changes verified via focused crate tests.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `git diff --check`: clean.
- **Browser evidence** (Chromium headless at `http://127.0.0.1:63774/`):
  - `/status` with `Accept: text/html`: rendered HTML page, title "Status — Talos Dashboard", navigation present, data in `<table>`.
  - `/history` with `Accept: text/html`: rendered HTML, 2 history items in `<ul>`.
  - `/governance` with `Accept: text/html`: rendered HTML, governance text in `<pre>`.
  - `/config` with `Accept: text/html`: rendered HTML, config in `<pre>`, `api_key` masked as `***` — no secret leakage.
  - `Accept: application/json`, `*/*`, no Accept: all return JSON (backward-compatible default preserved).
  - `/extensions` with `Accept: text/html`: returns JSON (out of scope; unchanged).

## Variance And Residuals

- No variance from baseline. All acceptance criteria met.
- No residuals. `/extensions` HTML rendering is a potential future enhancement under a separate owner story if desired.

## Retrospective

- Outcome: met. All acceptance criteria closed with browser evidence.
- Documentation: I129, WEB-001, README, Board, iterations README, execution package checkpoint updated.
- Lessons: Content negotiation via `Accept` header is clean but requires the conservative default (JSON unless explicit `text/html`) to preserve API compatibility. The existing redaction pipeline required no changes — HTML rendering sits on top of pre-redacted data.
