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

### Selection-time non-terminal inventory and disposition — 2026-11-18

**Snapshot:** `main@bc4a37eeafff5e0e172764e861fac6cb6341fd86`, after accepted #956 code and #972 I328 closeout. Source-of-truth check: `docs/iterations/README.md` Current Operating Set/Recent Non-Terminal entries, individual iteration owners, `docs/BOARD.md` current top sections, `docs/backlog/PRODUCT-BACKLOG.md`, effective GitHub PRs and #897/#670. This is a **select-or-retain/no-overlap decision for every applicable non-terminal owner**, not a new claim, scope transfer, completion assertion or endorsement of proposed states. Dated historical text in some index/owner rows is qualified by the current source owner/Board **and the actual GitHub merge status**. In particular, governance claims #918 (Azure) and #920 (Git compatibility) have already merged and must not be treated as still ineffective merely because older owner prose says proposed.

| Existing iteration / owner | Source-recorded status at I330 selection | Disposition relative to E2 |
|---|---|---|
| I320 / ARCH-035 E0/E1 #897 | **Complete / Closed** documentary slice; prior claim #905 Released, ADR-104 accepted placement-only | **Reuse accepted decision**, preserve old claim history; proposed I330 E2 is separately bounded |
| I328 / STORE-002 E #670 | **Complete / Closed**, #956 implementation and #972 owner closure accepted | **Prerequisite satisfied**, do not duplicate E `LocalStorageProfile`, domain contracts or tests |
| STORE-002 F / #670 | **Open / Unclaimed** full fault/recovery parity residual | **Retain Unclaimed**, no F conformance/backend work within E2 |
| ARCH-035 E3/E4 / #897 | **Blocked / Unclaimed**, multi-host migration and full equivalence | **Defer**, no CLI/TUI/Desktop/SDK mass migration, no overall #897 closure |
| SERVER-001 / #142 and SERVER-001-C / #361 | **Blocked** on ARCH-035 accepted Host composition | **Retain Blocked**; no new ServerHost binary/protocol/runtime |
| I329 / SEARCH-001-D3-T3 / #957 | Current I329 owner/Board: T3a offline **Complete / Closed**, T3b production **Blocked / Unclaimed** (some iteration-index proposal text is historical) | **Retain separate search ownership**, no endpoint/browser/permission/network change |
| I323 / PROVIDER-008-C / #914 | **Active / Claimed effective**: governance #918 merged `f61a15cd3f0be81489e69f92d374fc8ee41eddec`; iteration/owner still preserve earlier proposed wording, not current merge truth | **Retain effective Azure claimant**, no model/provider transport scope or acceptance transferred |
| I324 / GIT-COMPAT-001 / #919 | **Active / Claimed effective**: governance #920 merged `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80`; old owner/Board proposal wording is historical | **Retain effective Git-native/preflight owner** and protected review/Windows 2.36 gates |
| I318 / TUI-054 / #269 | **Review / Claimed**, native repeat-safe key handling, physical/IME acceptance deferred | **Retain Review**, do not touch TUI renderer/input or claim its native evidence |
| I302 / TEST-003 / #717 | **Review / Claimed**, Windows current-process handle/resource acceptance residual | **Retain Review**, no process-boundary or Windows native test claim |
| I277 / DESKTOP-001-D3 / #29 | **Review / Claimed**, human/device acceptance deferred (Current Operating Set) | **Retain Review**, E2 cannot transfer Desktop acceptance or change Desktop defaults |
| I249 / dependency upgrade pilot | **Planned / Unclaimed** | **Defer**, no Cargo/dependency upgrade beyond independently reviewed strictly necessary E2 selection |
| I164 / startup composer continuity | **Paused**, superseded layout target with no completion claim | **Retain Paused**, no TUI layout/composer work |
| MODEL-007 / deferred model picker | **Deferred / Unclaimed** behind Desktop schedule | **Defer**, no provider/model menu or reasoning variant admission |
| AUTO-HISTORY-001 / #916 | Owner **Review / Claimed**, implementation #964 merged `0cf52249dbd227bcd8357806326d32d0f3a5408b` but owner does not claim Complete | **Retain Review** and its residual acceptance, no approval history or permission change |
| EDIT-LOCATIONS-001 / #921 | Owner **Review / Claimed**, implementation #963 merged `e1f8c693d276ea046b7d6d7dfa3a6797dac02338` but owner acceptance remains open | **Retain Review**, no edit/diff line-location or tool output work |
| SESSION-UI-001 / #952 | Owner **Review / Claimed**, implementation #962 merged `08a302eaf22cc51166a1a8bc399aec443b8b5eae`; owner acceptance remains separately pending | **Retain Review**, no session actor/queue UI behavior changes; E2 consumes existing authority |
| Session recovery integration / archive+checkpoint emergency | Board **Partial / Emergency override**, integration PR #965/native evidence pending | **Retain Partial**, E2 must neither overwrite recovery rules nor claim its validation |
| Session recovery storage prerequisite | Board **Partial / Emergency override**, storage integration/review pending | **Retain Partial**, no shared Session recovery implementation or authority transfer |
| TUI-APPROVAL-001 | Board **Review / Claimed**, local emergency display candidate | **Retain Review**, no permission/approval presentation or authority change |
| GIT-SNAPSHOT-001 | Board **Review / Claimed**, tracked-source capacity candidate | **Retain Review**, no Git snapshot code/source-fence changes |
| TUI-PERF-001 | Board **Review / Claimed**, native/performance acceptance residual | **Retain Review**, no TUI frame/render scheduling and no reuse of pending acceptance as E2 proof |
| CLIPBOARD-IMAGE-001 / #894 | Board **Review / Claimed**, native clipboard ingestion candidate | **Retain Review**, no image/clipboard/TUI input change |
| WORKTREE-SANDBOX-001 | Board **Planned / Unclaimed** separate sandbox owner | **Retain Planned**, no sandbox or protected permission-boundary change |
| I305 / TEST-004 and I300 / structured Git repair | **Complete / Closed** per current iteration source (older index entries may describe earlier Review) | **Retain completed**, not reopened by E2 |

**Non-overlap conclusion:** the sole proposed E2 owner on this branch is ARCH-035 / I330 / #975. Accepted I320 E0/E1 and STORE-002 E are prerequisites; proposed/Review/Planned/Paused/Blocked owners above keep their own source, PR, native and manual acceptance. E2 is only an opt-in runtime-facing **selection** over accepted delegates. Implementation is not authorized until this governance-only claim+activation PR itself is reviewed and merged; E3/E4, F, and ServerHost remain separately gated. Future relevant main-branch owner changes must be rechecked during merge-time CAS.

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
