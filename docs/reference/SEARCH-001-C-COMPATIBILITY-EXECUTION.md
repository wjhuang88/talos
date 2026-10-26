# SEARCH-001-C: Dormant Bound Compatibility Execution

Private I293/#712/ADR-090 prerequisite, based on main
ae6a0791c6317ff7e384295ac9af74a6571572a7 (#737). The existing fixed request factories now also
offer prepare_search, which owns the protocol, original query, requested count and prepared
transport payload together. This is library-only: no official caller, model contract, default
search behavior, router, provider, public API, dependency, lockfile or permission change.

## Ownership And Dispatch

PreparedCompatibilitySearch has private fields, no Clone/mutators and redacted Debug. Its sole
production constructor validates count1..20, calls the bounded fixed factory, then owns the
original query. Preparation performs no HTTP/DNS, authority checks or budget reservations.
The same object must survive future permission admission. It is not an approval seal, query
fingerprint or globally single-use manifest; existing invocation and query-free Session scope
semantics remain unchanged.

Consuming execute accepts only the existing SearchTransport. It sends the captured immutable
request through execute_prepared, retaining canonical endpoint membership, live authority,
invocation binding, DNS/address/TLS/redirect controls and shared budget. There is no second
query, count, protocol, destination, header, callback or transport-trait override at this entry.
Transport errors return from this single-source execution without selecting another parser or
source. The owning chain separately permits typed DNS/connection-refusal and semantic continuation, as linked below.

| Captured protocol | Response parser | Captured parser inputs |
|---|---|---|
| Lite | DDG Lite | status, body, bounded20-candidate window |
| HTML | DDG HTML | status, body, bounded20-candidate window |
| Instant Answer | DDG JSON | status, body, original query, bounded20-candidate window |
| Bing | Bing CN | status, body, bounded20-candidate window |

The [result-admission prerequisite](SEARCH-001-C-COMPATIBILITY-ADMISSION.md) replaces the raw
compatibility wrapper with opaque redacted AdmittedCompatibilityResults. Whole-batch bounds and
family identity are checked; invalid URLs are filtered before captured requested-count truncation.
Consumption preserves original fields/order and the compound family's DuckDuckGo ID. This does
not attribute native Bing, deduplicate further, select a router winner, render or grant fetch
authority. Parser errors retain closed DDG/Bing classifications; new admission errors are fixed
content-free enums. No raw query, body, URL or dependency diagnostic enters Debug/Display errors.

## One Existing Execution Budget

After the consuming send, the existing shared transport deadline is checked before parsing,
then unconditionally after parsing, including parser failures. The parse result is saved before
the second check. Deadline therefore dominates a success or failure completed after expiry.
The same budget also fences result admission and its final success/error completion. No new
parse/admission deadline or budget is created; request/byte accounting remains in transport.

The private finishing helper receives the budget's live check from its sole production caller.
Tests inject deterministic check outcomes to prove ordering, including failure paths; this is
not an exposed clock or parser override. Synchronous DOM/JSON parsing cannot be preempted and
may occupy an async executor thread until it returns. These fences reject expired completion;
they do not establish a CPU deadline, cancellation responsiveness, OOM/abort recovery, panic-hook
suppression or official root integration.

## Evidence Boundary

Ten synthetic tests cover captured metadata/ownership/count, four-format parser selection,
status/challenge/whole-response errors, empty and unsafe candidates, redaction, cross-invocation
zero effects, wrong-format rejection and deterministic pre/post deadline dominance. Paused Tokio
clock tests also use the actual shared budget before send and before parsing.

Four formats each send real controlled loopback HTTP requests for count1 and2. The fixtures
record method, target, form/query bytes and fixed Content-Type, then assert matching parser,
ordering, family identity and Instant Answer fallback from the original Unicode/reserved query.
Responses are mutually incompatible synthetic HTML/JSON; no official host is contacted.

The relocation helper exists only inside cfg(test): it starts from the real fixed factory,
replaces destination/invocation through the existing validating constructor for an explicit
loopback scope, and asserts method/path/query/body/type preservation. Protocol/query/count remain
captured. This exercises consuming controlled dispatch into parsing; it is not official-host
HTTPS, live protocol/service parity, default-root or provider admission evidence. No production
destination override, new permission exception or fixture warm-up is introduced.

All new orchestration is Talos-authored. Existing parsers/helpers and their pinned upstream/full
MIT provenance remain unchanged and distributed with talos-tools. See the
[fixed requests](SEARCH-001-C-COMPATIBILITY-REQUESTS.md),
[DDG parsers](SEARCH-001-C-DDG-RESPONSE-PARSER.md) and
[Bing parser](SEARCH-001-C-BING-RESPONSE-PARSER.md) references.

The [fixed header snapshot prerequisite](SEARCH-001-C-COMPATIBILITY-HEADERS.md) now retains UA
and Bing language in the same request. Preparation includes local entropy access; it still has
no network/authority/budget effects. Header selection timing parity, probe/retry/source/proxy
snapshots, full protocol/parser parity,
D/privacy/result-use review, full result-policy acceptance, official CLI/SDK/MCP/standalone roots, router,
health, cancellation/rollback/default activation and release retain their gates. The private
request/response binding prerequisite does not complete those rows. I293 remains Active/Claimed,
Completion Pending; #643/#624 stay open, #705/#717/#735 remain independently owned.

The [dormant fixed-source chain](SEARCH-001-C-COMPATIBILITY-CHAIN.md) now owns all4 prepared
objects and consumes them sequentially through this entry, using one transport/budget and the
explicit typed availability/semantic continuation policy. It does not activate an official caller, add retries
or complete upstream source/probe/privacy parity.
