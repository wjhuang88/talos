# CAP-001-C: Plugin Capability Declarations And Carrier Adapters

| Field | Value |
|---|---|
| Story ID | CAP-001-C |
| Type | Plugin runtime integration |
| Parent | CAP-001 / #466 |
| Status | Refinement / Unclaimed |
| Selected Iteration | None |
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
