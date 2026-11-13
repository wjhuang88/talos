# ADR-104: Domain-owned persistence and cross-host runtime composition

**Status**: Proposed — E1 acceptance candidate; not effective until reviewed, approved and merged
**Date**: 2026-11-13
**Owner**: ARCH-035 / #897, I320 E1, effective documentation/architecture claim #905
**Existing accepted constraints**: ADR-042 durable embedded Session boundary; STORE-002 / #670 I321 A/B and I322 C
**Reversal trigger**: An accepted alternate domain repository implementation or host feature cannot preserve the authority/error/order/source-compatibility invariants below using the documented composition boundary; require a separately reviewed ADR amendment before changing them.

## Context and source evidence

Current product hosts have **different construction entrypoints**, not a single already-shipped provider factory:

- CLI selects local Session and runtime construction, including `TuiRuntimeBuilder` and `AppServerSession`; `talos-tui` renders and must not become a persistence factory.
- Desktop and embedders use `talos-runtime::RuntimeBuilder`; its `durable_session(DurableSession)` remains an optional, supported source-compatible API per ADR-042. The unbound in-memory path also remains supported.
- `talos-session::SessionManager` is the existing local directory, SQLite metadata index and TLOG/JSONL composition owner, while domain-local Memory, Evolution and Exploration repositories enforce their own invariants.

The reviewable [I320 source inventory](../reference/ARCH-035-I320-E0-CODE-TRUTH-2026-11-10.md) documents specific caller paths and baseline tests. Its 2026-11-10 snapshot is historical, not a claim of tests run for this ADR.

STORE-002 A/B implementation [#909](https://github.com/wjhuang88/talos/pull/909) accepted seven typed capability identifiers, **metadata only**. Accepted I322 C implementation [#912](https://github.com/wjhuang88/talos/pull/912), Completion Commit `3df409ef9ed707756350f2c7d4b2c8819f157222`, adds domain-owned `PendingSubmissionPersistence`, `TodoPersistence`, `MemoryPersistence`, `KnowledgePersistence` and `ExplorationPersistence` traits with direct forwarding to existing concrete stores. Session transcript `SessionStore` already exists, with file-format/path semantics.

STORE-002 D's governance-only claim [#932](https://github.com/wjhuang88/talos/pull/932) merged `e49ccde70556403a9d7b66551565e2b6c81ad32d`; this activates D work but is **not** its implementation or conformance acceptance. STORE-002 E local default composition and F storage parity remain separate owners. ARCH-035 cannot import their responsibilities.

## Decision: ownership and placement only

1. **Existing domain stores remain the persistence authority.** Session/Agent/permission/settlement/recovery logic stays in the runtime and consuming domains, including generation fencing, CAS, idempotency, first-terminal-wins, turn settlement, redaction and recovery. Runtime host selection may choose an implementation but must never implement these business semantics.
2. **Reuse STORE-002 contracts and concrete delegates.** Capability identity (Log, Relational, Object, Lexical Search, Vector Search, Embedding, Rerank), domain repository methods and future local adapter composition are owned by #670. ARCH-035 introduces no second storage contract hierarchy, general SQL/CRUD/transaction layer, or opaque `StorageProvider` in Server/CLI/TUI. Unavailable capability metadata never implies an operational backend.
3. **Host-facing assembly belongs at the supported runtime-facing composition boundary**, consumed by CLI, Desktop and embedded SDK and eventually Server. CLI keeps its host selection role, Desktop remains a thin adapter, TUI stays presentation, and Server must reuse the same runtime/Agent/Session authority rather than creating a parallel Session engine.
4. **Exact local delegation first.** The first host assembly path must select the same existing SQLite/TLOG/JSONL/FTS5 implementations and preserve constructor sequence, Session root/defaults, errors, lock ownership, retry and transaction/cross-resource ordering. No new backend, storage migration, path, SQL/schema, FTS ranking, permission policy or runtime behavior change is authorized by this ADR.
5. **Preserve public embedded compatibility.** `RuntimeBuilder::durable_session(DurableSession)`, its optional semantics and unbound builder path remain unchanged under ADR-042. New host selection may be additive only if proven necessary; no Rust API signature is approved by this placement decision.
6. **First inspect STORE-002 E's accepted local default composition** before planning ARCH-035 E2. If the upstream assembly already meets an integration call site, reuse it and record that sub-path as satisfied/no-op. If a verified cross-host gap remains, propose only the minimal additive runtime-facing delegator under an independent E2 claim with specific concrete type signatures and SDK migration review; do not invent a profile abstraction now.
7. **Truthfully retain the binary dependency and durability limits.** `talos-runtime -> talos-session -> rusqlite(bundled)` remains a physical compiled dependency until independently removed. Trait injection is not SQLite-free binary delivery. SQLite indexes and TLOG filesystem operations retain their present multi-resource recovery order; there is no claimed global ACID transaction.

## Explicitly rejected approaches

- Server-private `SessionManager` or `StorageProvider`, or duplication of the Agent Loop;
- moving DB constructors into `talos-tui`;
- a generic SQL/ORM/query abstraction or migrating all traits into `talos-core`;
- treating metadata capability registration as proof of operational binding;
- new PostgreSQL/Zvec/S3 engines, schema/format changes, default swaps or native dependency removal within ARCH-035 E-series;
- a speculative public `RuntimeBuilder` signature before accepted upstream D/E operational evidence.

## Acceptance, evidence and limitations

**This ADR accepts only the placement/ownership rule once approved and merged.** It does not approve an injection API or create an effective E2/E3 claim. E2/E3 remain gated by accepted usable STORE-002 local delegation/composition and their own governed iteration; E4 needs locked cross-host, native and SDK error/recovery/permission/format parity evidence before #897 closes.

For the E1 review, independently verify the actual I322 traits and direct forwarders, ADR-042 compatibility, the CLI `AppServerSession` versus Desktop/SDK `RuntimeBuilder` distinction, the absence of an invented provider or port, and the SQLite compiled dependency statement. The architecture [review candidate #934](https://github.com/wjhuang88/talos/pull/934) was merged as `d92aabaad4570d799c6e04d75c87abbb36a2b004` after [CI #4159](https://github.com/wjhuang88/talos/actions/runs/43359366292) passed and [read-only Copilot review](https://github.com/wjhuang88/talos/pull/934#issuecomment-6843004742) found no blockers. That is review of the **underlying proposed matrix**, not automatic approval of this new ADR.

A separate exact-head independent architecture/API review, docs CI with both governance validators, and final main/owner/no-overlap CAS are mandatory for this ADR. Shared-account Agent review is not independent human approval; single-maintainer architecture authorization must be recorded as such. No Rust/workspace regression tests are claimed from documentation-only checks.

## Consequences and implementation gate

The system keeps one authoritative runtime semantics path with host-selectable **future** assembly. This may leave existing CLI and embedded construction distinct until E2 proves an exact delegation point; forcing premature unification is rejected. Missing D/E composition or negative-case/SDK conformance remains a deliberate, explicit blocker. A future public API or storage implementation change requires its own reviewed evidence and may amend this ADR when new facts warrant it.
