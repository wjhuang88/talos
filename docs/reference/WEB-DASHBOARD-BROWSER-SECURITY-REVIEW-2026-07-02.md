# WEB Dashboard And Browser Boundary Security Review

**Status**: Review complete for I077/T112; T113 hardening fixes implemented
**Date**: 2026-07-02
**Scope**: WEB-001 read-only loopback dashboard and WEB-005 browser-page record mock boundary
**Related**: ADR-031, WEB-001, WEB-005, TOOL-014, I077/T112-T113

## Authorized Scope

This review keeps the existing product boundary:

- dashboard remains loopback-only, token-authenticated, read-only, and default-on only in TUI mode
  with `[dashboard] enabled = false` opt-out per the 2026-07-02 ADR-031 amendment;
- browser-page support remains mock/record-only, with no real connector, cookies, storage, DOM,
  screenshot, profile path, click, fill, upload, download, or automation support;
- `fetch_url` remains the unified model-facing web/page ingestion entry.

This review does not authorize remote dashboard access, web approvals, config writes, browser
automation, browser connectors, standalone browser tools, or permission-default changes beyond the
default-on local dashboard lifecycle amendment recorded in ADR-031.

## Findings

| Finding | Severity | Assessment | Resolution |
|---|---|---|---|
| Dashboard snapshot surfaces trusted upstream masking. | Medium | `/history`, `/status`, `/governance`, and `/config` served caller-provided snapshot data. Config tests covered a masked fixture, but the dashboard boundary itself did not recursively redact secret-like JSON keys or query-string assignments. | T113 added boundary redaction for JSON values and text snapshots, with regression coverage for API keys, bearer headers, cookies, and token query parameters. |
| Unknown dashboard paths without tokens were untested. | Low | Known routes required tokens. Unknown paths with valid tokens returned `404`, but no regression proved unauthenticated unknown paths could not bypass the middleware shape. | T113 added an authenticated fallback and regression proving unknown paths without a token return `401`. |
| Browser page selected link URLs were not normalized through the record sanitizer. | Medium | `BrowserPageRecord::new_mock` sanitized record URLs, but `with_links` accepted caller-provided link URLs directly. A selected link could carry credentials or token query parameters in the model-facing record. | T113 now sanitizes selected link URLs and tests userinfo/query-token redaction. |

## Residuals

- Dashboard redaction is a defensive display boundary, not a substitute for upstream masking.
- Browser-page support remains mock-only. A real connector still needs a connector-specific ADR.
- No `browser_page_read` permission facet is implemented in this review.
- No web write/action route is approved.

## Validation Evidence

- `cargo test -p talos-dashboard`
- `cargo test -p talos-tools browser_page`
- `cargo test -p talos-tools fetch_url`
- `cargo clippy -p talos-dashboard -p talos-tools -- -D warnings`
