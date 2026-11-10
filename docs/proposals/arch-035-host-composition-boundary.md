# ARCH-035: Proposed host composition decision (I320 E1)

> Status: Non-binding proposal — not accepted; governance claim PR #905. ADR number to be allocated at E1 acceptance, avoiding existing ADR-095.
> Date: 2026-11-10
> Owner: ARCH-035 / #897
> Dependency: STORE-002 / #670 (unclaimed at preparation time)
> Existing accepted baseline: ADR-042 embedded durable runtime session boundary

## Evidence before decision

E0 reproducible [cross-host code-truth inventory](../reference/ARCH-035-I320-E0-CODE-TRUTH-2026-11-10.md) identifies the current CLI `AppServerSession` construction, Desktop/SDK `RuntimeBuilder` usage, existing SQL/file domain stores, and baseline regression tests. Its paths are anchored to main at #905 merge `b211e590f6b1180af0679be8b1b03d77b0a19615`.

**Acceptance is deliberately split:** the host/domain ownership decision below is reviewable now. Public injection signatures and adapter selection **cannot** become Accepted without consuming concrete, accepted #670 contracts. No E2/E3 code may start merely from this proposal.

## Context

CLI/TUI, Desktop and embedders use existing runtime/session paths but differ in host-side construction. `talos-session::SessionManager` owns local directory, SQLite index and TLOG/JSONL stores; `talos-runtime` depends on `talos-session` and exports `RuntimeBuilder`; CLI TUI uses `AppServerSession` and a separate host builder; Desktop already imports `RuntimeBuilder`.

STORE-002 / #670 owns capability taxonomy and all domain persistence contracts and local adapters. This ADR cannot create an overlapping `StorageProvider` or promise a remotely portable `SessionStore` by repurposing the existing path-oriented file trait.

## Proposed decision (review gate, no implementation authority)

1. **One runtime execution authority:** preserve `talos-runtime` / `talos-agent::AppServerSession`, existing Session domain semantics and permission engine. CLI and Desktop remain product hosts, TUI remains presentation.
2. **Composition location:** host selects a named *local default composition profile* at the runtime-facing construction boundary. A future API may add a narrowly typed, additive selector/delegator after #670 contracts are accepted. Do not commit a new trait name or API signature in E1 before contract review.
3. **Local behavior:** initial profile delegates exactly to existing `SessionManager`, `DurableSession`, Memory/Evolution/Exploration stores as currently applicable. Preserve creation order, paths, errors, locks, recovery and redaction. No feature/default change.
4. **SDK compatibility:** retain `RuntimeBuilder::durable_session(DurableSession)` and unbound in-memory behavior per accepted ADR-042; additive composition must not make existing callers change.
5. **No duplicate contracts:** all Log/Relational/Object/Lexical/Vector/Embedding/Rerank definitions and domain adapters are exclusively #670. #897 integrates accepted seams only. If #670 already supplies host assembly, record it as satisfied rather than wrapping again.
6. **Dependency truth:** `talos-runtime -> talos-session -> rusqlite(bundled)` remains in the compiled dependency graph; injection is not an SQLite-free binary. Physical separation requires a separately accepted change.
7. **Server gate:** #361 may refine architecture but cannot claim production Host integration on this proposal. #360 remains sole owner of future remote persistence.

## Rejected alternatives

- A Server-private `StorageProvider` or SessionManager: duplicates domain authority.
- Rewriting all persistence to a generic ORM/SQL dialect: violates behavior preservation and #670 scope.
- Moving SQLite construction to `talos-tui`: renderer becomes a persistence owner.
- Moving every trait into `talos-core`: unnecessary dependency/ownership expansion.
- New backend, schema, format, FTS ranking, migration or transaction behavior: outside E-series.

## Compatibility / acceptance gates

- E0: exact-source dependency matrix and pre-change tests, including failure/restart/concurrency/permissions.
- E1: owner/API review of this proposal against accepted #670 deliverables; maintainers record explicit acceptance and any no-op disposition.
- E2/E3: blocked until #670 accepted and separately governed implementation activation; exact delegation only.
- E4: full locked workspace, cross-host and platform conformance, security/API review, exact-head CI, owner-first completion with existing evidence SHA.

No runtime implementation or test result is asserted by this proposal.

## E1 review questions and disposition conditions

1. Does preserving `RuntimeBuilder::durable_session(DurableSession)` (ADR-042) and CLI's direct `AppServerSession` path require an additive builder composition profile or can an existing #670 entrypoint be reused exactly? **Decision deferred pending #670**, not an invented public API.
2. Is there one terminal/recovery/permission authority under every supported host, with no product-owned persistence semantics? **Required**, verify against E0 call-site matrix.
3. Does TUI remain a renderer and does Desktop continue as a thin Runtime adapter? **Required**, no independent storage trait in either surface.
4. Do local SQLite/TLOG/JSONL/FTS5 behavior and bundled binary closure stay unchanged? **Required**, verified only once E2/E4 conformance evidence exists.
5. Which operations cannot be transparently injected (e.g., session ownership, migration/reset, generation fencing, cross-resource durability)? **Must be explicitly documented**; no generic CRUD facade or fictional atomicity.

Status remains Proposed until required independent API/architecture review and explicit maintainer acceptance. The nonbinding E0 inventory may be acknowledged separately; it does not unblock E2/E3.
