# Talos Three-Track Development Baseline

> Status: Established on 2026-08-13
>
> Common Base: `23e4174bcfb036602ce2145026b872ec5c517289`
>
> Establishing merge: PR #198, reviewed head
> `727d80564ea4554ae29f7c3085429f23bb50d79b`
>
> This baseline coordinates work; it does not activate an iteration or establish a Collaboration
> Claim.

`AGENTS.md` and the routed repository SOPs remain authoritative for this entire document. The
bounded-maintenance and emergency exceptions in `docs/sop/AGENT-COLLABORATION.md` remain available;
no topology or checklist statement below narrows them.

## Objective

Run three parallel delivery lanes while preserving one releasable mainline:

1. Dashboard product design and read-only UI development;
2. Desktop D0 and mock-only GPUI visual/i18n development;
3. shared runtime/session/tool/permission work continuing through mainline governance.

All released behavior returns to `main`. No lane may create a second runtime, session, permission,
work-state, or governance source of truth.

## Baseline Evidence

- PR #206 merged as `512ff32f389167364c02e7058151879b9ce6859a` after exact-head CI,
  independent natural-person review and merge-time CAS.
- I191/I192 closed owner-first through PR #208 at
  `d94c704e8cc0866d62e763bfdc298d08d89d6af8`, using the pre-existing #206 merge as
  Completion Commit evidence.
- PR #198 synchronized to that mainline, passed exact-head local preflight and CI run
  `31660804527`, received independent natural-person approval bound to
  `727d80564ea4554ae29f7c3085429f23bb50d79b`, and merged as the Common Base above.
- The #198 squash tree is `8fce4bc56abc4cd9af730a79a8fa0b85cae57fd4`, equal to the
  merge-tree result of its exact base and reviewed head.
- Both governance validators reported zero warnings at the #198 merge gate.

## Post-Baseline Non-Terminal Inventory

| Iteration | State | Disposition |
|---|---|---|
| I159 | Blocked | Keep blocked until TUI-037 has a recorded disposition. |
| I160 | Blocked | Keep blocked until I159 is Complete. |
| I161 | Blocked | Keep blocked until I160 is Complete and a security-review plan exists. |
| I162 | Blocked | Keep blocked until I161 is Complete and release-readiness authorization exists. |
| I164 | Paused | Preserve the superseded startup-inline target; do not resume through this baseline. |
| I188 | Planned / Claimed | Keep unactivated; its decision-only TOOL-024-A scope remains independent. |
| I189 | Planned / Claimed | Keep unactivated; its behavior-preserving PERM-006-A scope remains independent. |

There is no Active or Review iteration at establishment. ARCH-034-R04 remains Partial, SESSION-008
remains Ready / Released with B Ready / Unclaimed, and RUNTIME-005 remains blocked on B. This
inventory is an establishment fact, not future activation authority; every slice repeats the
current inventory before it starts.

## Git Topology

The three routes are logical delivery lanes, not long-lived divergent integration branches.

- `main` remains the only integration and release branch.
- Every normal governed implementation slice first lands an effective claim on `main`.
- Only after that claim merge may its short-lived implementation branch be created from the claim
  merge commit or a later `main` commit.
- Use lane-qualified names such as `feat/dashboard-<iteration>-<slice>`,
  `feat/desktop-<iteration>-<slice>`, and `feat/runtime-<iteration>-<slice>`.
- Every implementation PR targets `main`, carries exact-head validation/review evidence, and passes
  merge-time CAS.
- Shared crate/API changes land through the mainline lane before Dashboard/Desktop consume them.
- Dashboard/Desktop branches must not copy shared domain or runtime logic to avoid waiting.

Each authorized route may use an independent worktree and short-lived branch. Small slices integrate
continuously into `main`; there is no late three-way integration branch or lane-specific release.

## Lane A — Dashboard

### Initial Shape

Create a new governed child owner through requirement intake for Dashboard-wide information
architecture and the first read-only visual shell. Do not repurpose WEB-001's published acceptance.
This baseline describes the candidate; it does not authorize or claim it.

The first implementation slice should be limited to the existing GET-only loopback data surfaces:

- `/status`;
- `/history`;
- `/governance`;
- `/config` with existing masking;
- `/extensions` read-only presentation.

It may introduce a cohesive navigation/layout/design system and accessible responsive rendering.
It must retain ADR-031 loopback binding, GET-only routing, HTML escaping, output-boundary redaction,
and existing JSON/plain-text content negotiation.

### Separate Security Slices

The following remain separate owners/ADRs/claims and are not implied by Dashboard-wide design:

- SSE/live log transport;
- configuration writes;
- approvals or tool execution;
- session mutation/actions;
- WebSocket control;
- LAN/remote/tunnel access;
- browser automation.

TUI-037 remains an independent Refinement item and must not be folded into the Dashboard lane.

## Lane B — Desktop

This lane may open only from the Common Base or a later `main`, and only after its own owner and
effective claim have landed.

Two separately governed candidate slices are expected first:

1. D0 — renderer/dependency/host ADR and repository boundary decision;
2. a mock-only GPUI Execution-page visual/i18n slice covering `zh-CN`, `en-US`, bilingual layout,
   deterministic fallback, reduced motion, and Chinese IME for editable controls in scope.

The mock slice may use fixtures or presentation-local ephemeral state. It cannot claim real Mission
execution, persistence, completion, Evaluation, approvals, Artifact/Delivery, or session recovery,
and cannot add an alternate durable Mission/session/permission store.

Real Runtime/Work Graph/Evaluation binding waits for the relevant shared P0-P4 contracts and APIs
defined by DESKTOP-001. Desktop remains a host above `talos-runtime`, never a second agent engine.
SESSION-009 remains the gate for attach/reconnect/multi-client behavior, not for a local mock-only
single-window spike.

When P0 is formed, its dependency reconciliation must add discoverable Desktop downstream references
to the affected RUNTIME-001 and SESSION-009 owners without changing their completed evidence. P1
acceptance must require mechanical regression evidence for the existing `todo_*` tool contracts;
the prose compatibility promise alone is insufficient completion evidence.

## Lane C — Mainline Foundations

This lane delivers shared capabilities consumed by both product surfaces.

- Preserve ARCH-034-R04 as Partial; AG-11 and other children remain independently claimable.
- Preserve SESSION-008 as Ready / Released and SESSION-008-B as Ready / Unclaimed.
- RUNTIME-005 remains blocked on SESSION-008-B.
- I188/TOOL-024-A and I189/PERM-006-A remain Planned / Claimed and are not activated by this
  baseline.
- Issues #45, #49, and #59 remain open until their own owner evidence permits closure.
- Archival PRs #120/#121 remain untouched.
- The P0-P4 Work Graph/Evaluation chain uses separate governed slices on `main`.

SESSION-008-B is the recommended next shared-runtime candidate, but it requires its own effective
claim and must include SESSION-008-R1/R2 in its plan and acceptance. The authoritative record for
both residuals is [I192](../iterations/I192-session-runtime-recovery-closure.md); this baseline is
only a discoverability pointer and does not replace that owner document:

- R1: link I187 characterization as the current released-behavior truth source until B lands, while
  ADR-058 remains the target contract;
- R2: if the seven transient `talos-session` failures recur, record disk bytes/inodes, temporary
  paths, complete stderr, and default-parallel results; do not claim a concurrency defect or ENOSPC
  root cause without reproduction evidence.

This baseline does not claim or activate SESSION-008-B.

## Cross-Lane Ownership Rules

| Concern | Authoritative lane | Consumers |
|---|---|---|
| Runtime/session/tool/permission/domain APIs | Mainline | Dashboard, Desktop |
| Loopback HTTP presentation and web assets | Dashboard | TUI link may consume availability only |
| GPUI/native presentation and client-only locale/layout state | Desktop | None |
| Mission/Work Graph/Evaluation durable truth | Mainline P0-P4 | Desktop, later Dashboard views |
| Release/version/tag | Main only | All lanes after integration |

If a Dashboard/Desktop slice needs a shared API change, pause that portion, form a mainline owner
and claim, land it on `main`, then refresh the consuming branch. Do not widen the product-surface PR.

## Per-Slice Merge And Completion Gate

1. Inventory all Active, Review, Planned, and Blocked iterations and record disposition.
2. For normal governed implementation, establish the effective target-branch claim before creating
   implementation work. Resolve any conflict in this checklist in favor of the applicable SOP;
   bounded-maintenance and emergency work retain the explicit exceptions in
   `docs/sop/AGENT-COLLABORATION.md`.
3. Branch from the claim merge or later `main` and record the exact base SHA.
4. Keep one runnable/testable deliverable and its user-facing documentation in scope.
5. Run targeted acceptance and every preflight required by `AGENTS.md` and the applicable testing,
   Git, collaboration, and release SOPs; do not treat this summary as authority to skip a gate.
6. Obtain independent natural-person exact-head review; disclose identity on shared accounts.
7. Repeat merge-time CAS against `main` and merge without force-push.
8. Close owners afterward using the already-existing implementation/merge SHA as `Completion
   Commit`; a status-only commit cannot self-certify.

## Release Gate

Only `main` may produce a release. Before a release tag, all selected lane slices must already be
integrated, owner/Board truth synchronized, residuals recorded, and the versioned
`./scripts/release_preflight.sh vX.Y.Z` must pass. No lane branch may publish, tag, or claim release
readiness independently.
