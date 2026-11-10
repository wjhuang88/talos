# STORE-002 / I321 — Current Capability And Data Authority Inventory

> State: A/B review candidate, 2026-11-10
> Scope: #670 / STORE-002; **source-backed characterization, not a migration or runtime behavior change**
> Baseline: I321 claim merge `201a9b61d72e704da3a86f5db283b9f6e4d92938`
> Dependent consumer: ARCH-035 / #897 E2/E3 (Host composition); not owned by STORE-002

## Existing stores, authority and rebuildability

| Data family | Existing owner / source | Current storage | Authority and rebuild boundary |
|---|---|---|---|
| Session transcript, tree and turns | `crates/talos-session/src/store.rs`, `crates/talos-session/src/compact_text.rs`, `crates/talos-session/src/manager.rs` | `CompactTextSessionStore` TLOG default, `JsonlSessionStore` legacy; the existing `SessionStore` trait | **Authoritative** serialized conversation record; atomic durable-turn replace, encoding, redaction and idempotency stay in Session |
| Session listing, search and metadata projection | `crates/talos-session/src/sqlite.rs` | SQLite + FTS5 `SessionIndex` | **Derived/indexed** for search and metadata where reconstructable from transcripts; do not assume every index column or continuation record can be blindly rebuilt |
| External durable-session identity bindings | `crates/talos-session/src/durable.rs` | SQLite bindings with UUID-backed TLOG files | **Authoritative mapping**; the opaque external-ID → UUID identity cannot be reconstructed from filename guesses |
| Pending submissions, runtime generation and lifecycle | `crates/talos-session/src/pending_submission.rs`, `crates/talos-session/src/runtime_state.rs` | SQLite session-owned sidecars | **Authoritative custody/state** for incomplete work, generation fences, lifecycle and restart; **not** a disposable search index |
| Todo and dependency graph | `crates/talos-session/src/todo/repository.rs` | SQLite transactional repository | **Authoritative domain state**; preserve revision/CAS, rollback, dependency checks and write ordering |
| Memory facts, provenance, entity links and graph | `crates/talos-memory/src/store.rs`, `crates/talos-memory/src/graph.rs` | SQLite relational records and `memory_fts` | Facts/evidence/entities/edges are **authoritative**; FTS is **derived** from records in principle, but availability of an automated rebuild path is not asserted |
| Evolution observations, signals, patterns, conflicts | `crates/talos-evolution/src/store.rs`, `crates/talos-evolution/src/hook.rs` | SQLite `knowledge.db` | **Authoritative learned/domain state**; current migration/version and legacy v1 reset behavior must not be modified |
| Exploration runs, sources/chunks, claims/edges and syntheses | `crates/talos-exploration/src/lib.rs`, `crates/talos-exploration/src/ingestion.rs` | SQLite relational records and `source_chunks_fts` | Research provenance, citation edges and source data are **authoritative**; FTS is **derived** from stored text; no new rebuild guarantee |
| Quarantined historical model catalog | `crates/talos-models/src/store.rs` | SQLite `catalog.db` | **Non-runtime** historical catalog; MUST NOT become a CLI/Runtime dependency or enter local Host composition |
| Workspace artifacts and transient tool outputs | Existing local filesystem/Session/host-specific paths | Local filesystem or in-memory presentation state | Ownership depends on artifact type and host. No general `ObjectProvider` exists; ephemeral UI state is **not authoritative persistence** |

Data labeled `derived` is *conceptually* reconstructable from a source of truth; this document does **not** authorize index deletion, rebuild/replay, or changes to consistency and reconciliation guarantees.

## All seven capability categories

| Capability | Existing implementer / operations | First-phase contract rule |
|---|---|---|
| **Log** | Session TLOG/JSONL `SessionStore`: append, replay, atomic replacement | Reuse the existing Session trait/semantics; a generic Log interface must not weaken turn atomicity |
| **Relational** | Bundled `rusqlite` in Session, Memory, Evolution, Exploration; quarantined Models | Identify contracts without generic ORM, SQL DSL, transactional fiction or moving SQL out of domain stores |
| **Object** | Applicable local filesystem behavior, not a shared object service | Define type/availability boundary; no S3 or host filesystem rewrite |
| **LexicalSearch** | Session `SessionIndex`, Memory FTS5, Exploration FTS5 | Explicitly separate search capability from domain stores without changing filters, BM25, snippets or rank |
| **VectorSearch** | No bound production implementation | Explicit **Unavailable** until a typed provider is supplied; no Zvec dependency |
| **Embedding** | No dedicated infrastructure/storage-layer binding | Explicit **Unavailable** at this layer; model-side functionality is not silently counted as an installed storage provider |
| **Rerank** | Existing domain scoring/ranking (e.g. Memory retrieval), no standalone provider | Contract distinct from domain ranking; **Unavailable** for independent provider, never silently replace ranking |

Semantic retrieval is *composition* of available LexicalSearch, optional VectorSearch/Embedding and optional Rerank, **not** an eighth storage engine. A capability identity or descriptor alone is not an executable provider; later functional bindings require accepted domain/implementation contracts.

## Dependency and ownership proof boundaries

- [ADR-002](../decisions/002-local-storage-architecture.md) originally deferred trait extraction until a second implementation need. #670 records that concrete need, but still preserves the current implementation.
- [ADR-008](../decisions/008-sqlite-bundled-storage.md), [SQLite consumer inventory](SQLITE-CONSUMER-INVENTORY.md) and normative [SQLite consumer policy](SQLITE-CONSUMER-POLICY.json) restrict direct bundled SQLite users to five crates. Do not add a direct SQLite edge to `talos-core`, CLI, Desktop or Runtime.
- Existing `talos-core::capability::CapabilityDescriptor`, `ProviderDescriptor`, `CapabilityRegistry` and `CapabilityRequest` are the common provider identity/metadata foundation. I321 B introduces only a typed **storage-specific classification and bound/unavailable marker**, not another provider registry or a second `Provider` nomenclature.
- Domain invariants stay in `SessionStore`, `SessionManager`, `PendingSubmissionStore`, `TodoRepository`, `MemoryStore`, `KnowledgeStore`, `ExplorationStore`. B **does not** add any operational method to them.
- #670 owns future domain contracts and equivalent existing adapters. #897 owns cross-host selection/injection after accepting those concrete outputs; #361 ServerHost and #360 remote relational implementation remain separate.

## Validation targets and honest limitations

Current expected characterization includes:
- Session: durable turn idempotence, crash/recovery, owner/lock/generation fencing, transcript format.
- Relational: schema, SQL, migrations, transaction scope, busy/locked/error paths and Todo rollback.
- Memory: ADD-only conflict/dedup, evidence/graph, FTS score and bounded return.
- Evolution: schema/version and v1 legacy reset, observations/signals/patterns.
- Exploration: citation integrity, source FTS, reopen and Unicode snippets.
- CLI/TUI/Desktop/SDK: zero change to current defaults and public call surfaces.

Existing tests are inventoried in [ARCH-035 E0 code truth](ARCH-035-I320-E0-CODE-TRUTH-2026-11-10.md). **No production storage code changes in I321 B**, so the new contract tests cannot be represented as before/after implementation equivalence proof. C/D/E/F must run actual existing and new conformance suites before accepting adapter migration.

## Later slices (explicit residuals)

C: domain contract extraction with real operational signatures anchored in current stores; D: identical SQLite/TLOG/JSONL forwarding; E: one default local binding/profile; F: behavior, recovery and multi-platform equivalence audit. **Do not mark #670 Complete or unblock #897 E2/E3 on this A/B foundation alone.**
