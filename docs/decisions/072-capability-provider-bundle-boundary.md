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

## Acceptance checklist

- [ ] Independent architecture review accepts this ADR.
- [ ] CAP-001 child owners and dependencies are created after acceptance.
- [ ] Existing completed stories retain their original scope and evidence.
