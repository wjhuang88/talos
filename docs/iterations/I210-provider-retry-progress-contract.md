# Iteration I210: Provider Retry Progress Contract

> Document status: Planned / Unclaimed
> Planned date: 2026-08-17
> Objective: deliver PROVIDER-006 through a truthful semver-compatible progress contract for
> provider dispatch, retry backoff and first-packet wait.

## Selected Story

- `PROVIDER-006` - `docs/backlog/active/PROVIDER-006-bounded-retry-progress-contract.md`

## Activation Gate

- I209 has a terminal disposition and records the retry-progress transfer.
- A new ADR defines the provider progress contract and compatibility/migration boundary.
- Current non-terminal work is inventoried and an effective Collaboration Claim exists on target
  `main` before an implementation branch is created.

## Runnable Deliverable

A rebuilt CLI/TUI receives actual bounded provider dispatch/retry progress through the accepted
contract and can cancel during dispatch, backoff and first-packet wait, with deterministic tests.

## Scope

- ADR and semver-compatible provider progress contract.
- Actual attempt/backoff facts projected through runtime status.
- Cancellation and presentation evidence at all three provider wait boundaries.
- Directly affected provider/runtime/CLI documentation.

## Exclusions

No retry-policy redesign, timeout-default change, new dependency, persistence migration,
Desktop/Dashboard work or release/publication work.

## Exact-Main Non-Terminal Inventory - 2026-08-17

Baseline: `main@c738033272687a5066a32d2ed86826782ecdfce6`.

| State | Iterations | Disposition |
|---|---|---|
| Active | I209 | Continue its bounded implementation/review only; do not activate I210. |
| Review | I188 | Preserve its independent background-job decision closeout; no overlap or authority transfers to I210. |
| Planned / Claimed | I189, I195, I196 | Keep unactivated under their existing owners and claims. |
| Planned / Unclaimed | I197-I208 | Preserve published order and require separate claims. |
| Planned / Unclaimed | I210 | Publish this runnable follow-up only; require ADR, fresh inventory and effective claim before activation. |
| Blocked | None | Backlog-level blocked parents retain their own owner gates. |
| Paused | I164 | Preserve supersession; do not resume. |

Open PRs #120/#121 remain archival Drafts and #233 remains Dashboard-owned. This planning addition
does not take over, repair or merge them.

## Acceptance

- [ ] An accepted ADR defines compatibility, migration and rollback.
- [ ] Actual provider attempt/backoff facts drive bounded status without fabricated timing.
- [ ] Cancellation during dispatch, backoff and first-packet wait returns the durable terminal
      outcome promptly.
- [ ] Retry policy, dependency closure and unrelated consumers remain unchanged.
- [ ] Focused and workspace locked validation plus affected user documentation pass at exact head.

### Connecting/Reconnecting UI Row

- [ ] Initial model request displays `Connecting…`.
- [ ] A real retryable error or timeout projects `Reconnecting… (attempt n/m)` through the same
      structured progress contract used for dispatch/backoff facts.
- [ ] The row clears on success, terminal failure or cancellation; no text parsing or elapsed-time
      inference is used for `n/m`.
- [ ] OpenAI-compatible deterministic fixtures cover repeated timeout/error, bounded retries and
      final success/failure as observed through the model-request activity surface.

## Status

Planned / Unclaimed. Issue #278 is intake and planning evidence only. No claim, implementation
branch, public API change or code authorization exists.
