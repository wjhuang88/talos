# SEARCH-001-C: Dormant Compatibility Result Admission

Private I293/#712/ADR-085/090 prerequisite, based on main91c330ba (#739). The consuming fixed
compatibility path now admits display results after controlled transport and parsing. No official
root, router, provider, permission, public API/model contract, dependency or default behavior changes.

## Closed Result Boundary

AdmittedCompatibilityResults has private fields, no unchecked constructor/mutator/Clone and a
fixed redacted Debug. Consumption returns existing normalized fields unchanged. The only production
constructor validates already-materialized parser output; successful HTTP/parsing alone is not
enough. Empty and entirely URL-invalid results cannot produce the wrapper.

Captured protocol determines expected identity. Lite/HTML/Instant Answer/Bing remain the existing
compound DuckDuckGo family. Bing is not reattributed as a newly admitted native provider. Every
candidate's identity and field bounds are checked before selecting any output, including trailing
entries after enough valid URLs exist. A trailing identity/field overflow rejects the whole batch.

| Internal candidate bound | Value | Meaning |
|---|---|---|
| Requested count | 1..20 | Validated before request selection and at result admission |
| Parser window / input batch | <=20 candidates | Reuses parser cap; no scan beyond that window |
| Title / snippet | <=16384 UTF-8 bytes each | Whole-batch validation, not upstream DOM allocation cap |
| URL | <=8192 UTF-8 bytes | Checked before parsing; oversize rejects the batch |
| Output | 1..requested count | Invalid URLs filtered before truncation, order retained |

Compatibility parsers now receive the bounded20-candidate window rather than the smaller desired
output count. Admission then applies the captured count. This may inspect additional fields and
reject an oversized trailing entry that the earlier smaller parser window would not inspect. A
valid result beyond20 remains undiscovered; this is not full-response parity or unlimited scanning.
The parser-only modules still return untrusted candidates and retain their existing bounds.

## URL Meaning

Existing reqwest::Url parsing must yield HTTP/HTTPS with a host, empty parsed username and no password.
Malformed/relative/unsupported/credential-bearing URLs are filtered; mixed responses retain only
usable entries. No extra deduplication, title rewriting, canonical display replacement or source
combination occurs. All original title/URL/snippet fields remain inert untrusted display text.

This deliberately uses WHATWG parsing, not strict raw-syntax validation: repaired strings such as
https:example.invalid/path, extra slashes and embedded newline may parse. Original text is retained.
An empty userinfo marker may be discarded by parsing. Loopback/private result URLs may be displayed.
This grants no permission to fetch, render HTML,
follow a result/redirect or bypass the separate network/SSRF boundary. No validation fetch occurs.

URL parsing is guarded by catch_unwind and failures use closed, content-free error enums. That
does not suppress panic hooks or recover abort/OOM. Result Debug and errors carry no query, URL,
body, credentials or field content. The wrapper is a local data invariant, not an execution
authorization, provider-use approval, cryptographic seal or complete router-winner contract.

## One Budget And Evidence

The same shared transport budget is checked before parsing, unconditionally after parsing, during
whole-batch checks, before/after URL parsing and at completion. A final post-admission check runs
on success and error, so expired work cannot return a partial winner or replace deadline with an
admission error. No budget reset/new timer, synchronous preemption or caller cancellation proof.

Synthetic unit fixtures cover empty/invalid/mixed results, credentials, parsed/display semantics,
identity/order/fields/redaction, exact/overflow limits and partial-result deadline rejection.
Controlled four-format HTTP fixtures exercise unsafe-leading mixed results with requestedcount1
and entirely invalid responses through the real prepared request, captured parser and shared
budget. Existing cross-invocation/authority/redirect/encoding/parser-error fixtures remain required.
Actual local/native validation and protected review receipts are recorded in I293 and #643.
The new admission code is Talos-authored; existing parser/full MIT provenance is unchanged.

Source/header/request/charset/probe/retry/entropy timing parity, optional adapters, D/privacy/result
use, official CLI/SDK/MCP/standalone roots, router/health/cancellation/rollback/default activation
and release retain their gates. I293 remains Active/Claimed, Completion Pending; #643/#624 stay open.
