# SEARCH-001-A: Search Architecture Audit, Migration Matrix & ADR

**Status**: Complete / Claimed
**Type**: Architecture / Governance Story and evidence-led Spike
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-A |
| Source Issue | [#625](https://github.com/wjhuang88/talos/issues/625) |
| Priority | P2; maintainer selected 2026-09-30 |
| Selected Iteration | [I288](../../iterations/I288-search-architecture-audit.md) |
| Dependencies | None for audit; accepted ADR and effective claim are gates for subsequent implementation |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Sol / talos开发 session |
| Work Slice | SEARCH-001-A architecture audit, deterministic current-behavior characterization, provider-admission/multi-region evidence plan, ADR and B–I migration matrix only; no production search implementation |
| Claimed At | 2026-09-30 |
| Source Issue | #625 |
| Governance Claim PR | #627 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance claim PR #627 merged as `6f651a28fe901a894942ab1c56015a60491a054e` after exact-head reduced CI, both governance validators, remote Issue reconciliation and merge-time CAS. I288 is architecture-only and non-overlapping with other active slices. |
| Implementation PR | #629 |
| Last Updated | 2026-10-03 |
| Handoff / Release Condition | ADR-085 accepted; B/C/D are dependency-ready and separately unclaimed |

## Identity / Goal / Value

Establish the authoritative technical contract and evidence-led staged migration for reliable no-config web search. Separate user requirements from verified current code, distinguish candidate zero-key providers from accepted production channels, and prevent replacing one fragile library dependency with a new unreviewed one.

This child is **not** a production search implementation. Its result is a reviewable decision, compatibility baseline, provider-admission test plan and governance decomposition.

## Scope / Deliverables

1. **Current-code and ownership audit.** Document actual `WebSearchTool` call graph, `rust-websearch` version/behavior, Tavily/SearXNG/Wikipedia branches, error/fallback semantics, configuration and key paths, Network permission, Tool disclosure, cancellation and fetch separation. Record overlap with ADR-072/CAP-001, TOOL-009/012/013/014, WEBFETCH-001 and NET-001/#199.
2. **Reproducible behavior baseline.** Document controlled, deterministic tests for fast failed provider versus slow success, all-fail, zero-result, timeout/cancel and parse drift. Verify current state before proposing fixes. Do not introduce production behavior changes or new dependencies in this slice.
3. **Provider feasibility/admission matrix.** Define maintenance and terms assessment, parser fixture strategy, endpoint and infrastructure independence, privacy/query exposure, region-scoped test provenance, DNS/TLS/HTTP/parser observations and blocking/challenge interpretation. Specify how legitimate mainland-China environment evidence is to be collected without fabricated geography or circumvention assumptions.
4. **Search-domain ADR.** Decide whether a separate `talos-search` crate is necessary or the contract fits an existing module. Define ownership of Request/Result/Error, provider adapter, per-endpoint health/probe, bounded first-valid-success routing and no-startup-network default. Explicitly compose with shared network resilience #199 and ADR-072 rather than creating rival platform mechanisms.
5. **Compatibility/migration matrix.** State old/new behavior by CLI/TUI/embedded/RPC surface, tool schema/output, feature flags, zero-config default and premium override semantics, permission, config/secret handling, tests, activation, rollback and B–I ownership. State adoption criteria for two independent native zero-key providers and a measured `rust-websearch` retirement gate.
6. **Governance convergence.** Propose and review the ADR through normal processes; update this owner and parent map after acceptance. Create only dependency-ready future child Issues/owners, not placeholders.

## Exclusions

- No runtime behavioral change, production provider implementation, automatic downloads, new runtime dependencies, crate/public API changes or persisted config migration.
- No location-based hardcode, unverifiable availability guarantee, default relay, bypass of provider challenges, change to Network permission authority, or startup network probe.
- No claim that a credentialed provider can run by default or that an environment key alone opts users into paid usage.
- No implementation authorization from Issue creation, this owner intake or an unaccepted ADR.

## Decision Links And Constraints

- [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md) defines shared Capability/Provider/Plugin terms and lifecycle.
- NET-001 / [#199](https://github.com/wjhuang88/talos/issues/199) owns generic resilience; coordinate API timing rather than independently implementing a generic breaker.
- TOOL-009 is historical existing search delivery, not automatically a new implementation claim.
- Parent [SEARCH-001](SEARCH-001-zero-config-global-search.md) owns the target product outcome and acceptance, not this child's execution.

## I288 Architecture Candidate

Evidence package:

- [I288 Search Architecture Audit](../../reference/I288-SEARCH-ARCHITECTURE-AUDIT-2026-09-30.md)
- [ADR-085: Talos-Owned Web Search Routing And Backend Boundary](../../decisions/085-talos-owned-search-routing-boundary.md)

Current candidate decisions:

- keep `web_search` tool name/schema/family/Network permission stable;
- model engine implementations as private `SearchBackend` strategies below the built-in ADR-072
  Search Provider, rather than registering each engine as a platform Provider;
- keep the first boundary inside `talos-tools/network`; do not create a public `talos-search`
  crate until a second supported consumer or SDK/release boundary justifies extraction;
- Talos owns backend identity, typed errors, first-valid-success routing and bounded hedging;
- no startup probes, no country/GeoIP routing, and no query text in health state;
- credential presence makes a premium backend available but does not imply user consent to use it;
- Wikipedia is transitional knowledge fallback, not a target generic web backend;
- named native zero-key candidates remain unadmitted until SEARCH-001-D records terms,
  maintainability, fixture, independence and real regional evidence.

I288 also records a deterministic test characterizing the current first-completion
`tokio::select!` behavior; it does not alter production routing.

## Uncertainty And Validation Path

Explicitly answer: is a separate crate necessary; what is `rust-websearch` actually exposing in this project; which zero-key endpoints are technically, operationally and legally maintainable; how will real mainland-China routing evidence be acquired; which failure classes require retry vs fallback; and where exactly does NET-001's shared policy end and the search domain policy begin? If unresolved, document a bounded feasibility Spike rather than shipping a provider on an assumption.

## State / Status Owners

- Requirement and acceptance truth: this file; discussion and review: [Issue #625](https://github.com/wjhuang88/talos/issues/625).
- Epic closure and dependency map: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [#624](https://github.com/wjhuang88/talos/issues/624).
- Derived compact backlog: `docs/backlog/PRODUCT-BACKLOG.md`. Only after selection/claim should the active Board and iteration be updated.

## User-Facing Documentation

This architecture phase does not advertise new features. It owns the documentation impact matrix required before future zero-config routing and premium-override behavior changes.

## Required Reads

- [Issue #625](https://github.com/wjhuang88/talos/issues/625); parent [#624](https://github.com/wjhuang88/talos/issues/624); network [#199](https://github.com/wjhuang88/talos/issues/199); CAP-001 [#466](https://github.com/wjhuang88/talos/issues/466).
- `AGENTS.md`, `docs/sop/REQUIREMENT-INTAKE.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/START-ITERATION.md`, `docs/sop/CHANGE-CONTROL.md`.
- `docs/backlog/active/TOOL-009-internet-search-tool.md`, `docs/decisions/072-capability-provider-bundle-boundary.md`, NET-001 backlog owner (if any).
- `crates/talos-tools/src/web_search.rs`, `crates/talos-tools/src/lib.rs`, `crates/talos-tools/Cargo.toml`, `Cargo.lock`.

## Acceptance For Technical / Governance Work

- [x] A reviewed code-truth matrix with explicit version, actual routes, current permissions and observed failure evidence exists.
- [x] Deterministic baseline evidence characterizes first-completed vs first-valid-success and fallback/cancellation cases.
- [x] Provider admission, independence, fixture and multi-region verification plan includes real mainland-China evidence acquisition with provenance.
- [x] Proposed ADR addresses crate/module decision, domain and shared-network boundary, default/advanced configuration, security/privacy and no-startup-network constraints.
- [x] B–I compatibility, dependency, test, activation and rollback gates are documented and accepted in normal architecture review.
- [x] No production API/config/dependency/network behavior changed under this child, and repository owner/compact backlog/Issue statuses are synchronized.

## Completion Evidence

- Completion Commit: `4567bf85ef1fb151fbb07a30e1f5f26a81feeed7` (PR #629 merge)
- Validation: `scripts/validate_project_governance.sh .`,
  `bash scripts/validate_collaboration_claims.sh .`, and
  `cargo test -p talos-tools --features network --locked` (105 passed)
- Residual: provider feasibility and regional evidence remain SEARCH-001-D / #644; no native
  provider is admitted by this architecture decision.
