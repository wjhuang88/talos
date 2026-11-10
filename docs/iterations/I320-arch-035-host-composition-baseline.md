# I320 — ARCH-035 E0/E1 host composition architecture baseline

| Field | Value |
|---|---|
| Status | Active (proposed; ineffective until governance claim merges) |
| Story | ARCH-035 / #897 |
| Scope | E0 dependency/code-truth inventory and E1 ADR / API placement only |
| Owner | @wjhuang88 |
| Claim | See authoritative ARCH-035 owner; no production implementation authorized |
| Dependency | STORE-002 / #670 remains Intake / Unclaimed; E2/E3 blocked |
| Coordination | #361 ServerHost, #360 remote adapter, CLI/TUI/Desktop/SDK |

## Objective

Record an exact-source-path matrix for host constructors and Runtime/Session/Memory/Evolution/Exploration ownership. Accept a minimal host-facing composition decision that delegates to existing local implementations and does not create any capability/domain storage contracts owned by #670.

## Execution gates

- E0: inspect runtime builder, session manager/store, CLI host, TUI renderer, Desktop and SDK call sites; classify persistence authority and compiled SQLite dependency; record baseline tests and negative/recovery invariants.
- E1: review ADR for local default host profile, domain ownership, builder source compatibility and the #670 no-duplication matrix.
- No E2/E3 code, schema, SQL, file format, migration, dependencies or Server behavior.
- Validate docs/governance and exact-head CI before acceptance.

## Existing evidence / initial inventory (read-only, 2026-11-10)

| Component | Verified path | Observation |
|---|---|---|
| Runtime SDK | `crates/talos-runtime/src/lib.rs` | Runtime facade and shared composition feature; inspect builder implementation in E0 |
| Session domain | `crates/talos-session/src/manager.rs` | SessionManager owns directory, index and SessionStore delegates; default CompactTextSessionStore + JsonlSessionStore |
| CLI host | `crates/talos-cli/src/main.rs` | CLI owns mode selection and session commands; storage and runtime adapter modules need full tracing |
| Storage owner | `docs/backlog/active/STORE-002-storage-capability-contracts.md` | #670 is unclaimed; capability interfaces/adapters cannot be duplicated here |

These are preliminary facts, not an E0-complete inventory or accepted ADR.

## Acceptance / residuals

- Full dependency matrix and code references reviewed.
- ADR accepted with #670 owner separation and API compatibility evidence.
- No production behavior or dependency change.
- E2/E3 remain explicitly blocked until #670 accepted seams.
- Required governance claim PR merged before any governed implementation.
- Independent API review and exact-head checks as applicable; no Completion Commit until existing evidence commit exists.

## Iteration inventory / parallel authorization

I319 search legacy dependency retirement has a separate proposed claim; I318 TUI native key repeat is in Review, I300 and other historical active work remain independently owned. This I320 scope is docs-only architecture and does not touch their code or ownership. Parallel non-overlap must be reconfirmed at merge-time CAS.

## Additional source-path characterization (2026-11-10, governance PR #905)

| Surface | Observed construction/authority | E2/E3 implication |
|---|---|---|
| CLI session commands | `crates/talos-cli/src/session_setup.rs` directly accepts `&SessionManager`, calls resume/create/fork and resolves workspace | Host selection must preserve current workspace/session resolution and error paths |
| TUI runtime | `crates/talos-cli/src/tui_runtime_builder.rs` owns `TuiRuntimeBuilder` and `PreparedTuiRuntime::finish` constructing `AppServerSession::new` then `actor.set_persistence` | TUI renderer remains separate; preserve ordering of actor creation and persistence attachment |
| CLI RPC | `crates/talos-cli/src/runtime_adapter.rs` wraps canonical Session SQ/EQ and serializes EQ consumption; cancellation uses drop interrupt | Do not introduce another Agent/Session authority or change cancellation and terminal settlement |
| Runtime crate | `crates/talos-runtime/Cargo.toml` depends directly on `talos-session` | Trait injection alone does not make runtime binary SQLite-free |
| Session crate | `crates/talos-session/Cargo.toml` directly depends on bundled `rusqlite` | Physical SQLite dependency remains until separately governed change |

**Limitations:** Desktop composition source path, full Memory/Evolution/Exploration constructor tracing, SDK builder call graph, baseline test list and ADR approval remain unverified. This is an incremental E0 inventory, not acceptance of E0/E1.

## E0 domain/desktop inventory — additional verified paths

| Domain / host | Source | Current authority / constructor | Non-regression boundary |
|---|---|---|---|
| Desktop host | `crates/talos-desktop/src/main.rs`, `src/runtime_host.rs`, `Cargo.toml` | Desktop imports `talos_runtime::RuntimeBuilder` and `RuntimeHandle`; owns thin Tokio adapter and presentation forwarding; depends on `talos-session` and runtime with `shared-composition` | Keep one runtime authority, bounded command/event queues, approvals, and Desktop feature flags |
| Embedded SDK | `crates/talos-runtime/src/lib.rs`, `docs/decisions/042-embedded-durable-runtime-session-boundary.md` | `RuntimeBuilder::durable_session(DurableSession)` optional, established ADR-042 source compatibility | Preserve TLOG atomic rename/turn ID idempotency, redacted projection, failure non-commit |
| Memory | `crates/talos-memory/src/store.rs` | `MemoryStore::open(path)` and `open_memory()` own SQLite/FTS5 and migration; add-only/evidence provenance | No FTS scoring, dedup, migration, evidence semantics change |
| Evolution | `crates/talos-evolution/src/store.rs` | `KnowledgeStore::open(path)` and `open_memory()` own SQLite/migrations | Preserve historic migration/reset behavior; no new shared repository |
| Exploration | `crates/talos-exploration/src/lib.rs` | `ExplorationStore::open(path)` and `open_memory()` own SQLite/FTS5, citation integrity | Preserve citation validation, FTS and schema semantics |
| Session | `crates/talos-session/src/manager.rs` | `SessionManager::new()` composes directory, index, TLOG/JSONL delegates | Preserve startup reconciliation and multi-resource recovery |

### Initial conformance baseline (to execute after claim)

- `cargo test -p talos-runtime --locked` (builder/SDK source and durable sessions)
- `cargo test -p talos-session --locked` (recovery, idempotency, transcript persistence)
- `cargo test -p talos-memory -p talos-evolution -p talos-exploration --locked` (domain persistence)
- `cargo test -p talos-cli --locked` and supported Desktop feature checks, subject to platform/toolchain constraints
- `scripts/validate_project_governance.sh .` and `scripts/validate_collaboration_claims.sh .`; exact-head GitHub CI

**Not executed:** these commands are a proposed baseline, not claimed green evidence. Additional call-site tracing and acceptance review remain necessary.
