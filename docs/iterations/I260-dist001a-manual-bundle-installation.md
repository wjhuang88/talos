# Iteration I260: Verified Manual Bundle Installation

> Document status: Planned / Unclaimed
> Parent: CAP-001 / #466; DIST-001-A / #509
> Objective: deliver an offline, user-selected Bundle installation path with deterministic integrity, compatibility, destination, and rollback behavior.

## Collaboration Claim

Claim State: Unclaimed
Responsible Actor: Not assigned
Executing Agent: Not assigned
Implementation PR: Not started
Authorization: No implementation authorization until an effective claim reaches `main`.

## Scope

- Read a manually supplied local Bundle artifact using the I259 manifest contract.
- Validate identity, schema compatibility, digest, package-root and destination safety.
- Stage atomically; reject corrupt, partial, conflicting, or unsafe artifacts without changing installed state.
- Provide deterministic rollback evidence.
- Keep installation separate from activation, registration, provider resolution, and permission grants.

## Non-Goals

No network discovery/download, marketplace, automatic activation, executable loading, permission changes, Desktop/Dashboard work, or release/publication work.

## Runnable Deliverable

An offline installation API/CLI path plus fixtures and tests proving success, corruption rejection, path containment, idempotency, and rollback without activation side effects.

## Dependencies

- BUNDLE-001 / I259 Complete (`137da646`).
- CAP-001-C / I255 Complete.
- DIST-001 policy and ADR-072.

## Acceptance

- Valid manually supplied Bundles install to a deterministic destination after compatibility and integrity checks.
- Invalid or incomplete artifacts fail closed and leave prior installed state byte-for-byte unchanged.
- Repeating an identical installation is deterministic and does not duplicate state.
- Installation never loads, activates, registers, resolves, or grants permissions to a Plugin.

## Validation

Focused offline fixtures, locked workspace tests, path/security review, governance validators, and exact-head CI. User-facing installation documentation must describe installation versus activation separately.

## Handoff

After this iteration reaches Complete/Closed, DIST-001-B may be reconsidered. LANG-002 and LANG-003 remain independently governed.
