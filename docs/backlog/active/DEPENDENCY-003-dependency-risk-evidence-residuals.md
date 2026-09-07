# DEPENDENCY-003: Dependency Risk Evidence Residuals

> Document status: Refinement / Unclaimed

| Field | Value |
|---|---|
| Story ID | DEPENDENCY-003 |
| Type | Dependency Governance / Security Evidence Story |
| Priority | P2 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #502](https://github.com/wjhuang88/talos/issues/502) |
| Selected Iteration | None |
| Depends On | I250 final disposition; ADR-007; ARCH-034-R04; DEPENDENCY-001 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Own the post-I250 `libc` migration and dependency advisory/deprecation evidence residuals; no implementation is authorized |
| Claimed At | Not applicable |
| Source Issue | #502 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-07 |
| Handoff / Release Condition | Revisit by 2026-12-07 or the next workspace dependency upgrade, whichever is earlier; select separately protected implementation slices before changing native/security boundaries or audit semantics |

## Scope

- Preserve the I250 exception for `libc 1.0.0-alpha.3` until a separately governed migration has
  an API/call-site matrix, ADR-007 and escape-vector review, and Unix/Windows evidence.
- Preserve advisory and deprecation signals as `unknown` until a reproducible Rust-native evidence
  source, unavailable-source behavior and accepted-baseline compatibility are decided and tested.
- Route the native boundary through ARCH-034-R04 and recurring audit semantics through
  DEPENDENCY-001 without treating either completed/reference owner as implementation authority.

## Exclusions

No dependency, manifest, lockfile, sandbox, permission, process-hardening, audit implementation,
release or publication change is authorized by this intake owner. It does not reopen I250 or #474.

## Acceptance

- Each residual is completed under a separately effective claim or renewed with fresh evidence and
  a new dated revisit trigger.
- Missing advisory coverage can never be silently presented as safe.
- A failed `libc` migration restores the accepted manifest/lock resolution before merge.

## Verification Evidence

- I250 disposition and Issue #502 establish the initial reason, validation gap, trigger and
  containment/rollback contract.

## Completion Evidence

- Completion Commit: pending
