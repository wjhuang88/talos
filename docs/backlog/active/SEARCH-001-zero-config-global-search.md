# SEARCH-001: Zero-Config Global Search Architecture & Provider Internalization

**Status**: Intake / Unclaimed
**Type**: Architecture / Domain Epic
**Parent Epic**: None

| Field | Value |
|---|---|
| Story ID | SEARCH-001 |
| Priority | Proposed P2; not selected into the active Desktop cycle |
| Source Issue | [#624](https://github.com/wjhuang88/talos/issues/624) |
| First child | [SEARCH-001-A / #625](SEARCH-001-A-architecture-migration-contract.md) |
| Selected Iteration | None — Epic parents are not executable iterations |
| Related | TOOL-009; CAP-001 / #466; ADR-072; NET-001 / #199; WEBFETCH-001; TOOL-012/013/014 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | None — the Epic is a coordination owner; children require separate claims |
| Claimed At | Not applicable |
| Source Issue | #624 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None — Epic parents are not implementation units |
| Last Updated | 2026-10-03 |
| Handoff / Release Condition | Keep parent open until the terminal-state acceptance below is met and residuals are owned. |

## Identity / Goal / Value

A user should be able to issue a web search from a fresh Talos installation in a wide variety of networks, including real mainland-China network environments, without choosing a provider or obtaining keys. Network reachability should guide routing; explicit premium or self-hosted configuration should remain optional. This is a **best-effort** product objective; an offline machine, upstream access controls, regional/legal restrictions, or widespread upstream failure cannot be solved or guaranteed by a zero-configuration local-only client.

Talos must own the domain contract, routing decisions, safety boundaries and compatibility tests. No single external scraper crate should remain a critical dependency of the default search capability.

## Current Code Truth — Intake Snapshot (A must verify)

- `crates/talos-tools/src/web_search.rs` currently implements `WebSearchTool` plus DuckDuckGo via `rust-websearch`, optional direct Tavily/SearXNG requests and Wikipedia OpenSearch fallback.
- `crates/talos-tools/Cargo.toml` includes optional `rust-websearch = "0.1"` under the `network` feature; inspected lockfile contains `rust-websearch 0.1.2`.
- Existing `tokio::select!` picks a first-completed branch. Confirm and test whether a fast failure causes fallback while other providers remain in flight, contrary to intended first-valid-success behavior.
- ADR-072 already governs Capability/Provider vocabulary and lifecycle. NET-001/#199 already owns generic retry/circuit-breaker semantics; reuse rather than duplicate.

This snapshot is not a proof of supported regional endpoints, provider terms, or shipped auto-selection behavior.

## Scope

- Accept an architecture through separately governed child A before new runtime work.
- Preserve the externally visible `web_search` capability, Network permission and progressive Tool disclosure during migration.
- Make default routing no-config, runtime-owned and recoverable across network/proxy changes; never hardcode a country→provider rule or make a startup network probe mandatory.
- Establish security, privacy, provider terms/maintainability and evidence requirements before native zero-key provider admission.
- Incrementally introduce at least two meaningfully independent native zero-key routes into the default distribution, without a forced user setup step.
- Normalize opt-in premium/keyed/self-hosted override without automatically spending paid quotas simply because a key exists.
- Set a measured `rust-websearch` de-risk/retirement gate rather than promising an untested removal.

## Exclusions

- No production provider code, new crate, dependency, public API, persistence/config migration or behavior change is authorized by this parent.
- No guaranteed universal availability, default proprietary Talos relay, automatic VPN/proxy, CAPTCHA bypass, upstream access-control circumvention or unsolicited startup network traffic.
- No competing global lifecycle, retry system or circuit breaker independent of ADR-072 and #199.
- Do not pre-open all proposed future child Issues or select the Epic itself as an iteration.

## Child Dependency Map

The table is a proposal for child **outcomes**, not blanket implementation authorization. A decides exact crate boundaries and revised dependencies.

| Child | Outcome | Depends on | Issue / State |
|---|---|---|---|
| SEARCH-001-A | Code-truth audit, migration matrix, architecture ADR, provider qualification gates | None | [#625](https://github.com/wjhuang88/talos/issues/625), Complete / Claimed; ADR-085 accepted in #629 (`4567bf85`) |
| SEARCH-001-B | Domain extraction with current-behavior-compatible adapters | A ADR accepted | [#642](https://github.com/wjhuang88/talos/issues/642), Proposed / Unclaimed |
| SEARCH-001-C | Talos-owned auto router, deadlines, error semantics, #199 integration | B / A | [#643](https://github.com/wjhuang88/talos/issues/643), Proposed / Unclaimed |
| SEARCH-001-D | Provider feasibility and real multi-region (incl. mainland China) evidence gate | A; research may overlap B | [#644](https://github.com/wjhuang88/talos/issues/644), Proposed / Unclaimed |
| SEARCH-001-E | First Talos-native zero-key provider with fixtures | C / D | Proposed only |
| SEARCH-001-F | Second meaningfully independent native zero-key provider | C / D / E interface | Proposed only |
| SEARCH-001-G | Global auto-routing, change recovery, smoke/soak validation | E / F | Proposed only |
| SEARCH-001-H | Opt-in premium and self-hosted overrides, secret and fallback policy | C | Proposed only |
| SEARCH-001-I | Evidence-based `rust-websearch` de-risk/retirement gate | G / H | Proposed only |

After A is accepted, create only the next independently deliverable, dependency-ready children as linked Issues and owner documents. Each needs its own selected iteration, claim, tests, review and release-compatible rollback.

## Decision Links And Constraints

- ADR-072: Capability and Provider boundaries; provider implementation identity is not Plugin installation or permission grant.
- NET-001 / #199: shared bounded network retry and per-failure-domain circuit breaking; search policy may select providers, not quietly duplicate a global mechanism.
- TOOL-009, TOOL-012/013/014 and WEBFETCH: preserve search tool registration, permission and existing URL-fetch separation.
- Code dependencies and default configuration remain unchanged until a reviewed child explicitly authorizes a migration.

## Uncertainty And Validation Path

Child A must verify `rust-websearch` behavior and any regional routing claims against actual source and controlled evidence. Treat DDG/Bing/other endpoints as candidates, never promises; record provenance of mainland-China and other regional tests, DNS/TLS/HTTP/parser observations, provider terms risk, independent failure domains and update/fixture ownership. CI cannot stand in for missing geographic test evidence.

## State / Status Owners

- Epic contract, child map and final closure: this owner document.
- Requirement discussion and external progress: [Issue #624](https://github.com/wjhuang88/talos/issues/624).
- First executable architecture Story: [SEARCH-001-A](SEARCH-001-A-architecture-migration-contract.md) / [#625](https://github.com/wjhuang88/talos/issues/625).
- Compact inventory: `docs/backlog/PRODUCT-BACKLOG.md`; selected work only on `docs/BOARD.md`.
- Architecture decision: future SEARCH architecture ADR once accepted; no new ADR number reserved here.

## Required Reads

- [Issue #624](https://github.com/wjhuang88/talos/issues/624), [child #625](https://github.com/wjhuang88/talos/issues/625), [network #199](https://github.com/wjhuang88/talos/issues/199) and [CAP-001 #466](https://github.com/wjhuang88/talos/issues/466).
- `docs/sop/REQUIREMENT-INTAKE.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/START-ITERATION.md`, `AGENTS.md`.
- `docs/backlog/active/TOOL-009-internet-search-tool.md`, `docs/decisions/072-capability-provider-bundle-boundary.md`.
- `crates/talos-tools/src/web_search.rs`, `crates/talos-tools/Cargo.toml`, `Cargo.lock`.

## Acceptance / Completion Definition

- [ ] Search domain contract/router and fallback ownership are shipped with compatible tool and permission behavior.
- [ ] At least two independently admitted Talos-native zero-key providers can satisfy a default search without `rust-websearch`.
- [ ] Default auto routing and explicit optional premium override are user-documented, permission-safe and recover across network changes.
- [ ] Controlled tests and provenance-backed real-network evidence cover multiple regions including mainland China; limitations are honestly documented.
- [ ] Fixtures, cancellation, error classification, security/privacy gates and real smoke/soak results are recorded.
- [ ] `rust-websearch` is demonstrably no longer a critical default-search dependency; the disposition (optional adapter or removal) has a reviewed decision and rollback.
- [ ] All child completion evidence, owner/Board/backlog status and remaining residuals are reconciled before Epic closure.

Completing a trait, router scaffold or first provider alone **must not** close this Epic.
