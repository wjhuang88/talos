# CAP-001-C: Plugin Capability Declarations And Carrier Adapters

**Status**: Review / Claimed (#527 effective on main; local implementation candidate)

| Field | Value |
|---|---|
| Story ID | CAP-001-C |
| Type | Plugin runtime integration |
| Parent | CAP-001 / #466 |
| Status | Review / Claimed (#527 effective on main; local implementation candidate) |
| Selected Iteration | I255 |
| Source Issue | [GitHub Issue #513](https://github.com/wjhuang88/talos/issues/513) |
| Depends On | CAP-001-B; ADR-027; ADR-072 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Installed Plugin capability declarations, carrier adapter lifecycle, registry availability and existing explicit CLI Plugin integration. |
| Claimed At | 2026-09-10 |
| Authorization Evidence | Claim #527 effective at 07066e76e92c30d63a2a9068dfdd5e0737d3f1a1; head eb26aaef, CI 34449342743, independent Agent-role API/security plan review 5615007238 and CAS 5615007433. Shared account does not prove natural-person separation. Implementation needs fresh exact-head API/security review. |
| Governance Claim PR | #527 |
| Implementation PR | Not started |
| Authorization Mode | Independent review |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | #527 is effective; implementation needs fresh exact-head CI, independent API/security review and CAS before merge. No new carrier, permission, Bundle or product authority. |

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md), [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), and [ADR-027](../../decisions/027-plugin-runtime-boundary.md).
- [PLUGIN-001](PLUGIN-001-wasm-runtime-plugins.md) for preserved WASM/sandbox evidence; no carrier decision may weaken it.

## Goal And Scope

Connect executable Plugins to the Capability/Provider registry through explicit declarations and
carrier adapters. Preserve Plugin lifecycle, sandbox, timeout, provenance and permission boundaries.
Built-in, WASM, MCP, helper-process and remote carriers remain distinct adapters.

## Non-Goals

No Bundle installation/distribution, online resolution, new carrier security decision, language or
browser Provider, Desktop/Dashboard, public manifest rename, or expansion of WASM authority.

## Acceptance

- A Plugin can declare one or more Provider capabilities without conflating Bundle metadata.
- Adapter lifecycle maps load/initialize/activate/stop to registry availability explicitly.
- ADR-027 WASM restrictions and existing PLUGIN-001 evidence remain intact.
- Carrier failures, timeouts and incompatible descriptors fail closed and do not crash the host.
- Provider registration does not bypass permission or tool disclosure policy.

## Validation And Documentation

Carrier-specific contract fixtures, Plugin regression tests, security review, locked checks and
architecture/API documentation. Changed-file inventory must exclude Dashboard and unrelated lanes.

## I255 Selection Preparation (2026-09-10)

I254/CAP-001-B is Complete on main through closeout #526 (`cefb8320`), with existing
implementation `d03da494`/`63328e53`. The dependency is satisfied; this is not an implementation
claim. [I255](../../iterations/I255-plugin-capability-lifecycle.md) specifies the runnable Plugin
lifecycle integration, compatibility contract, failure/ownership/stop tests and current inventory.

The first executable adapter retains ADR-027 WASM restrictions. Other carrier identities remain
distinct and unsupported until an authorized adapter exists; declarations cannot make them Ready.
Legacy public manifest fields and persisted files are not renamed or rewritten. Additive APIs
must preserve old constructors and loading behavior; any required break needs decision control.
Real CLI and public lifecycle tests must prove the behavior, not descriptor registration alone.
All local subtasks stay under #513. Complete does not imply future carriers or Bundle installation.

## Atomic Claim Proposal (2026-09-10)

PR #527 proposes I255 Active / Claimed and this Story In Progress / Claimed atomically.
Earlier Unclaimed/preparation text is historical; neither proposed status is effective until
merge. Scope and compatibility contract are in I255; no implementation code accompanies this PR.

## Effective Activation (2026-09-10)

#527 merged at `07066e76e92c30d63a2a9068dfdd5e0737d3f1a1`; I255 records exact-head
CI, independent plan review and CAS. Claim/activation are now effective, superseding the dated
proposal state. Implementation remains local until the whole slice passes its stable-candidate
checkpoint; no implementation PR or completion evidence exists yet.

## Local Review Candidate (2026-09-10)

I255 now records the implemented lifecycle, core ownership leases, CLI integration, compatibility
adapters, tests and complete changed-file inventory. Initial HTTP fixture and initialization
panic findings were repaired locally. Final full validation and exact-head remote review remain
required; this is Review, not completion. #513 stays open until owner-first closeout reaches main.
