# Iteration I334: ARCH-035 E3a — CLI Actor And Embedded SDK Host Composition Parity

> Document status: Active / Claimed proposed by atomic governance PR #995 — ineffective until reviewed target-main merge
> Published plan date: 2026-11-20
> Parent: [ARCH-035](../backlog/active/ARCH-035-behavior-preserving-host-composition.md) / [Issue #897](https://github.com/wjhuang88/talos/issues/897)
> Objective: verify and minimally converge the actually supported CLI/TUI canonical Session actor path and embedded SDK RuntimeBuilder opt-in local binding on accepted STORE-002 E + ARCH-035 E2 concrete delegates without changing host defaults, startup/recovery, Session/permission authority, SDK source compatibility or storage paths/formats
> MVP: deterministic runnable CLI + embedded SDK conformance and justified no-op/delegate/migration disposition for those host paths; explicit Desktop Create/Existing inventory for a later separately claimed E3b
> Baseline rule: preserve this bounded objective; E3b, E4 and other scopes need separate owner decisions

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
| Implementation PR | Not started (implementation forbidden before #995 main merge) |
| Last Updated | 2026-11-20 |
| Handoff / Release Condition | E3a only effective after reviewed #995 claim+activation merge; E3b, E4, STORE-002 F, ServerHost and #897 closure separately gated |

## Published Baseline

### Accepted prerequisites and source truth

- I320 E0/E1 architecture inventory and accepted placement-only ADR-104 (#936): CLI/TUI creates `AppServerSession::new` then `actor.set_persistence`; Desktop/embedders use `RuntimeBuilder`; `talos-tui` is only presentation. Do not fake a single existing Host runtime constructor.
- STORE-002 E #956 accepted opt-in `LocalStorageProfile`; E2 #979 accepted additive `#[cfg(feature = "local-storage")]` `RuntimeBuilder::durable_session_from_custom_local_profile` calling existing `custom_session_manager().create_or_open_session` and `durable_session`, preserving old unbound/default SDK API.
- `SessionManager::new` performs default startup reconciliation that `with_dir` does not; CLI session selection and `AppServerSession::set_persistence` remain authoritative. CLI `maybe_set_memory_provider` opens only on first query if DB exists and caches; Todo prompt callback opens on demand. No eager optional DB.
- Desktop `DurableOpenMode::Create` vs `Existing` (open-or-create vs open-existing), transcript restore, hooks, session Todo and unbound mode cannot be replaced by the E2 custom-profile path without a separate exact-parity proof.
- Concurrent claimed/Review/Blocked owners remain unaffected: independent I332 Bedrock, I333 Search T4; I318 TUI-054 native keyboard, I302 Windows native lifetime, I277 Desktop device acceptance, session recovery emergency and Provider/Search owners retain their own scopes. STORE-002 F remains open/unclaimed; ServerHost #361 blocked. E2 I330 is Complete/Closed, not an E3a implementation.

### Selected bounded Work Slice

1. Refresh exact current CLI, TUI-renderer, embedded SDK and Desktop construction paths on the E3a implementation base; publish a host-by-host `retain/no-op`, `delegate` or `separately gated` decision with source references and before/after observables. A no-op disposition is valid where the existing Host already delegates to the same concrete domain store and any migration would alter authority.
2. Add deterministic runtime and CLI integration fixtures proving local Session identity/path, default-versus-custom startup index reconciliation, legacy unbound and explicit durable builder compatibility, storage error/no-create behavior, permission/cancellation/settlement ordering, Memory first-query laziness and Todo callback timing against unchanged current CLI actor authority. Use accepted domain constructors; do not introduce a fake second Host Session/Agent engine.
3. Where source-grounded equivalence supports it, allow only additive, explicit opt-in selection/conformance at the runtime-facing composition boundary; do not make E2 custom manager the default CLI manager or treat its `with_dir` behavior as equivalent to `new()`. An already equivalent CLI call site may be retained unchanged with an executable regression proof.
4. Exercise supported external SDK source-compatibility fixture with both feature-off legacy builder and feature-on opt-in path. Inventory Desktop Create/Existing/unbound and restore semantics, but defer Desktop source migration and all device/native acceptance to independently governed E3b.
5. Record any observed gaps honestly as E3b or E4 residuals; the E3a closeout cannot mark full E3, E4, parent #897, #670 F or ServerHost complete.

### Explicit Non-Goals

- No mass CLI/TUI/Desktop/SDK rewrite, TUI persistence factory, ServerHost/protocol, remote providers, SQL/schema/TLOG/JSONL/FTS5/index migration, new storage capability taxonomy, generic CRUD/ORM, compiled SQLite-free binary.
- No default or permission/sandbox/approval/lock/lease/recovery/actor policy change, hidden eager Memory/Todo/Exploration open, cross-resource ACID claim or change to protected Session custody/settlement behavior.
- E3b Desktop call-site migration and broader cross-host native parity; E4 final equivalence/cancellation/privacy/recovery matrix and #897 closure; STORE-002 F crash/fault parity.

### Acceptance And Validation Plan

- Source-grounded matrix names each CLI/TUI/SDK/Desktop constructor, default profile, storage root, error identity, OpenMode and preserved actor/permission owner, plus exact no-op/migrate decisions.
- Focused deterministic tests demonstrate default vs custom startup behavior, same Session ID/TLOG/index binding for selected and direct path, typed invalid ID without directory creation, no eager Memory/Todo/Exploration, and unchanged CLI actor ordering/receipt/interrupt semantics.
- External SDK fixture compiles with `RuntimeBuilder::new()`, `durable_session(DurableSession)` and unbound mode without local-storage; explicit opt-in feature path compiles and runs separately; no public breaking signature.
- Required local stage: `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo check/test -p talos-runtime --features local-storage --locked`, focused CLI tests/external SDK fixtures, `scripts/validate_project_governance.sh .`, `scripts/validate_collaboration_claims.sh .`, exact-head macOS/Windows CI and applicable independent public API/host integration review.
- No completion without pre-existing implementation/evidence commit SHA and later owner-first status closeout. Final merge CAS must check exact head, no overlapping claim, native CI and review. Parent #897 remains Open after E3a.

### Risk, Rollback And Handoff

- Risk: invalidly substituting E2's custom `with_dir` for CLI default reconciliation or Desktop Existing/open-only; eagerly opening lazy stores; changing Session ownership/cancellation/permission; expanding scope into another active owner.
- Rollback: remove E3a additive opt-in/tests while retaining accepted E2 and all original Host constructors. Any semantic change needs a different reviewed claim.
- Handoff: do not start E3a implementation until this iteration's finalized atomic claim+activation is independently reviewed, passes docs CI/both validators and is merged to main.

## Actual Activation And Execution

Finalized governance **proposal on Draft PR #995**: the ARCH-035 source owner and I334 together propose E3a `Claimed`/`Active`. Neither is effective before exact-head docs CI, both validators, independent Host/SDK/non-overlap review, single-maintainer CAS and target-main merge. No E3a implementation, API, tests, production Host change or Completion Commit in #995.

## Completion Evidence

Not started. The already accepted I330 E2 Completion Commit `21589346d99263c5675ad0b648df684801d2e861` is a **prerequisite**, not this E3a Completion Commit. No E3a implementation or tests claimed.

## Variance And Residuals

E3b Desktop and E4 broad conformance are not activated; #897 remains Open. #670 F and ServerHost remain separately governed.
