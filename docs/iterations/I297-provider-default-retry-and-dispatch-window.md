# Iteration I297: Provider Default Retry And Dispatch Window

> Document status: Planned
> Published plan date: 2026-10-09
> Planned objective: Deliver the maintainer-approved urgent default change from three to five provider retry dispatches and from a 60-second to a 300-second request dispatch/header limit.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A tested, configuration-compatible provider default policy that makes no classifier, stream-timeout, backoff, dependency or release-version change.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None; maintainer urgency request 2026-10-09 |
| Governance Claim PR | Pending |
| Authorization Mode | Pending single-maintainer merge |
| Authorization Evidence | This is expedited normal maintenance, not an emergency override. |
| Implementation PR | Not started |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Claim and activation become effective only when the finalized governance record merges to `main`. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PROVIDER-007 | PROVIDER-002; NET-001 related only | Ready / Unclaimed | Existing `ProviderTimeoutConfig` and dispatch/retry implementation | Default retry and dispatch/header values increase while explicit configuration remains authoritative. |

### Scope

- Set default `max_attempts` to 5 and default `dispatch_timeout_secs` to 300.
- Update default, parsing and override tests and the directly affected user/operator documentation.
- Prove OpenAI-compatible and Anthropic implementations preserve their current use of explicit
  values and their current retry/stream boundary.

### Non-Goals

- No retry taxonomy, backoff/jitter, cancellation, progress event or TUI behavior change.
- No first-packet or stream-idle timeout change.
- No generic retry/circuit breaker, dependency, lockfile, toolchain/MSRV, version or publication
  change.
- No reinterpretation of `max_attempts`; it remains the existing retry ordinal budget after the
  initial dispatch.

### Acceptance

- Given default provider timeout configuration, when it is created or omitted in TOML, then it has
  a 300-second dispatch/header limit and a retry limit of 5.
- Given explicit values in TOML, when parsed, then the caller's values win unchanged.
- Given the existing provider test matrix, when providers are supplied explicit short limits, then
  their bounded timeout and retry behavior remains deterministic and unchanged.
- Given the final diff, when reviewed, then it contains only this default-policy slice and directly
  affected documentation/tests.

### Planned Validation

- `cargo test -p talos-config --locked`
- `cargo test -p talos-provider --locked`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/PROVIDER-007-default-retry-and-dispatch-window.md`
- The affected configuration/reference documentation, if one is present in the current tree.
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md` and `docs/BOARD.md`.

### Risks And Rollback

- Risk: larger defaults extend perceived terminal failure time or unintentionally change explicit
  configuration behavior.
- Rollback: revert this small default-only implementation commit; no schema migration or persisted
  state is involved.

## Selection Inventory - 2026-10-09

Target baseline: `9e376155`. The selection is non-overlapping with current work.

| Owner | Current State | Disposition For I297 |
|---|---|---|
| I293 / SEARCH-001-C | Active / Claimed | Retain; search routing is unrelated. |
| I294 / SEARCH-001-D | Active / Claimed | Retain; provider-evidence gate is unrelated. |
| I291 / AUTO-UX-001 | Review / Claimed | Retain; locale acceptance does not alter provider defaults. |
| I277 / DESKTOP-001-D3 | Review / Claimed with deferred acceptance | Retain; no Desktop work in this slice. |
| I249 dependency pilot | Planned | Retain; no dependency or lockfile change is authorized. |
| I164 | Paused | Retain unchanged. |
| I296 / TOOLCHAIN-001 | Planned / Unclaimed on separate governance branch | Retain; it has not merged to the target baseline and no toolchain work is included here. |

No open PR on `main` owns these provider default values. Existing PR #682 is the unrelated locale
follow-up and draft #688 is the unrelated I296 toolchain governance proposal. No new GitHub Issue is
created: this is a narrow maintainer-selected child with no external handoff requirement.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-10-09 | Planning | Published as a separate urgent slice so runtime defaults are not mixed with I296 toolchain maintenance. Claim, activation, implementation and release evidence remain pending. |

## Verification Evidence

- Pending effective claim and implementation.

## Completion Evidence

- Completion Commit: Pending. A status-only documentation commit cannot be used as completion evidence.

## Variance And Residuals

- NET-001 remains the owner for generic retry/circuit-breaker policy and no such architecture is
  selected here.
- The observed pre-send context-budget telemetry mismatch is a separate residual; it does not
  change provider dispatch defaults and requires an independently owned slice.

## Retrospective

- Outcome: Pending.
- Documentation: pending implementation evidence.
- Lessons: pending.
