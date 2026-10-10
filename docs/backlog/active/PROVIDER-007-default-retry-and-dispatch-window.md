# PROVIDER-007: Default Retry And Dispatch Window

| Field | Value |
|---|---|
| Story ID | PROVIDER-007 |
| Parent / Related Owner | PROVIDER-002; NET-001 is related intake only |
| Status | Active / Claimed (proposed; ineffective until governance PR #689 merges) |
| Priority | P0 urgent reliability maintenance |
| Source | Maintainer request 2026-10-09; no new GitHub Issue |
| Selected Iteration | I297 - Active / Claimed (proposed; ineffective until governance PR #689 merges) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I297/PROVIDER-007 only: default `ProviderTimeoutConfig.max_attempts` 3 -> 5 and `dispatch_timeout_secs` 60 -> 300 seconds; direct config/provider tests and directly affected docs. No retry taxonomy, backoff, stream-limit, cancellation, dependency, toolchain, version, release or explicit-config behavior change. |
| Claimed At | 2026-10-09 |
| Source Issue | None |
| Governance Claim PR | #689 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer selected expedited normal maintenance and explicitly requested serial I297/I296 closeout on 2026-10-10. No independent human maintainer is available; independent Agent reliability/API review, exact-head CI and CAS remain mandatory. This is not an emergency override. |
| Implementation PR | Not started |
| Last Updated | 2026-10-10 |
| Handoff / Release Condition | Proposed claim and activation are ineffective until #689 merges to `main`; a fresh implementation branch then needs exact-head validation, required review and merge-time CAS. |

## Goal

Increase the defaults used only when a provider does not explicitly configure its timeout policy:

- `ProviderTimeoutConfig.max_attempts`: `3` to `5`.
- `ProviderTimeoutConfig.dispatch_timeout_secs`: `60` to `300` seconds.

The desired operator outcome is fewer premature request-header failures for slow providers and a
larger bounded recovery budget for retryable pre-stream failures, without removing a user's ability
to select stricter values.

## Scope

- Change the two defaults and their public doc comments in `talos-config`.
- Update default and deserialize/override tests so a provider's explicitly configured values remain
  authoritative.
- Verify the existing OpenAI-compatible and Anthropic paths consume the config values without
  changing their retry classifier or retry progress semantics.
- Update the directly affected configuration/operator documentation that describes these defaults.

## Fixed Semantics

- `max_attempts` retains the existing retry-module convention: it is the maximum retry dispatch
  ordinal after the initial request, not a new total-attempt interpretation.
- `dispatch_timeout_secs` covers only request dispatch through response headers. It does not expand
  `first_packet_timeout_secs` or `stream_idle_timeout_secs`, which remain `30` and `90` seconds.
- Explicit provider configuration continues to override the defaults exactly as before.

## Non-Goals

- No retryable/non-retryable failure taxonomy, backoff, jitter, retry progress UI or cancellation
  change.
- No stream first-packet or idle-timeout policy change.
- No generic network retry/circuit-breaker implementation; NET-001 remains an unclaimed intake
  owner.
- No provider dependency, lockfile, MSRV/toolchain, release-version or public-API shape change.
- No automatic retry/replay after text, tool-call or other irreversible stream output.

## Acceptance

- Given a provider with no timeout block, when config defaults are constructed or deserialized,
  then the dispatch/header limit is 300 seconds and the retry limit is 5.
- Given an explicit provider timeout block, when it supplies different values, then those values
  remain unchanged after parsing.
- Given the built-in OpenAI-compatible and Anthropic providers, when dispatch/retry is exercised
  with an explicit test configuration, then their existing bounded retry and dispatch timeout
  behavior remains unchanged.
- Given the staged diff, when reviewed, then it changes no stream timeout, classifier, backoff,
  dependency, release version or user-specified configuration behavior.

## Planned Validation

- `cargo test -p talos-config --locked`
- `cargo test -p talos-provider --locked`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`

## Required Reads

- `AGENTS.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/backlog/active/NET-001-network-resilience-policy.md`
- `crates/talos-config/src/types.rs`
- `crates/talos-config/src/tests.rs`
- `crates/talos-provider/src/retry.rs`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/lib.rs`

## State / Status Owners

- This document owns the selected default-policy maintenance slice only.
- I297 owns execution, validation and release-candidate evidence for this slice.
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md` and `docs/BOARD.md` are derived
  views and do not authorize implementation.

## Residuals

- The cross-network resilience taxonomy, replay policy and circuit-breaker design remain in
  NET-001 and are explicitly not implied by these two default changes.
- Request-context budget accounting and pre-send compaction telemetry are a separate product defect
  discovered during this selection. They must receive their own owner before implementation; they
  do not block PROVIDER-007.

2026-10-10 checkpoint: the separate bounded context-budget repair I298 is Complete through
#691/#692. Broader MEM-005 policy remains separately owned; this does not change the default-only
scope or transfer ADR-087's next-minor API change into a patch release. I297's owner records the
serial closeout contract; I296 remains separately unclaimed until its own target-branch claim.
