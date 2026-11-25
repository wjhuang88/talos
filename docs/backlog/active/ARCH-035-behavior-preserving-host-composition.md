# ARCH-035: Behavior-Preserving Domain Persistence And Host Composition Decoupling

| Field | Value |
|---|---|
| Story ID | ARCH-035 |
| Source Issue | [#897](https://github.com/wjhuang88/talos/issues/897) |
| Status | Active / Claimed — I334 E3a delivery in Review; #995 effective, #996/#998 partial source merged; E3b/E4 unclaimed, #897 Open |
| Priority | P1 — blocking architectural prerequisite for SERVER-001 / SERVER-001-C implementation |
| Type | Architecture / Runtime Composition / Equivalence Migration |
| Selected Iteration | [I334](../../iterations/I334-arch-035-e3a-cli-sdk-host-parity.md) — E3a effective claim #995, partial implementation #996/#998 accepted, additional startup acceptance in Draft #1010; I330 E2 and I320 E0/E1 Complete / Closed |
| Parent / consumer | [SERVER-001](SERVER-001-serve-connect-protocol-adapters.md) / #142; [SERVER-001-C](SERVER-001-C-standalone-server-host-composition.md) / #361 |
| Depends On | [STORE-002](STORE-002-storage-capability-contracts.md) / #670: accepted capability/domain seam; RUNTIME-001 SDK compatibility |
| Coordinates | SESSION-009 / #46; SESSION-011 / #612; DATA-002 / #141; SERVER-002 / #360 |
| Long-task planning | [SERVER-MVP-001](../../tasks/2026-11-10-server-mvp-storage-decoupled.md) proposed in draft PR #896 |
| E1 review artifact | [ADR-104](../../decisions/104-host-composition-ownership-boundary.md) — Accepted **placement/ownership only** via #936; [E1 proposal](../../proposals/arch-035-host-composition-boundary.md) remains historical evidence |
| E0 audit artifact | [Code-truth and regression inventory](../../reference/ARCH-035-I320-E0-CODE-TRUTH-2026-11-10.md) — published/accepted as documentary inventory via #906; runtime regression tests not rerun |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | ARCH-035 E3a only: deterministic CLI/TUI canonical AppServerSession and embedded SDK RuntimeBuilder local host-composition parity and justified no-op/direct-delegate decisions; preserve startup reconciliation, existing Session identity/path, lazy Memory/Todo/Exploration opens, typed errors, permission/settlement/cancellation and source-compatible feature-on/off SDK. Inventory Desktop Create/Existing/unbound read-only, no Desktop migration. Excludes E3b, E4, STORE-002 F, ServerHost, providers/SQL/schema/default behavior and protected authority changes. |
| Claimed At | 2026-11-20 |
| Source Issue | #897 |
| Governance Claim PR | #995 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I320 E0/E1/ADR-104 #936 accepted; STORE-002 E #956 Completion Commit `a28c1cccebad71eb5785d149129a640ceec3d098`; E2 #979 Completion Commit `21589346d99263c5675ad0b648df684801d2e861` with native CI #4412 8/8 and independent API/SDK review; I330 owner-first #987 merged `9f07fc6b8975939a4b54ddd40ba688fb9efa9f22` with docs CI #4455, independent review and CAS. #995 is governance-only: no E3a code/acceptance. Claim/activation require exact-head docs CI, both validators, independent Host/API/non-overlap review, single-maintainer authorization and merge-time CAS. |
| Implementation PR | E3a partial #996 (merged `51b6e54`) and #998 (merged `ce6c56d`); Draft #1010 startup reconciliation fixture (not yet accepted) |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | I334 E3a remains Active / Review until default startup and permission/fault/cancel/receipt parity are validated with exact-head CI and independent review; E3b Desktop, E4, STORE-002 F, ServerHost and #897 closure separately gated. |

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
- **Historical 2026-11-13 pre-ADR-104 dependency-reconciliation checkpoint:** the [E1 proposal](../../proposals/arch-035-host-composition-boundary.md) carried a contract-aware no-duplication review matrix and explicit acceptance conditions. **At that earlier point**, no ADR had yet been allocated and the proposal was non-binding. **Subsequently ADR-104 was allocated in #936**; even if its placement-only rule is accepted on reviewed merge, no concrete public runtime injection signature, production change or E2/E3 implementation is accepted.

**Next ARCH-035 gate:** independently review E1 ownership/API placement against I322 C and the eventual accepted I325 D / STORE-002 E host-facing composition, then record a defensible accepted decision or no-op integration disposition. E2/E3 require their own effective claim after usable accepted upstream integration evidence. No `talos-server` behavior or SQLite-free binary is implied by this checkpoint.

## E1 ownership decision candidate — 2026-11-13 (historical, before accepted #936 merge)

[ADR-104](../../decisions/104-host-composition-ownership-boundary.md) is proposed under the **existing effective I320 E0/E1 claim #905** to give the reviewed host-vs-domain placement an explicit authoritative ADR. This is **not yet Accepted** and does not alter the existing E1 proposed status or authorize E2/E3. It adds no public RuntimeBuilder signature, new provider, altered local default, production change or Completion Commit. The prior E0 inventory and E1 [proposal](../../proposals/arch-035-host-composition-boundary.md) remain evidence, not a substitute for exact-head independent ADR review, docs CI and guarded merge. Upstream D implementation and E/F acceptance remain separately owned by #670; the Story stays **Open / Review**.

### E1 decision acceptance gate — PR #936 (historical pre-merge checkpoint)

ADR-104's **host/domain ownership and composition-layer placement only** may be accepted on reviewed #936 merge after independent technical no-blocker review of the final head, exact-head documentation CI, both governance validators, single-maintainer authorization and merge-time CAS. The prior Copilot [initial review](https://github.com/wjhuang88/talos/pull/936#issuecomment-6843456241) found the placement decision supportable but requested correction of the 2026-11-13 historical E1 proposal sentence; this was corrected in the candidate and must be revalidated. The proposed ADR says **no shared callable runtime factory already exists**, no accepted public builder signature, no completed D/E provider binding or conformance results. Even if ADR-104 is accepted, this ARCH-035 owner remains **Review / Claimed**, E2/E3 stay **blocked/unclaimed**, and #897 stays **Open**; no Completion Commit for #897 is asserted.

## I320 E0/E1 bounded closeout — 2026-11-13 (parent #897 NOT complete)

**Delivery accepted: E0 source-truth inventory and E1 host/domain ownership placement only.** The previously existing evidence is:

- I320 governance/activation [#905](https://github.com/wjhuang88/talos/pull/905) merged `b211e590f6b1180af0679be8b1b03d77b0a19615`; bounded E0 inventory and non-binding E1 proposal [#906](https://github.com/wjhuang88/talos/pull/906) merged `70c0f7cd28d092cbd6ac0d310918fb97410b0cc4`.
- Accepted STORE-002 C trait boundary reviewed in E1 update [#934](https://github.com/wjhuang88/talos/pull/934) merged `d92aabaad4570d799c6e04d75c87abbb36a2b004`; no duplicate storage capability/provider or implementation.
- **ADR-104 placement-only acceptance** [#936](https://github.com/wjhuang88/talos/pull/936) merged **`07c99a8c2aeff615caeb62d38318cc2ca3c7bb3b`**. Its exact review head `3fd1b7176901832717b692202512dadb7364391f` has [CI #4173](https://github.com/wjhuang88/talos/actions/runs/43365335816) overall success, **reduced documentation validation** and both project/collaboration validators with 0 warnings. Its independent [read-only technical review](https://github.com/wjhuang88/talos/pull/936#issuecomment-6843928194) found no blockers; [single-maintainer authorization and CAS](https://github.com/wjhuang88/talos/pull/936#issuecomment-6852726331) recorded scope, lack of separate human reviewer and exact-head merge gate.
- A separate **remote Issue/owner reconciliation job failed** on that head because new *unrelated* SEARCH-001-D3 Issue #937 was missing from the global matrix; it is NOT counted as passed. #937's canonical registration and remote reconciliation subsequently merged through separately owned [#938](https://github.com/wjhuang88/talos/pull/938) as `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`, with that PR's exact-head remote owner job passing. This historical unrelated failure is preserved, not converted to a green check on #936.
- **I320 documentation slice Completion Commit: `07c99a8c2aeff615caeb62d38318cc2ca3c7bb3b`** (pre-existing merged evidence for architecture decisions, *not a production implementation or #897 Completion Commit*).

**Residual and next gate:** #670 I325 D [#935](https://github.com/wjhuang88/talos/pull/935) remains independently owned/open at this checkpoint despite an exact-head CI result; #670 E default local composition and F behavior parity are not delivered. No runtime host factory, new `RuntimeBuilder` injection signature, SQLite-free binary or E2/E3/E4 regression result was shipped. **No production E2/E3 work may start** until accepted usable STORE-002 integration evidence, new effective bounded ARCH-035 claim, SDK compatibility review and native before/after baseline. Keep #897 Open / Blocked, #142/#361 ServerHost production blocked; the completed I320 documentation claimant is explicitly **Released**.

## I330 E2 atomic claim+activation proposal — 2026-11-18

- **Owner-first selection:** [I330](../../iterations/I330-arch-035-e2-host-storage-selection.md) proposes only ARCH-035 **E2**. [Governance draft #975](https://github.com/wjhuang88/talos/pull/975) contains the same-branch `Claim State: Claimed` + `Active / Claimed` proposal with complete Work Slice, claimant and actual PR number; **no ownership or activation is effective until its authorized target-branch merge**. The claim PR carries no implementation code, tests, Cargo/API changes or new SQL.
- **Upstream acceptance**: STORE-002 C/D accepted; E feature-gated `LocalStorageProfile` accepted via #956 implementation Completion Commit `a28c1cccebad71eb5785d149129a640ceec3d098`, I328 owner-first closure #972 `bc4a37eeafff5e0e172764e861fac6cb6341fd86`. Parent STORE-002 #670 remains Open for unclaimed F broader recovery/conformance. ADR-104 #936 is placement only; no E2 injection method has been approved.
- **Historical claimant:** I320 E0/E1 governance #905 authorized documentary E0/E1 only; I320 docs and [ADR-104](../../decisions/104-host-composition-ownership-boundary.md) accepted via #906/#934/#936, then the old claim **Released** on 2026-11-13 and documented in #945. This prior `Claim State: Released` is historical, **not** a second current effective claim or an E2 implementation authorization.
- **Source-grounded non-overlap inventory:** CLI `AppServerSession::set_persistence`, `SessionManager::new` startup reconciliation vs `with_dir`, lazy cached Memory first query, Todo provider closure; Desktop/SDK optional `RuntimeBuilder::durable_session(DurableSession)` and in-memory unbound mode. E2 requires **one real opt-in functional Host selection/delegation** and targeted API+negative-path fixtures, without rewiring production defaults or migrating all Hosts. E3 multi-host migration and E4 overall equivalence remain separate, as do STORE-002 F and ServerHost. Concurrent SEARCH/TUI/Session/Provider/Git owners keep existing claims and protected source boundaries.
- **Historical claim gate now satisfied:** #975 merged as `799e3c35e6a5f9b2adb8ce2346762b148be940c0` after exact-head docs CI #4377, independent scope review and guarded CAS. E2 is effective for **only** the approved bounded implementation; it does not accept the new #979 public method. E3/E4, F and ServerHost stay separately gated; #897 and #670 OPEN.

## I330 E2 implementation review checkpoint — 2026-11-18 (#979)

- **Effective E2 claim:** governance #975 merged `799e3c35e6a5f9b2adb8ce2346762b148be940c0`; source owner and I330 claim `@wjhuang88`, E2 only. [Implementation #979](https://github.com/wjhuang88/talos/pull/979) is **Review / Claimed**, not accepted, with no Completion Commit or Host migration.
- **Implementation scope:** additive `#[cfg(feature = "local-storage")]` `RuntimeBuilder::durable_session_from_custom_local_profile(&LocalStorageProfile, &str) -> Result<RuntimeBuilder, SessionError>`, calls accepted `profile.custom_session_manager().create_or_open_session(external_id)` and existing `durable_session`. Unbound in-memory/old builder API remain intact; no default SessionManager reconciliation, optional DB eager opening, CLI/TUI/Desktop/SDK/ServerHost call-site rewire, new SQL or storage engine.
- **First native [CI #4383](https://github.com/wjhuang88/talos/actions/runs/44253925717) FAILED** on macOS explicit `local-storage` feature regression: new test incorrectly expected UUID TLOG under `sessions/` rather than the existing `sessions/durable/` subtree. This is a **test oracle error**, confirmed by `talos-session::durable::create_or_open`, while `durable-bindings.sqlite` remains at `sessions/`. The other seven native jobs succeeded. Source-only test fix was pushed on #979; first failure is retained and no CI green claim is made for its newer head.
- **Next gates:** exact-head macOS/Windows locked fmt/check/Clippy/tests and explicit feature tests, both governance validators and independent technical review of public builder API, source-compatible SDK and error/no-create semantics, plus owner/no-overlap merge-time CAS. E2 is not Complete. E3/E4, #670 F, ServerHost and entire #897 remain separately open.

## I330 E2 owner-first bounded completion — 2026-11-19 (parent #897 remains OPEN)

- **Completion Commit: `21589346d99263c5675ad0b648df684801d2e861`** — the actual pre-existing implementation merge of [PR #979](https://github.com/wjhuang88/talos/pull/979) on main, preceding this documentary closeout. Governance claim #975 `799e3c35` was the authorization, **not** an implementation completion commit.
- **Exact-head evidence:** final implementation HEAD `99d6ecf303a6ebb0d54f700de6613f857360ccc3` passed [native CI #4412](https://github.com/wjhuang88/talos/actions/runs/44457476055) **SUCCESS 8/8**. macOS locked release preflight, fmt/check/Clippy/workspace test and feature-on `talos-runtime --features local-storage`, Windows workspace/Rust and Desktop, Linux Desktop, installer and remote owner checks all passed. Project governance validator **0 warnings**, collaboration claim validator **PASSED with one unrelated SEARCH-001-D3 unclaimed intake warning**; the warning remains independently owned. Earlier [CI #4383](https://github.com/wjhuang88/talos/actions/runs/44253925717) FAILED one new mislocated TLOG assertion (95/96 passed); narrowed test-only repair led to old-head CI #4402 success 8/8 and final #4412 success, not a rewritten historical result.
- **Independent/authorization:** read-only [Copilot APPROVE on the final exact head](https://github.com/wjhuang88/talos/pull/979#issuecomment-6999596024) confirmed source-compatible opt-in API, with_dir vs new() startup distinction, typed invalid-ID/no-create and unchanged Session TLOG path, optional-store laziness and current SDK behavior. [Final single-maintainer authorization + CAS](https://github.com/wjhuang88/talos/pull/979#issuecomment-7002828221) recorded no overlapping E2 owner, mergeable branch, full validation, and unrelated Provider/Azure main movement per ADR-071. No separate human approval is claimed.
- **Accepted E2 only:** feature-gated `RuntimeBuilder::durable_session_from_custom_local_profile` delegates to accepted STORE-002 E `LocalStorageProfile::custom_session_manager().create_or_open_session` and unchanged `durable_session`. Tests compare Session ID/path with the direct constructor, validate the `sessions/durable/` transcript and `durable-bindings.sqlite` index, absence of eager Memory/Exploration/Todo DBs, typed invalid identity without creating root directories, and legacy unbound/bound API. Existing CLI/TUI/Desktop/SDK production callers, Session reconciliation, permissions, cancellation, SQL/schema, format and defaults remain unchanged; there is no SQLite-free binary or completed global Host equivalence claim.
- **Residual and release:** bounded I330 E2 is **Complete / Closed** and this parent's former E2 claim is **Released**. The parent ARCH-035 / #897 remains **Open / Blocked / Released** until separately authorized E3 product-host composition and E4 equivalence evidence and acceptance. STORE-002 F remains independently unclaimed under #670; ServerHost #361 cannot treat E2 as full architecture acceptance.

## I334 E3a effective claim and accepted partial source — 2026-11-23

- **Effective on main:** atomic E3a claim [#995](https://github.com/wjhuang88/talos/pull/995) merged `08f2a41c5f83ad7a87e310370df83d140c39e889`; historical proposal below is superseded for effective status.
- **Accepted bounded source:** [#996](https://github.com/wjhuang88/talos/pull/996) `51b6e548e26e98444459ce93340149aa7bfc4578` covers CLI/SDK positive conformance, and [#998](https://github.com/wjhuang88/talos/pull/998) `ce6c56d2290d11c058ec33b9876d546ac9dce056` covers independently locked SDK default/coding/local-storage external fixture. #998 [CI #4561](https://github.com/wjhuang88/talos/actions/runs/45144165793) 8/8 and independent review APPROVE; only partial E3a evidence, not completion.
- **Current work:** [Draft #1010](https://github.com/wjhuang88/talos/pull/1010) proposes a child-process `SessionManager::new()` vs `with_dir()` startup index parity test; unaccepted until exact-head native CI and code review. Broader permission/receipt/cancel/fault behavior remains gated. No production CLI/SDK/Session default migration or E3a Completion Commit.
- **Parent scope:** ARCH-035 [#897](https://github.com/wjhuang88/talos/issues/897) is **OPEN** (premature automatic closure from #998 corrected). STORE-002 [#670](https://github.com/wjhuang88/talos/issues/670) is OPEN for F. E3b, E4 and ServerHost remain separately governed.

## I334 E3a atomic claim and activation proposal — 2026-11-20 (ineffective until #995 merge)

- **Accepted prior boundary:** I330 E2 owner-first [#987](https://github.com/wjhuang88/talos/pull/987) merged `9f07fc6b8975939a4b54ddd40ba688fb9efa9f22`; underlying [#979](https://github.com/wjhuang88/talos/pull/979) E2 Completion Commit `21589346d99263c5675ad0b648df684801d2e861`. Historical I330 claim is Closed and must not be reopened.
- **Separate proposed claim:** [I334](../../iterations/I334-arch-035-e3a-cli-sdk-host-parity.md) / governance [#995](https://github.com/wjhuang88/talos/pull/995) proposes E3a only. Its `Claimed` and `Active` values are **ineffective until a reviewed and authorized main merge**. This PR does not implement E3a or approve a public API.
- **Source-backed non-overlap:** CLI `AppServerSession::new` / `set_persistence` retains Session/Agent authority; `SessionManager::new()` startup reconciliation must not be replaced with E2's custom `with_dir`. Memory lookup is query-time lazy; Todo prompt open is callback-time. Embedded `RuntimeBuilder::new` / `durable_session(DurableSession)` / unbound mode remain source-compatible. Desktop Create/Existing/unbound is inventoried but **not migrated** (E3b separately gated). E4, STORE-002 F, ServerHost and the parent #897 remain Open.
- **Next gate:** independent read-only Host/SDK and non-overlap review, exact-head docs CI, both governance validators, single-maintainer authorization and merge-time CAS, then an independent E3a implementation branch; no permission/SQL/schema/default behavior changes in this claim.
