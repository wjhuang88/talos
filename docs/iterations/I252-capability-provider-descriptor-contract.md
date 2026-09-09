# Iteration I252: Capability/Provider Descriptor Contract

> Document status: Planned / Claimed

| Field | Value |
|---|---|
| Story ID | CAP-001-A |
| Source | GitHub Issue #466 |
| Parent | CAP-001 |
| Depends On | ADR-072 Accepted; CAP-001-P0 Complete |
| Scope | Define versioned, UI-neutral Capability and Provider descriptors and conformance fixtures. |
| Exclusions | No registry/resolver, Plugin loader, Bundle install, network, Cargo dependency, or persisted schema migration. |
| Deliverable | Runnable contract types/tests documented for later CAP-001-B consumers. |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex Agent / single-developer unattended session |
| Work Slice | Descriptor identity, version/capability compatibility, provider metadata, validation errors, and offline conformance fixtures. |
| Source Issue | #466 |
| Claimed At | 2026-09-09 |
| Governance Claim PR | Direct commit 4f8e641a |
| Authorization Mode | Direct commit |
| Authorization Evidence | Maintainer authorization recorded in ADR-072; single-developer unattended session. |
| Implementation PR | Main commit `71cc03b3` (stable candidate; review pending) |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Independent exact-head review and owner-first closeout required before Complete. |

## Acceptance

- Descriptors have stable identifiers, version compatibility, provenance, and carrier metadata.
- Invalid/unknown versions fail closed with typed validation errors.
- Contract is UI-neutral and contains no Arborium/Tree-sitter concrete types.
- Fixtures prove deterministic serialization and offline validation.
- Existing PluginManifest and persisted configuration remain unchanged.

## Validation

- Focused unit and conformance tests for valid, invalid, and forward-compatible descriptors.
- Locked checks for affected crate(s), governance validators, and `git diff --check`.

## Execution checkpoint (2026-09-09)

Descriptor contract implementation is present on `main` at `71cc03b322d80d19c3767e4ef2014fecb84c1994`.
The focused `talos-core` test suite passed locally. Exact-head independent review remains pending;
this iteration is not marked Complete until that evidence and owner-first closeout are recorded.

Local correction checkpoint (2026-09-09): `ec7db027` adds explicit provenance/carrier enums and
`c28f9764` adds deterministic serde round-trip and unknown-value conformance tests; `8c888271`
ensures legacy descriptors with missing origin metadata remain readable but fail validation closed.
`cargo test --locked -p talos-core` passes (83 tests). These commits are local stable-candidate
work and do not change the completion state or authorize CAP-001-B.

## Residuals

Registry/resolver belongs to CAP-001-B; Plugin/Carrier adapters belong to CAP-001-C.
