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
| Responsible Actor | `@wjhuang88` |
| Executing Agent | GPT-5.6 Sol / talos开发 session |
| Work Slice | Produce and land the #502 `libc` version-route research/evidence report: upstream route, historical selection evidence, Talos call-site/ABI/platform matrix, dependency-resolution implications, three-path comparison, validation/rollback plan and maintainer recommendation. No dependency, lockfile, runtime, sandbox or process-hardening implementation is authorized. |
| Claimed At | Not applicable until claim merge |
| Source Issue | #502 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Repository maintainer requested completion of the 2026-09-22 research handoff in Issue #502. No distinct reviewer is attached to this governance-only research claim; exact-head CI, both governance validators and merge-time CAS are required before merge. Any later `libc` version or security-boundary implementation still requires separately governed ADR-007/escape-vector review. |
| Implementation PR | Not started |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Merge this research-only claim before committing the evidence report. The report may recommend a migration candidate but must not change manifests/lockfiles or authorize protected implementation; advisory/deprecation evidence remains a separate residual. |

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
