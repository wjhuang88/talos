# Iteration I310: Native Google Gemini

> Document status: Active
> Published plan date: 2026-10-30
> Planned objective: complete child B, including AI Studio and Vertex OAuth.
> Baseline rule: preserve this scope; changed objectives require a new iteration.
> MVP deliverable: executable stateless Gemini image/tool Agent round on both transports.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | PROVIDER-008-B native Gemini wire with AI Studio and Vertex authentication only |
| Claimed At | 2026-10-30 |
| Source Issue | #810 |
| Governance Claim PR | #811 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized completing parent #724; distinct human reviewer unavailable in this session. Exact-head checks, independent protocol/auth review and merge CAS required. |
| Implementation PR | #838 (Draft; not accepted) |
| Last Updated | 2026-11-03 |
| Handoff / Release Condition | Claim effective via #811; retain Claimed until full child acceptance, exact-head CI/review and merge CAS. |

## Scope and deliverable

One stateless native Gemini Agent round over streamGenerateContent, with text, ordered multiple
images, parallel/sequential tools and results, reasoning summaries/signatures, usage and terminal
errors. AI Studio API-key and standard Vertex project/location OAuth are distinct transport
profiles of the same adapter. Preserve existing Chat/Messages/Responses and centralized protocol
selection/provenance. Catalog activation is separately reviewed from runtime implementation.

## Acceptance

- Given either configured transport, a real permission-gated mock HTTP Agent completes image/tool
  turns using Gemini contents/parts and correct authentication, without cross-wire retries.
- Given signed model parts, preserve opaque signatures at their original positions; keep real
  function-call IDs when supplied, generate unambiguous internal IDs only when absent, correlate
  results without relying only on function name, and prevent readable secret/signature disclosure.
- Given safety blocking, malformed/unknown events, missing terminal or invalid credentials, return
  bounded explicit outcomes without panic, silent success or credential relocation.
- Given expired Vertex credentials, refresh through the reviewed credential provider; service-account
  and local ADC flows must work. A static externally supplied token alone cannot close this scope.
- Given old configs, custom gateways and explicit protocol overrides, retain behavior and expose
  effective protocol/source through picker/browser/debug surfaces.
- Given frozen reviewed catalog metadata, select only confirmed Gemini protocol/transport pairs;
  preserve non-routing fields and suppress inference for replaced/custom endpoints.

## Validation and documentation

Pinned Rust with --locked; request tests include two distinguishable images in order, tools/results,
reasoning and output limits. Sourced recorded SSE fixtures with license/hash/provenance; proptest
byte/UTF-8/chunk/EOF boundaries; parallel tool association; config precedence/round-trip and
catalog matrices; real Agent image/tool rounds; credential expiry/refresh/redaction/destination
isolation tests. Standard ./scripts/release_preflight.sh, native exact-head CI, independent
protocol/auth security review and merge CAS. Update README en/zh, protocol reference, architecture,
SDK contract, unreleased notes and owner/Board/index/Issue. No live credentials in tests.

## Non-goals

Interactions API or server-side sessions, grounding/caching billing, built-in vendor tools,
Azure/Bedrock, active endpoint probing, general capability inference, release/tag. Vertex express
API-key mode is not silently substituted for standard OAuth. No shelling out to gcloud at runtime.

## Dependencies and decisions

Child A Complete (#806 merge5d366db2fbb63301cdce83369fc63fedd1f28c5a) supplies resolver/factory.
ADR-094 defines Gemini boundary; ADR-013/021/034/050/051/062/075 remain constraints. Auth library
version/features and transitive native dependencies require explicit audit before any dependency
commit; no dependency is introduced by this governance candidate.

## Iteration inventory and disposition — 2026-10-30

Inventory of all nonterminal iteration status headers on main d15c7688:
I249 Planned: defer dependency pilot, separate scope; I277 Review: retain deferred human Desktop
acceptance; I293 and I294 Active: retain Search owners, no overlap (open #808); I302 Review:
retain Windows attribution evidence; I308 Active: retain permission/MCP owner, no overlap.
I306 Complete/Closed is the satisfied prerequisite; I307/I309 are not nonterminal candidates.
No existing Gemini owner or overlapping implementation PR was found. Parent #724 stays open;
C Azure and D Bedrock remain unclaimed. This inventory does not transfer those owners.

## Execution / completion

Governance-only #811 proposes Claimed/Active atomically. Both remain ineffective until target
merge; the actual PR number is backfilled. Independent review and exact-head checks remain required.
Completion Commit: Pending. No code, dependency, catalog data or implementation tests in this PR.

### Remote recovery checkpoint — 2026-10-31

Maintainer explicitly requested timely remote code checkpoints after workspace rollback.
This checkpoint reconstructs bounded credential-file admission only. It rejects executable /
external / impersonated credentials and endpoint relocation before signer or network creation.
It is not wired into runtime yet, is not a merge candidate, and has no passing test claim.
Remaining: request-owned OAuth refresh/signing, reviewed dependency graph and ADR amendment,
config/factory, Gemini wire/SSE, full acceptance and native CI. Keep Active; completion pending.
Historical auth audit and recovery rationale are durably recorded in issue #810.

Recovered authorized-user token refresh, bounded response validation, shared on-demand cache,
request deadline and redirect rejection, with regression tests. No new dependency introduced.
This checkpoint remains unwired and unverified; service-account signer, explicit ADC discovery,
metadata isolation, in-flight cancellation tests and Gemini adapter remain outstanding.

### Verified recovery checkpoint — 2026-10-31

Pinned Rust 1.99.0: cargo test --locked -p talos-provider --lib completed with
214 passed, zero failed/ignored/filtered. Includes eight Gemini auth tests; malformed, oversized,
invalid expiry/type/header responses cannot enter cache or leak response secrets through errors.
This validates the library checkpoint only: no Gemini runtime integration or whole-workspace
acceptance is claimed. Service-account signing, ADC and native wire remain outstanding.

### Local ADC checkpoint validation — 2026-10-31

Implementation 017cca1f0d41ac9b719608d96ff400f4e974c78c includes deadline admission
and bounded local ADC selection. Verified source blob 27936bb1af61003ac895e764495b0e666fcb80c2.
Rust 1.99.0, cargo test -p talos-provider --lib --locked: 216 passed, zero failed,
ignored or filtered (8.87s). Offline repetition also passed 216 tests (9.54s).
Local ADC resolves caller-supplied paths only; runtime platform discovery, metadata credentials,
service-account signing and Gemini native wire remain incomplete. This is library validation,
not full Gemini acceptance, workspace preflight, or merge authorization. Keep Active.

### Service-account implementation checkpoint — 2026-10-31

Implementation d04c09b89d8d7da28f607f72134984683fe574f7 restores local service-account
RS256 JWT signing and request-owned token exchange/cache, with a generated test-only RSA key.
Exact google-cloud-auth 1.18.0 source checksum and feature/native findings are in ADR-094.
Independent read-only source audit confirmed no network/background work on this signer path,
and explicitly identified non-preemptible synchronous signing and panic/native limitations.
Rust 1.99.0: cargo test -p talos-provider --lib --locked: 217 passed, zero failed,
ignored or filtered (6.43s). Includes mock service-account exchange/cache and invalid-key redaction.
No full workspace, exact-head CI, implementation security approval or Gemini runtime acceptance
is claimed. Metadata ADC, platform path discovery, config/factory and native Gemini request/SSE,
signature isolation and Agent acceptance remain pending. Keep Active; no merge/release.

### Metadata transport checkpoint — 2026-10-31

Implementation cf38416c79e51f0d6a88881691c30aa8f1f16ec4 adds explicit metadata token
source: fixed link-local IP, no proxy, no redirects, required Google Metadata-Flavor response,
bounded response/expiry validation and shared request-owned deadline/cache semantics.
Rust 1.99.0 cargo test -p talos-provider --lib --locked: 219 passed, zero failed,
ignored or filtered (5.40s). New mocks cover flavor rejection, valid cache reuse and no redirect
forwarding. No real metadata service was contacted. Source reference checked 2026-10-31:
https://docs.cloud.google.com/compute/docs/access/authenticate-workloads .
This constructor is not yet wired into runtime ADC selection. Platform discovery, endpoint
validation/configuration, native generateContent/SSE and Agent acceptance remain incomplete.
Independent metadata implementation review requested; no merge approval is claimed.

### Official endpoint admission checkpoint — 2026-11-01

Added dedicated official AI Studio and Vertex regional/global endpoint constructors. Resource
components reject path/query/fragment/authority injection before credential acquisition. This
module is not yet connected to factory/config dispatch. Custom-gateway credential profiles
remain separate and unimplemented. Formatted recovered gemini_auth.rs with pinned rustfmt.
Pinned rustc --edition 2024 --test gemini_endpoint.rs: 2 passed, zero failed; this standalone
check validates endpoint logic only. Full locked Provider tests are compiling dependencies;
no fresh Provider/workspace pass or Gemini acceptance is claimed.

### Credential-isolated HTTP transport checkpoint — 2026-11-01

Added GeminiTransport composition: official AI Studio key-header profile and validated standard
Vertex ADC profile; destination validation precedes ADC discovery. Secret headers are sensitive,
redirects are disabled, token acquisition and HTTP dispatch share the supplied deadline, upstream
error bodies are not exposed. Body streaming/retry/progress and public factory remain pending.
Rust 1.99.0 cargo test -p talos-provider --lib --locked: 226 passed, zero failed/ignored/filtered
(6.55s). This validates latest ADC selection/path tests, endpoint tests and three HTTP transport
mocks. No live vendor/metadata request, full workspace validation or Gemini acceptance is claimed.
Independent transport source review requested. Next: native request mapping and SSE/Agent wiring.

### Native request mapping checkpoint — 2026-11-01

Added stateless contents/parts request mapping for system/context/user/model messages, ordered
inlineData images through the existing TOCTOU guard, functionDeclarations.parametersJsonSchema,
functionCall/functionResponse association by unique call ID and maxOutputTokens. Reject orphan,
duplicate and unresolved tool history. Native signed/thinking history replay remains explicitly
unavailable: this checkpoint rejects reasoning-bearing messages rather than relocating signatures.
Schema source: https://ai.google.dev/api/generate-content (FunctionDeclaration JSON Schema field
verified against official reference/Google SDK). No server-side session/caching fields are used.
Pinned locked Provider lib tests: 229 passed, zero failed/ignored/filtered (5.44s). Includes two
independent images with exact byte/order assertions and same-name parallel tools/results reversed
in completion order. Prior transport status-classification correction also passes in this run.
Request mapping is not yet factory/SSE/Agent connected; full Gemini acceptance remains pending.

### Stream semantic decoder checkpoint — 2026-11-01

Added GeminiStream for decoded GenerateContentResponse JSON: distinct text/thought summaries,
native ordered parts/signatures retained privately without Debug, supplied tool IDs preserved,
unique local IDs when absent, duplicate/malformed calls rejected, safety/unsupported finish outcomes
explicit, missing terminal EOF rejected, trailing usage included before one TurnEnd. Truncated tool
responses fail rather than execute. Content/usage counters are bounded. Native replay integration
is still pending; this decoder does not yet emit durable reasoning metadata.
Rust 1.99.0 locked Provider library tests: 233 passed, zero failed/ignored/filtered (6.04s), including
semantic fixtures and Unicode text-sequence proptest. Fixtures are authored synthetic cases, not
recorded vendor evidence. SSE byte framing/chunk-boundary proptest, sourced vendor fixtures,
timeouts, factory and Agent wiring remain required. Independent read-only decoder review requested.
No complete Gemini/parent acceptance or merge authorization is claimed.

### Bounded SSE framing/reader checkpoint — 2026-11-01

Added GeminiSse byte framing with LF/CRLF/CR, arbitrary UTF-8 splits, 1 MiB record/line and
8 MiB total byte limits. Invalid UTF-8/JSON, incomplete record EOF or missing semantic terminal
fails closed. Function calls are retained until successful semantic EOF; late errors discard them.
Added request-owned HTTP response reader with fixed first-event, idle-read and shared total
deadlines plus bounded channel delivery; no detached read task. Heartbeat/comment packets do
not reset the first-event deadline. Runtime factory, signed replay and Agent acceptance pending.
Pinned locked Provider lib: 239 passed, zero failed/ignored/filtered (5.68s). Includes every UTF-8/
CRLF split, arbitrary chunk-size proptest, truncation/size errors, delayed tool release and loopback
heartbeat-first-event timeout. Other HTTP reader failure/cancellation/idle cases remain to extend.
Synthetic fixtures do not replace required sourced vendor samples. Independent reader review requested.

### Identity-scoped replay envelope checkpoint — 2026-11-01

Added private Gemini replay encoding/decoding in ReasoningBlock::Redacted with a versioned
wire-family marker and exact provider/model identity. Preserve full ordered original parts and
signatures; keep generated internal tool IDs in a separate ordinal binding table. Reject native
ID mismatch, missing/duplicate/out-of-range associations, foreign identity/format and oversize or
unsupported payloads. No Debug implementation on decoded private payload. Tests added for positional
round-trip and rejected associations, but execution remains at dependency restoration after workspace
maintenance pruned the previous build environment. Pinned rustfmt and diff whitespace checks pass.
This helper is not yet connected to SSE completion/request replay or Agent persistence; no passing
new test, runtime or complete signature-replay claim. Next: finish validation and wire the envelope
through response completion and identity-checked request history.

### Replay diagnostic confidentiality fix — 2026-11-01

Independent source review found that ReasoningBlock's derived Debug exposed Redacted envelope
content, including Gemini signatures. Replaced it with variant-preserving redacted Debug for
Thinking text/signature, Redacted data and Plain text, thereby protecting derived message/event
Debug too. Serde and lossless persistence are unchanged. Added core nested-message/event and
serialization regression plus concrete Gemini envelope Debug assertions. Independent read-only
review confirmed the original leak path is closed; it is not a merge approval or a claim that
all other message/tool strings are private. Rust 1.99.0 cargo test -p talos-core --lib --locked:
123 passed, zero failed/ignored/filtered (0.02s). Provider replay tests restarted after earlier
CONNECT proxy failure; no fresh Provider pass is claimed. SSE/request signed replay wiring remains
pending. No Complete, release or parent acceptance claim.

### Response-to-request replay composition checkpoint — 2026-11-01

Response semantics now retain internal-call-ID to original-part-index bindings. Only successful
semantic EOF permits replay encoding; SSE completion can stamp actual supplied transport identity
and emits private ReasoningComplete before the held tool calls and TurnEnd. HTTP reader requires
provider/model identity rather than deriving authority from payload contents. Added signed stream
through serialized/restored history into the next native request, including native and generated
IDs, reversed same-name tool completions and exact original-part comparison. Latest request-side
checkpoint 489ddcc also batches results in original call order to preserve idless associations.
Pinned rustfmt and diff checks pass. Locked Provider tests restarted and are compiling dependencies
after workspace maintenance; no fresh test pass claimed. Factory/LanguageModel/real Agent and
recorded vendor fixture acceptance remain outstanding; no completion or merge claim.

### Internal LanguageModel composition checkpoint — 2026-11-01

Added crate-private GeminiProvider composing request/replay, admitted streaming transport, SSE and
legacy native-tool ingress. Constructor checks transport model/stream mode and nonempty provider
identity. Default trait paths refuse unsupported browser/ephemeral/decision/compat modes before
network. Single dispatch only (reported max_attempts=0): retry integration remains pending.
One owned reader/pump task observes receiver closure, retains terminal events until successful
reader completion, and bounds text/terminal delivery by the shared deadline. Error notification
uses try_send to avoid indefinite backpressure. Added HTTP LanguageModel tool/replay/usage test,
nonstream/mismatched construction rejection and stalled-consumer deadline regression.
Independent review raised backpressure/stream-mode findings; corresponding fixes and tests were
added. Pinned rustfmt/diff checks pass, but Cargo is still restoring Git dependencies after toolchain
recovery: these new tests have not yet completed. Factory/config and real Agent acceptance remain
pending; this internal adapter is not a public completed integration or merge candidate.

### Internal adapter validation and shared retry — 2026-11-01

Rust 1.99.0 cargo test -p talos-provider --lib --locked completed: 250 passed,
zero failed/ignored/filtered (4.74s). Validates internal LanguageModel HTTP signed-tool/usage
composition, stalled-consumer deadline closure without late tool release, transport model/stream
admission and bounded 429 retry/progress ordering. Latest adapter reuses shared retry classification
and backoff only before response headers, preserving one total deadline across acquisition, sends
and backoffs. First-event timing starts at the successful dispatch attempt; no SSE/body error is
retried, no cross-wire fallback. fmt/diff checks pass. Independent current-source follow-up requested.
This remains an internal adapter: configuration/factory, capabilities/catalog, browser/ephemeral/
decision completeness, real permission-gated Agent validation and full CI/preflight remain pending.

### Explicit profile/factory checkpoint — 2026-11-01

Implementation e83501a18c54812ee0d6db4ae803b44a44682c10 adds google-gemini configuration
with explicit AI Studio or Vertex project/location/metadata-opt-in profile. Existing provider
literals retain gemini=None; no catalog/default auto-activation. Shared factory refuses arbitrary
AI Studio origins and all replacement Vertex ADC URLs before credential discovery. Vertex validates
resources synchronously and lazily initializes/caches admitted transport during request deadline,
including OnceCell wait. Unsupported reasoning options are explicitly rejected, not silently ignored.
Added config round-trip/unknown-field/old-default and factory profile/relocation/missing-profile tests.
Pinned rustfmt/diff checks pass. Locked config/provider check is compiling; tests await build lock.
No new passing result, public host/CLI credential flow or full Gemini acceptance is claimed.
Next: resolve compilation/tests, host credential and capability integration, actual Agent acceptance.

### Profile/factory integration corrections and validation — 2026-11-01

Implementation 4ec9f7bb9eb0de520023ac7b15f014c14a480c96 corrects two findings: effective
provider configuration now merges the explicit Gemini profile; API-key resolution/validation exempts
only effective google-gemini + Vertex profile so existing CLI/Desktop key entrypoints can reach
request-owned ADC. Old protocol/AiStudio without key remain rejected. Explicit Vertex endpoint
replacement is rejected while irrelevant inherited default roots are not used for its transport.
Recorded failure: first Provider run 250 passed/1 failed because profile merging was missing.
Final pinned locked library tests: config267 passed (0.91s), provider251 passed (4.78s), zero
failed/ignored/filtered. Locked config/provider check passes; test-only helpers restricted to cfg(test)
remove four production dead-code warnings. No full host/workspace/real Agent/Vertex round-trip
acceptance or catalog activation is claimed. Next: host builds, model/capability displays, real
permission-gated Agent image/tool rounds for both profiles and remaining acceptance checks.

### Real Agent acceptance fixture checkpoint — 2026-11-01

Added controlled HTTP tests using the real Agent, PermissionEngine and workspace ReadTool, with
two distinguishable granted PNG images, signed idless read call, actual tool result and follow-up
native request preserving the signed model part. Both key-header AI Studio and explicit local
ADC refresh/Bearer Vertex credential profiles are covered; token cache is expected to refresh once
across both Agent requests. Assert no key/access/refresh/client-secret in returned history and no
signature in Debug. Auth/destination test injection is cfg(test)-only. The tests build the admitted
internal provider directly, not public factory-to-host wiring, and make no live cloud requests.
Pinned rustfmt/diff checks pass. Cargo tests are still fetching dependencies after toolchain recovery;
no new passing Agent or Provider result is claimed. Keep fixture work unaccepted until executed.

### Controlled real Agent round validation — 2026-11-01

Provider library run: 253 passed, zero failed/ignored/filtered (5.36s). After exact image-sequence
matcher strengthening in fe5e3197ecd4e0fb484e937426c90dde4284233d, pinned locked targeted
Agent tests rerun: 2 passed, zero failed/ignored, 251 filtered (0.08s). Final fixture blob
008c1b978b227e0cadc9e7a435e4f16adfd17e40 matches the remote version. Covers AI Studio and
explicit authorized-user ADC refresh/Bearer profiles through real Agent+PermissionEngine+ReadTool,
two exact ordered images, signed next-turn replay and credential-free returned history.
Independent read-only fixture review found no new blocker. This is controlled local HTTP and
serde history evidence, not live cloud, actual TLOG recovery or public factory Vertex lazy-path
acceptance. Test-only source/endpoint injection bypasses factory for the Agent fixture. Remaining
host/factory Vertex entrypoint, capabilities/decision/ephemeral/browser, vendor fixtures, full
workspace preflight, exact-head CI/review and parent requirements remain open.

### Factory ADC host-admission validation — 2026-11-01

Implementation add3fcef3795291477a4a9375d42effe0886c5a4 adds isolated subprocess verification
of the public factory and Config validation/API-key entrypoint: explicit Vertex profile needs no
API key or HOME/APPDATA, factory defers invalid selected ADC until dispatch, and invalid explicit
file returns its static admission error without metadata fallback/path disclosure. No process-global
environment mutation is used. Provider lib: 255 passed, zero failed/ignored/filtered (5.34s), including
the child probe and parent verification. CLI locked check passed. The joint default CLI/Desktop
command skipped the feature-gated Desktop binary; it is NOT Desktop evidence. Correct locked
Desktop check with --features desktop-ui started separately and has no final result recorded yet.
This covers negative ADC admission, not a successful factory Vertex cloud endpoint exchange.
Remaining positive lazy-path/host, capability/decision/ephemeral/browser and full integration gates
continue to block completion. No merge/release/parent acceptance claim.

### Isolated decision/permission-review support — 2026-11-02

Implementation e86d63462390b97a4bcbcd344bcf5ef1dcacef34 supports bounded stream_decision
and inherited stream_auto_review delegation. Shared validation permits only System/User text and
positive token caps; cloned output/retry limits do not mutate conversation settings. No native
tools are declared. Private thinking/replay events are suppressed, and unexpected returned tools
produce failure without ToolCall/TurnEnd. Tests cover actual thought/signature filtering, output
cap and unchanged normal conversation budget, zero retry on 429 and assistant-history refusal.
Independent current-source review found no new blocker. First build failed E0460/E0463 due stale
shared talos_core-linked workspace artifacts; cleaned workspace crate build artifacts and reran.
Final Rust1.99.0 cargo test -p talos-provider --lib --locked: 257 passed, zero failed/ignored/
filtered (5.06s). Source and tests formatted; no claim of a new Desktop UI result (prior log lost
with workspace maintenance). Remaining ephemeral/browser compatibility, host/UI/catalog, vendor
fixtures, full preflight and precise CI/review continue to block Gemini/parent completion.

### Request-owned Gemini PNG checkpoint — 2026-11-02 (not accepted)

Restored remote 5cd7380ee6d6cd3b5b32e239b76e16a08776c1e0 and rebased all 43
Gemini checkpoints onto main 9ace8968603a67ada8d118459c81367611ad3fd2; range-diff
retains each existing change. Original dirty workspace is untouched. Native PNG ingress now
consumes request-owned EphemeralImage values into contents/inlineData using shared 32-image /
16 MiB admission. Earliest expiry bounds lazy ADC, token acquisition, headers and same-wire
retry/backoff; an admitted SSE response retains its original request/idle deadlines. History
and preview projections receive no image payload. Unsupported protocol/browser combinations
still fail before dispatch; browser original-argument support remains pending.

Added controlled tests for ordered two-image HTTP projection/no subsequent replay, expired and
unsupported admission, count/aggregate size limits, stalled headers versus delayed SSE body,
stalled Vertex token cancellation/no generation dispatch, and expiry during retry backoff.
Pinned Rust 1.99.0 restored; Provider --lib --locked test command started after downloading
fresh dependencies and is still compiling. No current test pass, full preflight or CI result
is claimed by this checkpoint. Initial independent read-only source review found no blocker
and requested the two deadline regressions now added; final source/test review remains pending.
Git CLI push dry-run failed because this host has no GitHub username credentials; use the
connected GitHub Git-data API with exact expected-head lease to persist checkpoints.
Completion Commit: Pending. No Gemini/parent completion, PR merge, release or tag.

### Request-owned PNG validation result — 2026-11-02

Remote implementation checkpoint f321da9f3dfee0c810802023d76754bfb0615ce9 has exactly
the same Git tree (abd6059378ad14bbb839614126cbd1809b6d47db) as locally reviewed
4fa0bb7d65cdd60be9f33b78ceba8131c2d65262. All 43 existing checkpoints were
replayed on main 9ace8968603a67ada8d118459c81367611ad3fd2, with identical per-commit
range-diff patches; each Git-data reconstructed tree was checked before exact-head CAS update.
No concurrent branch changes were overwritten.

Pinned Rust1.99.0 cargo test -p talos-provider --lib --locked: 263 passed, zero
failed/ignored/filtered (6.22s); repeated on final local source: 263 passed (6.13s).
Config/Core --lib --locked regression: 267 and 123 passed respectively, zero failures.
Formatting and staged whitespace checks passed. Independent final read-only source review
found no new blocker; verified bounded test receipt waits, delayed response beyond image expiry,
HTTP header timeout and in-flight Vertex token cancellation with zero generation requests.
That token test uses a prebuilt credential source and does not validate lazy ADC initialization.

Desktop check with explicit --features desktop-ui failed at yeslogic-fontconfig-sys because
pkg-config is absent; this is a host build prerequisite failure, not a passing Desktop result.
Initial apt-get update failed due unsupported setgroups/setuid in this host; environment repair
is being attempted separately. Standard release_preflight.sh is running; installer and project
governance checks have passed so far, but no final preflight/workspace/CI result is claimed.
Gemini remains Active / Claimed, Completion Commit: Pending. Browser argument integrity,
compatibility modes, reasoning/model support, host/UI/catalog integration, sourced fixtures,
full validation and exact-head review/CI/CAS still block completion and parent #724 closure.

### Full-check fixes and Desktop feature build — 2026-11-02

First standard preflight passed project/collaboration/text/classifier/projection/format checks
and cargo check --workspace --locked, then failed Clippy on two doc_lazy_continuation lines
in the Gemini api_key documentation. Fixed paragraph separation; the next Clippy run exposed
one existing nested-reader collapsible_if in GeminiProvider. Collapsed it without changing
short-circuit/error behavior; independent read-only diff review confirmed no semantic change.
Final cargo clippy --workspace --locked -- -D warnings passed (50.81s including lock wait).

Desktop's first feature-enabled check failed because pkg-config/fontconfig were absent.
APT's default user drop is unsupported here; initial install also hit the system cache's
permissions. Downloaded the repository-CI development packages to a task-local cache and
extracted them to a temporary sysroot, without changing Cargo dependencies or Desktop features.
With task-local PKG_CONFIG/PATH/SYSROOT/LD_LIBRARY_PATH, cargo check -p talos-desktop
--features desktop-ui --locked passed (1m03s). This proves compilation, not rendered UI or
live-provider acceptance. The initial failure remains part of the validation history.

Standard release_preflight.sh rerun has started after these fixes; full workspace tests and
its final status are still pending. No final CI, merge, release or Gemini completion claim.

### Catalog contract foundation and workspace Git admission — 2026-11-02

Added GoogleGemini to the already non-exhaustive shared CatalogProviderProtocol and its explicit
config mapping. A mixed Gemini/legacy provider TomlDataset regression verifies the new metadata
does not prevent loading the entire dataset and preserves the endpoint and old wire mapping.
No models.toml, protocol inference table, provider default or catalog activation is changed.
Independent read-only diff review found no blocker. Focused pinned/locked Config regression
passed (1 test); running its newly built full Config binary passed all 268 tests (1.95s).
The full locked Config/Core rerun is pending; this does not supersede earlier 267/123 evidence.

Full preflight's workspace tests stopped in talos-agent: 586 passed and one Git Auto-review
fixture failed because the entry Git canonical path differs from its internal maintenance
executable. On this host /usr/local/bin/git and /usr/local/libexec/git-core/git are hardlinks
but remain distinct canonical paths. The unchanged failing test binary passed when PATH starts
with the internal Git directory (1 passed, 586 filtered; 13.53s). No Git authority checks or
permission source was modified. A Cargo focused rerun is also in progress; rerun full preflight
with this explicit host PATH. Retain the failed baseline; do not claim workspace tests passed.

### Draft implementation PR and updated focused evidence — 2026-11-02

Opened Draft #838; backfilled both owner claim records and the Board. This is an unfinished
implementation candidate, not Ready/Complete. Latest catalog foundation 090ec083 has the same
Git tree as locally reviewed f3f093dc. Full locked Config/Core rerun: 268 passed (1.92s) and
123 passed (0.06s). The unchanged Git fixture also passed its pinned/locked Cargo rerun with
internal Git PATH (1 passed, 586 filtered; 15.30s). Full standard preflight using that explicit
PATH is running; do not equate the focused recovery with a workspace pass. Draft PR lists all
remaining acceptance blockers. No merge/release/tag or live-provider success claim.

### Local disk recovery — 2026-11-02

The next standard preflight reached workspace test compilation but failed with OS error 28
(No space left on device), before a final test result. Target cache had 37 GiB deps, 13 GiB
incremental and 1.8 GiB build artifacts after focused/workspace/Desktop feature variants.
Removed only this task's target directory; source, Git history, Cargo.lock and other worktrees
were retained. Free space returned to 53 GiB. Rerun with the same debug=0/incremental=0 profile
settings already used by repository CI (ci.yml), plus bounded build jobs and internal Git PATH.
This is an artifact-resource change, not a test skip or assertion/policy change.

Owner/Board backfill validation initially failed because cargo was absent from that shell PATH;
a retry during disk exhaustion produced no usable log/result. Fresh standard preflight must
revalidate both governance scripts with the pinned Cargo PATH. No governance/preflight pass is
claimed for the new metadata checkpoint yet. Draft #838 remains unaccepted; CI3732 was observed
in progress at source head 090ec083 and must not be treated as the final-head CI result after
claim metadata changes. The previously paused automation stays paused until reliable execution
is established; no background continuation is claimed.

### Confirmed-model reasoning controls — 2026-11-02 (not accepted)

Added exact-ID native thinking policy for Gemini 2.5 Pro/Flash/Flash-Lite budgets and
3.8 Flash / 4 Argon low/medium/high levels, sourced in ADR-094. Unsupported aliases,
unknown explicit settings, mixed budget/effort and replay=false fail before generation.
Factory catches replay=false before capability suppression; explicit reasoning_supported=false
suppresses thinkingConfig on both ordinary and isolated requests. No new dependency/schema
field or catalog activation. Existing output caps remain separate and include thinking tokens.

Isolated decisions use their own minimal controls (Flash/Lite 0, Pro 128, level models low,
includeThoughts=false), preserve the caller's bounded output/retry limits and filter private
reasoning events. HTTP regressions cover all three budget models and both level models, then
ordinary requests verify conversation controls are unchanged. Added invalid budget/combined
control/alias matrices, capability suppression HTTP and factory admission checks. Configuration
field documentation and protocol reference were updated; remaining B acceptance stays open.

Recovered prior preflight log ends during dependency compilation and its execution handle no
longer exists; there is no completed preflight result. First current focused Provider run failed
before tests with E0786 corrupt Cranelift metadata from the interrupted cache. Cleaned only
cranelift-codegen and wasmtime-internal-unwinder task artifacts and started a pinned --locked
Provider rerun with CI debug=0, incremental=0 and two build jobs. Result pending; do not reuse
prior 263 as proof of these new controls. Initial independent read-only review found no source
blocker and requested the HTTP matrices added above; final diff review remains in progress.

Draft #838 remains unaccepted. Remote CI3733 was pending at unchanged head 26a91246 before
this new checkpoint; it is not validation of subsequent reasoning changes. No merge/release/tag.

### Main synchronization and native CI recovery — 2026-11-02

Recovered remote c042a1a045c59bd07869e3e2fb6e0808c7ca19e6 after workspace maintenance
removed the newer task worktrees/toolchain/logs. Original dirty talos snapshot was not changed.
Main advanced to d324c3348f1bb64a3a7b0a0682e14fc97c6293c3, including #832's missing
Issue-owner matrix row. Rebased all 49 feature commits without conflict; every range-diff row
is unchanged. This incorporates the main fix rather than editing unrelated Search ownership.

Exact old-head CI3735/run41397974111 finished: six checks passed, two failed. Windows workspace,
Windows Rust workspace, Windows/Linux Desktop feature checks, installer and classifier passed.
The macOS job125129225344 had Provider 267 passed/one failure: cancellation produced native
ConnectionReset (code54), but its test required clean EOF. Global job125129175751 failed on
missing #832 matrix registration, now incorporated from main. No all-green CI claim.

Restored the pending test repairs: cancellation accepts only bounded EOF or ConnectionReset,
retaining cache-lock/cache-empty checks; identical SSE failure branches are collapsed without
changing the terminal-state assertion. Both diffs previously received independent read-only
review; macOS and final-head CI must rerun. Previous reasoning source c042a1a0 locally passed
268 Provider tests twice (6.42s/8.79s); its all-target Clippy exposed the SSE duplicate branch.
The subsequent fixes run was interrupted by workspace maintenance and has no recoverable result.
Pinned toolchain and new focused validation are being restored; no pass is claimed for these
new test repairs or the rebased candidate yet.

Browser entry remains refused. Read-only audit found that enabling Original dispatch alone
would leak raw browser arguments through the opaque JSON replay block into durable TLOG, and
would mismatch Agent's protocolVersion-only safe ToolCall input on the next signed round.
A model-private transient replay/persistence boundary and original JSON routing-key validation
must be designed and verified together before this guard can be removed; do not put raw browser
arguments into durable ToolCall.input or treat ReasoningBlock::Redacted as encryption. Draft
#838 and parent #724 remain open; no completion, merge, release or tag.

### Original native arguments checkpoint — 2026-11-02 (not accepted)

Recovered remote head 5001b7b4 and confirmed main d324c334 is already included; no additional
rebase or overwrite was needed. CI3741/run41407646766 has seven successful jobs; the macOS
job remains running, with its Release preflight step successful. This is not an overall CI pass.

Implemented a typed RawValue ingress before Value normalization. Duplicate routing fields,
including escaped equivalents and null-first duplicates, are refused without payload diagnostics.
Original function args retain whitespace, escapes, duplicate argument keys and numeric spelling
through the SSE-to-invocation boundary. Same-record single-candidate/ordered-part association
retains actual IDs and generated positional IDs. No tool is released before validated EOF.
Tests cover every byte split, parallel calls, missing args and clearing buffered calls on malformed
routing. Each duplicate fixture has a normalized-Value success control, so old terminal checks
cannot masquerade as duplicate-field protection.

Independent read-only Agent source review approved this bounded change after correcting a test
that originally sent duplicate records after STOP. Pinned Rust1.99.0 formatting and diff whitespace
checks pass. Locked Provider tests are still cold-compiling with CI debug=0/incremental=0/jobs=2;
no passing test result is claimed for this checkpoint. Browser ingress remains refused: transient
signed replay versus durable TLOG projection, compatibility modes and complete B acceptance still
require implementation and tests. #838 stays Draft; Completion Commit remains Pending.

Validation completed for original-arguments source checkpoint
`4c27bb2ce3ce7250e9e76daa88e94f3b441b43f5`: Provider locked library tests **270 passed,
0 failed** (4.93s); Provider locked all-targets Clippy with `-D warnings` passed (1m10s).
Source blobs independently reviewed: gemini_stream `93e90d4a`, gemini_sse `7cd8b64d`,
gemini_provider `242da570`; no remaining blocking finding in this bounded stage.
Project governance initially failed because the shell PATH omitted Cargo. After exporting the
pinned toolchain and Git internal executable paths, governance passed with zero warnings;
collaboration claim validation also passed with zero warnings. No permission check was weakened.
New exact-head CI3743/run41411287919 is pending, not passed. Earlier CI3741 still has seven
successful jobs and a running macOS job; do not infer its final outcome from the preflight step.
Next implementation gate remains transient browser signed replay and durable privacy projection,
then compatibility, positive lazy factory/ADC, catalog/host/UI/fixtures/docs and full acceptance.

### Positive lazy factory/ADC Agent acceptance — 2026-11-02

Remote base 96c4bee1 completed CI3744/run41411570761 with all eight jobs successful,
including native macOS preflight, Windows workspace and Linux/Windows Desktop. This does not
complete B. Later main c2d8b441 adds only SEARCH-001-H/I314 planning/activation documents;
AGENTS and runtime sources are unchanged. I314's effective claim explicitly preserves I310's
non-overlapping ownership. Per ADR-071, unrelated governance alone does not force a rebase.

Added isolated-process positive acceptance through the public factory and real Agent/PermissionEngine/
ReadTool: explicit authorized_user ADC, standard well-known user ADC, and explicit service_account.
The selected ADC file is absent when the factory is built, created before first dispatch, and removed
by the token fixture after initialization. The signed native tool continuation must succeed using exactly
one token exchange and two Bearer generation calls, without an API-key header. This exercises lazy
OnceCell/from_environment initialization and cached source/token reuse rather than manually injecting
an already-created credential source. Child processes clear inherited proxies and use loopback only.
The endpoint seam is cfg(test) at both module and call site; production admission is unchanged.

Checkpoint a0a9715c deliberately preserved uncompiled/unformatted code after toolchain/log loss.
Restored pinned Rust1.99.0 under the task workspace. The first build failed before tests because
an older target cache lacked serde_core rlib. Retained that failure and used a separate task-owned
CARGO_TARGET_DIR with CI debug=0/incremental=0/jobs=2; no dependency or assertion was weakened.
Clean focused acceptance passed (2 tests; parent covers all three credential/path cases, 2.33s).
Full locked Provider library regression then passed **272 tests, zero failures** (5.33s); locked
all-targets Provider Clippy with -D warnings passed (1m18s); rustfmt and diff whitespace passed.
Independent read-only source review approved final fixture blob 8fc77a00 and the cfg(test) seam.
Both governance validators passed with zero warnings. New exact-head CI is still required after
the formatted/evidence checkpoint; no merge is authorized.

Browser remains refused. Next implementation needs a nonserializable invocation replay carrier,
run-local bounded ownership and planner-bound request attachments; ReasoningComplete/Redacted are
durable and must not hold browser raw arguments. Hook edits, transcript pruning, token budgeting
and missing continuation on resume must be handled without reconstructing signed arguments.
Compatibility, host/UI, separate catalog activation, sourced fixtures/final docs and whole-candidate
review/CI/CAS remain open. Active/Claimed, Completion Commit Pending; parent #724 stays open.

### Successor-session recovery and ordinary textual tools — 2026-11-03

Recovered remote #838 head1eabe26f23b328a9b0e02c66e20c3b2b7b9a0f0f and main
58d8bf926beefe24df4e04c3d9c98f5ca62636cd into a new worktree. CI3788/run41624864516
has a successful workflow aggregate and seven successful native/classifier/installer/Desktop
jobs. The separate global Issue-owner reconciliation job failed on missing #844 and stale #390;
retain that advisory residual under ADR-071 rather than discarding successful implementation
validation or rerunning it solely for unrelated matrix drift. No merge or completion follows.

Added Compat/TalosStrict ordinary tool dispatch over the same Gemini wire, no native tool
declarations and no cross-wire fallback. Signed model text/parts are unchanged on continuation;
parsed call names/arguments/order and explicit wire IDs must match retained tool metadata.
ID-less legacy frames use retained local IDs, and duplicate/orphan/missing results fail before
credentials/HTTP. Signed native-tool history cannot be rewritten into textual mode. Browser
registry names remain refused in all modes; this does not implement private browser replay.
The additional bridge obeys the original absolute deadline even for a stalled receiver, cancels
upstream on receiver drop and rejects unterminated/malformed/native-call textual responses.
Request-local images retain order and one-request lifetime across all three explicit modes.

Pinned Rust1.99.0/locked Provider library tests passed twice: **282 passed, zero failed**
(5.64s, then 5.91s for final source including three-mode image coverage). Real Agent + actual
PermissionEngine/ReadTool fixtures cover both textual modes and signed tool continuation.
New regressions cover edited wire IDs/arguments, orphan/duplicate/missing outcomes, ID-less
legacy frames, malformed/strict batches, blocked-consumer deadlines and browser/Auto admission.
Both governance validators passed with zero warnings; rustfmt/diff whitespace passed.
The initial cold build failed archiving cranelift-codegen because one object file had length zero;
cleaning only that package's task artifacts and retrying passed. Keep the failure in this record.
Independent Agent source review approved bounded blobsff5b1346 (compat), b51f56b8 (provider),
81693f3f (Agent fixtures), 0a13e349 (module wiring). Shared repository identity; no human or
whole-candidate review claim. The first all-target Clippy run found two cloned-reference test-slice lints; replaced them with
std::slice::from_ref without changing assertions. Independent review approved the final compat
blob0a7ef648 after this equivalent test cleanup. Provider all-target Clippy then passed
(1m59s including lock wait). Staged self-review additionally added duplicate explicit ID batch
rejection before releasing any tool; independent review approved final compat blobff5b1346.
The final focused rerun is queued behind standard workspace preflight. Workspace check and
Clippy passed; full workspace test compilation is still running, so no preflight pass is claimed.

Resume from the feature branch/PR, preserving old-head CI evidence. Finish current validation,
then push one stable code/docs stage and obtain its fresh native CI. Next acceptance work remains
browser nonserializable run-local replay and durable privacy/planner binding, final host/UI,
separately frozen catalog activation, sourced fixtures and full protocol/auth review/CI/CAS.
B stays Active/Claimed, Completion Commit Pending; C/D and parent #724 remain incomplete.

Validation disposition for the compatibility candidate: final focused Provider **282/0**
(6.07s) and all-target Clippy **passed** (7.19s); source is checkpoint
5c2cde9c19e4d9b6183d0f31b800e49d479eb271, exact tree0ccac363. Standard preflight passed
workspace check/Clippy, then stopped after **1,974 passed / 2 failed** in unchanged MCP
checkout::case_04_approved/denied: AF_UNIX socket fixture creation returns EPERM. Both fixture
files match main58d8bf92 exactly. A standalone Python AF_UNIX socket creation also returns
EPERM before any Talos code. The remaining-package run exposed the same host restriction in
runtime_evidence_rejects_directory_symlinks_and_socket_entries (88 runtime tests passed,
1 failed). Do not weaken these assertions or claim a standard-preflight/workspace pass.

Remaining packages are being collected with --no-fail-fast to identify every host-blocked row;
remaining MCP integration targets and the standard external SDK fixture are also running.
These unfinished runs are not pass evidence. This host restriction is not a reason to repeat
known successful checks or discard CI3788. Submit this source-reviewed, focused-tested Draft
stage for fresh native CI; do not merge until the complete B acceptance and native gates pass.
Record later validation in PR metadata while its head is unchanged, and batch owner evidence
with the next substantive stage rather than interrupting native CI with documentation-only pushes.

### Native browser private-history implementation — 2026-11-04 (unverified checkpoint)

Resumed from validated PR head9d1771ce; CI3793 finished with successful aggregate and all seven
applicable jobs successful. Preserve advisory #844/#390 matrix failure and all historical local
failures; this is not an eight-job-success claim. Current main26160224 adds I315 governance only;
AGENTS/collaboration SOP unchanged. The effective I310 claim remains the same Work Slice.

The working stage adds a nonserializable provider replay carrier, safe marker/byte reserve,
Actor-owned live custody, exact post-hook and final-request binding, native Gemini original
argument ingress and private signed restoration only at HTTP encoding. Live Actor user turns
retain custody; reopened/forked/generation or persistence identity replacements do not inherit it.
Request-only compaction does not reclaim the hard32-block/16MiB canonical cap. Ordinary text
without an actual browser call/private context remains ordinary durable history. Browser textual
modes remain explicitly refused; this is native Gemini integration, no new browser capability.

Independent design/source review requested corrections, not approval: turn-local custody was
replaced by Actor custody; retained private filters now cover later text/tool-free replies and
mixed ordinary inputs; existing non-rescanning privacy filtering replaces recursive replacements;
private marker prefixes and unknown operations have explicit checks. Private data in routing
IDs/names/object keys is refused before observations, and original argument evidence must match
the same signed private call. These last corrections await final review/tests. Native tool
permission gates remain on the existing prepared-invocation path.

Core initially128 and Provider282 passed at earlier intermediate source before these final
privacy corrections; do not use them as final-stage proof. Current Core/Agent privacy tests and
new HTTP→two live Actor submissions→prepared browser→exact signed continuation→real TLOG and
JSON/Markdown export→reopen refusal test are being compiled. The old /tmp logs and root toolchain
were removed during the session transition; restored task-owned pinned Rust1.99.0 and new logs
under the sibling talos-build directory. Cargo.lock adds only the existing local talos-session
dev-dependency edge for the persistence acceptance test; no package version/source change.

Public API amendment/migration is in ADR-094: invocation enum becomes non_exhaustive with one
private carrier variant, and a default provider context entrypoint refuses unsupported custody.
Next pre-1.0 minor only, no patch/release claim. This checkpoint is UNVERIFIED and must not be
merged or made Ready. Finish focused acceptance, Core/Agent/Provider regression/Clippy, both
governance validators and independent final privacy review locally; then batch one substantive
PR candidate for fresh native CI. B/parent724 and remaining catalog/UI/fixtures/C/D remain open.

### Private replay checkpoint follow-up — 2026-11-04

Checkpoint 7d2cea27 added the mixed ordinary/browser test. At that exact source, pinned locked
Core 129, Agent 663 and Provider 283 library tests passed, as did both governance validators and
format/diff checks. The clean build recovered from registry rate limiting. The first Clippy pass
found new-source style and argument-count findings. The independent technical review requested
changes for unknown field-name disclosure, retroactive rewriting of sealed projections, and
ordinary input normalization before signed binding. This checkpoint is not a merge candidate.

Follow-up covers unknown-key refusal before observation, custody-verified stable projections,
and binding the original ordinary argument before the existing execution normalization step.
Tests and review of the follow-up must be recorded separately from the results above.

At ed1c5c8c the clean pinned locked library run passed Core 129, Agent 664 and Provider 284
tests. This includes both HTTP browser regressions and the custody-verified snapshot test.
Independent source review confirmed the three corrections at tree
5ae21de4297ea5e2e70a94be261f6b737706c64b. Its approval was bounded to those corrections.
A subsequent all-target Clippy run flagged three test-only findings (deprecated fixture
constructor, unnecessary clone, nested condition). The fixture-only fixes passed all-target
Clippy for Core, Agent and Provider with warnings denied. Standard release preflight was
started on the follow-up. Do not infer a workspace/preflight or final Gemini acceptance pass.

Standard preflight at 5349cd3c passed the workspace check and Clippy. Workspace test compilation
completed, but the I169 dispatch source guard still expected the old image-only provider method.
The guard is updated to require the sealed messages, tools and protocol at the new dispatch entry,
and to verify that replay is bound against those same messages before dispatch. Both guard tests
pass locally. A standalone AF_UNIX probe still reports EPERM on this host. This does not account
for the I169 failure, which was a source regression in this change. The workspace no-fail-fast
rerun and native CI are tracked separately.
