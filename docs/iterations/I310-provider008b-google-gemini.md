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
| Implementation PR | Not started |
| Last Updated | 2026-10-30 |
| Handoff / Release Condition | Claim and activation are ineffective until finalized governance PR merges; no implementation before then. |

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
