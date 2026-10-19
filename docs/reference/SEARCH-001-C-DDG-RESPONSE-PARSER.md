# SEARCH-001-C: Dormant DDG Response Parser

Private library prerequisite under effective I293/#712 and ADR-090, based on #732/main8fa2a50.
Implementations: crates/talos-tools/src/search_backend/{ddg_parser,instant_answer}.rs. No default WebSearch route,
request factory, public API, model contract, dependency/lock change, provider admission or release.
Only synthetic HTML/JSON fixtures are used; no live provider query or search response was acquired.

## Provenance And Distribution

rust-websearch0.1.2 archive SHA256:
`06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee`.
Fresh download verified before copying. Its VCS metadata pins
`b30180a03609580e8b28dfb682284cb2ba249fba`, path crates/websearch. The following archived files
match the retrieved immutable GitHub source byte-for-byte:

| Upstream path under crates/websearch | SHA256 | Use / modification |
|---|---|---|
| src/duckduckgo/lite.rs | 421b65792e4e3d82860c466fb0eebff3d09c7c7992751fd7de055191ff68fcc5 | Derived selector/text/snippet/URL/deduplication behavior; one bounded common traversal, closed errors and no coordinator/HTTP |
| src/duckduckgo/html.rs | dd9dcdd6fc617df1f8638325bbb7edb8c932b1123b7605aed22d1d5a4b2da550 | Same derivation with format-specific selectors and challenge classification |
| src/html_utils.rs | 94e06286456997e9c60cf1139c1b38133d05339502d97904dafa01bb41e0f759 | Reuse existing public clean_html/resolve_duckduckgo_url functions; not copied or modified |
| src/duckduckgo/instant_answer.rs | b3aad75760fcad2e079bb9d15ec155e072748482d6ca1906b3ad19e109ca6ced | Derived typed response schema, result order, topic traversal, limited cleanup and query fallback encoding; no HTTP/coordinator |

The pinned upstream root [LICENSE](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/LICENSE)
is MIT, Copyright(c)2026 Rust-Framework Contributors, previously recorded blob5760f660.
Its complete notice is retained at crates/talos-tools/LICENSES/rust-websearch-MIT.txt, inside
the crate's package boundary. The derived file names its source, notice and changes. Tests are
Talos-authored synthetic inputs, not downloaded result fixtures. Source licensing and service-use,
query privacy and result-reuse conditions are separate; D admission is still pending.

## Parsing Contract

Lite selects a.result-link and td.result-snippet; HTML selects a.result__a and a.result__snippet.
Snippet position advances for every inspected link before empty URL/`https:`/duplicate rejection.
Missing snippets become empty strings; excess snippets are ignored. Helpers normalize text and
unwrap protocol-relative/uddg candidates, then empty titles use the resulting URL. Deduplication
compares resulting strings exactly. Both formats retain the legacy DuckDuckGo backend identity.

Lite catches challenge-form/g-recaptcha/cf-challenge; HTML catches the first two plus HTTP202.
Lite HTTP202 remains Status, not Challenge. Non200 classification precedes body markers. The
shared implementation preserves these distinctions without describing them as comprehensive bot
or validity detection. No source helper may create a client, resolve DNS or fetch result URLs.

Candidates are not URL admission. javascript, relative and malformed values can remain, as in
the existing helper. Its uddg decoder consumes the entire remaining string, translates '+',
retains the original on bad percent/invalid decoded UTF-8 and skips unescaped non-ASCII during
percent decoding. Extra parameters are not stripped. Those quirks are explicitly characterized;
future C result admission must validate candidates before winner selection or any fetch/render.

The Lite fallback uses the identical link selector and URL predicate. With positive max, an empty
first deterministic pass cannot gain a first valid result by repeating it; the redundant fallback
is omitted. New limit/errors always fail the response, without partial success or fallback repair.

## Bounds And Migration Differences

| Boundary | Limit / behavior | Limitation |
|---|---|---|
| Input | <=1048576 bytes before DOM parse | Not a DOM/process memory cap |
| Result count | 1..20; stop after max accepted candidates | Does not inspect later fields, but still parses the whole bounded DOM |
| Fields | <=16384 UTF-8 bytes for decoded href, joined text, helper outputs and final fields | Reject whole response on overflow; no silent truncation or partial success |
| Joining | Check fragment plus inserted separator with checked arithmetic before append | Existing helpers may allocate bounded intermediate strings |
| Encoding | Strict UTF-8; reject leading UTF-8 BOM | Upstream response.text/charset/BOM replacement equivalence is not established |
| Failure output | Closed error codes and redacted DdgCandidates Debug | Consuming raw result data is intentional; callers still own safe logging/rendering |
| Dependency unwind | catch_unwind around DOM/helper work | Not abort/OOM recovery, a CPU timeout or suppression of the process panic hook |

Selector iterators avoid collecting all links/snippets into extra vectors; scraper still builds
the complete DOM. Upstream rust-websearch uses scraper0.22 while Talos directly uses0.27.
Matching selectors and bounded fixtures do not establish malformed HTML/DOM normalization parity.
No new dependency or version alignment is introduced in this stage.

Twelve fixtures cover both formats, Unicode/entities/nested text, positional snippets/duplicates,
helper quirks and unsafe candidates, status/challenge distinctions, empty/malformed input, exact
and overflow input/fields/helper expansion, stop-after-max, UTF-8/BOM, redaction and bounded
generated result counts. Generated cases do not claim execution of proptest or full upstream
differential tests. Validation results live in I293 and the candidate's exact-head CI/review.

## Remaining Activation Gates

This parser is intentionally dormant under the existing private seam's dead-code allowance;
no new suppression is added. The future prepared adapter must consume the same owned request
across admission and route bounded response bytes through reviewed parsing/result admission.
Request headers/UA profiles, probe/retry/source and proxy snapshots, Bing/optional
adapters, charset/status-read-order and malformed DOM parity, D terms/privacy/result use,
official CLI/SDK/MCP/standalone roots, router/deadline/rollback and default activation remain gated.
Wikipedia remains a knowledge fallback; paid availability is not automatic selection. No opaque
or legacy fallback after a protected denial. Full C/EP and #643/#624 remain incomplete/open;
#705/#717 retain separate ownership.

## Dormant Instant Answer JSON Stage — 2026-10-18

Based on merged #733/main dea3edccb163420806e08ddeef51c65f977bac59. Before deriving source,
reverified the retained immutable0.1.2 archive hash/VCS metadata; freshly retrieved the pinned
Instant Answer source and root MIT notice. Archived source exactly matches the retrieved source,
and the full retained tools-package notice matches the retrieved notice (trailing newline only).
This stage does not claim another fresh archive download or service-use admission.

Direct typed Deserialize retains upstream required, optional and default schema behavior,
including required-unused AbstractSource/Type/Icon.URL/content.data_type, duplicate known fields,
missing/null differences and ignored unknown fields. The entire schema is deserialized before
selection, even when max would be reached by Abstract. Actual hosted-service schema parity,
including grouped Topics requiring Text/FirstURL in this pinned source, remains unproven.

Order is Abstract, non-calc Answer, Definition, parent-first depth-first RelatedTopics, Infobox
meta, then content. Result Some("") suppresses a topic's Text but not its children. Topic titles
split the original selected text at .!?。？！ before limited cleanup, preserving anchor-URL
punctuation and empty-title quirks. No deduplication, trim, URL repair/admission or general HTML
cleanup is added. DefinitionSource can be a label; Infobox candidates retain empty URLs and raw
label/value strings. All retain the existing compatibility DuckDuckGo backend identity.

Limited cleanup removes only the pinned lowercase b/i/code/pre tags and replaces the three
specified br spellings. It then calls the existing seven-replacement decode_html_entities helper;
unlisted entities such as &#46;, uppercase/unknown tags and whitespace remain. Heading/query
titles and Infobox values do not receive this cleanup. Query fallback preserves alphanumeric
and -_.~, maps space to +, and percent-encodes every other UTF-8 byte, including * and control
bytes. It is deliberately different from form encoding. These strings are data, not authority.

The common1MiB body/16KiB field/1..20 count and strict UTF-8/BOM boundaries apply. Query itself is
limited to16KiB; fallback URLs and Answer:/Definition: title prefixes count toward output bounds,
with checked lengths before appending. Field limits apply to consumed text and emitted fields,
not all unused schema strings; those remain bounded by body size and typed validation. A field
after max may be skipped, but invalid schema anywhere still rejects the response. Any error
rejects the whole response, never returning partial candidates.

Every result addition honors max. This explicitly differs from upstream's first three additions,
which can exceed requested max1/2. Visited topic depth is limited to32 (top-level depth1); overflow
is Limit, while serde_json's unchanged default recursion-limit failure maps to Parse. Selection
may skip the topic-depth check after max, but the JSON guard still applies to the whole schema.
These are migration differences, not full parity. Eager JSON allocation, default recursion guards
and catch_unwind are not streaming, hard memory/CPU bounds, stack-overflow/OOM/abort recovery or
panic-hook suppression. Closed errors and the shared redacted DdgCandidates wrapper remain.

Seventeen Talos-authored synthetic tests cover schema/duplicates/null/default, ordering/max/calc,
depth/width, fixed cleanup/helper quirks, unsafe/duplicate/empty candidates, encoded expansion,
query/title prefixes, byte bounds, encoding/status and redaction. Generated Unicode/count cases
do not claim proptest or an upstream differential run. Local validation and exact-head CI/review
are recorded in I293 and the candidate PR. All previous activation gates remain effective;
having three dormant DDG parsers does not establish complete DDG protocol or full C acceptance.

The separate [dormant Bing compatibility response parser](SEARCH-001-C-BING-RESPONSE-PARSER.md)
continues the same compound-family migration; it admits no native provider and leaves request,
source/probe/retry, result admission and all activation gates effective.
