# PROVIDER-008-C — Azure OpenAI Transport and Per-Model Chat/Responses Routing

**Status**: Intake / Unclaimed
**Type**: Provider protocol family child / architectural intake
**Source Issue**: [#914](https://github.com/wjhuang88/talos/issues/914)
**Parent Epic**: [PROVIDER-008 / #724](PROVIDER-008-openai-responses-api-support.md)
**Selected Iteration**: None — I323 is a proposal, not an effective claim
**ADR**: ADR-103 proposed; no acceptance or runtime authority implied

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Owner intake and issue reconciliation only; implementation not authorized |
| Claimed At | Not applicable |
| Source Issue | #914 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Independently accept protocol/auth design and atomically claim+activate I323/ADR-103 on main before any code |

## Scope / decision boundary

Track the proposed Azure OpenAI transport without conflating it with the already completed
Responses/Gemini children. Reuse stateless Chat/Responses encoding and SSE semantics where
compatible; independently characterize Azure resource endpoints, deployment routing
separate from logical model selection, versioned deployment Chat and current v1 Chat/Responses
paths, API-key and explicitly supplied Entra bearer authorization.

No implicit credential discovery, managed identity or account-scope credential relocation
is granted. Such external login/credential-chain work remains [#132](https://github.com/wjhuang88/talos/issues/132).
No silent cross-protocol fallback, default provider selection change, unrelated Bedrock/Cohere
implementation or release authorization is part of this intake.

## Requirements before implementation

- Complete source-pinned Azure endpoint and auth decision, including conservative protocol
  precedence, catalog evidence, compatibility and migration behavior.
- Independently review protocol destination/auth/redaction boundaries and adopt ADR-103
  only through its own reviewed governance claim.
- Select/activate I323 atomically with the bounded child implementation claim; then require
  exact-head native CI, real Agent image/tool/reasoning turns, error/usage fixtures and
  independent technical/API/security acceptance.
- Keep parent [#724](https://github.com/wjhuang88/talos/issues/724) open until remaining
  C/D and parent acceptance close; B/#810 is independently complete.

## 2026-11-11 owner synchronization

Registered as an **Intake / Unclaimed** document-only owner for open Issue #914 to
restore remote Issue ↔ owner-document reconciliation. This is NOT an implementation PR,
I323 activation, ADR-103 acceptance or child completion. The separate SEARCH-001-L
PR #907 has no ownership over provider protocol behavior.
