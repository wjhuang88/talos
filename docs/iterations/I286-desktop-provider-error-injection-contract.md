# I286 — Desktop Provider Error Injection Contract

> Document status: Active / Claimed (proposed by #621; ineffective until merge to main)
> Planned objective: Define and implement a safe, isolated provider failure path that exercises the real Desktop error/timeout presentation without changing credentials, disrupting networking, or treating a mock response as native evidence.

## Scope

- Define an explicit locally scoped failure-injection contract at the configured Desktop provider boundary.
- Preserve production fail-closed behavior; injection must never be enabled implicitly.
- Exercise provider error, timeout, cancellation and retry presentation through the real Desktop host path.
- Add deterministic automated coverage and a native acceptance recipe for H1.

## Out Of Scope

- No credential mutation, network disruption, provider workaround, release behavior, or permission-policy change.
- Existing `MockProvider` and localhost fixtures do not satisfy native H1 evidence.

## Readiness And Acceptance

- Requires an accepted security/API contract before implementation authority is granted.
- Default configured-provider execution remains unchanged.
- Error and timeout states reach the real Desktop path without panic, hang, credential change, or network side effect.
- Locked tests cover error, timeout, cancellation and retry cleanup.
- The maintainer can reproduce and record native H1 evidence in #29 using the exact candidate build.

## Governance

PR #621 proposes the claim and activation atomically. Neither takes effect until its exact
governance candidate is merged to `main`; no implementation is committed or pushed before then.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I286 / Desktop H1: private debug-only provider failure injection, deterministic host/UI tests and native H1 acceptance; no Runtime/provider public API, permission policy, release behavior or I287 Evaluation source |
| Claimed At | 2026-09-29 |
| Source Issue | #29 |
| Governance Claim PR | #621 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer accepted the contract on 2026-09-29; exact-head governance CI and independent security/API review required before #621 merge |
| Implementation PR | Not started |
| Last Updated | 2026-09-29 |
| Handoff / Release Condition | H1 native evidence and implementation review required; I287/H6 separate; no release |

## Contract Acceptance Checkpoint — 2026-09-29

The maintainer accepted this security/API contract: only explicit debug injection is allowed;
default and release behavior stay unchanged. Native H1 will prove how the real Desktop UI handles
simulated provider faults, not that a real remote network request timed out. This acceptance does
not activate I286 or authorize committed implementation before the claim reaches `main`.

- The injection is a private Desktop adapter, not a public Runtime/provider API or persisted
  configuration. Only a debug build with the exact `TALOS_DESKTOP_PROVIDER_FAILURE=error` or
  `timeout` value enables it. Unset/unknown values and release builds use the configured provider.
- Injection starts after normal provider/model/credential configuration validation. It makes no
  provider network request, changes no credential, and does not alter permission or tool policy.
  The local test workspace should contain no sensitive data because normal Desktop session setup
  and storage still apply.
- `error` produces a provider network-error result. `timeout` waits for a bounded local first
  packet deadline and produces a provider error event. Both travel through the actual Desktop
  Runtime host and UI projection; neither is a fixture response or a real network timeout test.
- H1 native acceptance must observe the exact debug candidate in a real Desktop window: failure
  visibly terminates, the UI stays responsive, cancellation/shutdown finish, and a fresh launch
  without the variable can submit a real configured-provider request. Record build commit and
  observed results in the existing #29 ledger. Automated tests must also cover failure, timeout,
  cancellation and later-submit cleanup.
- Do not enable this in release builds, use it to claim provider transport correctness, or use
  its synthetic error as proof that a remote service actually timed out. The existing provider
  stream timeout tests remain the transport-level evidence.

The uncommitted provider experiment in the predecessor implementation worktree is disposable
investigation, not claim or implementation evidence. The current non-terminal inventory is I277,
I282, I284 and I285 in Review; I249 and I287 remain Planned. I286 is non-overlapping with the
open #619 browser contract and #620 CI documentation PRs. I285 retains its H1/H6 residuals in #29.
