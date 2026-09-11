# Iteration I259: Bundle Manifest Compatibility Implementation

> Document status: Review / Claimed
> Planned objective: Implement the ADR-073-compatible Bundle manifest adapter without changing installation or activation behavior.
> MVP deliverable: versioned Bundle manifest parsing and validation with legacy dual-read, controlled-write, unknown-field preservation, stable identity and rollback fixtures.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | `talos-plugin` Bundle manifest adapter, compatibility fixtures, identity validation and metadata-only rollback; no installer, network, activation, provider registration or permission changes. |
| Claimed At | 2026-09-11 |
| Source Issue | #514 |
| Governance Claim PR | #538 |
| Authorization Mode | Independent review |
| Authorization Evidence | Proposed atomic claim+activation; ineffective until finalized governance record merges. ADR-073 accepted in I258. Fresh exact-head CI and independent security/API review required before implementation merge. |
| Implementation PR | #539 |
| Last Updated | 2026-09-11 |
| Handoff / Release Condition | Create implementation branch only after this claim reaches main; preserve DIST-001-A as unclaimed and blocked on this implementation. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| BUNDLE-001 | CAP-001 / #466 | Refinement / Unclaimed | ADR-073 Accepted; CAP-001-C Complete | Compatible manifest adapter and conformance fixtures, with no installation side effects. |

### Scope

- Add the smallest UI-neutral adapter for legacy Plugin and versioned Bundle manifest forms.
- Enforce explicit schema/migration gates, unknown-field preservation or fail-closed rejection, stable identity and safe relative artifact paths.
- Prove metadata-only rollback and separation from installation, activation, registration and permission grants.

### Non-Goals

- No installer, network download, activation, provider registration, permission policy, release, publication, Desktop or Dashboard changes.

### Acceptance

- Given a valid legacy or versioned manifest, parsing returns a normalized in-memory Bundle form without rewriting source bytes.
- Given an unflagged write, unknown fields, duplicate roots, invalid identity or incompatible digest, the operation fails closed.
- Given migration failure, the original manifest/cache/artifact remain unchanged and only new metadata is removed.
- Given successful parsing or migration, no Plugin is executed, activated, registered or granted permission.

### Planned Validation

- Focused locked `talos-plugin` unit and compatibility fixture tests.
- Full affected-workspace locked checks and `./scripts/release_preflight.sh` before stable push.
- Governance validators, changed-file inventory, API/security review and exact-head CI.

### Documentation To Update

- BUNDLE-001 owner, architecture manifest boundary notes, and user-facing Bundle terminology.

### Risks And Rollback

- Risk: silent legacy-field loss, identity collision or migration side effect.
- Rollback: remove adapter integration while retaining legacy reader and original persisted bytes.

## Selection Inventory (2026-09-11)

I258 is Complete / Closed with ADR-073 Accepted. I259 is the only proposed implementation slice;
DIST-001-A/#509 remains Refinement / Unclaimed and blocked on BUNDLE-001. No Active or Review
implementation overlaps this scope; I164 remains Paused/superseded and I249 remains Planned/
Unclaimed. Existing CAP-001 children and LANG-002/003 retain their owner-defined gates.

## Completion Evidence

Pending implementation and independent security/API review. This proposal has no implementation
authorization until its claim record reaches main.
