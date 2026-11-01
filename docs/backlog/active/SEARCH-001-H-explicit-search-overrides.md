# SEARCH-001-H: Explicit Premium And Self-Hosted Search Overrides

**Status**: Proposed / Unclaimed
**Type**: Implementation / Policy Child
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-H |
| Source Issue | [#832](https://github.com/wjhuang88/talos/issues/832) |
| Depends on | Accepted ADR-085/ADR-090; SEARCH-001-C / #643 complete at #808 merge `eb870cdf0a63f45f20d89f1a50eba276f9198643` |
| Selected Iteration | None — intake only; no implementation authority |
| Implementation PR | None |
| Last Updated | 2026-11-01 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | None until a bounded iteration and claim are accepted |
| Claimed At | Not applicable |
| Source Issue | #832 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Intake only; Issue/owner existence does not authorize production changes. |
| Handoff / Release Condition | Select a bounded iteration and merge an effective claim before implementation. SEARCH-001-E/F remain separately blocked because D admission did not occur. |

## Goal

Make premium/keyed and self-hosted search providers explicit user-controlled overrides rather than
implicit members of the zero-configuration default route. A credential or endpoint merely existing
in the environment must not silently select paid usage or change the default search policy.

## Scope

- Define one explicit configuration/precedence contract for default auto/compatibility search,
  premium/keyed overrides and self-hosted overrides.
- Keep Tavily/other paid providers and self-hosted SearXNG opt-in.
- Define an explicit fallback-to-auto policy for override failures instead of relying on accidental
  control flow.
- Preserve SEARCH-001-C routing, cancellation/deadline, typed-error and permission boundaries.
- Define secret/redaction requirements for credentials, auth headers and credential-bearing URLs.
- Document compatibility and rollback for any accepted configuration migration.

## Constraints

- No implementation before a selected iteration and effective collaboration claim.
- Preserve the model-facing `web_search` schema and Network permission unless separately reviewed.
- Do not create a second generic retry/backoff/circuit-breaker system; reuse SEARCH-001-C and
  NET-001/#199 boundaries.
- No startup network probe, GeoIP/country routing, default Talos relay, automatic VPN/proxy or
  access-control/CAPTCHA bypass.
- SEARCH-001-D / #644 completed with BLOCKED / ZERO-ADMISSION; H does not admit Mwmbl/Purili and
  does not authorize SEARCH-001-E/F.
- Do not remove `rust-websearch`; SEARCH-001-I owns the later evidence-based dependency disposition
  after G + H.

## Acceptance

- The precedence contract makes default auto/compatibility, premium/keyed and self-hosted modes
  unambiguous.
- Ambient credentials/endpoints alone cannot silently select a paid/provider-specific route or
  incur paid use; regression fixtures prove this.
- Explicit override selection, explicit fallback-to-auto, invalid/missing configuration and
  provider-failure paths are deterministic and tested.
- Secrets are absent from logs, errors, health state and diagnostics, with redaction regressions.
- Existing `web_search` schema, Network permission and SEARCH-001-C deadline/cancellation/error
  behavior remain compatible.
- User/operator documentation explains opt-in behavior, fallback policy, privacy/secret boundaries
  and rollback.
- Locked tests, governance validation, focused independent review and merge-time CAS pass before
  completion.

## Non-goals

- Native zero-key provider admission or regional qualification (SEARCH-001-D/E/F/G).
- Generic network retry/circuit-breaker redesign (NET-001/#199).
- `rust-websearch` retirement (SEARCH-001-I).
- A public provider lifecycle or Plugin model that conflicts with ADR-072/ADR-085.

## Rollback

Retain the currently shipped compatibility configuration and provider adapters until an H
implementation has explicit migration/rollback evidence. Intake alone changes no runtime behavior.
