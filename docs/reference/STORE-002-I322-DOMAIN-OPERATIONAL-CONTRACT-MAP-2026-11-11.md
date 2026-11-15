# STORE-002 / I322 C — Domain Operational Contract Mapping

> State: implementation review candidate (not an accepted provider or migration)
> Claim authority: governance #911 merged `60da1bbadca00c9bdee6fbcb8f5ad990a4313331`
> Parent: #670, open until D/E/F; I321 A/B already merged via #909
> No SQL/SQLite/FTS5/TLOG/JSONL, file paths, migrations, transactions, locks, caller composition or dependencies changed

## Operational contracts and current concrete implementers

| Domain | Existing source of truth | Additive contract | Actual implementation | Important invariants preserved |
|---|---|---|---|---|
| Session transcripts | `talos-session/src/store.rs` | **Existing** `SessionStore` (reused, not duplicated) | TLOG `CompactTextSessionStore`, legacy `JsonlSessionStore` | Read/append, durable-turn atomic replace, bounded recovery encoding and file extension |
| Session pending submissions | `talos-session/src/pending_submission.rs` | `PendingSubmissionPersistence` | Exact forwarding `impl` for `PendingSubmissionStore` | Runtime generation CAS, activation stage/commit, submission custody/accept, committed receipt, running recovery |
| Session Todo state | `talos-session/src/todo/repository.rs` | `TodoPersistence` | Exact forwarding `impl` for `TodoRepository` | Existing session scope, batch update atomicity, dependency checks and work-graph identity |
| Memory | `talos-memory/src/store.rs` | `MemoryPersistence` | Exact forwarding `impl` for `MemoryStore` | ADD-only duplicate/conflict behavior, evidence provenance, bounded ranked FTS retrieval, graph recall/retention |
| Evolution | `talos-evolution/src/store.rs` | `KnowledgePersistence` | Exact forwarding `impl` for `KnowledgeStore` | Observation/pattern storage, confidence filtering, deactivation, existing schema/version/legacy migration |
| Exploration | `talos-exploration/src/lib.rs` | `ExplorationPersistence` | Exact forwarding `impl` for `ExplorationStore` | Source+chunk atomic insertion, citation and claim-edge integrity, FTS ranking, synthesis citations |

The new traits use the domains' **existing** types, error shapes, borrowing and mutation semantics. Forwarding methods call the concrete inherent implementations by their fully-qualified names, rather than reimplementing any business/transaction rule. No concrete existing method is removed, renamed, or rewired.

## Availability is distinct from metadata

- `talos-core::storage_capability::StorageCapabilityKind` remains only a seven-category identity/request classification: Log, Relational, Object, LexicalSearch, VectorSearch, Embedding and Rerank.
- `CapabilityRegistry` answers **descriptor metadata**, never operational backend readiness. I321 removed the unconstrained `Bound<T>` marker after independent API review.
- Current domain storage methods implement Session logs, local relational records and FTS5 lexical retrieval. This C declaration **does not** create seven interchangeable engines, a generic CRUD/SQL API or a cross-domain Provider registry.
- No shared object/blob provider, vector index, embedding provider or standalone reranker is installed by C. Domain ranking/graph recall must not be mislabeled as external Rerank or VectorSearch.
- The quarantined `talos-models` SQLite catalog is not added to runtime composition and no new direct SQLite crate dependency is introduced.

## Tests and acceptance boundaries

- Each new domain contract has a compile/type conformance test. Where the crate already supports an in-memory concrete store, tests exercise a trait object against that store without requiring a new provider factory.
- The existing concrete-store and workspace tests remain the regression authority; **no implementation substitution** is performed. Passing these tests demonstrates that the additive traits compile and forward correctly at the covered entrypoints, not that backend migration or cross-provider equivalence is complete.
- Required: exact-head format/check/clippy/tests, both governance validators and platform CI; independent Rust API/architecture review must confirm object-safety needs, operation coverage, domain boundaries and no misleading general-purpose binding.
- D is responsible for any subsequent migration through accepted local delegates, E for one local default profile, F for explicit before/after crash, persistence, FTS, error, concurrency and replay equivalence. Those slices have **not** been claimed by C. #897 E2/E3 remains gated until accepted functional delegates.

## Changes allowed in C

Additive declarations and concrete forwarding `impl` blocks in new `storage_contracts.rs` files, plus public exports from four domain `lib.rs` files. No changes to existing storage implementations, SQL, schema, public signatures of existing methods, `Cargo.toml` or `Cargo.lock`. No CLI, Runtime, Desktop, SDK, Server or Search changes.
