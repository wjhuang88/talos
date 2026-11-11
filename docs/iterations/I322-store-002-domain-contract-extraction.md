# Iteration I322: STORE-002 Domain-Owned Persistence/Retrieval Contracts (C)

> Document status: Review / Claimed — effective governance #911; C implementation candidate pending acceptance
> Published plan date: 2026-11-11
> Objective: extract existing domain-owned storage/retrieval operational interfaces **without changing implementation or a single existing caller**.
> Existing A/B baseline: I321 completed in implementation #909, merge `a9914b7a62b0873027997c69d9966d17f3fd48ba` (exact-head CI #4039 8/8, independent API review no blockers).
> Parent: STORE-002 / #670 remains OPEN until later D/E/F.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 I322 C only: extract minimal domain-owned typed persistence/retrieval traits from existing Session/PendingSubmission/Todo, Memory, Evolution and Exploration operations, documenting the seven A/B infrastructure categories, without changing existing concrete stores, SQL/paths/transactions, consumer dispatch, runtime/host composition or new backends |
| Claimed At | 2026-11-11 |
| Source Issue | #670 |
| Governance Claim PR | #911 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only C claim [#911](https://github.com/wjhuang88/talos/pull/911) merged `60da1bbadca00c9bdee6fbcb8f5ad990a4313331` after exact-head CI #4055, both governance validators 0 warnings, no-overlap CAS and documented single-maintainer path. Public C implementation requires independent API/architecture review. |
| Implementation PR | Pending C review candidate |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | C claim #911 is effective. C implementation needs exact-head platform CI, both governance validators and independent API/architecture review; D/E/F and #897 Host wiring remain separately gated. |

## Scope and source of truth

C0. Retain existing `talos-core::storage_capability::StorageCapabilityKind` seven-family IDs as **classification metadata only**. Explicitly do not resurrect generic `Bound<T>` or advertise a storage provider because it appears in `CapabilityRegistry`.

C1. Session transcript already has `talos-session::SessionStore` for JSONL/TLOG. **Reuse it; do not duplicate**. Document atomic turn replacement, recovery encoding, read/append/scan and file-format invariants. Extract domain-owned contract(s) only where absent and needed for PendingSubmission and Todo, preserving generation fences, admitted-work custody, atomic stage/commit, Todo revision/CAS/dependency semantics. Do not make path construction or SQL a generic Provider API.

C2. Memory `crates/talos-memory/src/store.rs`: add a narrow domain trait reflecting existing add-only, evidence/provenance and bounded FTS retrieval semantics. Preserve typed `MemoryStoreError` and existing scoring/rank. Graph operations remain explicitly domain-owned rather than generic vector-search.

C3. Evolution `crates/talos-evolution/src/store.rs`: add a narrow domain contract reflecting observations and active-pattern operations, conflict and schema semantics. Preserve existing error type/implementation; no schema/version/migration changes.

C4. Exploration `crates/talos-exploration/src/lib.rs`: add a narrow contract covering source/chunk provenance, citations and lexical search methods without changing FTS, source ingestion, transaction or result shapes. Existing `ExplorationError` remains the error contract.

C5. Object/VectorSearch/Embedding/standalone Rerank: classify as not bound at this layer; **do not** manufacture fake executable traits/providers. LexicalSearch and Relational remain owned by the Session/Memory/Evolution/Exploration domain methods; semantic retrieval stays a composition. The five runtime SQLite consumers accepted by ADR-008 (including quarantined `talos-models`) must remain unchanged.

## Source paths / compatibility baseline

- `crates/talos-session/src/store.rs` existing `SessionStore`; `crates/talos-session/src/pending_submission.rs`; `crates/talos-session/src/todo/repository.rs`; `crates/talos-session/src/todo.rs`
- `crates/talos-memory/src/store.rs`, `crates/talos-memory/src/lib.rs`, typed `MemoryStoreError`
- `crates/talos-evolution/src/store.rs`, `crates/talos-evolution/src/lib.rs`
- `crates/talos-exploration/src/lib.rs`, existing `ExplorationError`, `source_chunks_fts`
- Existing authority inventory `docs/reference/STORE-002-I321-CAPABILITY-AUTHORITY-INVENTORY-2026-11-10.md`; ADR-002, ADR-008 and normative `docs/reference/SQLITE-CONSUMER-POLICY.json`
- Existing architecture consumer `docs/backlog/active/ARCH-035-behavior-preserving-host-composition.md` / #897 E2/E3 remains blocked.

## Contract design requirements

1. Domain trait methods use **existing** domain data types and error/return shapes, borrowing and mutability; traits are object-safe only when actually required by an existing consumer, not speculatively.
2. **No implementation substitution in C.** Existing constructors, concrete stores and call sites remain authoritative. C should extract the interfaces with a clear path for later exact-delegation adapters in D; it must not switch any caller to a new binding or default.
3. No free-form SQL/ORM, `Arc<dyn GenericStorageProvider>`, fake vector embedding or universal CRUD. No SQLite connection or C/native edge in `talos-core`.
4. Concrete code and tests must compile on current supported platforms without adding dependencies; documentation tests should distinguish future contracts from currently operational bindings.
5. No new authorization for Server, CLI/TUI/Desktop/SDK wiring, background tasks or alternate engines.

## Acceptance evidence

- Existing `SessionStore` reused with no signature or format changes. Every added contract method is traceable to existing production code, with the original typed errors and domain invariants stated.
- All four domain families (Session/Pending/Todo, Memory, Evolution, Exploration) have explicit, minimal operational interface coverage; seven infrastructure categories and unavailable status remain accurately classified.
- Existing stored formats, FTS snippets/ranks, SQL, schema/version, transaction/CAS, error semantics and generation fencing are byte-for-byte untouched in implementation files, except additive declarations/export where necessary.
- No pre-existing public API is removed or changed, no existing constructor/caller is rewired, and no `Cargo.toml`/`Cargo.lock` change.
- `cargo fmt --all -- --check`, locked crate/workspace check/clippy/tests, both governance validators, cross-platform exact-head CI, and **independent public API/architecture review** are required before accepting any implementation PR.

## Out of scope, dependencies and stop conditions

- D: forward existing SQLite/FTS5/TLOG/JSONL implementations through accepted contracts with no behavior drift; separate claim.
- E: one default local implementation composition; separate claim.
- F: before/after characterization for crash/recovery, CAS, ranking, concurrency, migrations and SDK compatibility; separate claim.
- #897 E2/E3 Host injection consumes D/E contracts; no duplicate extraction, and not unblocked by C alone.
- #360 remote relational, #361 ServerHost, Zvec/STORE-001, S3, model embedding pipeline and any new persistence format remain out of scope.
- I319 Search, TUI and independent Git/Windows branches retain ownership; never rewrite their files to absorb drift.
- On public API design ambiguity or source-truth contradiction, stop and request independent architectural review rather than introducing generic operational bindings.

## Execution / Verification / Completion

Governance claim #911 merged to main as `60da1bbadca00c9bdee6fbcb8f5ad990a4313331` after exact-head [CI #4055](https://github.com/wjhuang88/talos/actions/runs/42948954104), both governance validators 0 warnings, and recorded merge-time CAS. **C code is authorized** under this bounded claim, but its implementation candidate is not accepted until independent public API review, exact-head CI and merge. D/E/F remain separately gated. `I322` is not Complete and #670 stays OPEN until an actual C implementation merge.

## I322 C execution checkpoint — 2026-11-11

- Effective governance: [#911](https://github.com/wjhuang88/talos/pull/911), merged `60da1bbadca00c9bdee6fbcb8f5ad990a4313331`.
- [Source-aligned contract map](../reference/STORE-002-I322-DOMAIN-OPERATIONAL-CONTRACT-MAP-2026-11-11.md): existing `SessionStore` retained; PendingSubmission/Todo, Memory, Evolution and Exploration domain contracts exported, with concrete forwarding implementations and compile/in-memory tests.
- No existing store, SQL/FTS, TLOG/JSONL, SQLite schema, transaction/locking, public constructor, runtime caller or Cargo manifest modified. Trait conformance does **not** mean alternate provider readiness.
- **Implementation PR, native CI, independent public API review and merge-time CAS pending.** No C completion commit; #670 stays OPEN and D/E/F/Host blocked.
