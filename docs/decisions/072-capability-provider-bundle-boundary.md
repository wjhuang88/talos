# ADR-072: Capability, Provider, Plugin and Bundle Boundary

*Status: Proposed*

## Context

Issue #466 requires a single vocabulary for capabilities, executable extensions, and
installation artifacts. Existing Plugin, Tool, language, browser, and distribution
documents use overlapping terms and must not be treated as one runtime contract.

## Decision (proposed)

- **Capability** is a stable core contract; it is not an implementation or install unit.
- **Provider** implements one or more capabilities behind a UI-neutral contract.
- **Plugin** is a loadable runtime extension with lifecycle, sandbox, timeout, provenance,
  and host-call boundaries.
- **Bundle** is an install/distribution/version unit. Installation never implies activation,
  registration, or permission.
- **Carrier** describes how a provider/plugin executes or connects (built-in, WASM, MCP,
  helper process, or remote connector).
- **Asset** is non-executable provider data (grammar, query, model, or template).

Object lifecycles remain separate: Bundle `Discovered -> Downloaded -> Verified -> Installed`;
Plugin `Unloaded -> Loaded -> Initialized -> Active -> Stopped`; Provider `Registered -> Ready
-> In Use -> Unavailable`; tool exposure remains governed by progressive disclosure policy.

Resolution may use an installed and already-authorized provider without network access at startup.
Automatic executable download is disabled by default. Missing optional capabilities must degrade
to a safe domain-specific fallback without crashing the process.

## Compatibility and migration contract

No persisted manifest field or public Rust API is renamed by this ADR. Any rename requires a
versioned migration matrix, dual-read/controlled-write period, rollback behavior, and a separate
accepted ADR. Existing ADR-027 WASM safety conclusions remain authoritative; only the package
terminology portion of ADR-029 is proposed for supersession.

## Scope and gates

This ADR is architecture-only. It authorizes no runtime, Cargo, persistence, network, permission,
Desktop, or Browser implementation. CAP-001-A/B/C and domain children require separate owner
documents, runnable iterations, effective claims, and exact-head validation.

## Rejected alternatives

- Treating `enabled` as a universal lifecycle state.
- Making Bundle installation auto-activate a Plugin or grant permission.
- Adding a second TUI/Desktop capability registry.
- Embedding all optional language parsers in the default binary.

## Migration matrix (proposed)

| Existing concept | Target concept | Compatibility rule | Rollback boundary |
|---|---|---|---|
| Plugin package references | Bundle references | Read existing package metadata; emit target vocabulary only after a versioned migration | Revert metadata migration without changing runtime state |
| `enabled` boolean | Installed/Active lifecycle states | Do not reinterpret persisted `enabled`; introduce explicit state fields in a separately versioned schema | Continue legacy interpretation until migration completes |
| Direct parser consumers | Language Provider consumer contract | Keep current consumers behind adapters until one provider vertical slice is validated | Remove adapter and restore existing consumer path |
| Tool/backend disclosure | Provider registration plus presentation policy | Registration never implies prompt exposure; preserve TOOL-012/014 decisions | Disable provider contribution while retaining disclosure policy |

Migration must be dual-read before controlled-write, must preserve unknown fields, and must fail
closed on invalid or incompatible versions. No network fetch, executable installation, permission
grant, or persisted schema rewrite is implied by this ADR.

## Staged child boundaries (proposed)

1. CAP-001-A: descriptor contracts and compatibility types.
2. CAP-001-B: offline registry/resolver and conformance tests.
3. CAP-001-C: Plugin contribution and Carrier adapters.
4. BUNDLE/TEXT/LANG/DIST/BROWSER children only after the contracts are accepted.

Each child must have its own owner, iteration, claim, acceptance evidence, and exact-head review.

## Acceptance checklist

- [ ] Independent architecture review accepts this ADR.
- [ ] CAP-001 child owners and dependencies are created after acceptance.
- [ ] Existing completed stories retain their original scope and evidence.
