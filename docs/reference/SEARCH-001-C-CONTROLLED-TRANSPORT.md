# SEARCH-001-C: Controlled Direct Transport

Library-only stage under effective I293/#712 and ADR-090; baseline #713 merge69e1e116.
Private implementation: crates/talos-tools/src/search_backend/transport.rs. Default WebSearch,
public API, model contract, dependencies/locks and provider selection remain unchanged.
This stage supplies a runnable direct-HTTP prerequisite, not full C/EP completion or activation.

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
Authorization or Host override are supported. POST has a fixed application/json body format.
This deliberately does not yet implement the upstream form protocol.

GET/HEAD may follow only predeclared301/302/303/307/308 targets. Every hop gets its own checks and
pinned dial set, under the same budget. POST retains body/method only on same-origin307/308;
other POST redirects fail. HTTPS downgrade, userinfo and fragments fail. A denied next host
receives neither DNS nor a connection. Already-sent input at an approved source cannot be undone.

Explicit proxy scopes return a closed UnsupportedProxy error; never silently dial direct.
Ambient proxy variables are ignored only in this dormant direct implementation. Preserve current
working proxy behavior until a separately validated snapshot/migration path permits activation.

## Bounds

| Resource | Bound | Evidence/meaning |
|---|---|---|
| One shared execution duration | >0, <=60s | Covers DNS, send, redirect and reads; explicit fences reject expired/late-ready outcomes |
| Hop attempt reservations | 1..8 shared atomically | Includes failed DNS and post-lookup revocation, all future candidates/hops; actual sends cannot exceed reservations; no automatic HTTP retry |
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

Real HTTPS certificate/SNI/address fixtures; explicit proxy and ambient configuration migration;
controlled existing compatibility request/form/parser/probe/retry/source parity; bounded D
terms/privacy/parser review and full MIT provenance before copying; real WebSearch and official
CLI/SDK/MCP/standalone dispatch, router/deadline/rollback acceptance. Preserve paid opt-in and
Wikipedia knowledge-only disposition. No default switch or opaque/legacy fallback after denial.

The module inherits the private SearchBackend seam's already existing staged dead-code allowance;
no new suppression is added. It is intentionally dormant and not a user-visible feature or new
public transport API. Future integration must consume this code and re-evaluate staged allowances.
Full C/EP rows remain Pending, D independently incomplete, E/F gated and #643/#624 open.
