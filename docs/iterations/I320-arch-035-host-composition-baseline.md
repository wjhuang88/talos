# Iteration I320: ARCH-035 E0/E1 Host Composition Baseline

> Document status: Active / Claimed (proposed via #905; ineffective until governance merge)
> Published plan date: 2026-11-10
> Planned objective: reproducibly inventory current cross-host persistence composition and propose a behavior-preserving architecture decision without competing with STORE-002.
> Baseline rule: this bounded E0/E1 documentation and architecture-review slice does not authorize production code, storage contract/adapters, dependency or default behavior changes.
> Deliverable: reviewable source-path matrix, conformance baseline and proposed host-composition decision; E2/E3 remain gated.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | ARCH-035 I320 E0 source inventory and E1 host-composition architecture review only; exclude production code, STORE-002 contracts/adapters, ServerHost and E2/E3 |
| Claimed At | 2026-11-10 |
| Source Issue | #897 |
| Governance Claim PR | #905 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User requested continued non-overlapping E0/E1 progress. No independent approval is recorded; docs-only single-maintainer option is proposed, subject to exact-head CI, both governance validators, recorded maintainer reason, no blocking feedback and merge-time CAS. |
| Implementation PR | Not started |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | The claim takes effect only after #905 merge. E0/E1 decision acceptance remains independently reviewable; E2/E3 require accepted #670 seams and separate activation. |

## Published Baseline

| Field | Value |
|---|---|
| Selected Story | [ARCH-035 / #897](../backlog/active/ARCH-035-behavior-preserving-host-composition.md) |
| Status at selection | Intake / Unclaimed; bounded documentation slice selected at user direction |
| Dependency | STORE-002/#670 contracts not yet accepted; E0/E1 independent read-only architecture analysis permitted |
| Coordination | SERVER-001-C/#361, SERVER-002/#360, CLI/TUI/Desktop/SDK |
| Non-goals | No new persistence provider, session authority, server endpoint, schema, SQL, TLOG/JSONL or dependency modification |
| Risk / rollback | Proposed decision may disagree with #670; keep proposal non-binding and revert documentation claim PR without touching production state |
| Acceptance | Code-backed E0 matrix, explicit no-overlap with #670, API/source compatibility constraints and reviewable E1 decision; no acceptance claimed until evidence reviewed |
| Planned validation | Both governance shell validators, exact-head CI, source-path review and independent API/architecture review as applicable |

## Objective

Record an exact-source-path matrix for host constructors and Runtime/Session/Memory/Evolution/Exploration ownership. Accept a minimal host-facing composition decision that delegates to existing local implementations and does not create any capability/domain storage contracts owned by #670.

## Execution gates

Decision review artifact: [ARCH-035 host-composition proposal](../proposals/arch-035-host-composition-boundary.md) (non-binding, no ADR number reserved).

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
