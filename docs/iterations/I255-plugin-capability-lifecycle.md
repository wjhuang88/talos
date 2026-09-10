# Iteration I255: Plugin Capability Lifecycle

> Document status: Planned / Unclaimed
> Published plan date: 2026-09-10
> Objective: Connect installed Plugin declarations and lifecycle to the shared Capability registry.
> MVP deliverable: An explicitly loaded local WASM Plugin declares capabilities, activates a Provider, executes through the existing permission pipeline, and becomes unavailable after stop.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Installed Plugin capability declarations, carrier adapter lifecycle, registry availability and existing explicit CLI Plugin integration. |
| Claimed At | Not applicable |
| Source Issue | #513 (parent #466) |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Preparation only; no effective implementation claim. |
| Implementation PR | Not started |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | Finalize atomic claim/activation, pass independent API/security review and exact-head CI, then merge before implementation. |

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

Completion Commit: pending implementation.
This plan cannot certify its own completion. Claim/activation preparation requires no Rust tests.

## Resume

Finalize this plan and CAP-001-C atomically under the actual governance PR number; obtain
independent API/security plan review and documentation CI. After claim merge, start implementation
from that merge or newer main, converge locally and submit one stable implementation candidate.
