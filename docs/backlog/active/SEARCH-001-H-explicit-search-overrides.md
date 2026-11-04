# SEARCH-001-H: Explicit Premium And Self-Hosted Search Overrides

**Status**: Complete / Closed
**Type**: Implementation / Policy Child
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-H |
| Source Issue | [#832](https://github.com/wjhuang88/talos/issues/832) |
| Depends on | Accepted ADR-085/ADR-090; SEARCH-001-C / #643 complete at #808 merge `eb870cdf0a63f45f20d89f1a50eba276f9198643` |
| Selected Iteration | [I314](../../iterations/I314-search-explicit-overrides.md) — Complete / Closed; claim #841 merged `c2d8b441c809c5b2e5cf74d2513d8cc8534a753a` |
| Implementation PR | [#842](https://github.com/wjhuang88/talos/pull/842) — merged as `9ba327728bfc87775ed31740d58636ba73a5c68e` |
| Last Updated | 2026-11-04 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol |
| Work Slice | SEARCH-001-H explicit premium/self-hosted override configuration and production composition; Tavily controlled transport; explicit fallback-to-auto; secret redaction. No SEARCH-001-E/F admission and no `rust-websearch` removal. |
| Claimed At | 2026-11-02 |
| Source Issue | #832 |
| Governance Claim PR | #841 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | On 2026-11-02 the maintainer explicitly answered `允许并行` after being told that I314 would be activated and implemented in parallel with the currently Active I310 and I313 scopes. |
| Handoff / Release Condition | Closed upon existing implementation merge `9ba327728bfc87775ed31740d58636ba73a5c68e` and acceptance reconciliation. No scope transfer; I310 and other owners remain independent. SEARCH-001-E/F stay blocked. |

Original claim #841 became effective at `c2d8b441c809c5b2e5cf74d2513d8cc8534a753a`; implementation #842 subsequently merged to main as `9ba327728bfc87775ed31740d58636ba73a5c68e`. This owner now records the terminal delivery and collaboration states, effective after this governance closeout merges.

## Completion Evidence — 2026-11-04

Completion Commit: `9ba327728bfc87775ed31740d58636ba73a5c68e` (existing main implementation merge commit, [PR #842](https://github.com/wjhuang88/talos/pull/842)).

- Final implementation head `46a5aef08f0b078b3ec46de0be66fddda04fa9ef` passed [CI3800](https://github.com/wjhuang88/talos/actions/runs/41787775314) **8/8**, including workspace Rust + Windows, Linux/Windows Desktop, release preflight, governance/owner reconciliation, installer and real CLI/TUI acceptance.
- Independent read-only security/API/config reviewer [Copilot comment 6624584293](https://github.com/wjhuang88/talos/pull/842#issuecomment-6624584293) reported **no blocking findings** on that exact head against main `261602242e49e6f921f5d3e4eac96eebbe92f8b3`. The proxied fallback request budget and old legacy constructor ambient-activation issues were corrected before merge.
- Network-feature search tests [run 41788026350](https://github.com/wjhuang88/talos/actions/runs/41788026350): 484/484 passed, including fallback budget and isolated environment-variable regression. That optional run was **not green overall** because an additional all-targets Clippy command found two pre-existing `search_backend/health.rs` test lints identical to main.
- Optional all-features run [41788314305](https://github.com/wjhuang88/talos/actions/runs/41788314305) had 1039 passes and 45 Git-executable-path identity test failures in unchanged Git test files, outside I314. Neither optional experiment substitutes for the fully green production CI.
- Actual-binary six-case zero-key/explicit/invalid/fallback/rollback smoke is incorporated into main CI; no paid Tavily or external SearXNG request was made. Provider-live verification is **not** claimed.
- Fallback stays ordered, Auto zero-key, credentials/URLs redacted, model-facing schema and Network permission stable. `rust-websearch` retained; E/F/Mwmbl/Purili not admitted. No migration required for rollback.

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

## Operator Documentation

[Explicit search override configuration, privacy, fallback and rollback](../../reference/SEARCH-001-H-EXPLICIT-OVERRIDES.md) describes the shipped additive persisted `[search]` contract. Exact-head CI3800, independent security review and merge CAS passed for #842. This closes only SEARCH-001-H, not the parent SEARCH-001 or E/F/G/I.

## Rollback

Retain the currently shipped compatibility configuration and provider adapters until an H
implementation has explicit migration/rollback evidence. Reverting I314 implementation must restore
the existing Auto-only production composition without requiring schema/data migration.
