# Iteration I320: ARCH-035 E0/E1 Host Composition Baseline

> Document status: Complete / Closed — **E0/E1 documentation and ADR-104 placement only**, not ARCH-035 #897 or E2/E3
> Published plan date: 2026-11-10
> Planned objective: reproducibly inventory current cross-host persistence composition and propose a behavior-preserving architecture decision without competing with STORE-002.
> Baseline rule: this bounded E0/E1 documentation and architecture-review slice does not authorize production code, storage contract/adapters, dependency or default behavior changes.
> Deliverable: reviewable source-path matrix, conformance baseline and proposed host-composition decision; E2/E3 remain gated.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Released — bounded E0/E1 review accepted via #936; effective on I320 closeout merge |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | ARCH-035 I320 E0 source inventory and E1 host-composition architecture review only; exclude production code, STORE-002 contracts/adapters, ServerHost and E2/E3 |
| Claimed At | 2026-11-10 |
| Source Issue | #897 |
| Governance Claim PR | #905 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim #905 merged as `b211e590f6b1180af0679be8b1b03d77b0a19615` after exact-head CI3992 passed, both governance validators reported 0 warnings and single-maintainer documentation-only fallback was recorded in PR comments; CAS confirmed no competing owner or branch lag. |
| Implementation PR | #906 E0/E1 review; #934 C contract reconciliation; #936 ADR-104 decision |
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

Historical 2026-11-10 decision review artifact: [ARCH-035 host-composition proposal](../proposals/arch-035-host-composition-boundary.md) (at that time non-binding with no ADR number reserved; **subsequently superseded for ownership/placement by accepted ADR-104**).

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
- **Historical 2026-11-10 pending-review note:** E1 architecture/API review and maintainer acceptance were then pending. Placement-only E1 decision was subsequently accepted via #936, while docs-only CI remains **not** runtime behavior-conformance evidence.
- STORE-002/#670 remains the sole contract owner and has not published accepted implementation seams at this checkpoint.
- No Completion Commit is claimed for #897. I320 may close only for its independently accepted documentation slice; #897 remains open through E2/E4.

## E1 dependency recheck checkpoint — 2026-11-13

Within the existing effective I320 E0/E1 claim #905, source-truth was rechecked after upstream STORE-002 progress. Accepted A/B PR #909 and C PR #912 (`3df409ef9ed707756350f2c7d4b2c8819f157222`) now supply a typed capability taxonomy and domain-owned persistence contracts with existing-store forwarding. I322 C closeout #930 merged as `a07b466d5aa2a03fb841cdcb918cd4968c778942`; governance-only D activation #932 merged as `e49ccde70556403a9d7b66551565e2b6c81ad32d`. Neither D implementation nor E local host composition/F parity is asserted here.

**Historical dependency-recheck disposition, before ADR-104 existed:** the E1 placement proposal was **Proposed**, awaiting architecture/API review. This did **not** mean STORE-002 D/E implementation was necessary to accept the narrow **ownership/placement rule**; D/E operational evidence gates the *future concrete host API, local composition, E2/E3 code and E4 parity*. Subsequent ADR-104 in PR #936 proposes that placement-only acceptance independently, conditional on final-head review/CI and guarded merge. The E0 baseline's original source-date and unexecuted-test qualifications are preserved. No runtime, public API, default SQLite/TLOG/JSONL/FTS5 implementation, migration or permission behavior changes; the ARCH-035 Story remains Review / Claimed and **E2/E3 remain unauthorized**. A future implementation claim must be independently established.

## Historical E1 formal ADR candidate — before #936 merge (2026-11-13)

After the reviewed E1 dependency reconciliation PR #934 merged `d92aabaad4570d799c6e04d75c87abbb36a2b004` (exact-head docs [CI #4159](https://github.com/wjhuang88/talos/actions/runs/43359366292), both validators 0 warnings, [read-only technical no-blocker review](https://github.com/wjhuang88/talos/pull/934#issuecomment-6843004742)), [ADR-104](../decisions/104-host-composition-ownership-boundary.md) was drafted as a *separate proposed* ownership/host-placement decision in the same authorized E1 scope. At this pre-merge checkpoint it had not yet been accepted; #936 subsequently accepted placement/ownership only. The ADR deliberately defers any public injection signature, runtime implementation, local profile binding and behavior-equivalence result until STORE-002 D/E and a separately governed ARCH-035 E2 claim. This I320 review remains open pending exact-head ADR review, CI and owner-first acceptance record; parent #897 stays Open.

### Historical E1 placement-acceptance candidate — subsequently fulfilled by #936 merge

Independent [initial proposed-head ADR review](https://github.com/wjhuang88/talos/pull/936#issuecomment-6843456241) found the ownership/placement decision acceptable in principle, but blocked on a contradictory historical proposal sentence. The candidate corrects that sentence, clarifies that the runtime-facing composition **layer** is not a currently unified callable API, and scopes ADR-104 as **Accepted on reviewed #936 merge for placement/ownership only**. The historical [CI #4164](https://github.com/wjhuang88/talos/actions/runs/43362479834) succeeded on the prior head; **at this pre-merge checkpoint** final-head CI, validators, independent review, owner/overlap/CAS and single-maintainer authorization remained mandatory. Their later disposition, including the unrelated remote-matrix residual, is documented in the final I320 closeout below. Even after ADR acceptance, I320/ARCH-035 E2/E3 implementation, any runtime API, upstream STORE-002 D/E operational readiness, E4 parity and #897 closure remain **not delivered**.

## I320 bounded final acceptance — documentation only (2026-11-13)

**I320 E0/E1 Complete / Closed** upon reviewed merge of this owner-first closeout, using the pre-existing E1 [ADR-104 PR #936](https://github.com/wjhuang88/talos/pull/936) merge **`07c99a8c2aeff615caeb62d38318cc2ca3c7bb3b`** as its **Completion Commit**. The earlier #905 governance claim activated E0/E1, #906 published the E0 code-truth and non-binding proposal, and #934 reconciled accepted STORE-002 I322 C contracts. ADR-104 fixes the architectural **ownership and composition layer** without naming a public API, changing runtime behavior or duplicating #670 providers.

Final #936 exact-head `3fd1b7176901832717b692202512dadb7364391f`; [CI #4173](https://github.com/wjhuang88/talos/actions/runs/43365335816) overall success, both required **governance validators 0 warnings**, [independent read-only technical review](https://github.com/wjhuang88/talos/pull/936#issuecomment-6843928194) no blockers, and [recorded single-maintainer CAS/authorization](https://github.com/wjhuang88/talos/pull/936#issuecomment-6852726331). The **unrelated global remote Issue/owner synchronization job was unsuccessful** due to new #937; that drift has separate [#938](https://github.com/wjhuang88/talos/pull/938) owner and was explicitly recorded before #936 merge. Do **not** claim every job or any Rust/native/SDK/behavior-conformance test passed for this slice.

The E0/E1 bounded claim is **released**; #897 remains Open and **Blocked / Unclaimed** for E2/E3 until STORE-002 D operational delegation and E local default composition are independently accepted, followed by a new ARCH-035 implementation claim. E4 full equivalence and ServerHost/#361 remain unfulfilled. Do not use this I320 Completion Commit as #897 completion evidence.
