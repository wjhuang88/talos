# Iteration I210: Provider Retry Progress Contract

> Document status: Planned / Unclaimed
> Planned date: 2026-08-17
> Objective: deliver PROVIDER-006 through a truthful semver-compatible progress contract for
> provider dispatch, retry backoff and first-packet wait.

## Collaboration Claim Preparation

| Field | Value |
|---|---|
| Claim State | Unclaimed; governance proposal not yet effective |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I210/PROVIDER-006 only: accept ADR-062; add a source-compatible defaulted provider progress entrypoint; project real built-in provider dispatch/backoff/first-packet facts through Agent/session/conversation; render Connecting/Reconnecting state; add deterministic compatibility, retry, cancellation and UI tests; update directly affected docs. No retry-policy, timeout, persistence, dependency, release, Desktop/Dashboard, I198 or I211 implementation. |
| Claimed At | Not effective until target-branch merge |
| Source Issue | #278 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review proposed |
| Authorization Evidence | Pending exact-head review, CI, both governance validators and merge-time CAS. |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Do not create an implementation branch until the finalized claim and accepted ADR exist on `main`; implementation must start from that merge or later exact `main`. |

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

## Exact-Main Claim Inventory - 2026-08-20

Baseline: `main@7c5cc8b7b4d75d7a71d2f632e6696d9023588396` after `git fetch origin`;
local and remote heads matched and the primary worktree was clean.

| State | Iterations | Disposition |
|---|---|---|
| Active | None | I210 may be proposed, but remains unactivated until this claim reaches `main`. |
| Review / implementation merged | I197, I200, I201, I212 | Preserve their Issue #302/I211 human-validation rows. Their TUI/model scopes do not transfer authority or overlap provider retry progress. |
| Planned / Claimed | I189 | Keep unactivated; its protected permission scope is independent of I210. |
| Planned / Unclaimed | I198, I206, I207, I208, I210, I211 | Preserve their owners and order. I210 is the next mainline child; none grants overlapping authority. |
| Paused | I164 | Preserve its superseded target; do not resume. |
| Blocked | None with a current iteration document status | Backlog-level blockers, including Issue #59 production children, retain their independent owners and gates. |

Open PRs #120/#121 are archival Drafts. No open implementation or claim PR targets Issue #278,
PROVIDER-006 or I210. The retained `/private/tmp/talos-i201-impl` worktree and its branch belong to
I201 history and must not be modified. Stashes `stash@{0}` and `stash@{1}` remain historical and
must not be restored as a unit.

## 2026-08-20 Decision And Claim Preparation

ADR-062 is Proposed. It defines a typed, non-secret request-local progress protocol, an additive
defaulted `LanguageModel` method for third-party source compatibility, the existing ordered
Agent/session event projection, and a distinct reconnecting phase carrying the provider-owned retry
ordinal and ceiling. It explicitly preserves retry policy and requires the public conversation enum
addition to wait for a pre-1.0 minor release rather than a patch.

The preparation branch is governance-only. Until the actual claim PR is finalized, independently
reviewed and merged, I210 remains Planned/Unclaimed and no Rust, Cargo, implementation branch,
version, tag or publication action is authorized.
