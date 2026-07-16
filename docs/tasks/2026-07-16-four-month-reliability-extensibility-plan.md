# Four-Month Reliability, Extensibility, And Memory Quality Plan

**Plan window**: 2026-07-16 through 2026-11-15
**Status**: Published baseline — authorized for one-pass unattended assignment; no implementation iteration active
**Execution owner**: `docs/tasks/2026-07-16-reliability-extensibility-execution-package.md`
**Handoff prompt**: `docs/tasks/2026-07-16-frontline-unattended-reliability-prompt.md`

## Outcome

Make Talos safer to run through provider failures, finish the already-built bounded local plugin
slice as a coherent product capability, replace memory admission only if a reproducible benchmark
proves it better, and finish with an independently replayable pre-1.0 readiness packet.

## Why This Sequence

1. `SESSION-006` is the only open P1 defect with a reproduced data-loss path.
2. `PLUGIN-001` is already implemented through a local explicit read-only T111 slice but remains
   in Review/partial documentation state; closure yields user value without inventing a new system.
3. MEM-009 has an accepted architecture but explicitly requires a benchmark before activation.
4. Security and release claims must be checked after, not assumed from, feature tests.

Desktop (#29), automatic health recovery (#32), persistent tasks (#38), multi-instance networking
(#40), and broad workspace permission changes (#22) are intentionally not selected. They remain
proposal, deferred, or security-gated work and would make unattended authority ambiguous.

## Four-Month Packages

Calendar windows are capacity guidance, not mandatory waiting periods. A package may begin early
after its predecessor passes and its activation checkpoint is committed.

The maintainer explicitly requires one uninterrupted N200-N250 run. Phase boundaries are internal
commit/push/checkpoint gates, not review pauses. The executor submits an acceptance request only
after all four months' packages are terminal and the final closeout is pushed.

| Window | Package | Iteration | Runnable/Testable Deliverable | Entry Gate | Exit Gate |
|---|---|---|---|---|---|
| Month 1: 2026-07-16–08-15 | N210 Session integrity | I135 | Resume retains one valid completed tool exchange after provider failure; durable failed turns still abort | N200 Start Gate | Runtime reconstruction proof + full validation |
| Month 2: 2026-08-16–09-15 | N220 Plugin closure | I136 | Explicit local read-only WASM plugin loads, appears in `/plugins`, invokes through permission/provenance, and fails safely | I135 Complete | Offline real-binary fixture + security regressions + full validation |
| Month 3: 2026-09-16–10-15 | N230 Memory benchmark | I137 | Deterministic offline baseline/ablation/candidate report produces predeclared Go/No-Go | I136 Complete | Repeated benchmark + reviewed report + no production change |
| Month 4: 2026-10-16–11-15 | N240/N250 Decision application and closeout | I138/I139 | Apply Go minimally or record No-Go/no-change; then independently replay and issue pre-1.0 readiness report | I137 terminal | Clean-state packet, owner sync, residual map |

## Published Scope Boundaries

### Authorized

- Edit repository code, tests, fixtures, and documentation needed by I135-I139.
- Run local locked Cargo checks, tests, release preflight without a release version, governance
  validation, clean-HOME fixtures, and existing CI-compatible scripts.
- Create conventional commits and push `main` after each completed package when all package gates
  pass and `main` remains fast-forwardable.
- Comment on originating GitHub Issues with factual status/commit evidence; close Issue #36 only
  after SESSION-006 reaches Complete.

### Not Authorized

- Release/tag/publish/deploy, crates.io operations, force-push, history rewrite, destructive data
  migration, permission-default broadening, credential/schema changes, session/TLOG format changes,
  remote plugin installation, write-capable plugin tools, desktop implementation, autonomous
  recovery, task runtime, multi-agent/multi-instance networking, or `v1.0.0` claims.
- New dependencies, `unsafe`, public-API breakage, sandbox behavior changes, or ADR reversal without
  explicit maintainer approval.

## Dependency And Status Inventory At Publication

| Owner | State | Disposition |
|---|---|---|
| I018 | Planned historical baseline | Explicitly deferred; not selected. |
| I019/I020 | Complete | Duplicate stale index entries removed; I020 header reconciled. |
| I048-I056 | Complete | Stale owner headers reconciled to recorded completion evidence. |
| I129-I134 | Complete | Prerequisites/history only. |
| I135 | Planned | First selectable iteration after N200. |
| I136 | Planned | Blocked on I135 Complete. |
| I137 | Planned | Blocked on I136 Complete. |
| I138 | Planned / conditional | Blocked on I137 Go/No-Go. |
| I139 | Planned | Blocked on I135-I138 terminal disposition. |
| SESSION-006 / Issue #36 | Open P1 | Selected in I135. |
| PLUGIN-001 / CMD-002 | In Progress/Partial | Explicitly held for I136; no parallel activation. |
| MEM-009 | Refinement | Selected first as benchmark-only I137. |
| Issues #29/#32 | Proposal/deferred | Not selected; no implementation. |
| Issues #38/#40 | Deferred by ADR-043/044 | Not selected; reversal triggers not met. |
| Issue #22 | Security residual after PERM-004/005 | Not selected; no permission broadening. |
| REL-002 | NO-GO | Remains independent; this program cannot satisfy or waive it. |

No Active or Review iteration is bypassed at publication. Backlog-level PLUGIN-001/CMD-002 work is
not treated as parallel authority; it is explicitly sequenced behind I135.

## Program Acceptance

- SESSION-006 is closed without contradicting ADR-042 or fabricating tool results.
- The local explicit read-only plugin slice has one reproducible user path and bounded failures.
- Memory admission changes only after a predeclared deterministic benchmark passes; ambiguous
  evidence produces No-Go and no production change.
- Unconfigured Runtime/CLI/TUI behavior, permissions, approvals, streaming event ordering, session
  formats, and secret boundaries remain compatible.
- Each package has its own reviewed commit(s), push evidence, checkpoint, runtime evidence, owner
  synchronization, and exact resume instruction.
- I139 reports actual release readiness but performs no release.
- No intermediate acceptance request or maintainer wait is required between packages; after each
  successful phase commit, push, and checkpoint, immediately activate the next eligible iteration.

## Residual Destination

- Session/provider follow-ups: SESSION-006 or a newly refined SESSION/PROVIDER story.
- Plugin expansion: PLUGIN-001, HOOK-001, DIST-001, or a new ADR-gated story.
- Memory benchmark failures or ideas: MEM-009 and ADR-046 execution notes.
- Security findings: PERM-005 or a new security owner; do not silently fold them into feature work.
- Release gaps: a dated release closeout task; REL-002 remains the sole v1 gate.
