# Iteration I210: Provider Retry Progress Contract

> Document status: Complete / Closed
> Planned date: 2026-08-17
> Objective: deliver PROVIDER-006 through a truthful semver-compatible progress contract for
> provider dispatch, retry backoff and first-packet wait.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I210/PROVIDER-006 only: accept ADR-062; add a source-compatible defaulted provider progress entrypoint; project real built-in provider dispatch/backoff/first-packet facts through Agent/session/conversation; render Connecting/Reconnecting state; add deterministic compatibility, retry, cancellation and UI tests; update directly affected docs. No retry-policy, timeout, persistence, dependency, release, Desktop/Dashboard, I198 or I211 implementation. |
| Claimed At | 2026-08-20 |
| Source Issue | #278 |
| Governance Claim PR | #321 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #321 exact head `4d45f1ba890fa7cb1ea6f6f058ecb0f0916eb639` passed CI `32322271343`, independent governance/architecture review `5350249740`, both governance validators and merge-time CAS, then merged to `main` as `e58fbd399a7071aad7ad8fd846a82f2745611fa0`. |
| Implementation PR | #323 |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | PR #323 passed exact-head CI/review/CAS and merged as `9d5c8a71`. I211 confirmed retry ordinals/cleanup but found initial connection and first-turn queue sequencing defects; TUI-060/#332 is the separate Ready/Unclaimed corrective owner. Keep I210 Review with no retry-policy or implementation authority transfer. |

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

Governance PR #321 established Active / Claimed plus ADR-062 acceptance on `main@e58fbd39`.
The implementation branch may now be created from this exact main, but no implementation commit,
release or publication action is implied by the governance merge.

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

The preparation branch was governance-only. PR #321 is merged and the claim is effective; the next
exact action is to create an implementation worktree from `main@e58fbd39` and implement only the
accepted I210 Work Slice. Version, tag and publication actions remain excluded.

## 2026-08-20 Implementation Checkpoint

Implementation commit `6efee2b8` adds the accepted typed provider-progress path and its runtime/TUI
projection. It preserves retry policy, dependency closure, persistence and release boundaries. The
implementation branch used the effective claim base and contains no Cargo, version, tag or
publication changes.

Validation completed against the implementation head: affected-crate locked tests passed (including
core/provider/agent/conversation/TUI), the full `talos-cli` locked suite passed with an isolated
writable `HOME`, formatting and strict affected-crate Clippy passed, and `git diff --check` passed.
The earlier CLI failure was an outer-sandbox configuration-I/O denial, not a product failure.

The implementation remains `Review / Claimed`: exact-head CI, independent technical review,
merge-time CAS, and the live retry-status human row in Issue #302 remain required. No
`Completion Commit` is recorded before those gates and the deferred human validation pass.

## 2026-08-20 Implementation Merge Disposition

PR #323 final head `c984ec483aaba5f6d4d1e96d288cfcb874b0f239` passed exact-head CI
`32333116774`, independent Agent technical re-review `5351610613`, both governance validators and
merge-time CAS, then merged to `main` as `9d5c8a71718b44d424092a45a75d3da0d593547d`.
Issue #302 comment `5351796088` now binds the merged implementation and the still-open natural-person
Connecting/Reconnecting matrix.

I210 remains Review/Claimed with `Completion Commit: Pending`. Its machine, technical-review and
merge gates are terminal; only the deferred human row prevents completion. This truthful Review
disposition permits the non-overlapping I198 claim to proceed under its own owner and effective
claim.

## 2026-08-20 I211 Human Validation Partial Failure

Natural-person validation on integrated `main@a2f43248` with a local OpenAI-compatible mock
provider observed the truthful `Reconnecting... (attempt 1/2)` value and terminal cleanup, but the
preceding `Connecting...` state was replaced too quickly to be observably stable. The same
walkthrough found that an idle first submission emitted `Message queued and will send after current
turn.` even though no earlier turn existed.

TUI-060 / Issue #332 separately owns observable initial connection status and first-turn queue-hint
semantics. At this historical checkpoint I210 remained Review with `Completion Commit: Pending`;
I211 granted no corrective implementation authority.

## 2026-08-28 Natural-Person Validation Closure

The maintainer confirmed the live request path displayed the expected initial `Connecting...`
state followed by provider-reported `Reconnecting... (attempt n/m)` during a real retry, with
transient activity clearing after the terminal outcome. Historical partial-failure checkpoints
remain unchanged.

I210 is now Complete / Closed. Completion Commit: `9d5c8a71` (implementation merge for PR #323).
