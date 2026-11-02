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
