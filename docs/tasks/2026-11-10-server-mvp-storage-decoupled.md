# SERVER-MVP-001 — Single-Node, Storage-Decoupled Talos Server

> Planning baseline (2026-11-10). **Status: Planning / Unclaimed.** This file is a proposed long-task coordinator, not an effective Collaboration Claim, implementation authorization, or evidence of a shipped server.
>
> Parent: [SERVER-001 / #142](https://github.com/wjhuang88/talos/issues/142). Host: [SERVER-001-C / #361](https://github.com/wjhuang88/talos/issues/361). Storage abstraction: [STORE-002 / #670](https://github.com/wjhuang88/talos/issues/670). Session: [SESSION-009 / #46](https://github.com/wjhuang88/talos/issues/46). Future remote persistence: [SERVER-002 / #360](https://github.com/wjhuang88/talos/issues/360).
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

**Initial checkpoint (2026-11-10):** Plan prepared from current repository Issues #142/#361/#360/#670/#46 and collaboration/long-task SOP; no implementation claim, no validation evidence, no database adapter implementation, and no production server is asserted. AG-UI child/version requires T0 refinement. #38 is already Closed / Superseded; it is not a dependency for this MVP.

## Closure rule

Mark `SERVER-MVP-001` Complete only when T0–T5 gates pass, all required/manual/security checks have evidence, docs and owner claims are consistent with merged commits, and unresolved optional/future work has an explicit residual destination. **Never close #142/#360/#361/#46/#670 merely because the narrower MVP runs.**
