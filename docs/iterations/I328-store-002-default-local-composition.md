# Iteration I328: STORE-002 Default Local Storage Composition (E)

> Document status: Review / Claimed — effective governance #955; E implementation #956 not accepted
> Published plan date: 2026-11-16
> Parent: [STORE-002](../backlog/active/STORE-002-storage-capability-contracts.md) / [Issue #670](https://github.com/wjhuang88/talos/issues/670)
> Delivery target: one verifiable, behavior-preserving **local-default composition** over accepted concrete stores/domain delegates; not cross-host injection, new engines, or F-wide equivalence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 E only: explicit local-default composition/selection of already accepted Session/Pending/Todo, Memory, Evolution and Exploration domain stores and trait delegation; preserve current paths, constructor timing, unavailable categories, error/recovery semantics and call sites. No F broad conformance, ARCH-035 Host/runtime/SDK injection, ServerHost, new SQL/vector/object engines, provider registry or speculative public framework. |
| Claimed At | 2026-11-16 |
| Source Issue | #670 |
| Governance Claim PR | #955 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim+activation #955 merged/effective `b524c7a9a0833cf9293c0fd77a730e1fb7fc1184` after CI #4256, both governance validators and independent no-blocker review/CAS. D #935 implementation/ #951 closeout accepted. E implementation #956 is Review, pending its own exact-head Rust/native CI, independent API/semantics review and guarded merge. |
| Implementation PR | #956 (Review; no Completion Commit) |
| Last Updated | 2026-11-16 |
| Handoff / Release Condition | E requires selected local profile/assembly with source-backed operational checks, applicable exact-head native CI and independent API/semantics review, then owner-first closeout; F remains unclaimed and #897 E2/E3 blocked until accepted usable E and its own claim. |

## Published Baseline

### Dependency/ownership inventory

- A/B metadata taxonomy #909 accepted; C domain-owned traits and existing forwarding #912 accepted; D real local trait exercises #935 accepted, I325 #951 Complete/Closed.
- Existing `SessionManager::new()` performs default-dir startup index and orphan reconciliation, but `with_dir()` does **not**; never silently unify their effects.
- CLI Memory opt-in prompt checks `~/.talos/memory.db` existence and lazily opens an existing file, skipping missing/unavailable storage instead of creating it. Exploration ingest opens on demand; query of missing DB remains informational/no creation.
- `RuntimeBuilder::durable_session(DurableSession)` is optional under ADR-042, leaving the unbound in-memory runtime intact. ADR-104 is **ownership/placement only**, not an approved Host/RuntimeBuilder injection API.
- `SessionIndex` is a derived metadata/search projection with bounded reconciliation; durable external-ID mapping, pending receipts and Todo/Memory/Exploration records remain domain-owned authority. Existing storage categories Object, VectorSearch, Embedding and standalone Rerank remain **unavailable** as generic operational providers.

### Selection-time non-terminal inventory and disposition — 2026-11-16

Source: `docs/iterations/README.md` and effective owners on `main@b524c7a9a0833cf9293c0fd77a730e1fb7fc1184`. This is a **non-overlap / select-or-retain record**, not a transfer of ownership or an assertion that unrelated iterations are completed.

| Existing iteration / owner | Recorded state at I328 selection | Disposition for I328 E |
|---|---|---|
| I325 / STORE-002 D | Accepted and closed (#935 implementation, #951 closeout) | Dependency **satisfied**, not reopened |
| I320 E0/E1 / ARCH-035 #897 | Bounded docs Complete / Closed; parent E2/E3 Blocked / Released | **Keep blocked** until accepted E and separate Host claim; E has no Host API mandate |
| I318 / TUI-054 #269 | Review with physical terminal / IME acceptance deferred | **Retain Review**, no TUI input/terminal changes in E |
| I302 / TEST-003 #717 | Review / Claimed with native Windows handle/resource residual | **Retain Review**, E neither asserts nor modifies native-process validation |
| I323 / PROVIDER-008-C #914 | Proposed governance activation | **Defer/retain separate claim**; Azure provider unrelated to storage E |
| I324 / GIT-COMPAT-001 #919 | Proposed governance claim (not effective at inventory snapshot) | **Defer/retain separate claim**; Git 2.36 preflight and protected review unaffected |
| I277, I249, I164 | Review, Planned, Paused respectively as recorded in README | **Retain Review / defer Planned / retain Paused**; no Desktop/device/release acceptance transfer |
| SEARCH-001 D3 and other separately owned search work | Independently owned source/research work; #957/I329 intake proposed separately | **Retain separate governance**, no shared search implementation source or provider admission |
| STORE-002 F / #670 | Unclaimed, next broad crash/recovery/parity slice | **Defer** until independently claimed after E implementation and closeout |

**Selection / non-overlap conclusion:** I328 E has the sole effective STORE-002 local-default composition claim through merged #955. No other non-terminal iteration owns this Work Slice; #956 changes only opt-in existing-store selection and its tests, bounded documentation and explicit feature CI. Concurrent search, provider, TUI, Git-compat, Windows diagnostics, Desktop and Host owners are retained, not superseded. No second active iteration or cross-owner implementation claim is inferred by this inventory.

### Work Slice

1. Identify the **smallest existing ownership-safe module/layer** for an additive local-default selection/composition over current domain stores; first prove its dependency graph and existing public API impact. Any genuinely new public API must have specific signatures and independent review before acceptance; do not introduce generic CRUD/SQL/transaction or opaque `StorageProvider`.
2. Bind the **same** local Session TLOG/JSONL + SQLite SessionIndex/sidecars/Todo, Memory SQLite/FTS5+graph, Evolution SQLite, Exploration SQLite/FTS5 as their **currently applicable** host paths. Preserve per-domain path, constructor order, lazy/optional opening, typed errors, generation fences, cross-file recovery and persistence/FTS behavior; do not force all stores to open at process startup.
3. Offer only currently operational domain delegate surfaces from C/D. Return explicit `Unavailable` metadata for unbound Object, VectorSearch, Embedding and independent Rerank; metadata capability IDs must not suggest generic executable adapters.
4. Prove explicit local composition with direct tests and temporary fixtures where possible (selected concrete types, absent optional store, error fallbacks, session custom-vs-default construction, no external-ID/path recomputation). Include an inventory of unsupported capabilities, broad conformance residuals and the exact interface consumed by future ARCH-035; do not rewire production CLI/TUI/Desktop/SDK/Server call sites in E.

### Non-goals

- No SQLite schema/SQL/transactions migration, no TLOG/JSONL format, FTS ranking, default storage engine, lock/fencing, permissions/recovery policy or source-of-truth changes.
- No cross-host `RuntimeBuilder` injection, SDK API migration, CLI/TUI/Desktop integration or ServerHost; those belong exclusively to ARCH-035 / #897 after E acceptance.
- No F-wide before/after cross-host/concurrency/crash/restart/migration parity; E requires narrow profile tests, F has a separate effective claim.
- No Zvec, PostgreSQL, S3, generic ORM, seven operational provider traits, runtime vector/embedding/reranker readiness or `talos-models` catalog admission.

### Acceptance

- A source-grounded API/constructor matrix identifies the single compositional entrypoint and unchanged product defaults, with no circular/new restricted SQLite dependency.
- Real existing domain operations succeed through the selected local composition and accepted trait interfaces; optional absent stores preserve existing no-create/no-fail behavior.
- Default vs custom Session setup, path roots, TLOG/JSONL, FTS and unbound/runtime-builder behavior are unchanged; explicit unavailable classes remain unavailable.
- Native exact-head Rust format/check/clippy/test and both governance validators pass; independent storage/API reviewer finds no breaking scope. Any necessary public API signature change is separately reviewed.
- D completion evidence remains unchanged; E completion only after implementation merge with existing Completion Commit and owner-first status closure. #670 stays Open for F; #897 Host E2/E3 remains separately gated.

### Planned validation and rollback

- Source/constructor and dependency-edge inspection; targeted existing domain-conformance tests plus new E composition tests; `cargo fmt --all -- --check`, locked workspace check/clippy/test and applicable platform CI.
- `scripts/validate_project_governance.sh .` and `scripts/validate_collaboration_claims.sh .`; exact-head native CI, independent read-only technical review and merge-time CAS.
- Rollback: E is additive, unconsumed by production Hosts until separately accepted ARCH-035. Remove E composition without rewriting existing stores/callers.

## Activation / verification / completion

Governance-only #955 claim and activation merged on main at `b524c7a9a0833cf9293c0fd77a730e1fb7fc1184` and authorized bounded E implementation. #956 is an unaccepted code candidate pending native exact-head CI, independent API review and CAS. Record actual claim PR SHA, implementation CI/review/merge and pre-existing Completion Commit in later append-only execution evidence. Parent #670 remains Open for E/F and #897 Host E2/E3 remains blocked.

## I328 E implementation checkpoint — 2026-11-16

- [PR #956](https://github.com/wjhuang88/talos/pull/956) proposes feature-gated concrete `LocalStorageProfile` selection in runtime-facing composition layer without changing RuntimeBuilder, CLI/TUI/Desktop/SDK or ServerHost. It reuses accepted C domain traits and existing store constructors; paths are explicit, missing read-side optional DBs are skipped, write/ingest only on explicit call, and Evolution hook/learned query path distinction remains.
- The first candidate omitted the optional dependency changes from `Cargo.lock`. Native Linux `--locked` check failed as expected, prompting a corrected lock-only commit; **the original CI failure is preserved**. Fresh exact-head CI, cargo format/test/clippy including `--features local-storage`, independent API/constructor review and all governance gates remain pending. No Completion Commit or E/F/Host acceptance.
