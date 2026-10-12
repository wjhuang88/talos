# RUNTIME-010: Missing Pending-Sidecar Startup Recovery

> Document status: Ready

**Type**: Technical / Runtime State Story
**Parent Epic**: None
**Selected Iteration**: I301

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Missing pending-sidecar recovery read fast path, storage/Actor regressions and startup documentation only; no existing-store durability or custody change. |
| Claimed At | 2026-10-12 |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer repeatedly authorized continuing the intermittent-timeout investigation and repair through GitHub CI. No separate maintainer reviewer is available; independent Agent storage/API review, exact-head CI, both validators and merge-time CAS remain required. |
| Implementation PR | Not started |
| Last Updated | 2026-10-12 |
| Handoff / Release Condition | No implementation before this atomic claim and activation merges into main; release only after evidence-bearing closeout or explicit handoff. |

## Identity / Goal / Value

A new or legacy Runtime caller with no pending journal should reach command admission without
creating SQLite solely to discover an empty inbox. Windows diagnostic #705 measured successful
startup samples spending seconds in PRAGMA/schema work before Submit reception. This is a bounded
unnecessary-work repair, not proof of the cause of every historical watchdog failure.

## Scope

Under the store's existing guard, distinguish a genuinely absent pending-sidecar file using an
error-preserving filesystem query. Return empty recovery without directory/database/schema creation
only for absence. Retain the existing connection/schema/query path for an existing journal.
Affected code: talos-session pending_submission recovery and its tests; talos-agent startup tests
only as needed to prove command admission. No new dependency or public API is required.

## Exclusions

No connection cache, async ownership rewrite, schema migration, WAL/FULL change, recovery bypass for
existing journals, permission/path-policy change, retry or deadline increase, process hardening,
PowerShell repair, provider/Search/Git work, release or reopening #590/I290/I291.

## Dependencies / Decision Links And Constraints

ADR-008 bundled SQLite and ADR-056 custody are accepted. ADR-056 explicitly treats legacy Sessions
without a sidecar as an empty inbox. Preserve ADR-058 durable finalization and current native error
propagation. The claim merge is the implementation gate; #705 remains an open diagnostic owner.
Concurrent instances use existing SQLite serialization; a missing-file observation is a read snapshot,
not a guarantee that another instance cannot later create or admit work. Do not weaken acknowledgements.

## Acceptance

- Given an absent sidecar and absent parent directory, when recovery reads run, then they return no
  work and create neither directories nor database; normal submission still creates durable storage.
- Given an existing journal containing pending and Running submissions, when recovery runs, then
  identity/state/order, Running reconciliation and restart custody remain unchanged.
- Given an inaccessible/malformed path or unsupported schema, when recovery runs, then a real error
  remains visible; absence handling must not hide I/O or schema failures.
- Given concurrent store instances, when reads and admissions interleave, then acknowledged work is
  durable/recoverable and existing idempotency/custody guarantees hold.
- Original Auto flow and ordinary workspace tests retain assertions, 15s watchdog, 10s cleanup and
  tool deadlines. Compare exact repair-head Windows startup evidence with #705's preserved baseline;
  successful timing samples are not a universal latency guarantee or a closed PowerShell risk.

## Uncertainty And Validation Path

Review filesystem absence/race semantics before implementation. Reject the fast path if it cannot
preserve the existing snapshot and custody contract. Operation timings include scheduling; no
Defender/disk/OS root cause is inferred. Avoid fragile millisecond unit-test thresholds: assert
structural absence and real persisted recovery, then use bounded Windows CI for timing evidence.

## Planned Validation / Documentation

Focused locked talos-session and real Actor/Runtime regressions; repository-pinned Rust 1.99.0;
standard release_preflight.sh, both governance validators, exact-head Windows/Unix CI and independent
Agent storage/API review. No ignored assertions or retry-to-green. Update user-facing Runtime session
storage documentation in README.md / README.zh-CN.md and relevant runtime SDK contract, then owner,
iteration, backlog and Board in that order. No release is authorized.

## State/Status Owners / Required Reads

This Story owns acceptance; I301 owns execution/evidence; indexes and Board mirror them.
Read AGENTS.md, collaboration/iteration/testing SOPs, ADR-008/056/058, pending_submission.rs and
pending_submission/storage.rs, Agent session.rs/custody.rs and #705 checkpoint
https://github.com/wjhuang88/talos/pull/705#issuecomment-6121305365.

## Residual Destination

PowerShell low-CPU active-child latency and attribution of previous failures remain in #705 and
WINDOWS-AUTO-TIMEOUT-DIAGNOSIS.md. Existing-store initialization latency is outside this first slice.
No Complete state without an already-existing implementation SHA and required evidence.
