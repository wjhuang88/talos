# SEARCH-001-C: Dormant Fixed Compatibility Requests

Private library prerequisite under I293/#712/ADR-090, based on main
2b015a6bba10e646ac4b022514177c4a92ba372c (#736). Implementation:
crates/talos-tools/src/search_backend/transport/compatibility.rs. This constructs the existing
owned PreparedTransportRequest for four upstream formats. It has no production caller, HTTP,
DNS, permission grant, budget debit, router/default or model-facing behavior change.

## Fixed Protocol Fields

| Format | Method and URL | Ordered form/query fields |
|---|---|---|
| Lite | POST https://lite.duckduckgo.com/lite/ | q=query, kl=wt-wt |
| HTML | GET https://html.duckduckgo.com/html/ | q=query |
| Instant Answer | GET https://api.duckduckgo.com/ | q=query, format=json, no_html=1, skip_disambig=1, t=rust-agent-websearch |
| Bing | GET https://cn.bing.com/search | q=query, mkt=zh-CN |

There is no caller URL, header or fixed-field override. The same bounded form serializer is used
for POST bodies and GET query strings; spaces become +, plus becomes %2B, UTF-8 bytes are escaped,
* is literal and ~ becomes %7E. Valid nonblank input retains its original bytes, including edge
whitespace and control characters; trim is only a blank check, matching WebSearchTool's boundary.
Raw input over16384 bytes is rejected before trimming or copying. This is distinct from the
Instant Answer response parser's fallback URL encoder.

Exact expansion is checked before serializer allocation, including all names, fixed values,
equals signs and separators. Entire GET query <=4096 encoded bytes; entire POST body <=16384.
The existing generic form path retains its16384-byte bound. Existing URL/path/method/membership
validation remains authoritative, with no limit increase, truncation or partial preparation.

destination() derives a query-free canonical endpoint from the actual fixed URL and method,
using the transport's existing request_endpoint. It aids trusted-host scope planning and grants
no authority. prepare() validates the actual constructed request against that manifest and binds
its owning, non-Clone payload to the manifest's fresh invocation ID. Existing execute_prepared
consumes it and rejects another invocation before DNS, authority checks or budget use.

Queries do not enter the approval scope, digest or Session grant identity. The same manifest can
prepare multiple different queries. This is not an approval seal, a query fingerprint, a global
single-use guarantee or proof that preparation preceded approval. Future admission must carry
the same owned object, rather than reconstructing from mutable model input. No new public API,
backend identity, dependency, lockfile, permission or execution seam is added.

## Provenance And Distribution

Freshly downloaded rust-websearch0.1.2 archive SHA256
06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee;
VCS b30180a03609580e8b28dfb682284cb2ba249fba, path crates/websearch.
Archived request sources exactly match freshly retrieved immutable pinned files:

| Pinned source | SHA256 |
|---|---|
| [Lite](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/duckduckgo/lite.rs) | 421b65792e4e3d82860c466fb0eebff3d09c7c7992751fd7de055191ff68fcc5 |
| [HTML](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/duckduckgo/html.rs) | dd9dcdd6fc617df1f8638325bbb7edb8c932b1123b7605aed22d1d5a4b2da550 |
| [Instant Answer](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/duckduckgo/instant_answer.rs) | b3aad75760fcad2e079bb9d15ec155e072748482d6ca1906b3ad19e109ca6ced |
| [Bing](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/bing.rs) | 73fe17e6156599bd0e0e109762f88bacf4b47d18024f5d7a3eb7e78dfb3334d0 |

The complete tools-package LICENSES/rust-websearch-MIT.txt matches the pinned root
[MIT license](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/LICENSE),
Copyright(c)2026 Rust-Framework Contributors, with a final LF retained. Source comments record
derivation and Talos modifications. This is source-license evidence, not D service-use, query
privacy, result reuse or native-provider admission. No live search query/response was acquired.

## Validation And Remaining Gates

Eleven synthetic tests cover destinations, fixed ordered bytes, injection resistance, blank and
preserved nonblank input, exact/+1 ASCII and Unicode expansion limits, bounded generated
roundtrips, wrong scheme/host/port/method/path, ownership and scope reuse, redaction and zero
execution effects on preparation/cross-invocation rejection. They inspect prepared bytes without
adding an endpoint override or performing an official-host request. Existing transport fixtures
independently test actual controlled sends; this stage does not claim new full-wire evidence.

Bing Accept-Language, common/random UA and headers, charset behavior, proxy/source snapshots,
probes/retries, parser-response binding, result admission, full service/protocol parity and
D/privacy remain pending. These factories are insufficient for adapter activation. Official
CLI/SDK/MCP/standalone roots, deadline/cancellation/router/rollback/default activation and release
retain their gates. Response prerequisites: [DDG](SEARCH-001-C-DDG-RESPONSE-PARSER.md) and
[Bing](SEARCH-001-C-BING-RESPONSE-PARSER.md). I293 stays Active/Claimed, Completion Pending;
#643/#624 stay open. #705/#717/#735 retain independent ownership.
