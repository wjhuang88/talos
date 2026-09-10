# CAP-001-C: Plugin Capability Declarations And Carrier Adapters

| Field | Value |
|---|---|
| Story ID | CAP-001-C |
| Type | Plugin runtime integration |
| Parent | CAP-001 / #466 |
| Status | Refinement / Unclaimed |
| Selected Iteration | I255 (preparation only) |
| Source Issue | [GitHub Issue #513](https://github.com/wjhuang88/talos/issues/513) |
| Depends On | CAP-001-B; ADR-027; ADR-072 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Capability declarations and carrier-specific adapter boundary for installed Plugins. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Requires a selected runnable iteration and effective claim after CAP-001-B. |

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
