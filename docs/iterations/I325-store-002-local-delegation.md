# Iteration I325: STORE-002 Existing Local Delegation (D)

> Document status: Review / Claimed — D local operational delegation test candidate, not Complete
> Selected: 2026-11-13
> Source: [STORE-002](../backlog/active/STORE-002-storage-capability-contracts.md) / [#670](https://github.com/wjhuang88/talos/issues/670)
> Baseline: A/B [#909](https://github.com/wjhuang88/talos/pull/909) and C [#912](https://github.com/wjhuang88/talos/pull/912) implemented; C bounded closeout [#930](https://github.com/wjhuang88/talos/pull/930) merged at `a07b466d5aa2a03fb841cdcb918cd4968c778942`. E/F and #897 Host remain gated.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 D only: verify and complete minimal functional local delegation of existing SQLite/FTS5/TLOG/JSONL stores via accepted I322 domain traits; add focused trait-directed conformance tests or narrowly necessary domain-local adapter glue without changing store implementations or production consumers. E local composition, F broad equivalence, ARCH-035/Host wiring and new engines excluded. |
| Claimed At | 2026-11-13 |
| Source Issue | #670 |
| Governance Claim PR | #932 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | D governance claim #932 merged `e49ccde70556403a9d7b66551565e2b6c81ad32d` after CI #4151 and independent claim review; D code requires its own CI and independent technical review. C implemented in #912 (`3df409ef9ed707756350f2c7d4b2c8819f157222`) and closed in #930 (`a07b466d5aa2a03fb841cdcb918cd4968c778942`). D governance [#932](https://github.com/wjhuang88/talos/pull/932) proposes an independent bounded work slice; source user authorized continued #670. Single-maintainer docs-only route needs exact-head CI, both validators, no-overlap CAS and explicit no-human-reviewer rationale before merge; implementation remains unauthorized on the open branch. |
| Implementation PR | #935 |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | After accepted bounded D delegation and independent API/runtime review, record Completion Commit, release claim, and keep E/F plus #897 independently gated. |

## Objective and dependency boundary

Preserve the first-phase zero-behavior-change constraint of Issue #670. **C already declares and directly forwards** `PendingSubmissionPersistence`, `TodoPersistence`, `MemoryPersistence`, `KnowledgePersistence`, `ExplorationPersistence`; Session transcripts already have `SessionStore`. D **must not redeclare those traits, create duplicate wrappers, introduce generic `Bound<T>`, or pretend metadata capability identities imply executable providers**.

D makes their existing local delegation demonstrably operational at accepted domain boundaries (real existing store fixtures and trait-directed invocation), and introduces only narrowly justified local binding glue if a concrete gap is proven. It does **not** select or inject runtime/Host backends; E owns one default local composition profile and #897 owns how Hosts consume accepted seams. F owns comprehensive before/after behavioral conformance and migration/recovery equivalence.

## Work Slice and acceptance

1. **Inventory actual C coverage, before editing**: list each existing domain trait, concrete implementer, constructor and operational test. Compare the I321 authority inventory, including `SessionIndex`, durable bindings and sidecars; record uncovered operations without silently expanding C contracts or changing source-of-truth ownership. If a new public API is genuinely required, seek an explicit owner/claim amendment and independent API decision first.
2. **Actual local operational delegation**: exercise accepted `SessionStore` TLOG/JSONL read/write and the existing Pending/Todo, Memory/FTS5+graph, Evolution patterns and Exploration sources/chunks/citations/FTS5 through already accepted domain trait signatures using real current implementations and temporary fixtures. Prove that the tested path goes through the trait-bound forwarding rather than only compile-time conformance. Do not duplicate the already present C `impl Trait for Store` or change any concrete method body.
3. **Typed preservation**: maintain existing typed errors, method receivers, CAS/generation fences, FTS ranking/snippets, source and citation integrity and transaction ownership. For shared variants, preserve exact constructor/default path and the absence of unbound providers. Only introduce additive domain-local adapter glue when justified by a specific acceptance gap; no generic CRUD/ORM, provider registry or new crate.
4. **Guarded verification**: relevant locked crate/workspace Format/Check/Clippy/Test, both governance validators and exact-head native CI on applicable platforms; independent Rust API/architecture review of any new interface or adapter glue. Store and test manifests, generated files, SQL, schema, migrations, TLOG/JSONL formats, FTS behavior, file paths, retry/locking, authority and production call sites must remain unchanged.
5. **Evidence and residuals**: map every current capability family (Log, Relational, Lexical, Object, VectorSearch, Embedding, Rerank) to existing accepted trait or explicit non-operational/unavailable status. Capture actual delegate exercise evidence, limitations, preserved defaults, and any uncovered SessionIndex/durable boundary as follow-up decisions, not imaginary coverage. Never claim E composition/F equivalence from D tests alone.

## Required reads and non-overlap

- [AGENTS](../../AGENTS.md), [collaboration SOP](../sop/AGENT-COLLABORATION.md), [testing SOP](../sop/TESTING.md), [local development SOP](../sop/LOCAL-DEV.md).
- [Current capability/authority inventory](../reference/STORE-002-I321-CAPABILITY-AUTHORITY-INVENTORY-2026-11-10.md), [I322 accepted domain contract map](../reference/STORE-002-I322-DOMAIN-OPERATIONAL-CONTRACT-MAP-2026-11-11.md), [SQLite consumer inventory](../reference/SQLITE-CONSUMER-INVENTORY.md), ADR-002 and ADR-008.
- No overlap with ARCH-035 / #897 Host E2/E3; no `talos-server`, CLI/TUI/Desktop/SDK composition rewiring, SEARCH-001, GIT-COMPAT-001, Windows #705, PROVIDER-008, `talos-models` runtime admission or Zvec/STORE-001 engine integration.
- Inventory other Active/Review/Planned claims and conflicting branches before claim finalization and again at implementation/CAS. Keep D/E/F separately governed; do not treat the C closeout as authority for D.

## Activation and completion gates

- **Effective claim #932** merged to `main` at `e49ccde70556403a9d7b66551565e2b6c81ad32d`. D implementation is in Review, and no acceptance is asserted before CI, independent review and merged implementation evidence.
- Completion requires a future **existing** implementation/evidence merge commit (not a status-only self-reference), exact-head CI, independent review appropriate to actual changed API/runtime scope, owner-first closure, derived index updates and #670 issue synchronization.
- Parent #670 remains OPEN for D/E/F; later E/default local composition and F/characterization require separate claims, #897 Host remains gated until accepted functional delegation and local profile.

## I325 D implementation candidate — 2026-11-13

- [Local delegation inventory](../reference/STORE-002-I325-D-LOCAL-DELEGATION-INVENTORY-2026-11-13.md) maps all accepted domain traits to existing constructors and real local operations, and retains SessionIndex/durable-binding residuals.
- Proposed tests use trait objects for Session TLOG/JSONL logs, Pending generation fencing, Todo transaction/dependency graph, Memory evidence/FTS/graph, Evolution patterns and Exploration atomic source/FTS/citations.
- No duplicate C forwarder, new provider, Host/runtime composition or SQL/data format change; independent API review and exact-head Rust CI **pending**, D **not Complete**, E/F gated.
