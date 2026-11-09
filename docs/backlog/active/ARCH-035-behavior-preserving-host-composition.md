# ARCH-035: Behavior-Preserving Domain Persistence And Host Composition Decoupling

| Field | Value |
|---|---|
| Story ID | ARCH-035 |
| Source Issue | [#897](https://github.com/wjhuang88/talos/issues/897) |
| Status | Intake / Unclaimed |
| Priority | P1 — blocking architectural prerequisite for SERVER-001 / SERVER-001-C implementation |
| Type | Architecture / Runtime Composition / Equivalence Migration |
| Selected Iteration | None |
| Parent / consumer | [SERVER-001](SERVER-001-serve-connect-protocol-adapters.md) / #142; [SERVER-001-C](SERVER-001-C-standalone-server-host-composition.md) / #361 |
| Depends On | [STORE-002](STORE-002-storage-capability-contracts.md) / #670: accepted capability/domain seam; RUNTIME-001 SDK compatibility |
| Coordinates | SESSION-009 / #46; SESSION-011 / #612; DATA-002 / #141; SERVER-002 / #360 |
| Long-task planning | [SERVER-MVP-001](../../tasks/2026-11-10-server-mvp-storage-decoupled.md) proposed in draft PR #896 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #897 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | Select bounded iteration and establish effective target-branch claim before code; SERVER-001-C implementation waits on accepted equivalent transformation |

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
