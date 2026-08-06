# ARCH-034-R04: Native And Panic-Boundary Containment

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Findings | ARCH-034-F13, ARCH-034-F19 |
| Status | Refinement - independent security review required |
| Priority | P1 |
| Selected Iteration | Not selected |
| Preserved behavior | Permission gates, native error mapping, process limits, storage format, and fallback policy |

## Problem And Boundary

`gix`, arborium/tree-sitter, bundled SQLite, subprocess spawning, and ADR-007 libc sites do not have
one current containment matrix. Some boundaries use timeout, error propagation, or `catch_unwind`;
coverage must be proven per call family rather than assumed.

## Scope

- Produce a call-site/failure-mode/containment/test matrix for every native or panic-capable boundary.
- Reconcile ADR-007 and R0 status facts without weakening their restrictions.
- After independent review, add only the narrow containment/tests required by proven gaps.

## Exclusions

- No sandbox, permission, process-hardening, `unsafe`, dependency, or policy edit before security review.
- No catch-all panic swallowing, silent fallback, or replacement of ADR-recorded dependencies.

## Readiness And Acceptance

- Independent reviewer records escape-vector and failure-mode analysis.
- Each accepted gap has one bounded implementation slice and explicit safe fallback.
- Process, permission, git, symbol, SQLite, and crash tests cover the reviewed boundary.
- Locked workspace, platform, security, governance, and ADR checks pass.

## Rollback / Residual

If independent review is unavailable, remain Refinement and do not edit protected code. New native
dependencies require a separate ADR.
