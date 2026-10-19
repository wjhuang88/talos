# SEARCH-001-C: Dormant Bing Compatibility Response Parser

Private library prerequisite under I293/#712/ADR-090, based on #734/main45cf9eb9.
Implementation: crates/talos-tools/src/search_backend/bing_parser.rs. It rehosts only the existing
compound compatibility family's Bing CN response parsing. No HTTP, request factory, header,
retry, new backend/provider, model contract, API/dependency/lock/permission or default change.
Only Talos-authored synthetic HTML is used; no live search query or response was acquired.

## Provenance And Distribution

Before deriving code, freshly downloaded rust-websearch0.1.2 and verified archive SHA256
`06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee`.
Its VCS metadata pins `b30180a03609580e8b28dfb682284cb2ba249fba`, path crates/websearch.
Archived src/bing.rs exactly matches the freshly retrieved immutable
[Bing source](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/bing.rs),
SHA256 `73fe17e6156599bd0e0e109762f88bacf4b47d18024f5d7a3eb7e78dfb3334d0`.
The existing network-free html_utils::clean_html is reused, not copied; archived helper SHA256
is `94e06286456997e9c60cf1139c1b38133d05339502d97904dafa01bb41e0f759`.

The complete retained crates/talos-tools/LICENSES/rust-websearch-MIT.txt matches the freshly
retrieved pinned root [MIT license](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/LICENSE),
Copyright(c)2026 Rust-Framework Contributors, except its final newline. The derived source records
provenance and modifications; the notice remains inside the tools package boundary. Source
licensing does not admit provider use, query privacy or result reuse; D review remains pending.

## Compatibility Contract

Select li.b_algo rows in document order. Each row uses only its first h2 a and first .b_caption p.
Missing links are skipped; missing href becomes empty. Trim the href, join title text nodes with
spaces and call the existing clean_html before rejecting empty URL/lowercase javascript:.
Only a remaining row reads/cleans its snippet, then deduplicates by exact trimmed URL string.
Missing snippets become empty; empty cleaned titles use the trimmed URL. Stop after max accepted
rows. No second-link recovery, extra-snippet fallback, case/slash/fragment normalization or
DDG redirect unwrapping is added. Oversized duplicate title/snippet still rejects the response;
max can stop processing before a later bad row. Limits/errors never return partial candidates.

The existing WebSearch compatibility normalizer assigns the compound family's DuckDuckGo ID to
results even when its opaque upstream chose Bing CN. This parser retains that private family ID;
it is not provider attribution, a new native route or Bing admission. Relative/ftp/malformed URLs,
`https:`, protocol-relative URLs and uppercase Javascript: may remain untrusted candidates.
Only literal lowercase javascript: and empty strings are skipped here, matching the pinned source.
HTTP/HTTPS result admission is a separate required stage before winner selection or fetch/render.

HTTP429 and503 map to closed RateLimited, all other non200 statuses to Status. For200, the entire
raw body is checked for case-sensitive g-recaptcha, cf-challenge, verifyContainer and bnp_dialog.
Markers in scripts, comments or ordinary text still classify Challenge; these compatibility
false positives are preserved, not called comprehensive validity/bot detection. RateLimited
grants no retry permission. Challenge/error classification does not activate another destination.

## Bounds And Migration Differences

| Boundary | Contract | Limitation |
|---|---|---|
| Input/count | <=1048576 body bytes, requested count1..20, checked first | New limits precede status classification; not streaming or a hard memory bound |
| Encoding | Strict UTF-8, reject leading BOM | Upstream response.text charset/BOM/replacement parity remains unproven |
| DOM-decoded href | <=16384 bytes before trim | Long surrounding whitespace can fail even when the trimmed URL would fit |
| Text/fields | <=16384 UTF-8 bytes for joined text, helper output and emitted fields | Checked fragment+separator length before append; not silent truncation |
| Selection | First link/snippet, stop at max | Later fields can be skipped; the whole bounded DOM is still parsed |
| Failure/data | Closed content-free errors, redacted BingCandidates Debug | Raw candidate consumption is explicit; downstream logging/rendering remains separate |
| Dependency unwind | catch_unwind around DOM/helper | Not OOM/abort/stack-overflow recovery, a CPU deadline or panic-hook suppression |

Selectors iterate without collecting additional row/link/snippet vectors. scraper still allocates
the full DOM. Talos uses existing scraper0.27 while upstream used0.22; malformed DOM normalization,
charset/status-read-order and resource behavior are not established by these bounded fixtures.
No dependency/version alignment or new suppression is introduced.

Fourteen synthetic tests cover row/first-element selection, empty/missing fields, exact trim
dedup, title-before-skip/snippet-before-dedup, unsafe candidates, all statuses/markers, malformed
and decoy input, exact/overflow body/fields, UTF-8/separator byte lengths, helper order and
redaction. Bounded generated Unicode/count cases do not claim proptest or upstream differential
execution. I293 and the candidate's exact-head CI/review record actual validation.

## Remaining Integration Gates

This is dormant under the existing seam's allowance and has no production caller. DDG parsers
are documented [separately](SEARCH-001-C-DDG-RESPONSE-PARSER.md). Requests/headers/UA, probes,
retry/source/proxy snapshots, optional adapters, full protocol/service/parser parity, D/privacy,
result admission, official CLI/SDK/MCP/standalone roots, router/deadline/rollback/default activation
and release remain gated. #643/#624 stay open; #705/#717/#735 retain independent ownership.
