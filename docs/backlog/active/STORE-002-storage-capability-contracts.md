# STORE-002: Storage And Retrieval Capability Contracts

**Status**: Review / Claimed — I322 C implementation candidate; D/E/F incomplete

**Type**: Architecture / Contract Convergence

**Source Issue**: [#670](https://github.com/wjhuang88/talos/issues/670)

**Parent / Related**: DATA-002 / #141; SERVER-001 / #142; SERVER-001-C / #361; SERVER-002 / #360; STORE-001; ARCH-035 / #897 (cross-host composition consumer; no duplicate capability extraction).

**Non-overlap note:** STORE-002 exclusively owns the complete storage/retrieval capability taxonomy, domain capability seams and behavior-preserving local implementation adaptation. ARCH-035 / #897 owns how product hosts and `talos-runtime` consume those accepted seams via equivalent composition, preserving existing CLI/TUI/Desktop/SDK behavior. If an interface/adapter is already delivered by STORE-002, ARCH-035 reuses it; it does not reimplement it.

## Objective

Converge Talos's existing persistence, retrieval, and semantic capability boundaries behind explicit contracts without changing current SQLite, FTS5, TLOG, JSONL, schema, transaction, migration, path, ranking, or runtime behavior.

## Scope

Define narrow capability contracts for Log, Relational, Object, Lexical Search, Vector Search, Embedding, and Rerank; preserve domain Stores/Repositories and their invariants; document authority and rebuildability; and provide one explicit local composition that delegates to the existing implementations.

Vector search, embedding, and other unavailable capabilities must have explicit unavailable semantics. Zvec, PostgreSQL, S3, generic ORM/query DSL work, and production behavior changes are out of scope for the first phase.

## I321 A/B merged baseline (completed bounded slice; #670 OPEN)

- Governance claim #908 merged `201a9b61d72e704da3a86f5db283b9f6e4d92938`; implementation [PR #909](https://github.com/wjhuang88/talos/pull/909) merged `a9914b7a62b0873027997c69d9966d17f3fd48ba`.
- [Exact-head CI #4039](https://github.com/wjhuang88/talos/actions/runs/42848943669) passed **8/8**, both governance validators **0 warnings**, independent [API review](https://github.com/wjhuang88/talos/pull/909#issuecomment-6772057381) and [delta check](https://github.com/wjhuang88/talos/pull/909#issuecomment-6772113212) no blockers, [CAS](https://github.com/wjhuang88/talos/pull/909#issuecomment-6785449447) recorded.
- Delivered: seven typed `StorageCapabilityKind` IDs, metadata-only `CapabilityRequest` helper and source-backed authority inventory. Speculative `Bound<T>` was removed; no SQL/runtime/default provider changes.
- A/B milestone is complete; **C/D/E/F remain separately gated** and #897 Host E2/E3 is still blocked.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 I322 C only: extract domain-owned persistence/retrieval operational traits from existing Session/PendingSubmission/Todo, Memory, Evolution and Exploration data/error methods; no concrete storage, SQL/transactions/formats/paths, consumer/default/Host/Server change or new engine |
| Claimed At | 2026-11-11 |
| Source Issue | #670 |
| Governance Claim PR | #911 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I322 C governance [#911](https://github.com/wjhuang88/talos/pull/911) merged `60da1bbadca00c9bdee6fbcb8f5ad990a4313331`; exact-head [CI #4055](https://github.com/wjhuang88/talos/actions/runs/42948954104) succeeded, both validators reported 0 warnings, docs-only single-maintainer authorization and CAS recorded. Independent API review of the C implementation remains mandatory. |
| Implementation PR | #912 (draft, Review; no completion evidence) |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Claim #911 effective. C implementation must pass exact-head native CI, both governance validators and independent public API/architecture review; D/E/F and #897 E2/E3 remain independently gated. |

## Governance

I321 A/B is **merged and accepted** as a narrow descriptor/inventory foundation, not as a functional provider binding. I322 C governance claim #911 **merged** as `60da1bbadca00c9bdee6fbcb8f5ad990a4313331`. Its narrow work slice is effective: add domain-owned operational interfaces and exact forwarding to unchanged stores; preserve all existing callers, SQL, error, schema and format behavior. The implementation is in Review and not yet accepted. D/E/F and #897 Host implementation remain gated. No generic `Bound<T>`, SQL/CRUD provider, or new backend is authorized.

## Acceptance Direction

- Every current persistence/retrieval responsibility has one capability classification.
- Domain invariants remain in domain Stores/Repositories.
- Existing SQLite/TLOG/JSONL implementations delegate through the contracts with zero behavior drift.
- A future talos-server profile and Zvec retrieval implementation can bind without another foundational rewrite.
- The issue remains open until the complete contract layer and evidence are merged.


## Intake history

Initially registered as Intake / Unclaimed on 2026-10-07; narrow I321 A/B claim proposed on 2026-11-10 via #908. Source Issue #670 remains open for C/D/E/F after A/B.

## I321 A/B implementation history — PR #909 (merged)

- [Current capability and authority inventory](../../reference/STORE-002-I321-CAPABILITY-AUTHORITY-INVENTORY-2026-11-10.md): authoritative/derived/ephemeral data, all seven capability families, existing paths and residuals.
- `crates/talos-core/src/storage_capability.rs`: typed `StorageCapabilityKind`, stable IDs and `CapabilityRequest` integration with existing `CapabilityRegistry` whose empty resolution returns `ResolutionResult::Unavailable`. No operational binding type, domain call site, or runtime behavior is introduced.
- Candidate code is additive and non-operational: it does **not** satisfy C/D/E/F, does not implement seven storage operation traits, and does not yet provide #897 a functional dependency to inject.
- Validation history: [CI #4030](https://github.com/wjhuang88/talos/actions/runs/42799763086) passed 8/8 on **previous** head `5c38e6e`; independent [Copilot review](https://github.com/wjhuang88/talos/pull/909#issuecomment-6765145292) identified one **blocking** unconstrained `StorageCapabilityBinding<T>` public API. The binding and its tests were removed. Final [CI #4039](https://github.com/wjhuang88/talos/actions/runs/42848943669) passed **8/8** on head `f3563138`, independent [no-blocker delta review](https://github.com/wjhuang88/talos/pull/909#issuecomment-6772113212) and merge-time CAS recorded. I321 A/B merged as `a9914b7a62b0873027997c69d9966d17f3fd48ba`.
- #670 remains open; avoid marking Complete or recording a fictitious Completion Commit. Future domain adapter delegation and local profile need separately authorized work.

## I322 C proposed activation

- [I322 domain-owned contract extraction](../../iterations/I322-store-002-domain-contract-extraction.md) proposes narrow interfaces anchored in the existing Session/Pending/Todo, Memory, Evolution and Exploration contracts with existing errors, while preserving local concrete behavior.
- Independent API review is required before C implementation merge. Explicit unavailable status for Object/Vector/Embedding/standalone Rerank does not imply fake executable providers.
- The C claim #911 is merged and effective; subsequent C implementation must satisfy exact-head CI and independent public API review before acceptance. D/E/F and #897 E2/E3 remain unclaimed/blocked.

## I322 C implementation review candidate

- [Domain contract mapping](../../reference/STORE-002-I322-DOMAIN-OPERATIONAL-CONTRACT-MAP-2026-11-11.md) documents Session (existing `SessionStore`, new PendingSubmission/Todo contracts), Memory, Evolution and Exploration.
- Four new domain-local `storage_contracts.rs` files declare typed traits and forward methods to existing inherent store methods; four `lib.rs` files export the traits. No existing storage engine, constructor, SQL, transaction, path, schema, runtime caller or Cargo dependency is changed.
- Status: implementation PR, exact-head Rust checks and independent public API/architecture review **pending**. No functional alternate provider, runtime swap, migration equivalence or C closure is asserted.
- C closes only after implementation merge with CI, independent review and CAS evidence. D/E/F and #897 Host remain gated.
