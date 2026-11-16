# STORE-002: Storage And Retrieval Capability Contracts

**Status**: Active / Claimed proposed by #955 — I321 A/B, I322 C and I325 D accepted; I328 E selected (ineffective until claim merge); F unclaimed; parent #670 Open

**Type**: Architecture / Contract Convergence

**Source Issue**: [#670](https://github.com/wjhuang88/talos/issues/670)

**Parent / Related**: DATA-002 / #141; SERVER-001 / #142; SERVER-001-C / #361; SERVER-002 / #360; STORE-001; ARCH-035 / #897 (cross-host composition consumer; no duplicate capability extraction).

**Non-overlap note:** STORE-002 exclusively owns the complete storage/retrieval capability taxonomy, domain capability seams and behavior-preserving local implementation adaptation. ARCH-035 / #897 owns how product hosts and `talos-runtime` consume those accepted seams via equivalent composition, preserving existing CLI/TUI/Desktop/SDK behavior. If an interface/adapter is already delivered by STORE-002, ARCH-035 reuses it; it does not reimplement it.

## Objective

Converge Talos's existing persistence, retrieval, and semantic capability boundaries behind explicit contracts without changing current SQLite, FTS5, TLOG, JSONL, schema, transaction, migration, path, ranking, or runtime behavior.

## Scope

Define narrow capability contracts for Log, Relational, Object, Lexical Search, Vector Search, Embedding, and Rerank; preserve domain Stores/Repositories and their invariants; document authority and rebuildability; and provide one explicit local composition that delegates to the existing implementations.

Vector search, embedding, and other unavailable capabilities must have explicit unavailable semantics. Zvec, PostgreSQL, S3, generic ORM/query DSL work, and production behavior changes are out of scope for the first phase.

## I321 A/B merged baseline (completed bounded slice; #670 OPEN)

- Governance claim #908 merged `201a9b61d72e704da3a86f5db283b9f6e4d92938`; implementation [PR #909](https://github.com/wjhuang88/talos/pull/909) merged `a9914b7a62b0873027997c69d9966d17f3fd48ba`.
- [Exact-head CI #4039](https://github.com/wjhuang88/talos/actions/runs/42848943669) passed **8/8**, both governance validators **0 warnings**, independent [API review](https://github.com/wjhuang88/talos/pull/909#issuecomment-6772057381) and [delta check](https://github.com/wjhuang88/talos/pull/909#issuecomment-6772113212) no blockers, [CAS](https://github.com/wjhuang88/talos/pull/909#issuecomment-6785449447) recorded.
- Delivered: seven typed `StorageCapabilityKind` IDs, metadata-only `CapabilityRequest` helper and source-backed authority inventory. Speculative `Bound<T>` was removed; no SQL/runtime/default provider changes.
- A/B milestone is complete; **C/D/E/F remain separately gated** and #897 Host E2/E3 is still blocked.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 I328 E only: select one explicit behavior-preserving default-local composition over accepted SessionStore, PendingSubmissionPersistence, TodoPersistence, MemoryPersistence, KnowledgePersistence and ExplorationPersistence with current TLOG/JSONL/SQLite/FTS5 stores. Preserve existing paths, default-vs-custom Session startup reconciliation, lazy optional opens, typed errors, pending fences and domain authority. No duplicate domain traits, SQL/schema/formats, generic CRUD/registry, runtime/CLI/TUI/Desktop/SDK/Server Host wiring, new backend, F-wide parity or #897 E2/E3. |
| Claimed At | 2026-11-16 |
| Source Issue | #670 |
| Governance Claim PR | #955 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User authorized continued #670 work. C #912 domain traits accepted; D implementation #935 Completion Commit `027b8a59194f5ba496b0048c7183ad236e2e2dd2` and I325 owner-first closeout #951 merged `8cb0f97a44259b4c7f4caee24c06ab02fc542ad7`. Governance-only Draft #955 proposes I328 E atomic claim/activation, ineffective until final exact-head CI, both governance validators, independent read-only technical review, single-maintainer rationale, CAS and merge. No production E implementation before this claim is effective. |
| Implementation PR | Not started — I328 E governance-only claim #955 |
| Last Updated | 2026-11-16 |
| Handoff / Release Condition | E activated only on reviewed #955 merge. Subsequent separate E implementation must preserve existing local defaults and pass native CI/independent API review; E owner closeout requires actual merged implementation Completion Commit. Parent #670 stays Open for F; ARCH-035/#897 E2/E3 and ServerHost remain gated. |

## Governance

I321 A/B is **merged and accepted** as a narrow descriptor/inventory foundation, not as a functional provider binding. I322 C governance claim #911 **merged** as `60da1bbadca00c9bdee6fbcb8f5ad990a4313331`. Its narrow work slice is effective: add domain-owned operational interfaces and exact forwarding to unchanged stores; preserve all existing callers, SQL, error, schema and format behavior. C implementation [#912](https://github.com/wjhuang88/talos/pull/912) is accepted and merged as `3df409ef9ed707756350f2c7d4b2c8819f157222` with exact-head CI #4127, independent public API review and merge CAS. This claim is released. D/E/F and #897 Host implementation remain gated. No generic `Bound<T>`, SQL/CRUD provider, or new backend is authorized.

## Acceptance Direction

- Every current persistence/retrieval responsibility has one capability classification.
- Domain invariants remain in domain Stores/Repositories.
- Existing SQLite/TLOG/JSONL implementations delegate through the contracts with zero behavior drift.
- A future talos-server profile and Zvec retrieval implementation can bind without another foundational rewrite.
- The issue remains open until the complete contract layer and evidence are merged.


## Intake history

Initially registered as Intake / Unclaimed on 2026-10-07; narrow I321 A/B claim proposed on 2026-11-10 via #908. Source Issue #670 remains open for C/D/E/F after A/B.

## I321 A/B implementation history — PR #909 (merged)

- [Current capability and authority inventory](../../reference/STORE-002-I321-CAPABILITY-AUTHORITY-INVENTORY-2026-11-10.md): authoritative/derived/ephemeral data, all seven capability families, existing paths and residuals.
- `crates/talos-core/src/storage_capability.rs`: typed `StorageCapabilityKind`, stable IDs and `CapabilityRequest` integration with existing `CapabilityRegistry` whose empty resolution returns `ResolutionResult::Unavailable`. No operational binding type, domain call site, or runtime behavior is introduced.
- Candidate code is additive and non-operational: it does **not** satisfy C/D/E/F, does not implement seven storage operation traits, and does not yet provide #897 a functional dependency to inject.
- Validation history: [CI #4030](https://github.com/wjhuang88/talos/actions/runs/42799763086) passed 8/8 on **previous** head `5c38e6e`; independent [Copilot review](https://github.com/wjhuang88/talos/pull/909#issuecomment-6765145292) identified one **blocking** unconstrained `StorageCapabilityBinding<T>` public API. The binding and its tests were removed. Final [CI #4039](https://github.com/wjhuang88/talos/actions/runs/42848943669) passed **8/8** on head `f3563138`, independent [no-blocker delta review](https://github.com/wjhuang88/talos/pull/909#issuecomment-6772113212) and merge-time CAS recorded. I321 A/B merged as `a9914b7a62b0873027997c69d9966d17f3fd48ba`.
- #670 remains open; avoid marking Complete or recording a fictitious Completion Commit. Future domain adapter delegation and local profile need separately authorized work.

## I322 C proposed activation

- [I322 domain-owned contract extraction](../../iterations/I322-store-002-domain-contract-extraction.md) proposes narrow interfaces anchored in the existing Session/Pending/Todo, Memory, Evolution and Exploration contracts with existing errors, while preserving local concrete behavior.
- Independent API review is required before C implementation merge. Explicit unavailable status for Object/Vector/Embedding/standalone Rerank does not imply fake executable providers.
- The C claim #911 was effective and its bounded implementation #912 has now merged; [I322](../../iterations/I322-store-002-domain-contract-extraction.md) records its final CI, review, CAS and Completion Commit. D/E/F and #897 E2/E3 remain unclaimed/blocked.

## I322 C implementation completed — PR #912 (2026-11-13)

- Completion Commit: `3df409ef9ed707756350f2c7d4b2c8819f157222` (existing implementation merge; not the subsequent docs closeout commit).
- [Domain contract mapping](../../reference/STORE-002-I322-DOMAIN-OPERATIONAL-CONTRACT-MAP-2026-11-11.md) documents existing `SessionStore` plus new PendingSubmission/Todo, Memory, Evolution and Exploration domain contracts. Four `storage_contracts.rs` files add typed traits and fully qualified forwarding to existing stores; four `lib.rs` files export them. No runtime consumer is migrated, no provider becomes operational merely by classification.
- [CI #4127](https://github.com/wjhuang88/talos/actions/runs/43249945758) on exact reviewed implementation head `f908f86633ccee5892bf91d738b41bf65eb33799` completed overall SUCCESS: seven scoped code, native platform and validator jobs passed; both project governance and collaboration-claim validators passed. A separate global remote Issue reconciliation job failed solely because already-closed unrelated #901 was still listed as Open. This is a separately owned advisory residual under [ADR-071](../../decisions/071-local-convergence-and-scoped-remote-gates.md), not a passing job.
- Independent [Copilot API and governance-diff review](https://github.com/wjhuang88/talos/pull/912#issuecomment-6828277614) found no blockers. This is independent Agent technical review under one maintainer identity, **not independent human approval**. [Final merge-time CAS](https://github.com/wjhuang88/talos/pull/912#issuecomment-6840919956) checked the effective #911 claim, no competing I322 PR, mergeability and exact head before squash merge.
- Historical macOS Session fork-recovery test `durable.rs:2363` failed on prior CI #4116 and on an unrelated historical #877 attempt, but **passed on final exact-head CI #4127**. Its intermittent cause remains unknown; no recovery/locking assertion, production store, SQL, FTS5, SQLite schema, TLOG/JSONL, transactions, dependencies or caller was changed to force a pass.
- This closes **only I322 C**; parent #670 remains Open for D existing-implementation delegation, E default local composition and F conformance/recovery evidence. Their separate governance claims are required before implementation. #897 Host E2/E3 is not unblocked by C alone.

## I325 D local delegation implementation — accepted 2026-11-14 (Complete / Released)

- [I325](../../iterations/I325-store-002-local-delegation.md) selects the existing-storage **D** work slice after C merged and closed. Governance [#932](https://github.com/wjhuang88/talos/pull/932) merged `e49ccde70556403a9d7b66551565e2b6c81ad32d`; D implementation #935 merged and accepted; Completion Commit `027b8a59194f5ba496b0048c7183ad236e2e2dd2` is pre-existing main evidence. The D claim is Released while parent #670 stays Open for E/F.
- C already implemented direct `impl Trait for Store` forwarding for the five new domain contracts; D must **not repeat or replace those interfaces or forwarders**. Its purpose is to demonstrate real local SQLite/FTS5/TLOG/JSONL operations through the accepted trait surfaces and add narrowly justified domain-local adapter glue only when evidence proves a gap, without moving SQL/transactions/paths or rewriting production callers.
- Preserve explicit non-operational/unavailable storage category semantics: capability descriptors do not establish runtime/provider readiness. `SessionStore` already handles TLOG/JSONL; SessionIndex/durable-binding coverage needs explicit inventory/disposition rather than blind new APIs.
- E owns default local composition, F owns full before/after equivalence and crash/recovery characterization, ARCH-035/#897 owns cross-Host consumption; no D claim transfers those work slices. Parent #670 stays Open.

## I325 D execution checkpoint — 2026-11-13

- [Source-backed local delegation inventory](../../reference/STORE-002-I325-D-LOCAL-DELEGATION-INVENTORY-2026-11-13.md) maps existing domain trait/Store operations and the seven capability categories, including SessionIndex/durable-binding gaps.
- Accepted tests exercise real local SQLite/FTS5/TLOG/JSONL through existing domain traits; no duplicate C forwarders or production caller changes. Exact-head validation, independent review and merge CAS accepted; E/F and #897 Host remain gated.

## I325 D completion evidence — 2026-11-14

- **Completion Commit: `027b8a59194f5ba496b0048c7183ad236e2e2dd2`** — actual implementation #935 squash merge, not this documentation closeout.
- Final reviewed head `259d240aa9f69d4c89060fd0891e61ea1b99eb19`: [CI #4223](https://github.com/wjhuang88/talos/actions/runs/43552408748) concluded overall SUCCESS; all seven applicable native/format/check/clippy/test/governance jobs succeeded, including both validators (one unrelated SEARCH-001-D3 collaboration warning). Separate **Remote Issue/Owner reconciliation FAILED** due global matrix missing #947 on the historical candidate and retaining closed #941. The failed job is not retroactively counted green; this closeout batches the separate register cleanup.
- Independent [exact-head technical APPROVE](https://github.com/wjhuang88/talos/pull/935#issuecomment-6871229199), [formal Copilot approval recommendation](https://github.com/wjhuang88/talos/pull/935#pullrequestreview-5901387865) with original dedup finding resolved, [single-maintainer CAS](https://github.com/wjhuang88/talos/pull/935#issuecomment-6871430719). No second human reviewer claimed.
- Delivered bounded Session TLOG/JSONL, Pending/Todo, Memory SQLite+FTS5/graph/content dedup, Evolution and Exploration trait-dispatch tests and inventory. No production code, SQL/schema/transactions/format/path, Cargo, default profile, runtime Host or SDK change.
- Parent #670 remains OPEN for E local-default composition and F wide fault/recovery parity, separately governed. #897 E2/E3 and ServerHost remain gated; SessionIndex/durable-binding and unavailable provider capabilities remain explicit residuals.

## I328 E default-local composition proposed activation — 2026-11-16

- [I328](../../iterations/I328-store-002-default-local-composition.md) selects only E following accepted #935 implementation and #951 D closeout. Draft [governance #955](https://github.com/wjhuang88/talos/pull/955) **proposes** atomically claimed/active E ownership; neither is effective until target-branch merge. No E implementation exists yet.
- E must select/assemble existing concrete local Session, Pending/Todo, Memory, Evolution and Exploration stores through accepted C/D delegate contracts, retaining all lazy/optional constructors and typed semantics. The seven `StorageCapabilityKind` entries remain metadata; Object/VectorSearch/Embedding/standalone Rerank remain explicitly unavailable. Do not invent a generic operational provider or SQL abstraction.
- `SessionManager::new()` must preserve its startup reconciliation that `with_dir()` lacks. Memory must not create absent `memory.db` merely to assemble a profile. Exploration read-only search must not create absent DB. No runtime `RuntimeBuilder` / Desktop / TUI / SDK / Server Host consumption: #897 owns those independently after accepted E. F owns broader fault/recovery/behavior parity.
