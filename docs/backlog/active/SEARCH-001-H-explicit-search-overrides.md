# SEARCH-001-H: Explicit Premium And Self-Hosted Search Overrides

**Status**: In Progress / Claimed
**Type**: Implementation / Policy Child
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-H |
| Source Issue | [#832](https://github.com/wjhuang88/talos/issues/832) |
| Depends on | Accepted ADR-085/ADR-090; SEARCH-001-C / #643 complete at #808 merge `eb870cdf0a63f45f20d89f1a50eba276f9198643` |
| Selected Iteration | [I314](../../iterations/I314-search-explicit-overrides.md) — Active / Claimed after this atomic activation reaches `main` |
| Implementation PR | Not started; implementation must branch from the effective activation merge/current `main` |
| Last Updated | 2026-11-02 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol |
| Work Slice | SEARCH-001-H explicit premium/self-hosted override configuration and production composition; Tavily controlled transport; explicit fallback-to-auto; secret redaction. No SEARCH-001-E/F admission and no `rust-websearch` removal. |
| Claimed At | 2026-11-02 |
| Source Issue | #832 |
| Governance Claim PR | #841 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | On 2026-11-02 the maintainer explicitly answered `允许并行` after being told that I314 would be activated and implemented in parallel with the currently Active I310 and I313 scopes. |
| Handoff / Release Condition | Keep I314 ownership until implementation/acceptance closeout or an explicit handoff. Existing I310/I313 ownership remains unchanged; SEARCH-001-E/F stay separately blocked because D admission did not occur. |

This claim/In Progress state is a candidate while #841 is open. It becomes effective only when #841 reaches `main`; no production implementation branch may start before that merge.

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

- Production implementation starts only after #841 reaches `main`.
- Parallel execution with I310/I313 is explicitly maintainer-authorized; their scopes and claims remain independent and unchanged.
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
implementation has explicit migration/rollback evidence. Reverting I314 implementation must restore
the existing Auto-only production composition without requiring schema/data migration.
