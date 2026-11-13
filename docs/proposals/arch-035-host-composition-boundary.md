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

## I320 E1 dependency review update — 2026-11-13 (non-binding)

This is a follow-up within claim #905 / ARCH-035 E0/E1, not an ADR acceptance, approved SDK signature, runnable conformance result or E2/E3 authorization. The original 2026-11-10 read-only E0 inventory remains its **historical baseline**. Subsequent STORE-002 progress must not be retroactively attributed to that inventory.

### Accepted upstream evidence and remaining boundaries

| Concern | Current accepted artifact | ARCH-035 permitted use / remaining gate |
|---|---|---|
| Seven capability categories | STORE-002 A/B PR #909, merge `a9914b7a62b0873027997c69d9966d17f3fd48ba` | Classification/registry is metadata, **not** executable storage injection. No second capability registry. |
| Session transcripts | Existing `talos-session::SessionStore` for TLOG/JSONL | Preserve file-oriented contract, SessionManager authority, durable turn and recovery rules. Do not recast it as a database-neutral repository. |
| Session custody / Todo | I322 C PR #912, merge `3df409ef9ed707756350f2c7d4b2c8819f157222` | Reuse domain-owned `PendingSubmissionPersistence` and `TodoPersistence` with direct existing-store forwarding; preserve generation fencing, CAS, custody and dependencies. |
| Memory / Evolution / Exploration | Same accepted I322 C implementation; bounded closeout #930, merge `a07b466d5aa2a03fb841cdcb918cd4968c778942` | Reuse `MemoryPersistence`, `KnowledgePersistence` and `ExplorationPersistence`; no new SQL, FTS ranking, schema, migration or business logic. |
| Local adapter delegation | STORE-002 I325 D governance #932, merge `e49ccde70556403a9d7b66551565e2b6c81ad32d` | **Only D claim/activation is accepted**, not D implementation at this checkpoint. Do not mark local delegation operational from a governance merge. |
| Local profile selection / broad parity | STORE-002 E / F (subsequent independently governed stages) | Do not claim a shipped default composition entrypoint, injectable backend, runtime API, or fault/recovery equivalence before evidence exists. |
| Host integration and SDK | ARCH-035 E2/E3 (not claimed) | Once upstream artifacts are accepted, adapt exactly one existing runtime path by direct delegation; no duplicated store/provider contracts, no Server-private agent/session authority. |

### E1 architecture/API placement under review

The proposed direction is intentionally **conditional**, rather than naming a speculative public `RuntimeBuilder` method:

1. Preserve `RuntimeBuilder::durable_session(DurableSession)` and optional unbound builder behavior under ADR-042. Existing embedders must remain source-compatible.
2. Keep `talos-cli` as its current host selection/composition root, `talos-tui` as renderer, and `talos-desktop` as a thin runtime adapter; the CLI's `AppServerSession` and Desktop/SDK `RuntimeBuilder` are currently **different construction entrypoints**, not already one shared host factory.
3. At E2 design time **first inspect** the accepted STORE-002 E default-local composition for a reusable supported entrypoint. If it serves the required host path, adopt it and record a no-op/satisfied disposition rather than inserting a wrapper. Only if a proven gap remains should ARCH-035 propose a *minimal, additive runtime-facing delegator*, with explicit API review, source-compatibility tests and no added persistence rules. A hypothetical API name/signature is not approved here.
4. Keep Session/Agent/permissions, first-terminal-wins, cancellation, pending custody, restart recovery, redaction and cross-resource ordering inside their current domain/runtime authority. Host selection must never implement those rules, imply global SQLite+TLOG ACID, or change errors/default paths.
5. Track the physical dependency chain `talos-runtime -> talos-session -> rusqlite (bundled)` as present until independently verified otherwise. An injected Rust trait does not prove an SQLite-free binary.

### Explicit E1 review questions before acceptance

- Is the no-duplication matrix above faithful to the accepted I322 C Rust traits/forwarders, including their real object-safety and typed-error constraints?
- Which accepted STORE-002 D/E operations truly allow host substitution without violating SessionManager lifecycle, provenance, error ordering or default paths?
- Can CLI's `AppServerSession` composition and Desktop/embedded `RuntimeBuilder` remain on their current execution authorities without introducing a second host persistence contract?
- Is an **additive** public API genuinely necessary, or can the accepted upstream assembly be reused with no new API? Identify the exact implementation caller and current tests before selecting a signature.
- Which negative-path, replay/restart, SDK fixture and native CI results must be frozen as before/after evidence for a future E2/E4 claim?

**Disposition at this checkpoint:** E1 stays Proposed, no ADR allocated, and the cross-host API choice stays open. I322 C supplies real domain contracts, but #932 supplies **authorization for D**, not its passing implementation. E2/E3 remain blocked on accepted usable upstream composition and an independent governed ARCH-035 implementation claim.
