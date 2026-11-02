# Iteration I314: Explicit Premium And Self-Hosted Search Overrides

> Document status: Planned
> Published plan date: 2026-11-02
> Planned objective: implement SEARCH-001-H as an explicit user-controlled premium/keyed and self-hosted search override layer without changing the zero-config default route or admitting native zero-key providers.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a configured explicit Tavily or self-hosted SearXNG override is selected deterministically, while ambient credentials/endpoints alone leave Auto unchanged; provider failure follows an explicit fallback-to-auto policy and secret-bearing values remain redacted.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned; this document is planning only |
| Claimed At | Not applicable |
| Source Issue | #832 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Planning only; no production implementation is authorized. |
| Implementation PR | Not started |
| Last Updated | 2026-11-02 |
| Handoff / Release Condition | Before activation, re-run the nonterminal iteration inventory and either wait for conflicting Active work to close or obtain explicit maintainer authorization for non-overlapping parallel execution; then merge one atomic claim+activation PR. |

This Planned record does not reserve ownership. Implementation must not start until a later governance-only PR records one effective `Claimed` / `Active` state on `main` under `docs/sop/AGENT-COLLABORATION.md`.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SEARCH-001-H` / #832 | `SEARCH-001` / #624 | Planned / Unclaimed | SEARCH-001-C complete at #808; ADR-085/ADR-090 accepted | Explicit override configuration and fallback semantics without ambient paid-provider activation |

### Scope

- Reuse the existing `SearchBackendSelection::{Auto, Explicit(...)}` intent seam and SEARCH-001-C composition/transport/error boundaries rather than creating another router, retry system or circuit breaker.
- Wire explicit premium/keyed and self-hosted provider selection into the real configuration / production composition path.
- Keep Tavily and self-hosted SearXNG opt-in. Merely setting a credential or endpoint in the environment must not change Auto selection or spend paid quota.
- Define one deterministic precedence contract across default Auto/compatibility search, explicit premium/keyed override and explicit self-hosted override.
- Define an explicit fallback-to-auto policy for override failures; fallback must be configured/documented rather than an accidental side effect of provider availability.
- Preserve the model-facing `web_search` schema, Network permission boundary, SEARCH-001-C deadline/cancellation/typed-error semantics and existing compatibility rollback path.
- Ensure secrets, auth headers and credential-bearing URLs cannot enter logs, user-visible errors, health state or diagnostics.

### Non-Goals

- Native zero-key provider admission, Mwmbl/Purili activation or regional routing; SEARCH-001-D closed BLOCKED / ZERO-ADMISSION and SEARCH-001-E/F remain unauthorized.
- GeoIP/country routing, startup probes, Talos relay, VPN/proxy automation or access-control/CAPTCHA bypass.
- Generic retry/backoff/circuit-breaker redesign; NET-001/#199 retains that scope.
- `rust-websearch` removal; SEARCH-001-I owns dependency disposition after G + H.
- New public provider lifecycle/plugin abstraction or breaking `web_search` API/schema changes.

### Acceptance

- Given default/Auto configuration with ambient `TAVILY_API_KEY`, `SEARXNG_URL` or equivalent provider-specific environment values,
  when production search composition is created,
  then no premium/self-hosted provider is selected solely because those values exist.
- Given an explicit supported override with valid configuration,
  when `web_search` executes,
  then the requested override is selected deterministically without entering the zero-key default provider race.
- Given an explicit override with missing/invalid configuration,
  when composition is created or search executes,
  then the result is deterministic and does not silently fall into paid/default behavior not selected by policy.
- Given provider failure with fallback-to-auto disabled,
  when the override fails,
  then the typed failure is returned without hidden Auto fallback.
- Given provider failure with fallback-to-auto explicitly enabled,
  when the override fails within the existing C/NET deadline and cancellation boundaries,
  then exactly the documented Auto path may run and no second generic retry policy is introduced.
- Given secret-bearing configuration and failures,
  when logs/errors/health/diagnostics are inspected,
  then secrets, auth headers and credential-bearing URLs are absent or redacted.
- Given existing callers and model schemas,
  when H is enabled or rolled back,
  then public `web_search`, Network permission and compatibility behavior remain semver-compatible.

### Planned Validation

- Focused unit/integration tests for Auto-vs-explicit selection and precedence.
- Regression tests proving ambient credentials/endpoints do not activate premium/self-hosted providers.
- Explicit override success, missing/invalid configuration, fallback-disabled failure and fallback-enabled Auto recovery fixtures.
- Redaction tests for API keys, auth headers, URL credentials and provider errors across logs/errors/health/diagnostics.
- Existing SEARCH-001-C cancellation/deadline/typed-error tests and Network permission tests remain green.
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `scripts/validate_collaboration_claims.sh .`
- Exact-head CI, focused independent API/security/configuration review and merge-time CAS.

### Documentation To Update

- `docs/backlog/active/SEARCH-001-H-explicit-search-overrides.md`
- Search/configuration user documentation and README surfaces that describe Tavily/SearXNG configuration.
- `docs/reference` search/configuration contract where appropriate.
- `docs/iterations/README.md` and `docs/BOARD.md` only when activation or later state changes become effective.
- Issue #832 synchronization comment after each effective governance transition.

### Risks And Rollback

- Risk: configuration migration accidentally converts an ambient credential into implicit paid usage.
  - Mitigation: Auto selection tests must run with ambient premium/self-hosted environment values present.
- Risk: fallback semantics duplicate or bypass SEARCH-001-C / NET-001 timeout, cancellation or error handling.
  - Mitigation: compose through existing selection/router/transport boundaries; no new generic resilience layer.
- Risk: provider-specific URLs or auth data leak through errors/diagnostics.
  - Mitigation: explicit redaction fixtures over every new observable surface.
- Rollback: retain the currently shipped compatibility adapters and config parsing; revert the H composition/configuration selection commit(s) without schema/data migration. No native provider admission is coupled to H.

## Nonterminal Iteration Inventory And Disposition — 2026-11-02

This planning record does not activate I314. Current relevant target-branch truth was checked before selection:

- I313 / T5 is effective `Active / Claimed` after #819 merged as `8b11e5f49794ea17f09c96eff8d59a6bf8f9a3d6`; retain ownership and execution unchanged.
- I310 / PROVIDER-008-B Gemini is `Active / Claimed` under #811; retain independent provider-protocol ownership unchanged.
- I302 / TEST-003 and I277 / DESKTOP-001-D3 remain Review/Claimed; retain their deferred evidence/acceptance work unchanged.
- I249 remains Planned and I164 remains Paused; neither scope transfers to I314.
- SEARCH-001-C/I293 and SEARCH-001-D/I294 are Complete/Closed. D closed with BLOCKED / ZERO-ADMISSION, so E/F remain unauthorized.
- No H implementation PR or effective H claimant exists at planning time.

I314 is technically non-overlapping with I310/I313/I302/I277, but repository policy permits additional Active iterations only with explicit approval. Therefore this plan stays `Planned / Unclaimed`; no production branch or implementation PR may start from this commit. A later atomic claim+activation must repeat the inventory and record the applicable maintainer authorization or serial activation condition.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-11-02 | Planning | I314 selected for SEARCH-001-H as Planned / Unclaimed only. Existing Active/Review/Planned inventory retained; no claim or implementation authority established. |

## Verification Evidence

- Planning-only candidate; implementation validation not yet applicable.

## Completion Evidence

- Completion Commit: Pending
- Status-only documentation commits must not cite themselves. I314 remains Planned until a later atomic claim+activation reaches `main`, and cannot become Complete without already-existing implementation evidence.

## Variance And Residuals

- Activation/claim remains pending.
- No runtime/configuration code, tests, dependencies or public API are changed by this planning record.

## Retrospective

- Outcome: pending
- Documentation: planning owner only
- Lessons: none yet
