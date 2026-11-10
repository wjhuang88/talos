# STORE-002: Storage And Retrieval Capability Contracts

**Status**: Review / Claimed — I321 A/B candidate #909; C/D/E/F incomplete

**Type**: Architecture / Contract Convergence

**Source Issue**: [#670](https://github.com/wjhuang88/talos/issues/670)

**Parent / Related**: DATA-002 / #141; SERVER-001 / #142; SERVER-001-C / #361; SERVER-002 / #360; STORE-001; ARCH-035 / #897 (cross-host composition consumer; no duplicate capability extraction).

**Non-overlap note:** STORE-002 exclusively owns the complete storage/retrieval capability taxonomy, domain capability seams and behavior-preserving local implementation adaptation. ARCH-035 / #897 owns how product hosts and `talos-runtime` consume those accepted seams via equivalent composition, preserving existing CLI/TUI/Desktop/SDK behavior. If an interface/adapter is already delivered by STORE-002, ARCH-035 reuses it; it does not reimplement it.

## Objective

Converge Talos's existing persistence, retrieval, and semantic capability boundaries behind explicit contracts without changing current SQLite, FTS5, TLOG, JSONL, schema, transaction, migration, path, ranking, or runtime behavior.

## Scope

Define narrow capability contracts for Log, Relational, Object, Lexical Search, Vector Search, Embedding, and Rerank; preserve domain Stores/Repositories and their invariants; document authority and rebuildability; and provide one explicit local composition that delegates to the existing implementations.

Vector search, embedding, and other unavailable capabilities must have explicit unavailable semantics. Zvec, PostgreSQL, S3, generic ORM/query DSL work, and production behavior changes are out of scope for the first phase.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 I321 A/B: source-truth authority/capability inventory, stable capability taxonomy and additive typed contract foundation for seven capability families; excludes domain adapter migration, SQL/format changes, host integration, Server and new engines |
| Claimed At | 2026-11-10 |
| Source Issue | #670 |
| Governance Claim PR | #908 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance claim #908 merged `201a9b61d72e704da3a86f5db283b9f6e4d92938`; exact-head [CI #4011](https://github.com/wjhuang88/talos/actions/runs/42789979266) passed, both governance validators 0 warnings, docs-only single-maintainer authorization and CAS recorded. Public B code candidate #909 still requires independent API review and tests. |
| Implementation PR | #909 |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | A/B candidate contracts must pass independent API/architecture review and all applicable tests; later C/D/E/F require separately scoped claims. E2/E3 ARCH-035 waits on accepted functional contract and local delegation. |

## Governance

I321 A/B became effective with governance #908 merge `201a9b61d72e704da3a86f5db283b9f6e4d92938`. Only bounded A/B inventory and additive typed capability foundation are authorized; C/D/E/F and #897 host implementation remain independently gated. Independent API/architecture review, exact-head locked CI, characterization evidence where applicable, and governance validators are required before B implementation acceptance.

## Acceptance Direction

- Every current persistence/retrieval responsibility has one capability classification.
- Domain invariants remain in domain Stores/Repositories.
- Existing SQLite/TLOG/JSONL implementations delegate through the contracts with zero behavior drift.
- A future talos-server profile and Zvec retrieval implementation can bind without another foundational rewrite.
- The issue remains open until the complete contract layer and evidence are merged.


## Intake history

Initially registered as Intake / Unclaimed on 2026-10-07; narrow I321 A/B claim proposed on 2026-11-10 via #908. Source Issue #670 remains open for C/D/E/F after A/B.

## I321 A/B implementation candidate — PR #909

- [Current capability and authority inventory](../../reference/STORE-002-I321-CAPABILITY-AUTHORITY-INVENTORY-2026-11-10.md): authoritative/derived/ephemeral data, all seven capability families, existing paths and residuals.
- `crates/talos-core/src/storage_capability.rs`: typed `StorageCapabilityKind`, stable IDs/requests reusing existing capability identity, and `StorageCapabilityBinding<T>` with explicit `Unavailable`. No runtime/domain call site is changed.
- Candidate code is additive and non-operational: it does **not** satisfy C/D/E/F, does not implement seven storage operation traits, and does not yet provide #897 a functional dependency to inject.
- Validation: **pending** exact-head `cargo fmt/check/test`, governance validators, CI and independent API/architecture review. This entry does not assert a passing Rust test result.
- #670 remains open; avoid marking Complete or recording a fictitious Completion Commit. Future domain adapter delegation and local profile need separately authorized work.
