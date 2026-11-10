# Iteration I321: STORE-002 Capability Contract Foundation (A/B)

> Document status: Active / Claimed (proposed in governance PR; ineffective until merge)
> Published plan date: 2026-11-10
> Planned objective: inventory and classify every current persistence/retrieval responsibility, then publish a minimal additive, typed infrastructure capability contract foundation that does not alter any consumer's behavior.
> Baseline rule: preserve SQLite/TLOG/JSONL/FTS5 schemas, SQL, paths, migration, transactions, locking, public domain interfaces and runtime behavior exactly.
> MVP deliverable: documented A taxonomy/authority matrix plus compile-checked B contract module with seven distinct capability categories, explicit missing capability semantics and tests; no concrete adapter rewiring.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 / #670 slices A/B only: inventory authority and existing capability usage; introduce narrow typed capability classification and common provider identity/availability foundation, without runtime wiring, domain adapters, new backends or host composition |
| Claimed At | 2026-11-10 |
| Source Issue | #670 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User requested #670 implementation; separate governance-only proposal from current main. Exact-head docs CI, two governance validators, recorded authorization and merge-time CAS required. No independent approving maintainer recorded at selection. |
| Implementation PR | Not started |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | Stop at reviewed A/B artifact; C/D/E/F and #897 E2/E3 remain independently gated, preserving no duplicate domain/host ownership. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| [STORE-002](../backlog/active/STORE-002-storage-capability-contracts.md) / #670 | DATA-002 / #141 | Intake / Unclaimed | Accepted ADR-002/008 and actual domain stores | Reproducible capability/authority inventory + additive contract foundation with explicit unavailable semantics |

### Existing active work and non-overlap

- I320 / ARCH-035 #897: E0/E1 review evidence merged in #906; E2/E3 cross-host composition must consume #670 accepted seams and **does not** own storage traits/adapters.
- I319 / SEARCH-001-L: standalone search compatibility/dependency retirement; do not touch SearchRouter, `Cargo.lock` or its fixtures.
- I318 TUI native key repeat, I310 model provider and other active/review iterations: unrelated work proceeds independently. Reconfirm conflicts at CAS.
- #361 ServerHost and #360 remote relational profile remain gated and are not built by this slice.
- I321 is permitted as a separate, user-directed, non-overlapping storage-contract iteration; no existing active claimant is shown in STORE-002 on `main` at selection.

### Scope

- A1. Inventory each current Session/Memory/Evolution/Exploration persistence and retrieval call family and quarantined Models, with authoritative/derived/ephemeral classification and rebuildability notes.
- A2. Record Log, Relational, Object, Lexical Search, Vector Search, Embedding and Rerank as distinct capability categories, their existing provider vs unavailable states, and domain ownership.
- B1. Reuse existing `talos-core` capability/provenance identity contracts where sensible; add minimal typed storage-specific capability contract APIs and explicit unbound/unsupported semantics, without generic SQL/ORM, generic catchall Provider, or speculative crate.
- B2. Add deterministic type/contract tests, compile compatibility and API/architecture evidence. Existing consumers must not be modified in B.

### Non-goals

- No SQL/schema/FTS/ranking/TLOG/JSONL/file/transaction/locking/migration/path changes.
- No persistence implementation, domain Store/Repository replacement, runtime constructor migration, Host/CLI/Desktop/SDK wiring, or ServerHost runtime.
- No PostgreSQL/Zvec/S3 or new Cargo dependencies; no binary SQLite-free claim.
- No C/D/E/F adapter delegation or full #670 closeout under this bounded Work Slice.

### Acceptance

- Given every existing data family, the inventory records its current code location, owning domain, authoritative or derived role and current provider without inventing portability.
- Given the seven contract categories, consumers can inspect typed capability identity/availability without requiring SQLite, registering new engines or changing the local default.
- Given unsupported Vector/Embedding and optional future Object/Rerank implementations, the new contract explicitly reports absence rather than silently falling back.
- Given current SDK/runtime/CLI/desktop code, all existing public domain APIs and defaults remain unchanged; no production consumer needs to adopt the new types in B.
- Given candidate changes, `cargo fmt --all -- --check`, `cargo check -p talos-core --locked`, `cargo test -p talos-core --locked`, both governance validators and exact-head relevant multi-platform CI pass; an independent API/architecture review accepts B before implementation merge.

### Planned validation and evidence

```bash
cargo fmt --all -- --check
cargo check -p talos-core --locked
cargo test -p talos-core --locked
scripts/validate_project_governance.sh .
scripts/validate_collaboration_claims.sh .
```

Run full relevant locked CI and independently review any public types/traits before code acceptance. Characterization of legacy SQLite/TLOG implementation behavior is not inferred from passing new contract tests; C/D/E/F will require before/after conformance.

### Documentation

Update this iteration, STORE-002 owner, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and a source-backed A inventory and B API contract decision if accepted. Update #670 once owner and derived views reconcile.

### Risks and rollback

- Trait proliferation/duplicate Provider: bind terminology to existing `talos-core::capability` metadata and retain domain authority; reject speculative CRUD/SQL traits.
- Advertised availability without operational implementation: distinguish descriptive capability and resolvable binding; future providers default unavailable.
- Source/API compatibility: additive code only, no existing domain method signature touched; revert A/B candidate if consumer compile or API/security review fails.
- Parallel conflicts: avoid `Cargo.lock`, Search/CLI/runtime/Server files; if `talos-core` overlaps another PR, stop and reconcile before merge.

## Actual Activation And Execution

- Governance claim PR Pending; **no effective claim until merged to main**, and no implementation branch authorized before then.

## Verification Evidence

- Not run at proposal stage. Record observed check outcomes and exact commit SHA after execution.

## Completion Evidence

- None. Existing implementation merge evidence is required before `Complete`.

## Variance And Residuals

- C/D/E/F local adapters, domain contracts, local profile and full legacy conformance remain unclaimed and incomplete.
- #897 cannot consume a merely descriptive taxonomy as a functional storage adapter; a later accepted local delegation remains prerequisite.

## Retrospective

- Pending.
