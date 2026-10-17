# SEARCH-001-C: Controlled Search Transport

Library-only stage under effective I293/#712 and ADR-090; baseline #713 merge69e1e116.
Private implementation: crates/talos-tools/src/search_backend/transport.rs. Default WebSearch,
public API, model contract, dependencies/locks and provider selection remain unchanged.
Direct HTTP merged in #714 (d78dc75c). The current TLS/CONNECT subset extends that dormant
prerequisite, not full C/EP completion or activation.

## Connection Contract

Only actual parsed URL/method -> exact endpoint membership + injected live permission authority
can proceed. No preparation-time DNS or startup probe. Before each hop, check deadline and
authority, resolve the host once, reject the entire empty/over-limit/mixed-denied set, then pin
all accepted addresses into a fresh reqwest client. Check deadline and live authority again
after DNS/client construction, immediately before logical send. Address ports must equal the
effective URL port. IP literals skip DNS but receive the same address check.

Clients disable implicit redirect, HTTP retry, Referer, ambient proxies and all optional decoders,
request identity encoding, use HTTP/1 and no cross-hop pooling. Original hostname remains the
Host/TLS identity; default Rustls certificate validation is retained. No caller headers, cookies,
Authorization or Host override are supported. Prepared JSON-byte requests retain fixed
application/json; prepared forms use bounded
application/x-www-form-urlencoded with ordered owned fields. Raw execute/execute_form adapters
exist only under cfg(test); production adapter code must consume a prepared request.
Neither activates an upstream route or establishes complete compatibility protocol parity.

GET/HEAD may follow only predeclared301/302/303/307/308 targets. Every hop gets its own checks and
pinned dial set, under the same budget. POST retains body/method only on same-origin307/308;
other POST redirects fail. HTTPS downgrade, userinfo and fragments fail. A denied next host
receives neither DNS nor a connection. Already-sent input at an approved source cannot be undone.

Explicit proxy scopes support only the limited CONNECT contract below; all other shapes return
a closed UnsupportedProxy error and never silently dial direct.
Ambient proxy variables are ignored only in this dormant direct implementation. Preserve current
working proxy behavior until a separately validated snapshot/migration path permits activation.

## Bounds

| Resource | Bound | Evidence/meaning |
|---|---|---|
| One shared execution duration | >0, <=60s | Covers DNS, send, redirect and reads; explicit fences reject expired/late-ready outcomes |
| Hop attempt reservations | 1..8 shared atomically | Direct hop reserves1; proxy compound hop reserves2 before DNS; includes failed DNS and post-lookup revocation, all future candidates/hops; actual sends cannot exceed reservations; no automatic HTTP retry |
| DNS addresses per hop | 1..16 | Retain seventeenth answer to detect overflow; reject whole set |
| Actual URL / encoded query | <=8192 / <=4096 bytes | Path uses the core2048-byte bound; no userinfo/fragment |
| POST body | <=16384 bytes per send | Total sends bounded; no GET/HEAD body |
| Response headers | <=64 entries / <=16384 name+value bytes | Post-parser acceptance limit before redirect/body, not pre-allocation proof |
| Body bytes read | 1..1048576 shared cumulatively | Chunked overflow fails without partial success; decoding disabled and nonidentity Content-Encoding rejected |
| Redirect hops | Core scope0..5 plus shared start limit | No reset of budget per hop or invocation candidate |

Pinned Hyper1.10.1 HTTP/1 has its own default finite parser buffer (8192+4096*100 bytes).
Our header acceptance check runs after that parser, and is not a16KiB parsing allocation cap.
Body chunks may already be allocated by the HTTP stack before rejection; these bounds are not
a whole-process memory guarantee. Exact execution budgets are internal staged limits requiring
compatibility review before activation, not approval TTLs or a generic NET retry system.

## Address Policy And Evidence

Conservative direct-search public-unicast policy excludes private, loopback, link-local, shared,
documentation, multicast/reserved IPv4; IPv6 permits ordinary2000::/3 unicast with conservative
special/tunnel/retired exclusions. Mapped IPv6 is classified using its underlying IPv4.
Exceptions match the exact original IpAddr: an IPv4 exception does not alias a mapped IPv6
exception. Every address in a DNS answer must pass; no filtering to hide a forbidden answer.

Reference snapshot read2026-10-13:
[IANA IPv4 special registry](https://www.iana.org/assignments/iana-ipv4-special-registry),
[IPv6 special registry](https://www.iana.org/assignments/iana-ipv6-special-registry),
[IPv6 unicast allocations](https://www.iana.org/assignments/ipv6-unicast-address-assignments).
The policy deliberately excludes some globally reachable special uses. It is not an exhaustive
future registry mirror or proof that locally routed public addresses cannot reach private services.
Explicit address exceptions are trusted-host configuration, not model input or automatic trust.

26 fixtures drive actual reqwest/loopback sockets plus cause-specific DNS/authority observations:
Host preservation and DNS pinning; real System DNS/IP literals; real Once proposal/admission and
clear revocation; wrong port/path/method/userinfo; mixed/over-limit/private/mapped addresses;
DNS-time revocation; denied redirect target; approved GET and same-origin POST redirects;
cross-origin POST body isolation; hop/request/body/header/encoding limits; concurrent budget;
hidden-retry rejection; expired/late-ready DNS; deadline and caller-drop peer EOF. An isolated
child process sets all proxy variables/clears NO_PROXY and proves target1/proxy0 connections
without mutating shared environment. Negative receipts join and drain the local listener queue.

These are local HTTP connections, not provider/regional observations. Injected deterministic DNS
tests pin actual sockets but do not claim a controlled authoritative-DNS rebinding laboratory.
No internal resolver/HTTP worker cancellation guarantee follows from observed local peer EOF.

## Remaining Activation Gates

Broader explicit proxy support and ambient configuration snapshot/migration;
controlled existing compatibility request/form/parser/probe/retry/source parity; bounded D
terms/privacy/parser review and full MIT provenance before copying; real WebSearch and official
CLI/SDK/MCP/standalone dispatch, router/deadline/rollback acceptance. Preserve paid opt-in and
Wikipedia knowledge-only disposition. No default switch or opaque/legacy fallback after denial.

The module inherits the private SearchBackend seam's already existing staged dead-code allowance;
no new suppression is added. It is intentionally dormant and not a user-visible feature or new
public transport API. Future integration must consume this code and re-evaluate staged allowances.
Full C/EP rows remain Pending, D independently incomplete, E/F gated and #643/#624 open.

## TLS And Explicit CONNECT Subset — 2026-10-13

Direct HTTPS retains original hostname SNI/Host, pinned dial addresses and Rustls verification.
The proxy subset accepts unauthenticated HTTP CONNECT only, with HTTPS IPv4-literal targets
and declared GET/HEAD/POST methods. Hostname/HTTP/IPv6 targets, HTTPS proxies and malformed
proxy method/path are UnsupportedProxy; authentication and SOCKS are not supported.

The declared proxy endpoint path `/` is its configuration locator, not the CONNECT request-target.
The latter is the approved literal target IP plus effective port. Target address policy runs
before proxy DNS. The entire proxy answer set must pass the same port/global-or-exact-exception
policy and is pinned. Both target and proxy live authority are checked before DNS and immediately
before the compound logical start. Two shared reservations cover CONNECT plus target request;
failed DNS/CONNECT consumes them conservatively. Ambient proxies remain disabled; there is no
implicit direct fallback, retry, cookie or Authorization forwarding.

Reqwest0.13.4 owns CONNECT→TLS→application internals. The authority fence covers the compound
start, not each subsequent physical substep. Revocation after that start can permit already
started application work; new redirects/hops must recheck and fail. A trusted proxy can choose
its own remote routing: literal authority and verified IP SAN do not prove its remote TCP peer
or end-to-end address pinning. Hostname CONNECT is rejected because reqwest delegates its
resolution to the proxy; local target resolution would not enforce that destination.

Pinned hyper-util0.1.21 CONNECT parsing has an8192-byte buffer/100-header bound, separate from
our target response post-parser acceptance limits. Oversized CONNECT response tests are not
proof of a16KiB pre-allocation limit. No whole-process memory/cancellation guarantee follows.

Thirteen added fixtures cover direct SNI/Host/pinning; trusted wrong SAN, untrusted CA, expiry
and CONNECT IP SAN; HTTPS downgrade; literal CONNECT/POST isolation; real Once proxy+target
admission/revocation; denial before DNS; mixed answers and DNS-time revocation; unsupported
shapes;407/oversized CONNECT; two-unit budget/deadline; compound-start revocation across a
redirect; and caller-abort peer closure. Together with the original26, focused transport runs39.
No live provider or regional availability claim is made.

The test-only Python stdlib loopback server is an explicit host-utility tradeoff: existing runner
availability avoids a new production/native dependency; missing Python fails rather than skips.
Replace when a Rust-native TLS test-server dependency is approved. See the public synthetic
certificate [fixture provenance](../../tests/fixtures/search-tls/README.md). The CA is merged
only under cfg(test); production trust and validation are unchanged. Fixtures relay only to their
own fixed loopback TLS socket. Terminal receipts require drained accepts and joined workers.
Startup/operation and child polling are timed; kill/wait/reader join have an OS cleanup tail
that is not independently interruptible. Do not call every cleanup step hard-deadline-bounded.

Initial focused validation had33pass/3fail: repeated SNI callbacks on one TCP connection were
mistaken for extra requests, and the fixture treated legitimate absent IP SNI as a counter,
raising TypeError in its TLS callback. Corrected receipt handling and physical connection counts
produced39pass/0fail. No packet-trace diagnosis of repeated callbacks is claimed. All-target
Clippy's manual range-pattern finding was corrected without a lint suppression. Fresh locked
workspace validation and protected exact-head review/CI/CAS are recorded in I293.

Cross-platform correction2026-10-14: initial CI3172 macOS positive TLS cases failed while
Windows passed. The25-year synthetic leaf violated Apple's825-day TLS leaf limit; regenerated
365-day positive/host-only leaves remove that known incompatibility while retaining platform
verification, all negatives and production trust. Closed Request errors did not prove the sole
cause. Fresh corrected-head macOS acceptance remains required; see I293 and fixture provenance.

## Bounded Form POST Prerequisite — 2026-10-14

#715 TLS/CONNECT merged after corrected-head native macOS/Windows acceptance; see I293.
The next dormant form path uses only fixed internal Json/Form content types and the existing
URL serializer. Preflight checks UTF-8 byte expansion, '='/'&' separators and checked arithmetic
before allocation; encoded size <=16384 implies at most8192 fields, even for empty pairs.
Per-field raw length is also checked before scanning. This bounds encoding work/output, not
caller-owned input allocation or whole-process memory. The serializer's synthetic form.invalid
URL is an in-memory container only; it receives no permission, DNS or connection activity.

Spaces become '+', literal '+'/'~'/controls/non-ASCII bytes are escaped; order, duplicate keys
and empty keys/values are retained. Existing target URL/query is never rewritten. The encoded
body is owned, and POST307/308 keeps its bytes/type only within the same origin. Shared deadline
fences run before encoding, after it and at the existing send path; oversize uses no DNS or
attempt reservation. Actual request authority/redirect/TLS/CONNECT/address controls are reused.
No caller content-type/header override, extra retry, probe, ambient proxy or default activation.

Eight added fixtures and prior39 form a47-case focused checkpoint. Golden output and bounded
generated decoder round-trips cover serialization without introducing proptest/dependency or
claiming proptest execution; real socket/TLS fixtures cover body/type, exact/overflow bounds,
redirect isolation, denial/deadline and proxy visibility. This is not copied upstream parser code,
provider admission, prepared-object fingerprint integration or full protocol parity.

## Owned Request Payload — 2026-10-17

Pure private preparation owns the actual parsed URL, HTTP method, encoded body and fixed type,
validates the existing URL/body/method bounds and exact manifest endpoint membership, and records
its invocation ID. json_bytes labels already-encoded bytes application/json without parsing or
validating JSON; form completes bounded encoding during preparation. Neither constructor accepts
caller headers or performs live-authority checks, DNS, sends or execution-budget reservations.
Debug and all errors remain content-free. There is no payload hash or query digest in Session scope.

The production transport entry accepts only an owned PreparedTransportRequest. Consumption checks
invocation ID before authority, DNS and budget reservations, then reuses all existing execution
fences. The object exposes no mutation or Clone API. Source URL/body/form mutations after preparing
owned copies cannot change sent bytes. Different invocation IDs sharing the same Session scope
cannot exchange objects, while query-independent Session reuse remains the ADR-090 contract.

This is an ownership prerequisite, not an approval seal: one manifest can prepare multiple objects,
and consuming an object is not a global exactly-once guarantee. Future official prepared tool
invocations must hold this same object across approval; this stage does not enforce when it was
constructed, integrate Agent input/fingerprint binding, or complete provider protocol parity.
No default activation, upstream protocol copying or D admission follows.

Five added prepared fixtures plus the prior47 exercise this entry. The real Once-permission
fixture now prepares before admission and retains a second object across clear/revocation;
execution still rejects it before another DNS request. Existing HTTP/TLS/CONNECT/redirect and
budget tests pass through test-only raw adapters into the consuming entry. Local results and
exact candidate gates are recorded in I293; these tests do not establish official root integration.
