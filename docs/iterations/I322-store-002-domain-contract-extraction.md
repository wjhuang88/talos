# Iteration I322: STORE-002 Domain-Owned Persistence/Retrieval Contracts (C)

> Document status: Complete / Closed — bounded I322 C accepted; parent #670 remains Open for D/E/F
> Published plan date: 2026-11-11
> Objective: extract existing domain-owned storage/retrieval operational interfaces **without changing implementation or a single existing caller**.
> Existing A/B baseline: I321 completed in implementation #909, merge `a9914b7a62b0873027997c69d9966d17f3fd48ba` (exact-head CI #4039 8/8, independent API review no blockers).
> Parent: STORE-002 / #670 remains OPEN until later D/E/F.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 I322 C only: extract minimal domain-owned typed persistence/retrieval traits from existing Session/PendingSubmission/Todo, Memory, Evolution and Exploration operations, documenting the seven A/B infrastructure categories, without changing existing concrete stores, SQL/paths/transactions, consumer dispatch, runtime/host composition or new backends |
| Claimed At | 2026-11-11 |
| Source Issue | #670 |
| Governance Claim PR | #911 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim #911 merged `60da1bbadca00c9bdee6fbcb8f5ad990a4313331`; implementation #912 merged `3df409ef9ed707756350f2c7d4b2c8819f157222` after [exact-head CI #4127](https://github.com/wjhuang88/talos/actions/runs/43249945758), both validators, independent [API review](https://github.com/wjhuang88/talos/pull/912#issuecomment-6828277614) no blockers and [CAS](https://github.com/wjhuang88/talos/pull/912#issuecomment-6840919956); single-maintainer path, no independent human approval claimed. |
| Implementation PR | #912 (merged; Completion Commit `3df409ef9ed707756350f2c7d4b2c8819f157222`) |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | I322 C is complete after #912 merge and formal closeout. Later D/E/F require distinct effective governance claims, and #897 Host E2/E3 remain gated. |

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

Governance claim #911 merged to main as `60da1bbadca00c9bdee6fbcb8f5ad990a4313331` after exact-head [CI #4055](https://github.com/wjhuang88/talos/actions/runs/42948954104), both governance validators 0 warnings, and recorded merge-time CAS. C implementation [#912](https://github.com/wjhuang88/talos/pull/912) merged as `3df409ef9ed707756350f2c7d4b2c8819f157222` after exact-head CI, both governance validators, independent review and CAS. **I322 C is Complete / Closed**; #670 remains OPEN for D/E/F. No Host or new storage-engine scope was admitted.

## I322 C execution checkpoint — 2026-11-11

- Effective governance: [#911](https://github.com/wjhuang88/talos/pull/911), merged `60da1bbadca00c9bdee6fbcb8f5ad990a4313331`.
- [Source-aligned contract map](../reference/STORE-002-I322-DOMAIN-OPERATIONAL-CONTRACT-MAP-2026-11-11.md): existing `SessionStore` retained; PendingSubmission/Todo, Memory, Evolution and Exploration domain contracts exported, with concrete forwarding implementations and compile/in-memory tests.
- No existing store, SQL/FTS, TLOG/JSONL, SQLite schema, transaction/locking, public constructor, runtime caller or Cargo manifest modified. Trait conformance does **not** mean alternate provider readiness.
- **Implementation PR, native CI, independent public API review and merge-time CAS pending.** No C completion commit; #670 stays OPEN and D/E/F/Host blocked.

## Completion Evidence — 2026-11-13

- **Completion Commit: `3df409ef9ed707756350f2c7d4b2c8819f157222`** — actual #912 squash-merge on `main`, independent of this documentation closeout.
- Exact-head `f908f86633ccee5892bf91d738b41bf65eb33799`: [CI #4127](https://github.com/wjhuang88/talos/actions/runs/43249945758) completed overall SUCCESS; macOS full Format/Check/Clippy/Test release preflight, native Windows workspace and Windows Rust, Linux/Windows Desktop, Windows installer and change scope all passed. Both project governance and collaboration claim validators passed. The *separate global remote Issue owner reconciliation* job failed due to already-closed unrelated #901 remaining in the Open Issue matrix; accepted ADR-071 classifies this as advisory for an unrelated implementation PR. The failed job is not misreported as passing.
- Independent [public API/architecture and three-document rebase review](https://github.com/wjhuang88/talos/pull/912#issuecomment-6828277614) explicitly found no blocking findings on the exact head, and independently checked formatting/diff cleanliness. Agent review shares maintainer GitHub identity; this is not a second human approval. [Final CAS and single-maintainer authorization](https://github.com/wjhuang88/talos/pull/912#issuecomment-6840919956) recorded before guarded merge.
- Delivered: new minimal domain-owned traits for PendingSubmission and Todo, Memory, Evolution, Exploration, implemented by forwarding into existing concrete stores. Existing `SessionStore` reused. All existing schemas, transaction/recovery behavior, SQL/FTS5, TLOG/JSONL, ranking, formats, callers and manifests preserved; no runtime provider binding or new backend claimed.

## Variance And Residuals

- Older CI #4116 failed macOS `talos-session` existing `durable.rs:2363` fork-recovery fixture once (retired=0, expected=1), a historically observed unexplained intermittent. The **final exact-head #4127 macOS job passed**; root cause is not fixed or claimed fixed. A separately owned recovery diagnostic is required if it recurs; no assertion weakening or scope expansion occurred.
- D (existing local delegation), E (default local profile) and F (conformance/compatibility) require new effective claims. Parent #670 and #897 E2/E3 Host wiring remain Open/gated; C completion does not establish provider execution readiness or authorize model-owned SQL/embedding/vector providers.
- The global Issue-matrix reconciliation residual for closed #901 belongs to its own governance owner under ADR-071 and does not alter the reviewed C implementation evidence.
