# SEARCH-001-C: Talos-Owned SearchRouter

**Status**: Active / Claimed; selected in I293
**Type**: Implementation Child
**Source Issue**: #643

Own first-valid-success routing and bounded hedging after SEARCH-001-B. The claim activates only
this bounded C slice; implementation must preserve ADR-085, the model-facing contract and the
existing paid-provider/Wikipedia compatibility constraints.

Iteration: [I293](../../iterations/I293-search-router.md). Effective governance claim: #664,
merged at `d801790454e252699684253166984662ea9dab6a`.

## Current Execution Boundary

The current session permits architecture/evidence and test-only characterization, not production
search behavior changes. The merged #681 test-only stage records the existing first-completion race;
it does not implement or complete SearchRouter. Runtime routing, typed failures, bounded hedging,
deadline/cancellation, health state and binary acceptance remain pending. Keep #643 and #624 open.

Existing test evidence: #681 merge `41381f8055b66d54de42988e2b936a12e5af3ac3`.
The [router integration plan](../../reference/SEARCH-001-C-ROUTER-INTEGRATION-PLAN.md)
audits the caller-context gap and supplies the pending implementation/acceptance matrix;
it does not activate a router or complete C.

The 2026-10-10 I293 stage adds a disposable twelve-case offline scheduling experiment, with
paused-clock, launch/drop and bounded-concurrency observations. It exercises already-classified
synthetic outcomes and control-boundary races; it does not call production search or establish
HTTP cancellation, admission, caller-context, health or binary integration. All full C acceptance
rows remain pending. See the iteration owner for recovery provenance and current validation.

#696 scheduling stage merged at `444a8137f054b33802f26bffdb55fb0f83a7a861` with full
exact-head CI3129 and independent Agent technical review. The next bounded test-only stage
experiments with URL admission/filtering/truncation and injected-clock validation boundaries.
It also records URL-parser repair behavior as a policy gap, not strict syntax or fetch safety.
Neither stage integrates production search or completes C-V02/C-V05; full C acceptance remains
pending. The iteration owner retains validation, limits and remaining integration gates.

#706 result-admission stage merged at `4b03149ca452bbaf4e96b580ab74135bc4ac670d` with full
CI3136 and independent technical review. The next test-only target drives actual Agent permission
dispatch and Session Interrupt using an offline Network stand-in, to characterize approval TTL,
the detached coordination token and owned-future cleanup. It does not call WebSearchTool or prove
HTTP/DNS cancellation, deadline propagation or full C-V10 acceptance. C and Epic remain open;
the iteration owner records fresh validation and residuals.


#707 caller stage merged at `7e817ab59364bf282c49941ec501f82b0c6c40fb` with six successful
CI3141 jobs and independent exact-head Agent review. The next test-only stage uses the real
registered WebSearchTool metadata and PermissionSessionState without executing search: default
Ask cannot compile a typed resource proposal; explicit fixture Allow can admit without a grant.
Production registration uses the Agent pipeline; the apparent CLI/SDK PermissionAwareTool classes
are test-only. This records a configuration-specific resource gap, not binary/transport acceptance
or authorization to repair policy. See I293 and the router plan for source audit and fresh gates.
All full C acceptance remains Pending; #643/#624 stay open.

#708 merged as `f036111f6e969feea47ad784f3cb3effd7e20a33`, with corrected exact-head review,
all six CI3146 jobs successful and three cases executed on macOS/Windows without filters.
The next architecture-only [endpoint-authority packet](../../reference/SEARCH-001-C-ENDPOINT-AUTHORITY-PLAN.md)
distinguishes proposal compilation from execution enforcement and defines planned carrier,
compatibility/fallback, redirect/connection and real-dispatch acceptance gates. It changes no
production behavior or permission API, activates no PERM-007-H work and admits no provider.
C/I293 and all full acceptance remain incomplete/Pending; #643/#624 remain open.

## Authorized Scope Addition — 2026-10-13

The maintainer accepted the described production endpoint-authority/search wiring scope. The
architecture/test-only boundary above is preserved as prior-stage history; the expanded phase
requires an effective I293 scope-amendment claim and accepted, independently reviewed
[ADR-090](../../decisions/090-invocation-bound-search-endpoint-authority.md) before implementation.
The draft remains governance-only and cannot activate code. Scope is search-only carrier,
permission/Agent/tool wiring, controlled existing compatibility transport and real-dispatch tests;
all-built-in Auto policy, Git, generic NET retry/breaker, new native admission and release stay out.
Preserve model-facing contract, no startup/GeoIP probing and explicit paid consent. Full C/EP
acceptance remains Pending; #643/#624 stay open. See I293 for authorization, inventory and gates.


## Effective Carrier And Direct Transport Stage — 2026-10-13

#712 merged989fec2e establishes the accepted ADR-090 contract and amended I293 scope.
#713 carrier merged69e1e116 after exact-head protected Agent review6138838842, full
CI3168/run38350762077 and CAS6139059609. This is partial infrastructure evidence.
The next private library-only [controlled direct transport](../../reference/SEARCH-001-C-CONTROLLED-TRANSPORT.md)
stage is Review under I293, with actual local HTTP/DNS/permission/redirect/budget fixtures;
no default WebSearch, new API/dependency or provider admission is activated. Full C/EP,
TLS/proxy/compatibility parity and bounded D migration gates remain Pending; #643/#624 open.


#714 direct transport merged `d78dc75c` after protected review6140200974, full CI3170 and CAS.
The next dormant TLS/CONNECT subset stage is Review under I293:39 focused fixtures; direct
SNI/certificate verification and unauthenticated HTTP CONNECT to HTTPS IPv4 literals only.
No default activation, full C/EP completion, provider admission or release. Exact-head review/CI,
broader proxy/ambient migration, compatibility parity, D and official-root acceptance remain.
See the transport contract and I293 for authority timing, trusted-proxy limits and validation.

#715 original CI3172 is failed (macOS445pass/5fail; all13 TLS cases pass on Windows).
The25-year synthetic positive leaf violates Apple's825-day lifetime requirement; a coherent
365-day leaf bundle correction retains production trust/verification and all negative cases.
Fresh exact-head review/full CI required, no merge/skip/sole-cause inference. I293 records gates.

#715 corrected head merged `d66e380a` with protected review6158318325, all six CI3173 jobs and
CAS6158600236; all13 TLS cases pass on macOS/Windows, old failedCI retained. Next I293 bounded
form transport prerequisite is Review:47 focused fixtures, no default activation/dependency/API
or upstream parser copy. Full local/remote gates and broader proxy/ambient/protocol/D/official-
root acceptance remain Pending; #643/#624 open. I293 records the authoritative stage evidence.

#716 original protected review6158827975 is retained; CI3175 failed Windows process-wide
handle counting (+11) while five jobs passed. A bounded test-only isolated-worker correction
preserves all40 faults and +2 threshold, using existing outer Job ownership. Fresh protected
review/native CI/CAS remain; no production process change, skip or retry-to-green claim.

CI3176 failed the isolated worker as well:73→76, original +2 assertion intact; five other jobs
passed. Root cause remains unknown. #716 prepares a bounded test-only41-sample trace, preserving
all40 attempts/threshold/cleanup. This is diagnostic evidence, not a fixed or merge-ready stage.

CI3177 trace failed cold73→76: first Pipe+1/first Assignment+2, later samples stable. Known cleanup
BOOL checks are prepared without new native calls; richer self-only metadata requires separately
accepted diagnostic scope/ADR/claim. No warmup or original-threshold change; #716 stays unmerged.

2026-10-15: #718 merged f4313237 establishes the separate accepted TEST-003/I302/ADR-091
self-only diagnostic scope. Recovered #716 candidate incorporates current main9b7ba281 and
keeps I293 search commits separate from I302 observer work. Native execution/review/acceptance
pending; original40/+2 and all failed history retained. Diagnostic collection does not fix the
original defect or authorize716 merge, default Search activation or full C/Epic closure.

CI3190 on published a39ee959 failed unchanged cold73->76; Windows453passed/1failed, five other
jobs passed. Successful I302 observer/native fixtures are diagnostic evidence only. New bounded
stage controls and typed-evidence interpretation remain in TEST-003/I302, with fresh protected
review/native CI required. Main23e659e2 is included without taking over I303/I304 work. I293/C
stays Active/Claimed/incomplete, #643/#624 open; production/default activation gates unchanged.

CI3197 on770d856e failed unchanged cold73->76; five other jobs passed, I302 modes1–4
succeeded. Preparation-only controls did not increase counts; direct API-return/cleanup
investigation stays in TEST-003/I302. C acceptance, #716 merge and #643/#624 closure remain
blocked by the unresolved original failure and existing activation gates; no Search change.

Local validation residual: the new full preflight also failed unchanged Linux fixture
`enclosing_deadline_covers_dns_and_response_wait_and_drops_socket` at its2s EOF receipt
(tools457pass/1fail). It lacks proof that the150ms request reached an accepted connection;
other read terminal states also lack receipts. Actual cause remains unknown. Preserve this
C-fixture residual and the failed run; no deadline/filter change or full-local-pass claim.

Follow-up is bounded failure telemetry only: retain up to4 read ErrorKinds and report accepted
count, up to4 request(length,headers-complete) shapes and EOF count when that assertion fails.
No content/paths/arguments are output, no error is counted as EOF, and150ms/2s/fixture10s limits
are unchanged. Revalidate the new diagnostic source; retain the prior failure even if it passes.
This is not a production fix or proven cause. Separate I293 commit; I302 owns Windows observers.

New diagnostic-source local full preflight passed3717primary+1existingchild and both SDK modes,
with only the prior authorized local filter; old457/1 failure remains. No repair/rootcause or
native pass is inferred. Fixed-source Windows cross-check/Clippy/graphs/gates pass; fresh exact
protected review/full unfiltered CI remain. I293 remains Active/Claimed/incomplete, #643/#624 open.

## Merged Bounded Form Stage — 2026-10-17

#716 merged as b5b8241e2d8fa72d6e469bd96ccbdf54ff851c69 after exact198cb751/tree29ac46f2,
all6CI3221 jobs, protected full-stage review and CAS6233550377. This adds only dormant private
bounded form transport and its tested TLS/redirect/authority constraints; no default WebSearch or
provider admission. Full C/EP/protocol parity, prepared-binding, D/privacy/root/rollback and actual
activation gates remain Pending; this owner stays Active/Claimed, #643/#624OPEN.
TEST004/I305 closes only its known-resource test criterion with existing merged evidence. #717
retains unexplained native resources and prior failures. Passing captured assertions are not raw
numeric receipts or cache/global-leak proof; #705 and completed I301/I303/I304 remain separate.

2026-10-17 next dormant prerequisite: I293 owns immutable, invocation-bound transport payload
preparation and consuming execution. Its production transport entry consumes the prepared URL/body
instead of accepting replacement raw parameters; test-only raw adapters retain the fixture suite. This proves ownership
and per-invocation object matching, not construction before approval or official Agent binding.
See the controlled-transport reference and I293 validation journal. Full C/EP/parity/D/root/rollback
and default activation remain Pending; #643/#624 stay open, #717/#705 retain separate residuals.
