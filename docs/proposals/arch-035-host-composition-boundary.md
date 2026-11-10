# ARCH-035: Proposed host composition decision (I320 E1)

> Status: Non-binding proposal — not accepted; governance claim PR #905. ADR number to be allocated at E1 acceptance, avoiding existing ADR-095.
> Date: 2026-11-10
> Owner: ARCH-035 / #897
> Dependency: STORE-002 / #670 (unclaimed at preparation time)
> Existing accepted baseline: ADR-042 embedded durable runtime session boundary

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
