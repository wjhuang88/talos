# Iteration I314: Explicit Premium And Self-Hosted Search Overrides

> Document status: Complete
> Published plan date: 2026-11-02
> Planned objective: implement SEARCH-001-H as an explicit user-controlled premium/keyed and self-hosted search override layer without changing the zero-config default route or admitting native zero-key providers.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a configured explicit Tavily or self-hosted SearXNG override is selected deterministically, while ambient credentials/endpoints alone leave Auto unchanged; provider failure follows an explicit fallback-to-auto policy and secret-bearing values remain redacted.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol |
| Work Slice | SEARCH-001-H explicit premium/self-hosted override configuration and production composition; Tavily controlled transport; explicit fallback-to-auto; secret redaction. Excludes SEARCH-001-E/F admission and `rust-websearch` removal. |
| Claimed At | 2026-11-02 |
| Source Issue | #832 |
| Governance Claim PR | #841 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly answered `允许并行` on 2026-11-02 after the activation gate was explained as I314 running concurrently with the currently Active I310 and I313 scopes. |
| Implementation PR | #842 — merged as `9ba327728bfc87775ed31740d58636ba73a5c68e` |
| Last Updated | 2026-11-04 |
| Handoff / Release Condition | Closed after existing implementation merge `9ba327728bfc87775ed31740d58636ba73a5c68e` and post-merge owner/Issue reconciliation; no scope transferred to I310/I313. |

The original Active/Claimed state became effective on main at #841 merge `c2d8b441c809c5b2e5cf74d2513d8cc8534a753a`. Implementation #842 subsequently merged as `9ba327728bfc87775ed31740d58636ba73a5c68e`. Complete / Closed in this owner is proposed by the separate governance closeout and takes effect after its own merge.

## Post-merge Acceptance And Closure — 2026-11-04

Completion Commit: `9ba327728bfc87775ed31740d58636ba73a5c68e` (already exists on main, implementation PR #842).
Exact implementation head: `46a5aef08f0b078b3ec46de0be66fddda04fa9ef`.

- [CI3800](https://github.com/wjhuang88/talos/actions/runs/41787775314): **8/8 success**, exact head, including release preflight and real CLI/TUI Auto / configured search smoke, Windows Rust workspace/Desktop, Linux Desktop, installer, classification and governance owner reconciliation. Both project and collaboration governance validators passed in the required workflow.
- Independent [security/API/config review](https://github.com/wjhuang88/talos/pull/842#issuecomment-6624584293): **no blocking findings** on that head, following corrections for proxied fallback 24→26 units and zero-key public legacy construction. Auto still 135s/24; opted-in Tavily/SearXNG + ordered fallback 150s/26; one shared request budget, no hidden race/paid activation.
- Additional `--features network --lib` search test run [41788026350](https://github.com/wjhuang88/talos/actions/runs/41788026350) passed 484/484 including three new boundary/ambient regressions. Its optional all-targets Clippy failed only on two pre-existing `health.rs` test lints identical to main; **do not label the optional workflow green**.
- Additional `--all-features --lib` run [41788314305](https://github.com/wjhuang88/talos/actions/runs/41788314305) passed 1039 and failed 45 existing Git subprocess identity fixtures due to `/usr/bin/git` vs `/usr/lib/git-core/git`; unchanged Git source files and outside I314. This optional run is **not** acceptance authority for I314.
- Offline real-binary six-case evidence (run `41746957277`) covered inert ambient keys, fail-closed missing explicit configuration, explicit stored credentials/endpoints and Auto rollback without paid-provider network calls. Separate live Tavily/SearXNG integration was not executed and is not claimed.
- Merge-time CAS: #842 was Ready, exact head unchanged, 0 behind main, mergeable clean, zero unresolved review threads and independent claim #841 effective. GitHub merged #842 with expected head SHA, yielding `9ba327728bfc87775ed31740d58636ba73a5c68e`.
- Bounded outcome: only SEARCH-001-H / I314 closes. SEARCH-001-E/F remain unauthorized after D's BLOCKED/ZERO-ADMISSION research disposition, G remains blocked, I still requires its own dependency gates; parent #624 remains Open. `rust-websearch`, model-facing schema, network permissions and rollback are retained.
- Operator reference: [SEARCH-001-H explicit overrides](../reference/SEARCH-001-H-EXPLICIT-OVERRIDES.md).

Historical implementation-stage snapshots below are retained for audit, not current status.

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
- `docs/iterations/README.md` and `docs/BOARD.md` when activation or later state changes become effective.
- Issue #832 synchronization comment after each effective governance transition.

### Risks And Rollback

- Risk: configuration migration accidentally converts an ambient credential into implicit paid usage.
  - Mitigation: Auto selection tests must run with ambient premium/self-hosted environment values present.
- Risk: fallback semantics duplicate or bypass SEARCH-001-C / NET-001 timeout, cancellation or error handling.
  - Mitigation: compose through existing selection/router/transport boundaries; no new generic resilience layer.
- Risk: provider-specific URLs or auth data leak through errors/diagnostics.
  - Mitigation: explicit redaction fixtures over every new observable surface.
- Rollback: retain the currently shipped compatibility adapters and config parsing; revert the H composition/configuration selection commit(s) without schema/data migration. No native provider admission is coupled to H.

## Nonterminal Iteration Inventory And Disposition — 2026-11-02 Activation Preflight

Target-branch truth was rechecked immediately before activation from `main` `069c5da196e492bd8120cc45f50e45438dd19423`:

- I313 / T5 remained effective `Active / Claimed`; implementation PR #837 was Draft/Open and independently owned.
- I310 / PROVIDER-008-B Gemini remained effective `Active / Claimed`; implementation PR #838 was Draft/Open and independently owned.
- I302 / TEST-003 and I277 / DESKTOP-001-D3 remained Review/Claimed; their deferred evidence/acceptance work was unchanged.
- I249 remained Planned and I164 remained Paused; neither scope transferred to I314.
- SEARCH-001-C/I293 and SEARCH-001-D/I294 remained Complete/Closed. D closed with BLOCKED / ZERO-ADMISSION, so E/F remain unauthorized.
- No other branch or target-branch record claimed SEARCH-001-H / I314 implementation ownership.

I314 is non-overlapping with I310/I313. On 2026-11-02 the maintainer explicitly authorized parallel execution after being told that this would activate and implement I314 concurrently with those Active iterations. That authorization satisfies the plan's explicit-parallel activation path; it does not transfer, weaken, or merge the existing I310/I313 claims.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-11-02 | Planning | I314 selected for SEARCH-001-H as Planned / Unclaimed only. Existing Active/Review/Planned inventory retained; no claim or implementation authority established. |
| 2026-11-02 | Parallel authorization | Maintainer explicitly answered `允许并行` after I314's concurrent activation with I310/I313 was explained. |
| 2026-11-02 | Activation | #841 passed exact-head CI3756 / run `41439465737`, both governance validators, exact-head Agent review and CAS; merged as `c2d8b441c809c5b2e5cf74d2513d8cc8534a753a`. I314 became effective Active / Claimed. |
| 2026-11-02 | Implementation | Draft PR #842 opened from the activation merge. Initial bounded stage prepares redacted/sensitive credential-header ownership inside the controlled search transport without activating Tavily or changing Auto selection. |

## Verification Evidence

### 2026-11-04 implementation candidate evidence — historical, superseded by merged acceptance

- Current candidate implementation commit: `8f065c962d0f05ea6e83489b4f3ceb19430886b7` on PR #842 (Draft; not merged). This is **not a Completion Commit**.
- Latest effective main last verified for this checkpoint: `261602242e49e6f921f5d3e4eac96eebbe92f8b3`. Historical safety archive `archive/i314-pre-main-sync-20261103` remains unmodified.
- Earlier CI3797 / run `41663411803` exposed three `talos-tools` Clippy findings and twelve missing exhaustive `Config.search` test initializers; all corrected in `464c51dea426f9ebc9b7ff53f0139f5ad8390a10`.
- CI3798 / run `41745290734` exposed three stale CLI registry symbols under `-D warnings`; current candidate reuses the old MCP status tool and preserves compatibility wrappers.
- **Independent, source-equivalent binary acceptance:** isolated run `41746957277` on the exact candidate source plus a temporary diagnostic workflow succeeded. Its temporary workflow was removed after use and never entered PR #842.
- Acceptance command chain:
  `cargo clippy --workspace --locked -- -D warnings` — **success**;
  `cargo test -p talos-config --locked --lib` — **268 passed, zero failed**;
  `cargo build -p talos-cli --locked` — **success**;
  `python3 scripts/accept_search_overrides.py target/debug/talos` — **success**.
- Actual `talos --mock --print --no-init --no-context` fixture transcript: `SEARCH_OVERRIDE_E2E auto_with_ambient_optional_values PASS`; `tavily_missing_explicit_credential_fail_closed PASS`; `tavily_explicit_stored_credential PASS`; `searxng_missing_explicit_endpoint_fail_closed PASS`; `searxng_explicit_endpoint PASS`; `rollback_auto_with_persisted_optional_credential PASS`.
- This confirms real print-mode persisted-config loading and registry construction **without** invoking a search query or consuming paid provider quota. It complements the selected-candidate/transport/permission/router unit fixtures, not an actual external-provider live integration claim.
- CI3799/run `41746709995` passed **all eight exact-head checks** on `8f065c962d0f05ea6e83489b4f3ceb19430886b7`; this is historical, not acceptance evidence for any later changed head.
- Independent Copilot review [PR #842 comment 6619286884](https://github.com/wjhuang88/talos/pull/842#issuecomment-6619286884) found two issues: a 24 vs 26 proxied request-unit envelope for explicit + Auto fallback, and ambient optional-provider selection in the legacy public constructor. The subsequent unmerged candidate corrects both and adds regression coverage; renewed exact-head CI/review are required.
- **Historical gate status at earlier head**: exact-head full CI and independent read-only security/API re-review were pending at that time; all completed on later head as recorded in the Post-merge Acceptance section above.
- Operator contract and rollback: `docs/reference/SEARCH-001-H-EXPLICIT-OVERRIDES.md`. No native Mwmbl/Purili admission or SEARCH-001-E/F authorization, no `rust-websearch` removal.

- Activation: #841 merge `c2d8b441c809c5b2e5cf74d2513d8cc8534a753a`; CI3756/run `41439465737` success; both governance validators zero warnings; exact-head Copilot gate review approved.
- Implementation: #842 is Draft; exact-head implementation validation remains in progress and no stage is accepted merely by publication.

## Completion Evidence

- Completion Commit: `9ba327728bfc87775ed31740d58636ba73a5c68e` — existing main implementation merge commit from #842.
- The separate governance closeout only records already-existing implementation evidence; this status-only change never cites itself as the implementation completion.

## Variance And Residuals

- Production implementation #842 is merged as `9ba327728bfc87775ed31740d58636ba73a5c68e`; acceptance and independent review passed, with optional broader test residuals disclosed above.
- I310/I313 remain independently Active/Claimed under explicit maintainer-authorized parallel execution.
- SEARCH-001-E/F, Mwmbl/Purili admission and `rust-websearch` removal remain outside I314 authority.

## Retrospective

- Outcome: Complete / Closed after implementation merge; no premature claim of paid-provider live tests.
- Documentation: owner, inventories, Board and Issue synchronized post-merge by separate governance closeout.
- Lessons: explicit parallel authorization keeps non-overlapping work moving without weakening ownership boundaries.
