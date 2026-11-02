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
| Last Updated | 2026-11-01 |
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

The table is a proposal for child **outcomes**, not blanket implementation authorization. Completed
children retain their own authoritative owners; blocked/proposed children still require their own
accepted claims before executable work.

| Child | Outcome | Depends on | Issue / State |
|---|---|---|---|
| SEARCH-001-A | Code-truth audit, migration matrix, architecture ADR, provider qualification gates | None | [#625](https://github.com/wjhuang88/talos/issues/625), Complete / Claimed; ADR-085 accepted in #629 (`4567bf85`) |
| SEARCH-001-B | Domain extraction with current-behavior-compatible adapters | A ADR accepted | [#642](https://github.com/wjhuang88/talos/issues/642), Complete / Claimed; I292; #653/#656 plus closeout evidence merged |
| SEARCH-001-C | Talos-owned auto router, deadlines, error semantics, #199 integration | B / A | [#643](https://github.com/wjhuang88/talos/issues/643), Complete / Closed; I293; Completion Commit `eb870cdf0a63f45f20d89f1a50eba276f9198643` (#808) |
| SEARCH-001-D | Provider feasibility and real multi-region (incl. mainland China) evidence gate | A; research may overlap B | [#644](https://github.com/wjhuang88/talos/issues/644), Complete / Closed with **BLOCKED / ZERO-ADMISSION** outcome; I294; #831 merge `bdb98b2b47dbb76111f5b053a226efbb033c6b34` |
| SEARCH-001-E | First Talos-native zero-key provider with fixtures | C + D admission | **Blocked** — D admission did not occur; no implementation authorization |
| SEARCH-001-F | Second meaningfully independent native zero-key provider | C + D admission + E interface | **Blocked** — D admission did not occur; no implementation authorization |
| SEARCH-001-G | Global auto-routing, change recovery, smoke/soak validation | E / F | Blocked on E/F |
| SEARCH-001-H | Opt-in premium and self-hosted overrides, secret and fallback policy | C | [#832](https://github.com/wjhuang88/talos/issues/832) / [owner](SEARCH-001-H-explicit-search-overrides.md), Proposed / Unclaimed; dependency-ready intake only |
| SEARCH-001-I | Evidence-based `rust-websearch` de-risk/retirement gate | G / H | Proposed only; blocked on G and incomplete H |

After A is accepted, create only the next independently deliverable, dependency-ready children as linked Issues and owner documents. Each needs its own selected iteration, claim, tests, review and release-compatible rollback. The existence of #832/H is intake only and does not authorize implementation.

## Current Coordination Checkpoint — 2026-11-01

SEARCH-001-C is complete at #808. SEARCH-001-D is also complete as a research gate, but its final
#831 disposition admitted **zero** native providers. Therefore E/F remain unauthorized and G cannot
start. H is now the only dependency-ready proposed child because it depends on C rather than D
admission; #832 and its owner record intake only. Parent #624 remains open.

## Decision Links And Constraints

- ADR-072: Capability and Provider boundaries; provider implementation identity is not Plugin installation or permission grant.
- ADR-085 / ADR-090: accepted SearchBackend/SearchRouter and invocation-bound endpoint-authority boundaries established by completed A/C work.
- NET-001 / #199: shared bounded network retry and per-failure-domain circuit breaking; search policy may select providers, not quietly duplicate a global mechanism.
- TOOL-009, TOOL-012/013/014 and WEBFETCH: preserve search tool registration, permission and existing URL-fetch separation.
- Code dependencies and default configuration remain unchanged until a reviewed child explicitly authorizes a migration.

## Uncertainty And Validation Path

D's blocked closeout does not establish regional availability or native-provider admission. Its
accepted evidence provides a reproducible bounded acquisition path and records why Mwmbl/Purili
remain unadmitted. Any future native-provider admission requires new governed evidence including
real target-network observations; CI cannot stand in for missing geographic evidence.

## State / Status Owners

- Epic contract, child map and final closure: this owner document.
- Requirement discussion and external progress: [Issue #624](https://github.com/wjhuang88/talos/issues/624).
- Architecture decision: [SEARCH-001-A](SEARCH-001-A-architecture-migration-contract.md) / #625 and accepted ADR-085.
- Completed router child: [SEARCH-001-C](SEARCH-001-C-search-router.md) / #643.
- Completed blocked provider-evidence child: [SEARCH-001-D](SEARCH-001-D-provider-evidence-gate.md) / #644.
- Dependency-ready intake: [SEARCH-001-H](SEARCH-001-H-explicit-search-overrides.md) / #832.
- Compact inventory: `docs/backlog/PRODUCT-BACKLOG.md`; selected work only on `docs/BOARD.md`.

## Required Reads

- [Issue #624](https://github.com/wjhuang88/talos/issues/624), completed children #625/#642/#643/#644, proposed child #832, network #199 and CAP-001 #466.
- `docs/sop/REQUIREMENT-INTAKE.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/START-ITERATION.md`, `AGENTS.md`.
- `docs/backlog/active/TOOL-009-internet-search-tool.md`, `docs/decisions/072-capability-provider-bundle-boundary.md`, ADR-085 and ADR-090.
- `crates/talos-tools/src/web_search.rs`, `crates/talos-tools/Cargo.toml`, `Cargo.lock`.

## Acceptance / Completion Definition

- [x] Search domain contract/router and fallback ownership are shipped with compatible tool and permission behavior. SEARCH-001-C completed at #808.
- [ ] At least two independently admitted Talos-native zero-key providers can satisfy a default search without `rust-websearch`. SEARCH-001-D closed blocked with zero admissions.
- [ ] Default auto routing and explicit optional premium override are user-documented, permission-safe and recover across network changes. H remains Proposed / Unclaimed.
- [ ] Controlled tests and provenance-backed real-network evidence cover multiple regions including mainland China; limitations are honestly documented.
- [ ] Fixtures, cancellation, error classification, security/privacy gates and real smoke/soak results are recorded for the terminal chain.
- [ ] `rust-websearch` is demonstrably no longer a critical default-search dependency; the disposition (optional adapter or removal) has a reviewed decision and rollback.
- [ ] All child completion evidence, owner/Board/backlog status and remaining residuals are reconciled before Epic closure.

Completing the router or closing D with blocked admission **must not** close this Epic.
