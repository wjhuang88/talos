# Iteration I330: ARCH-035 E2 — Behavior-Preserving Host Storage Selection

> Document status: Active / Claimed — atomic E2 claim+activation proposed by #975, ineffective until merge
> Published plan date: 2026-11-18
> Parent: [ARCH-035](../backlog/active/ARCH-035-behavior-preserving-host-composition.md) / [Issue #897](https://github.com/wjhuang88/talos/issues/897)
> Baseline rule: preserve accepted STORE-002 E, ADR-104 and original host behavior; new implementation requires a separately effective atomic claim
> MVP: one executable, opt-in, host-facing local storage selection path that reuses accepted concrete domain stores without changing any existing production default or external SDK entrypoint

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | ARCH-035 E2 only: minimally expose and exercise host-facing storage selection/injection by direct delegation to the accepted STORE-002 E LocalStorageProfile/domain stores, with source-compatible current RuntimeBuilder/CLI entrypoints, no default behavior changes. E3 multi-host call-site migration, E4 broad equivalence, STORE-002 F, ServerHost, new database/storage adapters, SQL/schema/formats and product behavior changes are excluded. |
| Claimed At | 2026-11-18 |
| Source Issue | #897 |
| Governance Claim PR | #975 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Accepted STORE-002 E implementation #956 Completion Commit `a28c1cccebad71eb5785d149129a640ceec3d098`; I328 owner-first #972 merge `bc4a37eeafff5e0e172764e861fac6cb6341fd86`; I320 E0/E1 ADR-104 accepted. E2 claim proposed by governance PR #975, **not effective until exact-head docs CI, both validators, independent read-only scope/Host API review, single-maintainer CAS and target merge**. No independent human review or implementation claimed. |
| Implementation PR | Not started |
| Last Updated | 2026-11-18 |
| Handoff / Release Condition | E2 claim becomes effective only after reviewed #975 merges. Code implementation cannot start until that merge; E3/E4 and STORE-002 F remain independently unclaimed, #897 and dependent ServerHost Open. |

## Published Baseline

### Accepted dependencies, not reimplementations

- **E0/E1 architecture**: I320 documentation Complete/Closed, accepted ADR-104 via #936 merge `07c99a8c2aeff615caeb62d38318cc2ca3c7bb3b`; **placement/ownership only**, no approved callable injection signature or production Host migration.
- **STORE-002 A/B/C/D/E**: existing domain SessionStore and accepted Pending/Todo, Memory, Knowledge, Exploration persistence contracts; local delegates accepted #935/#951; opt-in `talos-runtime/local-storage::LocalStorageProfile` accepted #956 **Completion Commit `a28c1cccebad71eb5785d149129a640ceec3d098`**, and I328 owner closure #972 at `bc4a37eeafff5e0e172764e861fac6cb6341fd86`. Parent #670 remains **Open for separately unclaimed F**.
- **Current Host authority**: CLI `AppServerSession::new` and `set_persistence` remain the authoritative CLI actor integration; `SessionManager::new` runs default startup index/reconciliation while `with_dir` does not; Todo opens within the existing host closure; Memory is first-query lazy with caching and existing-path guard. Desktop and embedded SDK use `RuntimeBuilder::new` and optional `durable_session(DurableSession)`; unbound memory-only path remains valid. TUI is presentation, not a database factory.
- **Data roots / failure modes**: Session TLOG/JSONL and index, Pending fences/receipts, Memory SQLite+FTS5, Evolution hook `evolution/knowledge.db`, learned index `index.db`, Exploration database and typed/no-create optional semantics remain domain-owned. `LocalStorageProfile::memory_if_exists` itself opens when called; do **not** move it into an eager Host constructor.

### Selection-time non-terminal disposition

| Owner / work | State at selection | I330 E2 disposition |
|---|---|---|
| I328 / STORE-002 E #670 | Complete/Closed, #956 implemented and #972 owner-first closed | **Accepted prerequisite only**; do not reopen or duplicate local profile/traits |
| STORE-002 F #670 | Open, unclaimed | **Defer**, no wide crash/recovery/behavior conformance or backend changes |
| I320 / ARCH-035 E0/E1 | Documentation Complete/Closed, former claim Released | **Retain history**, propose separate I330 E2 claim |
| ARCH-035 E3/E4 #897 | Blocked/unclaimed, same Story future slices | **Defer** CLI/TUI/Desktop/SDK migrations and full multi-host equivalence |
| SERVER-001 / SERVER-001-C #142/#361 | Dependent, no accepted Host composition | **Remain blocked**, no ServerHost or protocol implementation |
| Concurrent SEARCH, TUI, Session, Git, Windows and Provider owners | Independently claimed/reviewed or proposed | **Retain without transfer**; no shared protected implementation code or claim |

## Proposed bounded E2 Work Slice

1. Refresh the exact current CLI, Desktop and SDK construction call sites; inventory the existing `RuntimeBuilder` and `AppServerSession` API/behavior before proposing a public signature. Document a minimal selection/injection API *only if* functional delegation requires it; submit it for independent public API and dependency review prior to acceptance.
2. Connect **one explicit opt-in host-facing selection path** to accepted `LocalStorageProfile` and existing domain delegates through the runtime-facing composition boundary. The path must execute real existing local operations, never be metadata-only or a redundant wrapper. Keep `RuntimeBuilder::new`, `durable_session(DurableSession)`, CLI defaults and unbound in-memory behavior unchanged unless a separately reviewed source-compatible extension is strictly necessary.
3. Prove the old direct constructors and the opt-in selected path use the **same** concrete implementation and preserve: default-vs-custom Session reconciliation, lazy Memory/Todo/Exploration opens, typed and no-create failures, pending fences, startup/restart/settlement/cancellation/privacy/permission ownership. Any conditional `local-storage` feature must stay opt-in; do not eagerly create databases.
4. Record no-op dispositions for already-equivalent call sites, and an API migration inventory for future E3 without changing product Host call sites prematurely. Preserve TUI rendering and SDK entrypoint source compatibility.

### Explicit non-goals

- **Not E3/E4**: no wholesale CLI/TUI/Desktop/embedded SDK migration, multi-host deployment parity, full crash/recovery/fault matrix, or #897 closeout.
- **Not STORE-002 F**: no storage contract/adapter reimplementation, generic provider registry/CRUD/SQL abstractions, databases, schema/format, locking, TLOG/JSONL/FTS ranking, or unsupported Object/VectorSearch/Embedding/Rerank backend admission.
- **Not ServerHost**: no server binary, transport/remote API, new Agent/Session/permission authority, or proxy bypass.
- No default-on configuration, production paths/constructor-timing changes, hidden writes or claims of an SQLite-free binary.

### Acceptance

- One reviewed, callable E2 Host-facing opt-in path actually delegates to pre-existing concrete stores. Public API signatures and crate dependency edges are source-grounded, independently reviewed and avoid duplicated C/D/E seams.
- Regression fixtures show original `RuntimeBuilder` default/unbound/durable behavior, `SessionManager` constructors, CLI memory laziness/Todo timing, missing DB typed results, stable roots/format/permission and non-accidental creation remain unchanged. A no-op call-site disposition is acceptable where equivalence already holds.
- Focused feature-on and feature-off tests plus locked workspace format/check/Clippy/tests, both governance validators and native macOS/Windows CI pass at the final exact implementation head; independent API/storage/SDK compatibility review and final merge CAS are recorded.
- A later owner-first closeout cites a **pre-existing actual implementation merge SHA**. #897 remains Open for E3/E4 until separately accepted; #670 stays Open for F.

### Validation plan / risk and rollback

- Inspect canonical source and implement pure opt-in tests with isolated temporary directories: default-vs-custom reconciliation, no-create optional stores, same type/delegate semantics, SDK source compatibility, cancellation/permission failures without changing existing policies.
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo check/test -p talos-runtime --features local-storage --locked`, required native CI and both governance validators.
- Risk: eagerly opening optional stores, changing default durable binding/recovery, exposing a premature generic API, or introducing a crate dependency cycle. Rollback is removal of the strictly opt-in E2 path; original Host constructors stay untouched.
- Original CLI/Desktop/SDK source behavior is the baseline oracle; broader equivalence observations are E3/E4 or F and cannot be marked accepted by E2 tests.

## Activation / execution / verification

**Finalized proposal in governance-only PR #975**: source ARCH-035 owner, this I330 record and derived Board/index/backlog jointly propose Claimed / Active. These states are **ineffective until reviewed exact-head CI/validators/CAS and merge to main**. No E2 implementation branch, production code, API, tests or acceptance exists in this governance PR; preserve effective-claim-first rule.

## Completion evidence

Not started. Never use this governance draft/claim commit as an implementation Completion Commit. #897 stays Open.
