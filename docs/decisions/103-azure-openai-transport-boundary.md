# ADR-103: Azure OpenAI transport and body-shape boundary

**Status**: Proposed (atomic acceptance with I323 governance; ineffective until reviewed merge)
**Date**: 2026-11-11
**Owner**: PROVIDER-008-C / #914 / I323; parent #724

## Context and decision

Azure is a transport family over existing Chat/Responses bodies. The parent Issue's 2026-10-20
path sketch is not a universal endpoint: versioned Chat embeds deployment in its path; current
v1 Chat/Responses place deployment in body `model`. Do not append Responses to a deployment path.

Use a dedicated Azure module to validate configuration, compute the full destination and auth,
then reuse the existing message builders, streaming parsers, retry/progress and tool-mode logic.
No Azure conditional forest in the Chat encoder, no second Responses parser and no new runtime
SDK dependency. Preserve logical model and provider identity for reasoning/replay; only wire
`model` and deployment path use the separately resolved deployment. No server-side sessions:
Responses keeps store=false and does not send previous_response_id.

Expose `azure-openai` as the family, with explicit transport profile and body shape
`chat` or `responses`. Provider shape is the default, model-local shape can override it; explicit
model protocol still overrides provider protocol. Resolve shape from explicit model, explicit
provider, confirmed frozen catalog signal, then conservative Chat. A selected non-Azure protocol
must retain its existing semantics, without silently borrowing Azure credentials/transport.
The UI must expose effective family, body shape and source so two Azure models are distinguishable.

| Mode | Chat | Responses | Version policy |
|---|---|---|---|
| v1 (new profile default) | /openai/v1/chat/completions | /openai/v1/responses | No dated api-version; reject an incompatible configured dated version |
| deployment (explicit compatibility) | /openai/deployments/{deployment}/chat/completions | Refuse; select v1 explicitly | Require explicit api-version; document 2024-10-21 as a sourced Chat example, not a global latest value |

A deployment mapping is optional and falls back to the selected model ID. Validate deployment
as one resource identifier, never a URL/path/query. Require explicit resource origin; accept
HTTPS, or loopback HTTP for local controlled servers. Disallow credentials/query/fragment and
unexpected paths; preserve known origin/root normalization only. Model endpoint replacement must
not relocate profile credentials: reject before acquisition/dispatch. Redirects are disabled.
No endpoint or deployment is synthesized from a catalog model name.

## Authentication and security

API-key profile uses the existing configured credential source in an `api-key` header, including
on v1 where the Microsoft REST example supports it. Entra supplied-token profile uses only
`Authorization: Bearer` and a separately named environment source, read at dispatch for external
rotation. Never send both. Missing/invalid tokens fail explicitly and never fall back to API key.
Bound headers and mask all credential display/Debug/error/request-preview surfaces.
This child implements supplied bearer tokens, not token minting, login, managed identity or
automatic OAuth refresh. Those flows remain #132. The parent explicitly allows API key or Entra;
no claim is made that an environment token closes a general credential-provider contract.

No host-based ambient authentication discovery. Explicit credentials target only the configured
resource; official cloud origin checks must be exact if any default behavior is added later.
No new native dependency or custom signing. Unknown events and malformed/unterminated streams
retain bounded explicit errors; no automatic retry using a different body or protocol.

## Catalog and migration

Audit current source metadata and pin revision/hash before activating any inference. Azure/azure-
cognitive-services may be admitted only through verified model shape plus explicit resource/profile;
unknown SDK/shape/endpoint, aliases and ambiguous matches cannot establish a routing fact.
Any catalog regeneration is a separate reviewed data change. Existing explicit protocols and
unconfirmed gateway omission behavior remain unchanged. Do not claim old snapshot counts current.

Public enum/profile/model config additions require schema, serialization and SDK fixture coverage,
exhaustive-literal/catch-all migration documentation and the next pre-1.0 minor release, never a
patch. No release/tag is authorized. Existing Gemini private replay custody is not altered.

## Evidence and acceptance

Primary sources checked 2026-11-11:

- https://learn.microsoft.com/en-us/azure/foundry/openai/api-version-lifecycle?tabs=key
  (v1 Chat/Responses, optional version, deployment model, REST api-key and Bearer forms).
- https://learn.microsoft.com/en-us/rest/api/aifoundry/azureopenai/responses
  (v1 Responses request and authentication contract).
- https://learn.microsoft.com/en-us/azure/cognitive-services/openai/reference
  (versioned deployment Chat / 2024-10-21).
- https://learn.microsoft.com/en-us/azure/ai-services/openai/how-to/responses
  (stateless-compatible Responses input, deployment and streaming examples).

These links establish the proposal, not live acceptance or frozen fixtures. Pin source revisions
and fixture provenance during implementation. Controlled actual Agent rounds, auth/header/path
negative tests, old-config precedence matrices, stream proptest, locked full preflight,
exact-head native CI and independent protocol/auth/API/security review are acceptance gates.

## Alternatives and consequences

v1-only would drop the parent's versioned deployment compatibility requirement. Legacy-only would
misroute current Responses. Two explicit modes retain both without silent migration or retry.
A full Azure identity SDK now would expand #132 and require a dependency/native audit; explicit
supplied tokens keep this child bounded and accurately documented. Deployment aliases must not
accidentally change reasoning support or replay identity; regressions must test that separation.
