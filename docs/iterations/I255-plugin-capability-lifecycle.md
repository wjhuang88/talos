# Iteration I255: Plugin Capability Lifecycle

> Document status: Complete / Closed
> Published plan date: 2026-09-10
> Objective: Connect installed Plugin declarations and lifecycle to the shared Capability registry.
> MVP deliverable: An explicitly loaded local WASM Plugin declares capabilities, activates a Provider, executes through the existing permission pipeline, and becomes unavailable after stop.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Installed Plugin capability declarations, carrier adapter lifecycle, registry availability and existing explicit CLI Plugin integration. |
| Claimed At | 2026-09-10 |
| Source Issue | #513 (parent #466) |
| Governance Claim PR | #527 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim #527 effective at 07066e76e92c30d63a2a9068dfdd5e0737d3f1a1; head eb26aaef, CI 34449342743, independent Agent-role API/security plan review 5615007238 and CAS 5615007433. Shared account does not prove natural-person separation. Implementation #528 head e1b9997a passed CI 34471207754 and independent API/security APPROVE 5617962523; CAS 5618114017 preceded merge 9404c338. |
| Implementation PR | #528 |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | #528 merged with exact-head CI, independent API/security approval and CAS; acceptance evidence below closes this slice only. Other CAP-001 children remain independently governed. |

## Required Reads

- [CAP-001-C](../backlog/active/CAP-001-C-plugin-capability-carriers.md), [CAP-001](../backlog/active/CAP-001-progressive-capability-provider-architecture.md).
- [ADR-072](../decisions/072-capability-provider-bundle-boundary.md), [ADR-027](../decisions/027-plugin-runtime-boundary.md), [ADR-032](../decisions/032-wasmtime-dependency-security-review.md).
- [PLUGIN-001](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md) completed read-only WASM boundary.
- [I254](I254-capability-registry-resolver.md) registry API and cooperative cancellation limitations.

## Published Baseline

### Selected Story

| Story | Status at selection | Dependencies | Deliverable |
|---|---|---|---|
| CAP-001-C / #513 | Refinement / Unclaimed | I254 Complete; ADR-027/032/072 Accepted | Plugin declaration-to-activation-to-stop integration with conformance and real CLI evidence. |

### Scope And Compatibility Contract

- Parse explicit versioned capability declarations without executing code during discovery.
  Keep the existing public `PluginManifest`/`PluginMetadata` fields, constructor compatibility,
  legacy manifest spellings and legacy package acceptance. An additive declaration envelope or
  companion parser may reuse the legacy parser; do not add required public struct fields or rename
  persisted fields. No manifest rewriting is performed. Invalid explicit declarations fail closed.
- Define lifecycle transitions Unloaded -> Loaded -> Initialized -> Active -> Stopped and map
  Provider availability to actual activation, never mere installation or parsing. Initialization
  must validate executable bindings; a claimed capability without an adapter implementation is
  not available. Existing tool declarations have an explicit legacy mapping.
- Use one core CapabilityRegistry and typed Provider descriptors. Add the smallest additive
  withdrawal/ownership mechanism needed by stop/failure; one Plugin must not remove or replace
  another Plugin's provider. Partial activation failure leaves no stale available providers.
- Keep Carrier kinds distinct. The executable implementation uses the existing accepted WASM
  carrier; built-in/MCP/helper/remote declarations cannot silently execute as WASM or become Ready
  without an authorized adapter. Unsupported carriers return typed unavailability. This does not
  claim implementation of all carriers or authorize a new carrier security policy.
- Preserve existing WASM confinement, fuel, wall timeout, no ambient host calls, read-only tool
  facets, provenance, collision handling and progressive disclosure. Registration alone never
  grants execution or prompt exposure. Existing CLI print/TUI explicit Plugin composition must
  consume the same lifecycle implementation, not maintain a second registry algorithm.
- Stopped handles cannot execute stale tools. Admission/stop race behavior must be explicit and
  tested; already executing work retains existing bounded WASM cancellation/timeout guarantees.
  Failure, cancellation and cleanup must preserve unrelated providers and tools.
- Retain library APIs for embedders without claiming runtime-default Plugin discovery/loading.
  Keep existing default workspace members/features and dependency versions unchanged.

### Non-Goals

No Bundle schema rename/installer, automatic discovery/download, language/browser implementation,
new executable carrier, write-capable WASM tool, host-call expansion, permission-policy rewrite,
Dashboard/Desktop, release/version/tag/publication or reopening completed PLUGIN-001.
Any required breaking API/schema change or departure from accepted sandbox policy stops this
slice for explicit decision control; it cannot be hidden as an implementation detail.

### Acceptance

1. Given a legacy local read-only package, existing CLI/plugin fixtures retain their names,
   provenance, permission outcomes and output; public legacy constructors still compile.
2. Given explicit valid capability declarations, parsing/initialization alone does not resolve
   a provider; activation makes the implemented capabilities available through I254's registry.
3. Given malformed/incompatible declarations, missing binding or unsupported carrier, no
   executable activation or provider availability is produced; the host remains healthy.
4. Given stop, activation failure, timeout or trap, availability/cleanup follows the declared
   lifecycle, stale calls fail closed, and unrelated providers remain unchanged.
5. Given duplicate identities or partial registration failure, no cross-owner overwrite/removal
   or partial availability survives; repeated cleanup is safe.
6. Given denied tool permission or hidden tool family, capability registration never bypasses
   permission/disclosure; approved read-only invocation still works through the real CLI.
7. A checked-in fixture and real CLI test prove explicit load -> activation -> tool execution;
   public lifecycle integration tests prove stop -> unavailable and stale-call denial.
8. No optional network/startup dependency or default build expansion is introduced; updated
   user/API docs distinguish implemented lifecycle from future carriers and installation.

### Validation And Documentation

- Focused core registry, Plugin WASM, CLI plugin, permission/provenance/disclosure regressions.
- Adversarial fixtures for path escape, invalid module, trap, fuel/timeout, declaration mismatch,
  duplicate ownership, stop/execute race, stale handles and rollback after partial failure.
- Full locked `./scripts/release_preflight.sh` with the pinned toolchain before stable push;
  explicit `COLLABORATION_VALIDATION_BASE=origin/main` for both governance validators.
- Independent exact-head API/security review, fresh Linux/Windows CI and merge-time CAS.
- Update public rustdoc, README Plugin usage, `docs/reference/ARCHITECTURE.md`, CAP-001-C/I255
  owners, then parent/Board/backlog/iteration index/manifest and #513 status.
- Local fixes stay in I255/#513; no new Issue or PR for intermediate tests or review findings.

### Risks And Rollback

Primary risks are available-but-unimplemented descriptors, stale execution after stop, ownership
collision and permission/disclosure bypass. Preserve the old public entrypoints through adapters;
rollback removes the new integration and restores explicit legacy loading without rewriting files.
No new dependency or carrier decision is assumed.

## Selection Inventory (2026-09-10)

Target `main@cefb8320ffd3fc53d7c333d19f08346c4e1c5ec0`; fetched before preparation.

| Owner | State | Disposition |
|---|---|---|
| Active / Review iterations | None | I254 closeout #526 is on main; #512 closed afterward. |
| I249 | Planned / Unclaimed | Dependency pilot remains unselected; not part of #513. |
| I164 | Paused, superseded | Retain history; do not resume. |
| Blocked iterations | None | Unselected backlog gates remain independent. |
| I162 | Historical terminal record | Owner is Complete with a recorded Review outcome, not an active Review iteration. |
| I255 / CAP-001-C | Planned / Unclaimed preparation | No code authority until atomic claim merge. |
| Remaining #466 children; #520 | Unclaimed | No authority transfer. |

One clean worktree, no stash, no open PR at selection. No implementation branch is created.
Existing merged branches are historical pointers, not active claimants. Inventory uses current
owner headers; historical timeline status words and template rows do not activate work.

## Completion Evidence

Completion Commit: e1b9997abc57a17a2f536c97f063830b569dc4c0
Implementation PR #528 merged as `9404c3383a3ac076af086a5f6e0bd72fae5e27b8`.
This pre-existing implementation commit, not this status change, supplies completion evidence.

## Resume

Implementation is complete. Finish review/merge of this evidence-only closeout and close #513;
then select the next separately governed CAP-001 child. Do not resume I255 implementation or
infer completion of #466. Dated checkpoints below retain their original historical states.

## Atomic Claim Proposal (2026-09-10)

PR #527 proposes Active / Claimed for I255 and CAP-001-C together. The selection table above
preserves the earlier Unclaimed inventory; only merge establishes the new authority.
Implementation starts from that merge or later main, in a new implementation branch. Core registry
withdrawal, Plugin lifecycle and CLI integration are within this slice; permission policy,
Dashboard and other carriers are excluded. Published Baseline is unchanged from `2f8c0657`.
Both governance validators passed for draft preparation (0 warnings); final proposal checks and
independent API/security plan review remain required. No implementation or completion is claimed.

## Effective Activation (2026-09-10)

Claim #527 merged at `07066e76e92c30d63a2a9068dfdd5e0737d3f1a1`, from head
`eb26aaef1ea0224d3c1a263bc6cfb8625f23c488` and base
`cefb8320ffd3fc53d7c333d19f08346c4e1c5ec0`. Exact-head CI `34449342743` passed;
independent API/security plan approval `5615007238` and merge-time CAS `5615007433` are
recorded on #527. This supersedes proposal-only current statements, preserving dated records
and Published Baseline. Branch `feat/i255-plugin-capability-lifecycle` starts at that merge.
Implementation now converges locally; no implementation PR exists and Completion Commit is pending.

## Local Implementation Checkpoint (2026-09-10)

The candidate implements owned registry publication/withdrawal, optional version-1 declarations,
legacy mapping, static WASM binding validation, and the shared load/initialize/activate/stop
controller consumed by explicit CLI print/TUI composition. Original manifest constructors,
provenance, tool names, dependency versions and default feature/member sets are retained.
The legacy file loader rejects explicit new declarations with migration guidance rather than
silently ignoring them; typed legacy build/register adapters keep their prior contract.

Stop and invocation admission linearize under one mutex. Already-admitted calls may finish with
existing WASM bounds; new/stale calls fail closed. Trap, timeout, cancelled invocation and final
handle drop withdraw only the owned lease. Stopped host tool entries are retained but cannot
execute; host registry rebuild is the presentation cleanup boundary. No new carrier is enabled.

Local independent Agent-role API/security pre-review found missing unwind containment around
initialization. It was repaired before submission and covered by injected panic testing. This
pre-review is not final exact-head approval and does not claim natural-person separation.

Focused core ownership tests passed (3); Plugin tests passed including lifecycle, trap/fuel,
permission, stale ownership and no-start-on-initialize checks. Two real CLI binary tests use a
bounded local model endpoint: explicit package execution returns 7 through the real permission
pipeline; the same package outside workspace is denied. Runtime-default disclosure remains off.
The first workspace preflight failed on the new HTTP test fixture: macOS accepted sockets could
inherit nonblocking mode and read returned WouldBlock. Explicit blocking mode with bounded read
timeout repaired it; both binary paths then passed. Final full preflight is pending below.

Changed-file inventory (all I255):

- Core: `crates/talos-core/src/capability.rs`, `crates/talos-core/tests/i255_provider_ownership.rs`.
- Plugin: `crates/talos-plugin/src/{lib,lifecycle,wasm}.rs`,
  `crates/talos-plugin/tests/i255_lifecycle.rs`, and the two `capability-demo` fixture files.
- CLI: `crates/talos-cli/src/registry.rs`, `crates/talos-cli/tests/i255_plugin_e2e.rs`.
- User/API docs: `README.md`, `docs/reference/ARCHITECTURE.md`.
- Owners/derived views: this owner, CAP-001-C, CAP-001 parent, Board, PRODUCT-BACKLOG,
  iterations README, governance manifest, and the Issue/owner matrix.

No Cargo/lock/default build change, Dashboard, permission policy, release or publication edit.
Resume: finish full local checks, bind the actual implementation PR, obtain fresh exact-head
CI/API/security review, CAS merge, then owner-first closeout using pre-existing implementation SHA.
Completion Commit remains pending; #513 remains open and #466 is not complete.

### Stable Local Validation

`env COLLABORATION_VALIDATION_BASE=origin/main CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 ./scripts/release_preflight.sh` passed
after the socket fixture correction: pinned Rust 1.97.0, locked workspace check/Clippy/tests,
format, both governance validators (0 warnings), text boundary and classifier checks.
The final owner/derived Review synchronization is documentation-only and is checked again before
commit; no implementation completion or remote exact-head validation is inferred from this run.

## Acceptance Closeout (2026-09-10)

Implementation head `e1b9997abc57a17a2f536c97f063830b569dc4c0`, reviewed base
`07066e76e92c30d63a2a9068dfdd5e0737d3f1a1`, reached main through #528 merge
`9404c3383a3ac076af086a5f6e0bd72fae5e27b8`. CI `34471207754` attempt 2 has five
successful jobs, including macOS full locked preflight and Windows workspace/CLI smoke.
Only the Issue reconciliation job was rerun after status comment `5617924067`; original
successful Rust job timestamps were retained. Independent API/security APPROVE `5617962523`
and merge-time CAS `5618114017` bind that head/base. The reviewer independently ran lifecycle
integration 6/6, ownership 3/3 and lifecycle unit tests 3/3; shared GitHub identity establishes
Agent-role separation only, not natural-person separation.

| Published acceptance | Evidence and result |
|---|---|
| 1. Legacy compatibility | `legacy_non_semver_and_names_preserved_without_public_struct_changes`, existing WASM package tests and legacy file-loader rejection test passed; constructors, names and permission facets retained. |
| 2. Availability tracks activation | `activate_execute_stop_rejects_stale_handles_and_allows_new_owner` asserts unavailable after load/initialize and available after activation. |
| 3. Invalid declaration/binding/carrier | `malformed_explicit_declarations_and_unsupported_carriers_fail_closed`, `partial_initialization_and_path_escape_never_publish_tools` and initialization binding tests passed; panic containment injection passed. |
| 4. Stop/failure and stale calls | Lifecycle trap/fuel test withdraws availability; existing `timeout_handled` exercises bounded WASM timeout/fuel failure without distinguishing which limit fired. Unit tests `admitted_before_stop_can_finish_but_later_calls_are_denied` and `cancelled_invocation_closes_future_admission` cover admission/cancellation. |
| 5. Ownership/atomicity | Three `i255_provider_ownership` tests cover invalid/colliding batches, replacement/other owners and registry snapshots; lifecycle duplicate activation and repeated stop passed. |
| 6. Permission/disclosure | Lifecycle test checks hidden/default versus explicit presentation; real CLI `capability_registration_does_not_bypass_outside_workspace_permission` denies the request. |
| 7. Real execution and stop | Real CLI `explicit_capability_package_executes_through_print_permission_pipeline` returns fixture result 7; public lifecycle test proves stop and stale-call denial. |
| 8. Defaults/documentation | No Cargo/Cargo.lock or default feature/member edits. README, architecture and public lifecycle rustdoc describe implemented WASM scope, not future carriers. |

All eight acceptance rows are satisfied within the published slice. The final diff remains the
20-file inventory recorded above; no Dashboard, release, permission policy or carrier expansion.
Stopped host tool entries may remain displayed until host registry rebuild but cannot execute;
already-admitted calls retain the existing bounded WASM execution policy. These are the published
compatibility semantics, not a new unload/cancellation guarantee. Bundle installation remains
BUNDLE-001/DIST-001-A, language integration LANG-001/002/003, on-demand resolution DIST-001-B,
and Browser integration BROWSER-001. No downstream activation is included in this closeout.
