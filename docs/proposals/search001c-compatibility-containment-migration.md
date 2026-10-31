# SEARCH-001-C Compatibility Containment Migration Amendment Proposal

> Status: Proposed / ineffective until reviewed and merged.
> Owner: SEARCH-001-C / I293 / #643 under effective ADR-090 / #712.
> Related evidence owner: SEARCH-001-D / I294 / #644.
> Activation candidate: #808.
> Baseline: `02fee3cc9537e6bf688d167036389b4af3011aeb`.

## Problem

ADR-090 correctly requires terms/privacy/query-exposure review before the Talos-owned protected
SearchRouter replaces the existing opaque compatibility coordinator as the default `web_search`
path. The 2026-10-30 bounded D refresh records a real residual rather than an implementation gap:
public DuckDuckGo terms/AUP do not establish that Talos may re-display the search-service result
surface inside a distributed product, and the current Microsoft/Bing agreement does not establish
general product-building/redistribution authorization for Bing materials. Bing also documents
query and related-data exposure for Bing-powered experiences.

That residual already applies to the legacy compatibility behavior. The #808 candidate does not
introduce a new native provider or a new model-facing operation; it replaces opaque execution with
an invocation-bound, Talos-controlled path over the same core compatibility family while reducing
several uncontrolled behaviors. Blocking a security/privacy containment replacement indefinitely
because the pre-existing external-use question is unresolved is one possible policy, but it should
be an explicit decision rather than an accidental consequence of the migration.

The maintainer additionally requires that this containment migration **must not reduce practical
search availability**. Security hardening is not authority to make working search unavailable on a
previously supported network/configuration, or to abandon a still-viable compatibility source so
early that the protected path materially regresses successful searches. This availability condition
is an activation gate, not a best-effort target.

This proposal defines the alternative decision permitted by ADR-090's migration clause. It does
**not** claim provider permission, legal compliance, native admission, or completion of SEARCH-001-D.

## Proposed Decision

Permit a **containment-only technical migration** from the existing legacy compatibility execution
to the protected SearchRouter even while the route-specific external-use residual remains tracked
under #644, but only when every condition below is satisfied.

1. **No provider-set expansion.** The migration may use only the compatibility services already
   exercised by the existing zero-key legacy path. It cannot add another zero-key provider, paid
   route, self-hosted instance, knowledge source, redirect destination, startup probe or ambient
   credential-selected route.
2. **No query-content expansion.** The model query is not enriched with identity, account, locale,
   precise location or other new user data before external dispatch. Health state stays query-free.
3. **Per-invocation query-disclosure non-regression.** Compare query-bearing logical sends separately
   from query-free reachability probes. For the same deterministic endpoint outcomes, the protected
   path must not disclose the query to more compatibility destinations merely because it imposes a
   shorter source slice than the legacy path. Worst-case aggregate request counts are not sufficient
   evidence. The acceptance matrix must include slow-but-successful primary cases, fast failures and
   reachable fallbacks and record legacy versus candidate query-bearing destinations.
4. **Availability non-regression.** A previously supported compatibility environment must not become
   unusable solely because of the migration. Acceptance must cover direct networks, configured-proxy
   networks that the legacy path supports, slow-but-successful sources, primary failure with reachable
   fallback, and ordinary fast success. Any newly unsupported retained configuration or deterministic
   legacy-success fixture is an activation blocker unless a separately accepted migration explicitly
   retires that support.
5. **Truthful destination authority.** Every possible destination is declared before permission and
   checked at logical request start. Redirects remain disabled unless separately scoped. Missing,
   stale or mismatched authority fails closed.
6. **Proxy preservation, not silent bypass or disablement.** Supported proxy/self-hosted behavior must
   be represented truthfully in prepared authority and exercised through controlled transport. A
   configured proxy may never be silently bypassed. Fail-closed `Unavailable` is appropriate for a
   genuinely unsupported configuration during development, but it is not sufficient activation
   evidence if the legacy production path supports that configuration and would therefore regress
   availability.
7. **No hidden paid selection.** Ambient Tavily/SearXNG/provider configuration does not enter the
   default Auto candidate set. Explicit optional-provider policy remains separately owned.
8. **Stable user contract.** The `web_search` name, model schema, result projection and Network
   identity remain compatible across migration and rollback.
9. **Version/deployment rollback only.** A protected invocation never falls back at runtime to the
   opaque coordinator. Rollback is an explicit version/deployment revert to the previous release
   or a separately reviewed migration.
10. **External-use residual remains visible.** #644 stays open. Release/owner records must state that
    the migration is a technical containment decision and does not establish DuckDuckGo/Bing
    permission or waive applicable terms, privacy or result-use obligations.
11. **Fresh exact-head gates.** Activation still requires independent security/API review, locked
    exact-head CI, rebuilt-binary evidence, official-root wiring evidence and final main/head CAS.

If any condition fails, this amendment does not authorize default activation.

## Availability And Exposure Comparison Gate

The comparison is behavioral, not merely a comparison of nominal constants.

- **Legacy timing:** the compatibility dependency's 15-second value is a per-request timeout, not an
  enclosing search deadline. Retry behavior can extend total legacy search time beyond one request
  timeout.
- **Rejected candidate behavior:** #808 previously imposed one 15-second enclosing search budget and
  divided remaining time among remaining sources. That stricter envelope could abandon a valid
  slow primary after roughly one quarter of the legacy request window, disclose the query to a later
  destination and turn a legacy success into a protected-path failure.
- **Current candidate timing:** #808 head `eaa8c5cbdad00eb3e8f2bc03fc80e9c181db0a29`
  preserves the legacy 15-second timeout **per query-bearing fixed-family request**. Lite, HTML and
  Bing may retry one typed Timeout/Network failure after 500 ms; Instant Answer does not retry. The
  retry remains on the same already-approved destination before any later-destination disclosure.
  The protected path also retains the legacy request-time, query-free DuckDuckGo/Bing reachability
  probes (HEAD first, GET only after a non-success/non-redirection HEAD, 3 seconds per probe request,
  30-second cache) and the pre-existing Wikipedia OpenSearch last resort (10-second window, no
  retry). The four fixed-family destinations therefore contribute at most seven query-bearing
  attempts, and Wikipedia contributes at most one additional query-bearing attempt, for **eight
  query-bearing attempts across five fixed query destinations** under a separate **135-second**
  enclosing invocation cap. Deterministic fixtures cover slow primary success, same-destination
  retry, reachable fallback, family-unreachable disclosure gating, query-free probe behavior and
  the Wikipedia last resort.
- **Rejected proxy behavior:** #808 previously returned `Unavailable` whenever standard proxy
  variables were present. That prevented silent direct bypass but regressed a proxy-only environment
  that the legacy reqwest path could use.
- **Current candidate proxy behavior:** supported HTTPS environment proxy and `NO_PROXY` routing are
  frozen once at production tool construction, before a query exists. The sanitized proxy CONNECT
  destination is included in prepared authority while credentials remain private. Direct/NO_PROXY
  destinations retain local target DNS/address containment; proxied hostname destinations use the
  explicitly configured trusted proxy for remote target resolution while Talos locally resolves and
  pins the proxy peer. Unsupported selected proxy forms remain fail-closed rather than silently direct.

These changes are implementation candidates for the two non-regression gates; **they are not yet
accepted evidence merely because they are present in the branch**. Fresh exact-head CI, rebuilt
binary evidence, deterministic direct/proxy/slow/fallback parity, independent security/API review
and final CAS remain required. A runtime fallback to the legacy opaque coordinator is not an
acceptable remedy.

## Why This Is Narrower Than Provider Admission

SEARCH-001-D asks whether a route is suitable as an admitted native provider, including terms,
privacy, parser ownership, independence, regional evidence and future E/F use. This proposal does
not answer that question. It treats only the replacement of an already-existing compatibility
execution path with a more constrained implementation.

Accordingly:

- DDG/Bing do not become admitted native routes.
- Two-route independence is not claimed.
- Mainland-China or other regional reachability is not claimed.
- No new provider implementation authority follows.
- #644 and E/F remain independently gated.

The only accepted fact, if this proposal is merged, is that continuing the pre-existing
compatibility service set through a strictly more controlled Talos execution boundary is an
approved technical migration despite the recorded external-use residual, **provided the final
candidate also proves availability and query-disclosure non-regression**.

## #808 Candidate Mapping

The current #808 implementation candidate at this proposal snapshot is exact head
`eaa8c5cbdad00eb3e8f2bc03fc80e9c181db0a29`. It is evidence under validation, not part of this
documentation change and not activation-ready by declaration alone.

- Production construction moves from legacy `WebSearchTool` to prepared SearchRouter at the shared
  network contribution seam.
- The protected and legacy surfaces retain the same name/schema/Network identity/summary/read-only
  contract; protected direct legacy execution fails closed.
- Zero-key production composition admits only the retained compatibility behavior and does not
  discover ambient paid providers.
- Before query-bearing sends, the protected path retains the legacy query-free DuckDuckGo/Bing
  reachability gate: roots are prepared before permission; HEAD is attempted first; GET is used only
  after a non-success/non-redirection HEAD; each probe request has a 3-second window; results are
  cached for 30 seconds. An unreachable family opens no query-bearing connection to that family.
- The fixed query-bearing source order remains Lite / Instant Answer / HTML / Bing. Each request has
  the legacy 15-second timeout. Lite, HTML and Bing may retry one typed Timeout/Network failure after
  500 ms on the same destination; Instant Answer does not retry. These four destinations contribute
  at most seven query-bearing attempts. After ordinary four-source exhaustion, the pre-existing
  Wikipedia OpenSearch last resort may execute once through the same controlled transport with its
  legacy 10-second window and no retry. The complete compatibility path therefore has at most **eight
  query-bearing attempts across five fixed query destinations** under a separate **135-second**
  enclosing cap.
- Supported HTTPS environment proxy plus `NO_PROXY` state is snapshotted into immutable prepared
  configuration. Proxy credentials are retained only in the private frozen URL; manifest/profile
  authority exposes the sanitized proxy destination. Runtime ambient proxy discovery stays disabled.
- The direct compatibility authority contains exactly nine fixed endpoint variants before permission:
  five query-bearing endpoints (Lite, Instant Answer, HTML, Bing and Wikipedia) plus four query-free
  DDG/Bing HEAD/GET probe endpoints. Proxied HTTPS reserves CONNECT plus application send; with the
  retained probes and Wikipedia fallback, the exact compatibility-parity production ceiling is **24
  physical request units**. Generic transport bounds remain 120s/16; only this exact production
  compatibility envelope uses 135s/24.
- A loopback-only hostname CONNECT/TLS fixture admits only fixed witness authorities and checks
  CONNECT metadata, target SNI, proxy-peer resolution and application-query isolation.
- Retryable transport classification is intentionally narrow: typed transient connection reset/
  abort/broken-pipe/not-connected/timeout outcomes may retain same-destination retry behavior, while
  malformed/truncated HTTP body framing, TLS/certificate, CONNECT protocol and HTTP protocol failures
  remain terminal `Request` outcomes.
- The locked legacy `rust-websearch` build uses reqwest without its SOCKS feature, so the current
  HTTP/HTTPS proxy migration does not retire an actually built legacy SOCKS transport capability;
  unsupported selected forms remain fail-closed.
- No startup network probe, native provider, paid route or runtime fallback to `WebSearchTool` is
  introduced. The retained reachability checks are request-time, query-free legacy compatibility
  behavior executed through prepared authority.
- Final CI/review/rebuilt-binary evidence is deliberately not frozen into this decision document;
  the #808 exact-head validation record is the source of truth. Activation requires those gates to
  be green for the final implementation head.
- At this snapshot, main is `02fee3cc9537e6bf688d167036389b4af3011aeb` and #808 is directly
  1 commit ahead / 0 behind with exactly **23 SEARCH-001-C files changed**.

This mapping must be refreshed if #808's implementation head or main advances. A transient CI status
change alone does not require a documentation commit; final acceptance evidence belongs to #808's
final exact implementation head.

## Privacy And Terms Record

The bounded 2026-10-30 refresh is recorded in #644 comment `6517387257`. Its purpose is to keep the
residual explicit rather than manufacture an acceptance:

- DuckDuckGo's privacy policy describes substantially reduced search-history/IP retention, but its
  published use restrictions do not resolve Talos result re-display authorization.
- Microsoft's Bing materials terms do not establish general distributed-product use, and Bing
  privacy material documents query/related-data collection and third-party Bing-powered query
  exposure.

Merging this proposal would therefore be a **maintainer risk/technical migration decision**, not an
assertion that those external terms grant Talos additional rights. If the project requires explicit
provider authorization before any continued compatibility use, this proposal must be rejected and
#808 must not activate the direct family.

## Acceptance / Rejection Consequences

### If accepted

Acceptance authorizes the containment migration direction only. It does **not** waive the availability
and query-disclosure parity gates above. #808 may proceed to activation only after its implementation
is updated as necessary and fresh exact-head evidence proves those gates. #644 remains open and future
provider/native-route work remains blocked by its own acceptance matrix.

### If rejected or left unmerged

#808 remains Draft and must not switch the production default. The project must instead establish
applicable route authorization or design another separately reviewed migration that does not
silently disable working search. C-V11 and I293 completion remain Pending.

## Required Review

Because this proposal changes an activation gate rather than implementation mechanics, merge
requires explicit maintainer acceptance plus independent security/API review of the exact document
head. Shared GitHub identity review is technical independent-agent evidence, not distinct-human or
legal approval. Documentation CI and final base/head CAS are also required.
