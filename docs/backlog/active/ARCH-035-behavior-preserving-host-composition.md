# ARCH-035: Behavior-Preserving Domain Persistence And Host Composition Decoupling

| Field | Value |
|---|---|
| Story ID | ARCH-035 |
| Source Issue | [#897](https://github.com/wjhuang88/talos/issues/897) |
| Status | Review / Claimed — E0/E1 code-truth and E1 proposed architecture in PR #906; E2/E3 blocked |
| Priority | P1 — blocking architectural prerequisite for SERVER-001 / SERVER-001-C implementation |
| Type | Architecture / Runtime Composition / Equivalence Migration |
| Selected Iteration | [I320](../../iterations/I320-arch-035-host-composition-baseline.md) — E0/E1 active via claim #905 merge `b211e590f6b1180af0679be8b1b03d77b0a19615` |
| Parent / consumer | [SERVER-001](SERVER-001-serve-connect-protocol-adapters.md) / #142; [SERVER-001-C](SERVER-001-C-standalone-server-host-composition.md) / #361 |
| Depends On | [STORE-002](STORE-002-storage-capability-contracts.md) / #670: accepted capability/domain seam; RUNTIME-001 SDK compatibility |
| Coordinates | SESSION-009 / #46; SESSION-011 / #612; DATA-002 / #141; SERVER-002 / #360 |
| Long-task planning | [SERVER-MVP-001](../../tasks/2026-11-10-server-mvp-storage-decoupled.md) proposed in draft PR #896 |
| E1 review artifact | [Host composition proposal](../../proposals/arch-035-host-composition-boundary.md) — non-binding until reviewed and accepted |
| E0 audit artifact | [Code-truth and regression inventory](../../reference/ARCH-035-I320-E0-CODE-TRUTH-2026-11-10.md) — review candidate, existing tests not rerun |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | E0 code-truth inventory and E1 ADR / cross-host composition API placement only; no production code, no STORE-002 capability contracts or adapters |
| Claimed At | 2026-11-10 |
| Source Issue | #897 |
| Governance Claim PR | #905 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | #905 merged `b211e590f6b1180af0679be8b1b03d77b0a19615` following exact-head [CI #3992](https://github.com/wjhuang88/talos/actions/runs/42784639228), both governance validators with 0 warnings, recorded single-maintainer documentation-only fallback and passing merge-time CAS; this authorizes E0/E1 only. |
| Implementation PR | #906 |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | E0/E1 only; E2/E3 blocked until accepted STORE-002 seams and separately governed activation; no implementation before effective claim |

## Requirement and value

Talos has several host/product surfaces (CLI/TUI, Desktop, embedded SDK, planned Server) but still exposes concrete storage construction through product and domain implementation paths. A Server-specific `StorageProvider` would introduce a second architecture; moving SQLite construction into `talos-tui` would confuse rendering with application composition.

**Required outcome:** Existing products and a future `talos-server` consume the **same authoritative Session/Agent/permission/domain contracts**, while host composition chooses a concrete local implementation. The initial migration is **behaviorally equivalent**: the original local SQLite + TLOG/JSONL/FTS5 implementation remains the only production storage implementation and its external behavior remains unchanged. New database adapters and Server runtime functionality are **subsequent, separately claimed** changes.

This is a cross-host architecture integration Story, not a new storage engine and not the parent of #670.

## Existing-code baseline to characterize, not silently change

- `talos-session::SessionStore` is a path/file-format trait for TLOG/JSONL, not an independent relational backend contract.
- `SessionManager` currently composes local directory, SQLite index, TLOG/JSONL. `talos-session` itself depends on `rusqlite`.
- `talos-runtime` depends directly on `talos-session`, provides `RuntimeBuilder`/`RuntimeHandle` and optional shared composition.
- `talos-cli` is a product composition root; `talos-tui` is a renderer, not a DB factory.
- TLOG settlement, pending-submission custody, generation fencing, privacy and recovery are domain/runtime authorities, not database choice or transport-specific rules.

These are starting observations to verify again on the exact implementation base.

## Hard design invariants

1. **Equivalent transformation precedes extension.** Move dependency/constructor selection and extract small forwarding seams without changing any observable existing behavior. Any behavior change needs a separate Owner and review.
2. **Domain contracts are owned by their domains**; `talos-core` stays minimal. No generic CRUD, mega-Storage trait, SQL dialect, file path or transaction handle in shared domain or public host APIs.
3. **#670 owns storage capability taxonomy/contracts and implementation adaptation:** Log, Relational, Object, Lexical Search, Vector Search, Embedding, Rerank; semantic retrieval is composition. No duplicate capability trait, domain port or local adapter invented by this Story. Consume the already accepted #670 seams and evidence.
4. **ARCH-035 owns host integration:** where concrete defaults are selected, supported builder/SDK injection and compatibility, existing CLI/TUI/Desktop composition migration, and cross-host conformance. Reuse #670 work directly; if it already covers a specific path, record it as satisfied instead of rewriting.
5. **One runtime authority:** the host selects local capabilities; `talos-runtime` and domain libraries keep Agent/Session/permission/settlement/recovery semantics. No server-private session manager, actor or event bus.
6. **Physical dependency decoupling is a separate question:** source-level trait injection does not remove the compiled `talos-runtime -> talos-session -> rusqlite` chain. This requirement does not claim an SQLite-free binary.
7. **No false cross-resource atomicity:** SQLite index/journal and TLOG filesystem operations retain their current order, crash behavior and reconciliation. The port specifies observed semantics, not a fictional cross-file ACID transaction.

## Equivalent transformation plan / required evidence

| Gate | Deliverable | Evidence / completion condition |
|---|---|---|
| E0 | Inventory actual storage construction, imports, runtime/session/permission and host ownership paths; record accepted baselines | Source-path dependency matrix + regression/failure/restart characterization; no implementation |
| E1 | ADR for placement and scope of host-facing composition boundary; map accepted #670 contracts and default local profile | Reviewed decision, no-duplication matrix, public API/source-compatibility plan |
| E2 | Introduce minimal host-facing injection/selection by **direct delegation** to existing local constructors; preserve source-compatible RuntimeBuilder/CLI entrypoints | Old and new paths share one concrete implementation; no storage format/schema/SQL/transaction/FS behavior changes |
| E3 | Converge actually supported CLI/TUI/embedded SDK/eligible Desktop host paths on equivalent composition, without dragging renderer into storage logic | Deterministic multi-host conformance including error/permission/ID/order/cancellation and default config parity |
| E4 | Finalize equivalence audit, public API docs, independent review, exact-head CI and owner-first governance closeout | Existing implementation commit SHA, no unresolved equivalence failures; Server prerequisite acceptance recorded |

The E0/E1 architecture slice may be separately accepted while #670 implementation progresses, but **E2/E3 must not reimplement or pre-empt a claimed #670 seam**. No X-series/new-server-backend implementation starts merely because E0/E1 exists.

## Acceptance criteria

- [ ] Source-path matrix states what is core/domain semantics, what is capability contract, what is local implementation, and what is host composition.
- [ ] Accepted ADR defines API/trait placement and keeps `talos-core` lightweight.
- [ ] Full #670 capability contract coverage is consumed via its owner; no second storage provider hierarchy or duplicated local implementation.
- [ ] Existing CLI/TUI and supported embedded SDK behavior is unchanged; eligible Desktop integrations are characterized and migrated only within their existing authority/compatibility limits.
- [ ] `RuntimeBuilder::durable_session` remains source compatible or an independently reviewed migration is recorded; injected profiles use the same domain execution path.
- [ ] No storage-specific code moves into `talos-tui`, no independent `talos-server` persistence/Agent/Session engine.
- [ ] Existing SQLite schema/SQL/FTS5 ranking, TLOG/JSONL format/paths, migration, transaction and recovery ordering are unchanged.
- [ ] Idempotent submission, terminal settlement, lost-ack, generation fencing, Session ownership and permissions are not weakened or converted to generic CRUD.
- [ ] Error, cancellation, privacy/redaction, unsupported capability, concurrency and restart behaviors have explicit before/after tests.
- [ ] Current local adapter remains default and requires no external DB; no unproven SQLite-free compiled dependency closure claim.
- [ ] Code/review/CI tests pass at an exact commit head, and Completion Commit references an already-existing implementation SHA; owner/Board/Issues synchronized.

## Explicit exclusions

No PostgreSQL/MySQL/Zvec/S3 implementation, no new server binary or protocol endpoint, no AG-UI/ACP/A2A integration, no distributed DB/active-active, no independent persistent task runtime, no generic ORM, no full domain rewrite or relocation of every trait into `talos-core`.

## Server dependency / completion rule

**ARCH-035 is a hard pre-implementation prerequisite** for the SERVER-001 / SERVER-001-C `talos-server` production runtime composition and for the Server MVP functional integration. Related architecture research/ADR and isolated non-production experiments may continue without this prerequisite being Complete, but a Server feature/implementation PR must not pretend the required boundary already exists.

SERVER-002 / #360's future remote persistence implementation must also consume the same domain contract rather than inventing parallel Session truth; it remains a distinct unclaimed requirement.

Record acceptance on this Story with exact evidence and update #142/#361/#360 before dependent implementation activation. **Intake registration alone does not authorize code, claim ownership or satisfy the prerequisite.**

## Required reads / next governing action

- `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/LONG-RUNNING-TASK.md`, `docs/sop/START-ITERATION.md`
- `docs/backlog/active/STORE-002-storage-capability-contracts.md`
- `docs/backlog/active/SERVER-001-serve-connect-protocol-adapters.md`
- `docs/backlog/active/SERVER-001-C-standalone-server-host-composition.md`
- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- `crates/talos-runtime/src/lib.rs`, `crates/talos-session/src/store.rs`, `crates/talos-session/src/manager.rs`

## E1 contract-dependency reconciliation — 2026-11-13 (review candidate only)

This is a continuation of the **existing effective I320 E0/E1 documentation claim #905**, not an E2/E3 implementation claim or a new architecture decision. It preserves the Story's **Review / Claimed** status and leaves all acceptance checkboxes unclosed.

- STORE-002 / #670 I321 A/B capability *metadata* foundation is accepted via implementation PR #909 (`a9914b7a62b0873027997c69d9966d17f3fd48ba`). The metadata registry is **not** evidence of a bound operational backend.
- I322 C domain-owned contracts and existing-store direct forwarding are accepted via implementation PR #912 (**`3df409ef9ed707756350f2c7d4b2c8819f157222`**), with bounded closeout PR #930 merged (`a07b466d5aa2a03fb841cdcb918cd4968c778942`). These contracts are available for architecture review but have **not** rewired current production host constructors.
- I325 D *claim/activation*, not D implementation, became effective on governance PR #932 merge (`e49ccde70556403a9d7b66551565e2b6c81ad32d`). D owns proof of local operational delegation; #670 E owns local default composition; F owns broad storage equivalence. ARCH-035 must not absorb those slices.
- The [E1 proposal](../../proposals/arch-035-host-composition-boundary.md) now carries a contract-aware no-duplication review matrix and explicit acceptance conditions. It remains **Proposed / non-binding**, with no independently accepted public runtime injection signature, no new ADR number and no production source change.

**Next ARCH-035 gate:** independently review E1 ownership/API placement against I322 C and the eventual accepted I325 D / STORE-002 E host-facing composition, then record a defensible accepted decision or no-op integration disposition. E2/E3 require their own effective claim after usable accepted upstream integration evidence. No `talos-server` behavior or SQLite-free binary is implied by this checkpoint.
