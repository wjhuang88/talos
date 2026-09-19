# ADR-072: Capability, Provider, Plugin and Bundle Boundary

*Status: Accepted*

Accepted by maintainer authorization on 2026-09-09. This acceptance covers the architecture
boundary and migration contract only; implementation remains delegated to separately claimed
CAP-001 child stories.

## Context

Issue #466 requires a single vocabulary for capabilities, executable extensions, and
installation artifacts. Existing Plugin, Tool, language, browser, and distribution
documents use overlapping terms and must not be treated as one runtime contract.

## Decision

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
terminology portion of ADR-029 is superseded by this ADR.

## Scope and gates

### Repository layout clarification — I278 / CAP-001-D

Concrete optional Plugin/Provider sources belong under repository-root
`plugins/<domain>/<provider>/`. Host infrastructure remains in `crates/talos-plugin/`.
Generated Bundle directories and installed package locations are not source ownership. I278
delivers independently locked Rust/Python guest builds and explicit packaging under ADR-079;
the root host dependency graph must not statically absorb those optional implementations.
Shared source-only query modules may be reused from the full checkout without importing host
UI/runtime dependencies. [Plugin build guide](../../plugins/README.md) records exact inputs.

The original #466 TypeScript/Browser/tool-example tree is illustrative future domain ownership,
not blanket implementation authority. No empty placeholder can serve as delivery evidence.
I253's historical R1 governance satisfaction did not implement this source tree; CAP-001-D owns
that recovered omission and I278 records executable acceptance separately.

This ADR is architecture-only. It authorizes no runtime, Cargo, persistence, network, permission,
Desktop, or Browser implementation. CAP-001-A/B/C and domain children require separate owner
documents, runnable iterations, effective claims, and exact-head validation.

## Rejected alternatives

- Treating `enabled` as a universal lifecycle state.
- Making Bundle installation auto-activate a Plugin or grant permission.
- Adding a second TUI/Desktop capability registry.
- Embedding all optional language parsers in the default binary.

## Migration matrix

| Existing concept | Target concept | Compatibility rule | Rollback boundary |
|---|---|---|---|
| Plugin package references | Bundle references | Read existing package metadata; emit target vocabulary only after a versioned migration | Revert metadata migration without changing runtime state |
| `enabled` boolean | Installed/Active lifecycle states | Do not reinterpret persisted `enabled`; introduce explicit state fields in a separately versioned schema | Continue legacy interpretation until migration completes |
| Direct parser consumers | Language Provider consumer contract | Keep current consumers behind adapters until one provider vertical slice is validated | Remove adapter and restore existing consumer path |
| Tool/backend disclosure | Provider registration plus presentation policy | Registration never implies prompt exposure; preserve TOOL-012/014 decisions | Disable provider contribution while retaining disclosure policy |

Migration must be dual-read before controlled-write, must preserve unknown fields, and must fail
closed on invalid or incompatible versions. No network fetch, executable installation, permission
grant, or persisted schema rewrite is implied by this ADR.

## Staged child boundaries

1. CAP-001-A: descriptor contracts and compatibility types.
2. CAP-001-B: offline registry/resolver and conformance tests.
3. CAP-001-C: Plugin contribution and Carrier adapters.
4. BUNDLE/TEXT/LANG/DIST/BROWSER children only after the contracts are accepted.

Each child must have its own owner, iteration, claim, acceptance evidence, and exact-head review.

## Acceptance checklist

- [x] Maintainer authorization accepts this ADR; independent review evidence is recorded separately.
- [x] CAP-001 child owners and dependencies are created after acceptance: CAP-001-B #512,
  CAP-001-C #513, TEXT-001 #511, LANG-001 #510, LANG-002 #516, LANG-003 #517,
  BUNDLE-001 #514, DIST-001-A #509, DIST-001-B #515 and BROWSER-001 #508. These owners
  remain separately unclaimed and do not authorize implementation.
- [ ] Existing completed stories retain their original scope and evidence.
