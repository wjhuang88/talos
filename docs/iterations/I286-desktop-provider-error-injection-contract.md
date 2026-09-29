# I286 — Desktop Provider Error Injection Contract

> Document status: Review / Claimed
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

## Local Convergence Checkpoint — 2026-09-30

The Desktop provider boundary now has a debug-build-only, opt-in failure injector for local H1
reproduction. `TALOS_DESKTOP_PROVIDER_FAILURE=error` returns a provider error without dispatching
network traffic; `timeout` emits a bounded first-packet-timeout event after five seconds through
the real provider stream path. Release builds ignore the variable, and the default configuration
path is unchanged.
`cargo check -p talos-desktop --locked`,
`cargo test -p talos-desktop --features desktop-ui --locked --bin talos-desktop-mock`
(109/109, repeated after the five-second change), and
`cargo clippy -p talos-desktop --features desktop-ui --all-targets --locked -- -D warnings`
pass locally. These checks include provider and real RuntimeHost error, timeout,
cancellation, and later-submit coverage. Native H1 evidence is still outstanding, so no
completion claim is made.

The debug-only boundary was checked separately with
`cargo check --release -p talos-desktop --features desktop-ui --locked --bin talos-desktop-mock`
(pass); the release build excludes the injection adapter. `./scripts/release_preflight.sh`
passed its site, installer, governance, text-boundary, workspace check, and workspace Clippy
stages, but workspace test compilation stopped with `No space left on device` before the test
suite could run. This first run was not a passing preflight. After clearing generated build
artifacts, the full `release_preflight.sh` passed with `CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2`; it included workspace tests,
doctests and the external Runtime SDK fixture. The full preflight was repeated after the
five-second injection change and passed. Native H1 observation remains outstanding.

## Native H1 Acceptance Recipe

Build the exact candidate with
`cargo build -p talos-desktop --features desktop-ui --locked --bin talos-desktop-mock` and record
its commit SHA. Use a non-sensitive local workspace and an
already configured provider; normal provider/model/credential validation still applies. Run each
case in a fresh process, and record the observed UI state and candidate SHA in Issue #29:

1. Launch `TALOS_DESKTOP_PROVIDER_FAILURE=error target/debug/talos-desktop-mock --live`, select
   the workspace and submit a short prompt. Verify a visible terminal provider error, a responsive
   window, and that a later submission is accepted without stale activity.
2. Launch `TALOS_DESKTOP_PROVIDER_FAILURE=timeout target/debug/talos-desktop-mock --live` and
   submit once. Wait for the five-second synthetic first-packet timeout; verify a visible terminal
   error and responsive window. Submit again and cancel before five seconds; verify Cancelled is
   terminal and no delayed timeout appears. Close the window and verify clean shutdown.
3. Launch `target/debug/talos-desktop-mock --live` with the variable unset, submit a short prompt
   and verify the configured provider can respond normally. Do not use this synthetic timeout as
   evidence of a real network timeout or provider transport correctness.

## Governance

PR #621 merged the claim and activation atomically to `main` as `33875dd3` on 2026-09-29.
Implementation starts from that merge or later `main`.

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
| Authorization Evidence | Maintainer accepted the contract on 2026-09-29; #621 exact-head CI `36590316149`, independent Agent-role review comment `5893341827`, and merge-time CAS recorded below |
| Implementation PR | Not started |
| Last Updated | 2026-09-29 |
| Handoff / Release Condition | H1 native evidence and implementation review required; I287/H6 separate; no release |

## Activation Checkpoint — 2026-09-29

Claim PR #621 merged from exact head `dde67a11cf60596f54285d1218100ba848d8d8d7`
against base `eb23c3d8f0d4574a1f98fd87ebd82892afb75bcd` after exact-head CI
`36590316149` passed, independent Agent-role review was recorded in comment `5893341827`,
and merge-time CAS confirmed a clean, non-overlapping candidate. Merge commit:
`33875dd3d099d51e4e6a9c038ba7d134e496262a`. Shared GitHub account review
separates Agent roles, not natural-person identities. This activates I286 only; it does not
complete H1 or authorize I287/H6 implementation.

## Contract Acceptance Checkpoint — 2026-09-29

The maintainer accepted this security/API contract: only explicit debug injection is allowed;
default and release behavior stay unchanged. Native H1 will prove how the real Desktop UI handles
simulated provider faults, not that a real remote network request timed out. Acceptance alone did
not activate I286; the claim became effective only when #621 merged.

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
open #619 browser contract. #620 has merged. I285 retains its H1/H6 residuals in #29.
