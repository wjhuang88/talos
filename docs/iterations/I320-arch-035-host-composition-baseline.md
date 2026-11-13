# Iteration I320: ARCH-035 E0/E1 Host Composition Baseline

> Document status: Review / Claimed (effective claim #905; E0/E1 review candidate #906; no E2/E3 authorization)
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
| Authorization Evidence | Claim #905 merged as `b211e590f6b1180af0679be8b1b03d77b0a19615` after exact-head CI3992 passed, both governance validators reported 0 warnings and single-maintainer documentation-only fallback was recorded in PR comments; CAS confirmed no competing owner or branch lag. |
| Implementation PR | #906 |
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

## Activation and E0/E1 execution ledger

| Date | Stage | Evidence and limitations |
|---|---|---|
| 2026-11-10 | Atomic claim+activation | Governance #905 squash merge `b211e590f6b1180af0679be8b1b03d77b0a19615`, exact-head [CI #3992](https://github.com/wjhuang88/talos/actions/runs/42784639228) succeeded; project-governance and collaboration-claim validators each 0 warnings; documented single-maintainer fallback; no overlapping PR or target-branch claimant at CAS. |
| 2026-11-10 | E0 source inventory candidate | [Code-truth matrix](../reference/ARCH-035-I320-E0-CODE-TRUTH-2026-11-10.md) reviews Rust paths and existing test entrypoints on claim merge base. No regression tests run locally in this docs-only slice; source inventory is not behavioral equivalence proof. |
| 2026-11-10 | E1 design review candidate | [Proposed host-composition decision](../proposals/arch-035-host-composition-boundary.md) and [PR #906](https://github.com/wjhuang88/talos/pull/906) bind the E0 matrix and defer public injection signatures until #670 accepts concrete seams. No ADR accepted and no E2/E3 code implemented. |

### Review and residuals

- The E0 evidence doc enumerates existing tests and missing fault-injection/ABI/native evidence; E2/E4 require before/after runs against real code changes.
- Required architecture/API review and maintainer acceptance for E1 are pending. Do not treat docs-only CI as runtime behavior conformance.
- STORE-002/#670 remains the sole contract owner and has not published accepted implementation seams at this checkpoint.
- No Completion Commit is claimed for #897. I320 may close only for its independently accepted documentation slice; #897 remains open through E2/E4.

## E1 dependency recheck checkpoint — 2026-11-13

Within the existing effective I320 E0/E1 claim #905, source-truth was rechecked after upstream STORE-002 progress. Accepted A/B PR #909 and C PR #912 (`3df409ef9ed707756350f2c7d4b2c8819f157222`) now supply a typed capability taxonomy and domain-owned persistence contracts with existing-store forwarding. I322 C closeout #930 merged as `a07b466d5aa2a03fb841cdcb918cd4968c778942`; governance-only D activation #932 merged as `e49ccde70556403a9d7b66551565e2b6c81ad32d`. Neither D implementation nor E local host composition/F parity is asserted here.

The E1 proposal has been reconciled against these accepted boundaries; its decision remains **Proposed** pending independent architecture/API review and the remaining applicable STORE-002 operational evidence. The E0 baseline's original source-date and unexecuted-test qualifications are preserved. This checkpoint changes no runtime, public API, default SQLite/TLOG/JSONL/FTS5 implementation, migration or permission behavior; the ARCH-035 Story remains Review / Claimed and **E2/E3 remain unauthorized**. A future implementation claim must be independently established.
