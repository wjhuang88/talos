# SERVER-MVP-001 — Single-Node, Storage-Decoupled Talos Server

> Planning baseline (2026-11-10). **Status: Planning / Unclaimed.** This file is a proposed long-task coordinator, not an effective Collaboration Claim, implementation authorization, or evidence of a shipped server.
>
> Parent: [SERVER-001 / #142](https://github.com/wjhuang88/talos/issues/142). Host: [SERVER-001-C / #361](https://github.com/wjhuang88/talos/issues/361). Storage abstraction: [STORE-002 / #670](https://github.com/wjhuang88/talos/issues/670). **Hard architecture prerequisite:** [ARCH-035 / #897](https://github.com/wjhuang88/talos/issues/897) (host-composition equivalence and contract consumption). Session: [SESSION-009 / #46](https://github.com/wjhuang88/talos/issues/46). Future remote persistence: [SERVER-002 / #360](https://github.com/wjhuang88/talos/issues/360).
>
> **Work mode: Standard**. No deferred-human-validation tracker; required manual/security acceptance cannot be assumed passed.

## Outcome / hard finish line

Produce one **single-node, headless, API-usable** `talos-server` that composes the existing `talos-runtime` without its own Agent loop, supports a bounded **AG-UI HTTP/SSE** client interaction vertical slice (version pinned by an ADR), and retains/reloads the currently supported local durable Session state across process restart.

**Storage rule: no database brand is bound into the runtime, ServerHost, SessionHost, protocol DTOs, or shared storage contract.** Phase one reuses SQLite + TLOG/JSONL exactly as they exist through explicit capability/domain ports; this MVP does **not** implement or require PostgreSQL, MySQL, Zvec, S3, SQLx, or a remote DB. PostgreSQL remains only a possible *future, separately claimed* SERVER-002 adapter, not the chosen Server backend or a requirement to compile or start the MVP.

One end-to-end demonstration must prove: launch isolated single-node host -> create/attach Session -> submit a real turn using a deterministic Fake provider -> receive ordered text/tool/lifecycle events -> resolve an approval through the canonical permission path or fail closed -> cancel a turn -> disconnect/reconnect with a cursor/snapshot -> stop/restart the host with the same local storage root -> read committed history and submit another turn. No ambiguous unfinished tool action is replayed. Optional live-model smoke test must not replace deterministic CI evidence.

**Delivery label:** `Local Durable Server MVP`, **not** production-ready distributed/remote-database Server. An independent MVP milestone can close while #142, #361, #360, #46 and #670 remain open if their wider owner acceptance is not fully satisfied.

## Boundaries

### In scope
- A small one-process `talos-server` composition root; no TUI/Desktop/dashboard dependency, duplicate Session engine, event bus, or permission engine.
- Server/Runtime lifetime independent of one network request; explicit SessionHost and bounded client attachment/connection semantics.
- Full first-phase STORE-002 taxonomy (Log, Relational, Object, Lexical Search, Vector Search, Embedding, Rerank, semantic retrieval as composition), all current responsibilities classified. The initial local composition delegates to **existing implementations without behavior drift**; unavailable capability slots report unsupported explicitly.
- **Domain-semantic persistence ports** needed by SessionHost: Session catalog/history, durable turn settlement, submission custody/receipts/reconciliation, and generation/ownership control. Reuse existing domain Stores where possible; do not create a general SQL/CRUD/ORM abstraction or rewrite TLOG internals for symmetry.
- A locally durable, single-active-owner Session profile with no shared-filesystem / NFS-WAL assumptions; use existing safe ownership, fail-closed recovery, and shutdown contracts. Local persistence is authoritative only for that local host/root.
- Small AG-UI projection, loopback-only initial HTTP listener, authentication and canonical authorization for state changes, bounded SSE, deadlines, cancellation, and readiness/liveness diagnostics.
- Contract/conformance tests, integration test client, API documentation, and deployment/runbook for a **single local host**.

### Explicitly excluded
- PostgreSQL or any other newly implemented database driver, migrations or production schema; dual backend parity claims.
- Public/LAN/Internet bind; TLS termination, ingress, multi-tenancy, user identity provisioning, public authentication federation, or production K8s HA.
- Active-active Session ownership, distributed locks/leader election, multi-node migration, automatic failover and remotely authoritative state.
- `talos connect` / ACP implementation (#47), A2A (#40), S3 tools (#362), general MCP serving, arbitrary AG-UI state synchronization, custom UI and frontend side-effect tools.
- Re-activating the closed/superseded persistent-task engine #38; only existing Work/Goal/Mission and Session semantics may be consumed.
- OpenAI-compatible Chat Completions or custom proprietary chat/session API.
- Changing local SQLite schema, SQL statements, FTS5 semantics, TLOG/JSONL formats, paths, transactions, write ordering or migration behavior in STORE-002 phase one.

## Storage architecture decision to ratify in T0

```text
talos-server (headless host, local-only MVP)
    -> SessionHost / connection + control
    -> talos-runtime (one Agent/permission/execution authority)
    -> domain repositories / custody / settlement contracts
    -> capability composition
         local = existing SQLite metadata/FTS + existing TLOG/JSONL transcript
         tests = deterministic fake implementations where a narrow port requires it
         later = optional adapters by separate owners (e.g. SERVER-002)
```

**Separate two layers**:
1. Capability taxonomy for `Log / Relational / Object / Lexical / Vector / Embedding / Rerank` and explicit capability presence/absence. This is not one `Storage` trait or an engine-independent SQL query DSL.
2. Domain ports retaining atomic and security semantics (names non-binding): `SessionRepository`, `DurableTurnStore`, `SubmissionCustodyStore`, `SessionOwnership`. The business domain owns turn finalization, idempotency, cancellation, receipts, and invariant checks; a database adapter may fulfill the contract but must not create new business truth.

**Critical invariant:** SQLite and TLOG are not one cross-resource SQL transaction. The local profile must preserve today's persisted order, first-terminal-wins, crash recovery and ambiguity handling; do **not** claim global ACID over multiple files. A future relational adapter may implement the same semantic result with its own native transaction, but SQL or transaction handles must not leak upward.

Ports must cover: stable Session/Turn/Submission identities; ordered commit and authoritative recovery; same-ID/same-payload idempotency; same-ID/different-payload rejection; pre/post-commit failure; stale-generation fencing; no duplicate side effects after timeout/crash; payload redaction/privacy; bounded shutdown and ownership release. Do not confuse accepting a command with completed execution. Avoid async trait churn in product crates; decide sync/async adapter boundary via measurements and ADR.

**No-DB-lock-in test:** ordinary `talos-runtime` and `talos-server` source and public API must not name `PgPool`, `Postgres`, `MySql`, `sqlx::Transaction`, vendor SQL types, or a DB-specific protocol identifier. The default local profile must be independently constructible and testable without any remote database service or environment variable.

**Interaction with #360/#361:** existing larger owner documents currently describe PostgreSQL-backed server acceptance. This task intentionally narrows to a **separately governed local MVP child**; it does not silently change, satisfy or close those wider issues. T0 must document the revised delivery sequence and add cross-links/change control before implementation, with SERVER-002 treated as an optional later remote-state vertical slice.

## Dependencies and prerequisite decisions

- **Independent pre-implementation prerequisite: ARCH-035 / #897.** Its E0–E4 behavior-equivalent cross-host composition gates must pass using STORE-002 / #670-owned storage contracts before ServerHost integration or AG-UI protocol implementation may activate. Architectural refinement may proceed beforehand, but no Server feature implementation can bypass this dependency. ARCH-035 is an independent unclaimed owner; this long-task planning record does not authorize or claim its code.
- T0 requires current main/PR/iteration inventory and an accepted ADR defining local MVP authority, persistent facts, port boundaries, AG-UI version/HTTP semantics, identity/control rules and failure semantics.
- T1 consumes #670's approved first-phase capability direction; preserve full scope and zero-change criteria, not just those needed by Server. If STORE-002 implementation is independently selected, its effective claim/iteration belongs to #670, not this coordinator.
- T2 consumes #46 session-owner contract and #614 tracked submission where needed; do not duplicate SDK ownership or treat #46 as completed by the MVP subset.
- T3 consumes #361's host architecture and completed #49 shutdown; no unaudited network/public bind.
- T4 requires independently accepted AG-UI protocol child or refinement attached to #142, and protected permission review for #52 boundary.
- T5 validates the integration and residuals. Security-sensitive changes require the independent review required by AGENTS.md.

If a prerequisite is not effective on `main`, stop the dependent implementation item and record its blocker; other explicitly independent, claimed items may continue.

## Ordered long-task items

| ID | Task / expected output | Depends on | Completion gate | Fallback | Status |
|---|---|---|---|---|---|
| T0 | Current-truth inventory, compatibility/ownership/ADR decision, exact MVP scope, child owner map and test matrix | None | Accepted ADR + effective planning/owner alignment; no duplicate claims | Keep blocked; retain architecture proposal | Planned |
| T1 | STORE-002 full taxonomy + domain/capability composition over unchanged local implementation | T0 and #670 claim | Exact before/after behavior characterization, no schema/format/query/API behavior drift, fake/unavailable fixtures | Narrow contract per domain without premature generic core; unresolved #670 stays open | Planned |
| T2 | SessionHost attachment/observer/controller, cursor/replay, command receipt/generation/fencing via canonical runtime | T0, accepted #46/#614 boundaries | Deterministic session/approval/reconnect/race/cancel/idempotency tests | Defer interaction until contract accepted, never invent per-protocol Session | Planned |
| T3 | Standalone `talos-server` process and local durable composition, startup/readiness/shutdown, loopback-only network | T1, T2, reviewed #361 child | Server process restarts with same local root; no duplicate Agent loop and no unapproved remote bind | Host-only reviewable skeleton with readiness false and no customer-ready claim | Planned |
| T4 | AG-UI versioned bounded HTTP/SSE adapter + non-UI fixture client | T2, T3, accepted protocol/security child | Text/tool/order, exact event/cursor mapping, cancellation, approval/deny and invalid principal tests; no custom chat API | Keep protocol child Blocked if AG-UI mapping cannot preserve core semantics | Planned |
| T5 | Full integration, failure matrix, documentation, exact-head CI/independent review, milestone evidence | T1–T4 | Real process/restart end-to-end, negative controls, native platform checks, security/API review and owners synchronized | Remain Review/Partial with explicit residuals, no false Complete | Planned |

**Task rule:** One item may require its own non-overlapping child Story/Iteration/claim. Intermediate code review and fixture patches remain in the same claimed Work Slice; do not open an Issue per checklist row. The coordinator itself does **not** claim or activate any of the existing owners.

## Mandatory acceptance / end-to-end failure matrix

- [ ] Existing CLI/TUI/Desktop continue to open, read, write, recover, search and archive local Sessions without changed semantics.
- [ ] Server starts with **no PostgreSQL service** and no database-vendor configuration, using an isolated local durable root.
- [ ] Server starts and serves only on loopback in the MVP; non-loopback configuration is refused until separately authorized.
- [ ] Invalid/missing credentials, unauthorized Session ID, cross-attachment command and stale controller/generation are denied without mutation or leakage.
- [ ] Two client connections share one logical Session; one controller and observers have deterministic, enforced authority.
- [ ] Snapshot/replay use authoritative ordered events; slow or disconnected clients cannot block the canonical Session actor.
- [ ] Lost acknowledgement and retry do not create duplicate submissions, outcomes, transcript records or tool side effects.
- [ ] Running/cancelled/error/successful turns have truthful, durable terminal outcomes; interrupted ambiguous effects are never automatically retried.
- [ ] Restart after committed and in-flight work preserves original history/receipts, rejects stale owners, and allows a **new explicit** turn when safe.
- [ ] Storage errors, corrupt/unreadable state, lock contention and invalid migrations fail closed without success claims or content loss.
- [ ] Real tool approval, deny and cancel use the same canonical permission pipeline as local runtime; no protocol bypass.
- [ ] No secrets/private tool payloads/hidden reasoning leak to SSE, health, logs or committed transcript beyond existing privacy policy.
- [ ] Linux/macOS native CI and Windows native tests where supported; pinned Rust toolchain, `--locked` workspace checks, Clippy, tests, docs/governance validators, and relevant conformance/negative tests pass.
- [ ] Test fixtures document what is synthetic vs live; external model/DB credentials are **not** required for required CI.
- [ ] Owner/iteration/Board/README zh-CN+EN, ADR and runbook are consistent with actual evidence. Completion references existing merged implementation SHA(s).

## Execution / safety contract

- **Branch / worktree:** repository main is truth; use a dedicated worktree for each effective child claim, short-lived `docs/` governance branch for this proposal, then `feature/<story>-...` only after claim merge. Rebase/refresh before PR, never force-push main. Existing unrelated dirty work and branches must be preserved.
- **Claim/activation:** `docs/sop/AGENT-COLLABORATION.md` and `docs/sop/START-ITERATION.md` apply. One draft governance PR with real number backfill; effective Claim+Active only after merge and CAS. This planning file does not authorize implementation. Record actual responsible actor/model/scope and authorization then.
- **Allowed now:** read-only inventory, documenting/proposing scope and architecture, draft planning PR. **Not authorized by this plan alone:** production changes, third-party service provisioning, database migration, paid model calls, non-loopback deployment, release/tag, force-push, deletion of data, or permission/security bypass.
- **Irreversible operations:** none planned. Use disposable local database/fixture roots. Production/user Session data migration and credential handling require separate explicit authorization and reviewed rollback.
- **Time / cost / resources:** no fixed background runtime or budget inferred. Local/container fixtures only; no paid provider calls. If an item exceeds its selected iteration budget, preserve truthful Review/Blocked state and propose change control.
- **Failure policy:** stop at contract violations, ambiguous writes, security failures, data-corruption risk and unavailable review. Retry transient CI/network only under bounded evidence-backed policy; never classify a flaky rerun as root-cause resolution.
- **Foreseeable default decisions:** favor existing Rust components and narrower interfaces; no generic ORM; no cross-backend SQL translation; no new service discovery; no new global event bus; no remote state as a hidden dependency.
- **Residual owner:** new unmet server capabilities return to #142/#361; remote database to #360; storage capability gaps to #670; Session attachment to #46; protocol to the accepted AG-UI child; security to #52/#141. Update each owner explicitly; do not absorb unrelated scope.

## Checkpoint / recovery journal

At each item boundary append immutable facts under this section, then update the authoritative Story and iteration **owner first**, followed by Board/derived views and originating Issue.

```text
Date / task ID:
Effective claim and owner:
Base SHA / branch / PR / exact head SHA:
Completed output and existing Completion Commit (if any):
Checks actually run and their results (including failures):
Review / CI / security gate:
Risks, deviations and residual owner:
Next item / exact prerequisite / recovery command:
```

**Initial checkpoint (2026-11-10):** Plan prepared from current repository Issues #142/#361/#360/#670/#46 and prerequisite #897 and collaboration/long-task SOP; no implementation claim, no validation evidence, no database adapter implementation, and no production server is asserted. AG-UI child/version requires T0 refinement. #38 is already Closed / Superseded; it is not a dependency for this MVP.

## Architecture refinement — domain-first ports, host-selected adapters (2026-11-10)

### Decision: extract once for every host, not once per Server

The server request exposes a broader architecture opportunity: CLI/TUI, Desktop, embedded SDK and Server must consume **the same domain/session contracts**. A `talos-server::StorageProvider` wrapper around concrete SQLite types is not the required decoupling.

**But "move data abstractions into core and instantiate SQLite in TUI" is too literal.** Repository source currently shows:
- `talos-core` is the minimal shared type/trait foundation; avoid giving it SQL/FS, migrations, connection pools, `PathBuf` persistence ownership or domain-specific repository methods.
- `talos-session::SessionStore` is a *format/path* abstraction for TLOG/JSONL, not a repository capable of fulfilling remote database Session semantics.
- `talos-session::SessionManager` currently owns local session-directory, SQLite index and file-format composition. `talos-session` itself depends on `rusqlite`; simply adding a trait does not make the **compiled dependency closure** database-free.
- `talos-cli` is the existing product composition root; `talos-tui` is a renderer. `talos-runtime` already has `RuntimeBuilder` and optional `shared-composition`. Reuse these boundaries before inventing another shared composition crate.

### Target layering and placement rule

```text
PRODUCT HOSTS (composition roots)
  talos-cli (drives TUI) | talos-desktop | talos-server | embedded host
       │ select profile + inject capabilities, no domain rule implementations
       ▼
talos-runtime (stable application/SDK facade)
       │ composes shared Session/Agent/services; no vendor-specific API
       ▼
DOMAIN-OWNED PORTS + SEMANTICS
  talos-session: Session repository, durable Turn settlement,
                 submission custody and generation/lease invariants
  talos-memory / evolution / exploration: own their domain persistence
  storage capability taxonomy: Log/Relational/Object/Search/Vector/etc.
       ▲ concrete adapters implement ports
       │
LOCAL IMPLEMENTATION (existing SQLite/TLOG/JSONL) | FUTURE REMOTE IMPLEMENTATION
```

Place a trait where its **domain semantics are owned** (normally in the corresponding domain crate, or a small neutral contract module if actual dependency direction requires it), not automatically in `talos-core` and never exclusively in Server/TUI. Shared cross-domain capability identifiers/types may live in the minimal foundation, but `SessionRepository` is not a generic `RelationalStorage`.

Do not turn each capability taxonomy entry into a dynamic trait without a consumer. Stage #670 taxonomy and explicit availability first; extract behaviorally meaningful ports at real call sites. No `dyn` object, factory crate or new dependency is justified merely by conceptual symmetry. The public SDK remains additive/source compatible through a default local factory and opt-in host injection.

### Two different kinds of decoupling (do not conflate)

1. **Contract decoupling, MVP requirement:** Domain APIs, runtime construction and protocol DTOs must contain no PostgreSQL, SQLite SQL syntax, SQL transaction handles or vendor-specific error types. Local adapter reuses current storage implementation; no behavior changes.
2. **Physical crate/dependency decoupling, separately accepted step:** Today `talos-runtime -> talos-session -> rusqlite`. If a future backend profile must produce a binary **without compiling/bundling SQLite**, separate local adapter code from the domain contract into a physically independent crate/module boundary in a reviewed additive migration. Trait injection alone does **not** meet that requirement. No premature crate split in the first no-behavior-change phase.

### Repository contracts should preserve behavior, not mimic an RDBMS

- A Session operation is not a CRUD aggregate: turn commit, first-terminal-outcome-wins, custody/receipts, idempotency, recovery and generation fencing have explicit invariants.
- Preserve current local multi-resource ordering between SQLite and TLOG; no false claim of one transaction spanning both. Domain contract specifies externally observed result/recovery, not the native transaction method.
- Keep storage capability selection distinct from **runtime capability selection** (file, shell, network, S3 tools) and from Session ownership. Do not inject a generic `Provider` because Talos also has LLM/provider terminology.
- Separate session authority data from rebuildable FTS/search indexes and from semantic/vector computations. One vendor need not implement every capability.
- Profile default: `Local` reuses current production code; test profile supplies fake/controlled failures; remote profile is **future** and separately validated. Database-agnostic API does not by itself prove backend portability: require a second implementation plus a common conformance suite before claiming it.

### Implications for task order

- **T0:** register ARCH-035 / #897 as the Server prerequisite, and include a crate dependency/call-site matrix; record existing local factory and UI vs host ownership, pin semantics to protect, and decide exact trait home(s). Reconcile this with SERVER-001, STORE-002, SERVER-002 and SESSION-009 without reopening their independent claimed scopes.
- **T1:** classify all #670 capabilities and extract the minimum domain seams *across existing consumers* with exact behavioral characterization. Preserve current `RuntimeBuilder::durable_session` and all current CLI/TUI/Desktop paths. Do not move storage construction into `talos-tui`.
- **T2/T3:** shared SessionHost and Runtime receive capability bundles through one accepted composition contract; CLI continues to choose current local default, Server chooses the same local default for MVP. Avoid separate Server `SessionManager` and separate Agent loop.
- **T4:** protocol adapter only authenticates/translates/projections; it cannot directly construct SQLite or issue persistence writes.
- **T5:** verify an unchanged CLI/SDK path and Server path using the **same domain contracts**, plus absence of vendor types from public API. Treat optional SQLite-free binary packaging as a separate, explicitly owned acceptance.

### Architectural optimization threshold

Do **not** use this MVP to rewrite all Session, Memory, Evolution and Exploration concrete backends, flatten the domain modules into one storage layer, create a universal ORM, or move business logic into a new `talos-core` mega-crate. If a second backend later reveals a genuine missing port, revise that domain's contract with a controlled migration and preserve prior tests. **The optimization is dependency direction and ownership, not maximizing the number of interfaces.**

## Behavior-preserving architecture extraction protocol — equivalent transformation before expansion

**Binding planning principle:** Architecture optimization uses **equivalent transformations first, functional extensions second**. Do not combine storage contract extraction, changed persistence behavior and new backend admission in one implementation Work Slice. Every equivalent transformation is reviewable, reversible and backed by characterization evidence. A new backend is never a prerequisite for validating the initial abstraction.

### Equivalence boundary

The "same behavior" contract is stronger than API compatibility. For the existing local profile, preserve:
- all public and in-tree input/output semantics; session IDs and operation IDs; source-compatible supported SDK entry points;
- SQLite schema, SQL, migration, WAL/locking/retry assumptions, index and FTS5 behavior;
- TLOG/JSONL encoding, paths, history/archival/fork/recovery behavior, event order and transcript redaction;
- turn finalization's atomic/idempotent observable semantics, first terminal outcome wins, pending custody, reconciliation and generation fencing;
- errors (including closed-failure branches), permission decisions, cancellation and shutdown behavior;
- CLI/TUI/Desktop startup/default storage selection and identical runtime results.

Changes to execution latency or internal allocation are not in general byte-for-byte equal: compare bounded concurrency/failure/ordering contracts and collect baseline evidence. Do not claim mathematical equivalence from a passing happy-path test.

### Ordered transformation gates (T1 sub-checkpoints)

1. **E0 — Preserve baseline:** inventory actual dependencies and write paths in `talos-cli`, `talos-runtime`, `talos-session`, Memory, Evolution and Exploration. Capture deterministic golden/characterization tests including failure injection; identify the authoritative owner and data invariants for each call.
2. **E1 — Make boundaries explicit without rerouting:** name and document every #670 capability and the semantics required by its current consumer. Keep actual implementations, call sites, constructors and dispatch unchanged. Unavailable vector/embedding/rerank explicitly report unavailability rather than returning imaginary functionality.
3. **E2 — Extract domain ports by *semantic forwarding*:** introduce only the smallest traits/typed façades needed at real existing call sites; first implementation delegates directly to the old concrete method and preserves its errors/ordering. Do not translate to generalized SQL or reorganize transactional code at the same time.
4. **E3 — Converge local composition:** move default adapter **selection** to a shared host-composition entry point where justified; keep the original `talos-cli` / `talos-runtime` constructors source compatible and route them to the exact same local factory. TUI stays a renderer, not a SQLite wiring owner. If a public API migration is unavoidable, stop and seek an ADR plus compatibility owner.
5. **E4 — Retire redundant glue only after proof:** show that no business-rule implementation was duplicated, and all currently enabled local workflows invoke the canonical seam. Do not remove the old backend, change the storage layout or claim SQLite-free binary packaging. Record before/after API/test/failure/performance evidence; close #670 only if **its full** stated acceptance passed.
6. **X1 — Extend after E4 is accepted:** a separately owned implementation may provide a genuinely new adapter, storage profile or Server behavior. Reuse the E-series conformance matrix; add backend-specific transaction/ownership failure checks. New functionality is a new implementation iteration, not a "refactor follow-up" hidden inside E2/E3.

### Two protected invariants

- **Domain semantics before technology:** a Session's durable settlement and custody cannot be reduced to CRUD or generic relational/SQL operations; semantic repositories remain owned by their corresponding domains. Generic capability names do not imply that an arbitrary backend supports the same atomicity.
- **Implementation vs dependency separation:** `talos-runtime -> talos-session -> rusqlite` is a current dependency fact. Injecting a trait preserves source-level decoupling but does **not** magically remove the SQLite dependency from a Server binary. Actual physical crate splitting is optional, must be justified by a concrete requirement and requires its own equivalence gate.

### Review rule

Each E-series PR must state: exact unchanged behavior, original concrete call path, new indirection path, characterized negative cases, rollback path, validated target SHA and excluded future expansion. A passing test suite is evidence, not proof of all semantic equivalences. If any behavior must change, split it into an independently authorized X-series change. If equivalence cannot be established, stop the current migration at the previous accepted commit and record the uncovered invariant.

## Closure rule

Mark `SERVER-MVP-001` Complete only when T0–T5 gates pass, all required/manual/security checks have evidence, docs and owner claims are consistent with merged commits, and unresolved optional/future work has an explicit residual destination. **Never close #142/#360/#361/#46/#670 merely because the narrower MVP runs.**
