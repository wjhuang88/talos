# Iteration I252: Capability/Provider Descriptor Contract

> Document status: Review / Claimed

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
| Implementation PR | #507 |
| Completion Commit | `71cc03b322d80d19c3767e4ef2014fecb84c1994` |
| Merge Commit | `0835afc4b982a5ed409ccfb55538bbeaf6fe5eab` |
| Exact-head CI | `34294884578` (success; including Windows) |
| Independent Review | Handoff reports approval; original exact-head/base review record awaits verification. PR #507 has no public review/comment record. |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Verify original exact-head review and merge-time CAS evidence before completion. CAP-001-B/C remain separately unclaimed and unauthorized. |

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

## Closeout checkpoint (2026-09-09)

I252 remains Review / Claimed pending verification of original review evidence. Implementation commit `71cc03b322d80d19c3767e4ef2014fecb84c1994`
is present in the merge of PR #507 as `0835afc4b982a5ed409ccfb55538bbeaf6fe5eab`.
Completion Commit: `71cc03b322d80d19c3767e4ef2014fecb84c1994` (pre-existing implementation
commit; this governance closeout is not its own evidence).
Exact-head CI `34294884578` passed all required jobs, including Windows, at head
`f0103a7f4e78e42fc88364f6e4cbb801b1c96553`. The handoff reports Agent-role approval,
but the original exact-head/base review and CAS records have not yet been verified in this
resumed session. No public review identity or unavailable comment ID is asserted.
The closeout changes only owner and derived governance records. CAP-001 remains Refinement /
Unclaimed; CAP-001-B and CAP-001-C are not activated or authorized by this closeout.

## Residuals

Registry/resolver belongs to CAP-001-B; Plugin/Carrier adapters belong to CAP-001-C.
