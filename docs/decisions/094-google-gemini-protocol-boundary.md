# ADR-094: Native Gemini protocol and authentication boundary

**Status**: Accepted (effective only when governance #811 merges)
**Date**: 2026-10-30
**Owner**: PROVIDER-008-B / #810 / I310

## Context

Parent #724 requires Gemini generateContent, including AI Studio and Vertex. A supplies shared
model-level resolution. Current primary documentation distinguishes generateContent from
Interactions and permits function-call IDs; the historical assumption that IDs never exist is
not a valid wire contract. Signatures must remain attached to the original model parts.

## Decision proposal

Use one dedicated native generateContent request/SSE module group and shared factory dispatch.
Map systemInstruction and ordered contents/parts, inlineData images, function declarations and
function responses. Retain supplied IDs; for missing IDs assign stable per-response part keys and
keep explicit call-to-result association. Preserve signed parts without concatenating signatures;
opaque replay must respect model identity and reject incompatible replay explicitly rather than
forward confidential state to another provider. Do not log signatures or credential material.

Select AI Studio API-key authentication or standard Vertex OAuth via explicit transport config.
AI Studio uses the generativelanguage origin and key header; Vertex uses validated project/location
resource paths and Bearer credentials. Never infer OAuth from a user-controlled hostname or send
Google ADC to an arbitrary replacement gateway. Explicit custom destinations require a separate
credential/profile configuration and deny implicit ADC relocation. Disable credential-bearing
redirects. Bound credential acquisition/refresh to request deadlines and cancellation.

Prefer Google's Rust google-cloud-auth credential provider over handwritten signing/refresh or
runtime gcloud subprocesses. Support service-account and local ADC OAuth, refreshing expired tokens;
metadata credentials only in explicitly selected Vertex ADC mode. No ADC discovery for other wires.
Audit exact version, feature graph, native dependencies and secret-safe errors before dependency
commit; a failed audit requires an ADR amendment, not an unreviewed substitute. No SDK generation
client is required: retain native HTTP/SSE mapping and the existing retry/progress boundary.

Handle safety blocks and nonterminal EOF explicitly, account for usage/thinking tokens, and never
fall back to Chat after a request starts. Configure reasoning by confirmed model support; reject
unsupported settings rather than inventing defaults. Keep custom gateways conservative and catalog
activation separate, frozen and exhaustively audited. New enum/config fields require documented
pre-1.0 minor migration, not patch release.

## Alternatives and consequences

Interactions/stateful SDK conversations would change the accepted wire and local-history authority;
excluded. Handwritten OAuth and gcloud subprocess dependencies add security/host assumptions;
prefer the maintained Rust auth library. Static Bearer-only implementation is useful for fixtures
but insufficient to close Vertex scope. Auth dependency audit and exact wire fixtures are mandatory
implementation gates, not evidence of an already delivered feature.

## Primary evidence (checked 2026-10-30)

- https://ai.google.dev/api/generate-content
- https://ai.google.dev/gemini-api/docs/generate-content/thought-signatures
- https://ai.google.dev/gemini-api/docs/generate-content/function-calling.md
- https://docs.cloud.google.com/docs/authentication/application-default-credentials
- https://googleapis.github.io/google-cloud-rust/configure_client.html
- https://docs.rs/google-cloud-auth/latest/google_cloud_auth/credentials/struct.Builder.html

Pin specific schema/fixture/library revisions during implementation; these current documentation
links alone are not a stable fixture or a dependency approval. Independent protocol/auth review
must accept this decision with the finalized claim before implementation begins.

## Auth integration amendment — 2026-10-31 (implementation candidate)

The generic Google credential cache cannot satisfy caller-owned cancellation (see #810).
Use exact google-cloud-auth 1.18.0, defaults disabled, default-rustls-provider enabled,
only service_account::Builder::build_signer for local RSA-PKCS1-SHA256. Talos owns
fixed-endpoint HTTP token acquisition, bounded responses and request-owned cache refresh.
No generic Credentials/ADC builder, remote signer, or background refresh is used.

Archive SHA256 d563c7b987f40249c274a4e6a671661df6be2d084f08476dda2f0f931576cfcf.
The resolved graph adds bs58 0.5.1, cmov 0.5.4, ctutils 0.4.7, google-cloud-auth 1.18.0,
google-cloud-gax 1.15.0, google-cloud-rpc 1.6.0, google-cloud-wkt 1.10.0,
hashbrown 0.12.3, hmac 0.13.0, indexmap 1.9.3, schemars 0.9.0,
serde_with/serde_with_macros 3.24.0. No existing package version is replaced.
Provider feature-tree comparison retains aws-lc-rs 1.17.1 / aws-lc-sys 0.42.0 and
prebuilt-nasm features; rustls log/logging features become reachable. This does introduce
new native RSA private-key parsing/signing use, not merely reuse existing TLS behavior.

The dedicated signer has no HTTP/spawn path. Its async sign method performs synchronous
key parsing and signing in one poll: it cannot be preempted by Tokio timeout or cancellation.
Bound key input to 16 KiB and credential input to 1 MiB, contain Rust panics at construction
and signing, discard upstream errors, and check deadline before HTTP exchange. Do not log
signer Debug/error sources. catch_unwind cannot contain native aborts or suppress panic hooks;
no such guarantee or private-key zeroization is claimed. Independent read-only source audit
confirmed these limits. Exact implementation security review and CI remain merge gates;
this candidate does not establish completed Gemini acceptance.

## Reasoning controls — implementation candidate, 2026-11-02

Primary contract checked at https://ai.google.dev/gemini-api/docs/generate-content/thinking:
2.5 Pro budget 128..32768; 2.5 Flash 0..24576; 2.5 Flash-Lite 0 or 512..24576.
Initial exact model IDs are gemini-2.5-pro, gemini-2.5-flash and gemini-2.5-flash-lite.
Gemini 3.8 Flash and Gemini 4 Argon support low/medium/high levels; admit their exact IDs
(gemini-3.8-flash, gemini-4-argon), not guessed prefixes, gateway aliases or preview variants.
Existing ReasoningOptions.budget_tokens maps only to budget models; effort only to level models.
Never send both fields. An absent budget preserves vendor dynamic thinking; Argon explicitly
uses its documented medium default when an options block has no effort. includeThoughts=true
requests summaries. maxOutputTokens remains a separate hard cap including thinking tokens.
Unsupported explicit model/setting combinations fail before credentials or generation dispatch.

No new public config field is introduced. replay=false is explicitly refused for this native
adapter, because silently stripping positional function-call signatures would break the accepted
replay contract. Isolated decisions use an independent minimum-latency policy (Flash/Lite budget 0, Pro budget
128, level-based models low), includeThoughts=false, and private reasoning-output suppression;
ordinary conversation settings remain unchanged. Model capability declarations
continue to suppress unsupported explicit reasoning controls through the shared factory policy.
These are bounded initial mappings, not a claim of all-model parameter support or live acceptance.

## Textual tool compatibility — implementation candidate, 2026-11-03

Explicit Compat and TalosStrict use the existing shared textual-tool parser over Gemini SSE,
without native declarations or cross-wire fallback. Parsed tool metadata is separate from signed
model text: preserve that text and its original signed parts exactly, and send tool results as
compatibility user text. Verify parsed name/argument order against stored calls before replay;
the existing Gemini request builder then verifies signed text and identity. Generated textual
call IDs are local association keys, not added to signed parts. Signed native-tool history cannot
switch to textual mode by rewriting the envelope; reject that transition before HTTP. Unresolved
Auto and all browser ingress remain refused. Browser support still requires the separately pending
run-local replay/persistence boundary. This stage changes no public API or credential destination.

## Native browser signed history — implementation candidate, 2026-11-04

The Gemini wire must preserve signed function parts, while existing browser tools forbid raw
original arguments in durable conversation events. Carry signed browser history through a
nonserializable TransientReplay invocation event, never ReasoningComplete/Redacted payload data.
A unique live AppServerSession Actor owns private custody across user submissions. Standalone
Agent calls use execution-local custody; callers needing live cross-turn continuity use the Actor.
Generation/persistence-identity replacement, reopened Actors and durable forks receive no private
custody. Missing/foreign/modified/malformed/repeated markers fail explicitly before HTTP.

Persist only an opaque marker and conservative full-private-byte reserve plus projected calls.
Verify the exact effective assistant after text/tool hooks, before tool effects, then rebind only
through the trusted privacy projection. Validate marker identity and the complete sealed request
again before restoring signed parts inside the provider encoder. Hooks see projected inputs;
permission/execution retain original admitted arguments. Budgeting conservatively reserves one
token per serialized private byte, with normal safety/output reserves; malformed marker budgets
saturate. Protected hook messages cannot be edited, stripped or duplicated. Complete checkpoint
projection may omit an entire exchange; partial surviving calls/results fail. Canonical custody
has a hard 32-block/16-MiB limit; request-only compaction does not silently reclaim that limit.
Uncommitted canceled entries are pruned against canonical history at next preparation.

The existing non-rescanning Agent privacy filter registers bounded explicit private fields before
any text/thought/tool observations and retains tool-owned tokens in live custody. Limits are256
values/64KiB, with synchronous filter-work and output admission. Only validated public operation
discriminators are exempt; unknown operation strings remain private until semantic admission.
Merely presenting a browser tool does not turn a plain response into private history. Native
browser mode buffers its response through validated EOF. Compat/TalosStrict browser entry remains
refused in this stage; ordinary textual tools remain supported. No browser capability, permission
policy, stored-secret encryption, external credential destination or release is added here.

API migration: ProviderInvocationEvent gains TransientReplay and becomes #[non_exhaustive].
Downstream exhaustive matches must add a refusal/catch-all arm in the next pre-1.0 minor release;
this must not ship as a patch. LanguageModel gains a default stream_with_replay_context entrypoint,
which refuses private markers/attachments unless explicitly implemented. The default delegates
unchanged requests to the existing image/invocation path. New custody types intentionally implement
neither Serialize nor Clone; Arc ownership remains session/request-local. These are draft
implementation decisions requiring final privacy/protocol review and acceptance, not delivery.

The live store preserves a previously published projection only after it verifies the whole
message against custody. A value first classified as private in a later provider response
filters subsequent output. It does not retroactively rewrite a published sealed projection.
Unknown browser v2 field names reject the batch before any protected output is emitted.
Mixed ordinary tool calls bind the original signed argument before permission normalization.
The privacy filter covers the original string and its deterministic trimmed/path-cleaned forms.
Substring collisions with routing fields still fail closed, including collisions with public
schema names. Browser result content is outside this raw-invocation filter unless the tool
explicitly supplies private tokens.


## Textual browser continuation amendment — 2026-11-05

The next candidate extends the same custody boundary to Compat and TalosStrict through
`stream_with_replay_context`. Buffer the complete response, validate the shared textual batch
against original argument syntax, then publish custody before any text/thought/tool event.
Restore exact signed text before compatibility history projection; omit native declarations.
Auto remains refused. Tool calls in truncated responses are rejected before execution.

With a browser registered, signed ordinary tool frames also require custody: Agent text cleanup
may otherwise remove fenced calls or surrounding whitespace. Pure no-call responses without
private history do not acquire custody. Thus an ordinary tool response in this protected route
can consume the same 32-block/16-MiB limit and makes a durable reopen require missing-context
refusal. This is an explicit availability tradeoff for exact signed replay, not durable secrets.

The shared textual JSON envelope now rejects duplicate name/args/id fields before routing,
including escaped duplicate keys. Non-string, non-null IDs are rejected instead of generating
an ID. Unknown envelope fields, ordinary nested arguments and legacy name-plus-JSON framing
retain their previous handling. Original browser arguments still undergo the stricter protected
extractor. No browser permission, credential destination or public API change is added here.
