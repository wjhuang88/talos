# Iteration I334: ARCH-035 E3a — CLI Actor And Embedded SDK Host Composition Parity

> Document status: Active / Review — effective claim #995; bounded E3a implementation evidence #996, #998, #1010 and #1029 accepted; full E3a conformance remains incomplete
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
| Implementation PR | #996 merged source `51b6e548e26e98444459ce93340149aa7bfc4578`; #998 merged source `ce6c56d2290d11c058ec33b9876d546ac9dce056`; #1010 merged source `d1b565e75634887ad5af7f332784bc92a0bba406`; #1029 merged `3393ca2994a63613d81906743ec87a5e0cab09aa`; Memory first-query candidate pending CI/review |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | I334 E3a remains Active / Review until default-vs-custom startup, permission/cancellation/receipt/fault parity and independently reviewed exact-head tests are accepted; E3b, E4, STORE-002 F, ServerHost and #897 closure separately gated |

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

The atomic E3a Claim [#995](https://github.com/wjhuang88/talos/pull/995) **merged** to main as `08f2a41c5f83ad7a87e310370df83d140c39e889` on 2026-11-20, activating the bounded I334 work slice; the original proposed-status wording above is historical, not current activation status.

- [#996](https://github.com/wjhuang88/talos/pull/996): accepted **partial E3a** CLI actor/Todo lazy-registration and SDK public integration fixtures; main source `51b6e548e26e98444459ce93340149aa7bfc4578`. Native CI #4496 had 8/8 applicable jobs successful and independent read-only code review. This preserves production CLI/Session actor constructor, permission and cancellation ownership.
- [#998](https://github.com/wjhuang88/talos/pull/998): accepted **partial E3a** independent Cargo-root SDK fixture with explicit `local-storage` and one direct Talos dependency, authentic Cargo v4 lock with optional Memory/Evolution/Exploration closure, feature-off `default`/`coding` and opt-in `local-storage` `--locked` coverage, typed invalid-ID/no-create, lazy optional stores and durable TLOG reopen. Main source **`ce6c56d2290d11c058ec33b9876d546ac9dce056`**; exact-head [CI #4561](https://github.com/wjhuang88/talos/actions/runs/45144165793) **8/8 SUCCESS**; [independent code-only APPROVE](https://github.com/wjhuang88/talos/pull/998#issuecomment-7096365502) and [single-maintainer CAS](https://github.com/wjhuang88/talos/pull/998#issuecomment-7097798160). Final source diff comprises only five SDK fixture/validator paths, no production Host/Agent/Session or CI workflow behavior.
- **Accepted startup evidence:** [#1010](https://github.com/wjhuang88/talos/pull/1010) merged `d1b565e75634887ad5af7f332784bc92a0bba406`; [CI #4641](https://github.com/wjhuang88/talos/actions/runs/45424783612) 8/8 SUCCESS, [independent APPROVE](https://github.com/wjhuang88/talos/pull/1010#issuecomment-7135994551). `i334_default_custom_startup.rs` proves isolated default vs custom index reconciliation for stale and missing rows.
- **Accepted CLI selection evidence:** [#1029](https://github.com/wjhuang88/talos/pull/1029) merged `3393ca2994a63613d81906743ec87a5e0cab09aa`; [CI #4657](https://github.com/wjhuang88/talos/actions/runs/45653152919) attempt 2 8/8 SUCCESS, [independent APPROVE](https://github.com/wjhuang88/talos/pull/1029#issuecomment-7168178118). Both selector fixtures passed on Linux, macOS and Windows. Attempt 1 Git discovery failure remains historical evidence, not a proven root-cause fix.
- **Current work:** Candidate: `e3a_cli_memory_registration_defers_open_until_first_query` uses the actual CLI callback registration and Agent request preview with an isolated SQLite path. Extracts only a private path-binding helper; default home resolution, enabled guard, existing-file check, store mutex/cache, failure handling and prompt formatting unchanged. Tests missing store no-create, existing empty file untouched at registration, migration on first query, and persisted memory retrieval after initialization. No ambient home mutation. Exact-head native CI/review pending; default-home lookup and cache reuse count not directly measured. Nonempty interactive selector, combined default-host selection, permission/fencing, receipt/cancel and fault evidence remain open.

## Completion Evidence

**I334 E3a is not Complete.** Accepted #996, #998, #1010 and #1029 source commits above are partial implementation evidence, not an I334 Completion Commit or owner-first closeout. I330 E2 Completion Commit `21589346d99263c5675ad0b648df684801d2e861` is a prerequisite only. A final E3a Completion Commit may be assigned only from already-existing accepted source after all E3a acceptance tests, review and host boundaries have been verified.

## Variance And Residuals

- **Remaining E3a acceptance:** real default-host selection and remaining selector cases, Memory first-query laziness, permission denial and identity fencing, receipt settlement, interrupt/cancellation ordering and bounded fault scenarios. No waiver from prior Agent/Session test failures.
- **Separately governed:** E3b Desktop Create/Existing/unbound source migration, E4 broad cross-host equivalence, STORE-002 F/#670 crash/fault provider conformance and ServerHost/#361. Parent [#897](https://github.com/wjhuang88/talos/issues/897) **stays OPEN** after #998; automatic closure caused by the partial fixture PR was corrected via explicit reopen.
