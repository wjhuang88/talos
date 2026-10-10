# Iteration I297: Provider Default Retry And Dispatch Window

> Document status: Review
> Published plan date: 2026-10-09
> Planned objective: Deliver the maintainer-approved urgent default change from three to five provider retry dispatches and from a 60-second to a 300-second request dispatch/header limit.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A tested, configuration-compatible provider default policy that makes no classifier, stream-timeout, backoff, dependency or release-version change.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I297/PROVIDER-007 only: default `max_attempts` 3 -> 5 and `dispatch_timeout_secs` 60 -> 300 seconds, direct config/provider tests and directly affected docs. No retry taxonomy, backoff, stream-limit, cancellation, dependency, toolchain, version, release or explicit-config behavior change. |
| Claimed At | 2026-10-09 |
| Source Issue | None; maintainer urgency request 2026-10-09 |
| Governance Claim PR | #689 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer selected expedited normal maintenance and explicitly requested I297 then I296 closeout on 2026-10-10 in single-maintainer mode. No independent human maintainer is available; independent Agent reliability/API review, exact-head CI and merge-time CAS remain mandatory. This is not an emergency override. |
| Implementation PR | #694 |
| Last Updated | 2026-10-10 |
| Handoff / Release Condition | Claim effective through #689; implementation requires exact-head validation, independent Agent review and merge-time CAS. |

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
| 2026-10-09 | Proposed atomic claim+activation | #689 proposes I297/PROVIDER-007 as Claimed / Active under a single-maintainer merge path. Both remain ineffective until this exact claim record reaches `main`; no implementation has started. |
| 2026-10-10 | Effective activation | #689 merged as `71fc8bce8e80c32a7fc9b9b641d482c57dc70544`; head `82b71aecfd1a41653663a14ceb17446caaf82964`, base `6b4bfb8d3f20faa6e711b218d1631603da53b572`, CI `38010968762`, independent Agent review comment `6091864356`, merge-time CAS comment `6091887597`. Implementation starts from this merge on `fix/i297-provider-defaults`. |

## Verification Evidence

- 2026-10-10 local implementation: defaults now dispatch/header 300 seconds and five retries
  after initial dispatch. Explicit TOML overrides and first-packet/idle/backoff remain unchanged.
- `cargo test -p talos-config --locked`: 226 unit tests and one doctest passed.
- `cargo test -p talos-provider --locked` (low-debug environment): passed, including both
  protocols' dispatch limits, retry progress and stream-boundary tests.
- `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 ./scripts/release_preflight.sh`:
  passed; includes workspace check, Clippy, all workspace tests/doctests, external Runtime SDK
  fixture in both modes, site/installer checks and both governance validators (zero warnings).
- Locked all-target workspace check and all-target/all-feature Clippy with `-D warnings`: passed.
  Existing transitive `block v0.1.6` future-compatibility notice remains outside this slice.
- Actual CLI `config list`, using an isolated temporary configuration, reported defaults
  `dispatch_timeout_secs=300`, `max_attempts=5`; explicit 17/2 remained 17/2. The nested
  `config get` key is not supported by the existing CLI; evidence uses parsed `config list` TOML.
- Independent local Agent review identified the Anthropic default progress test's old expectation;
  corrected to 5. OpenAI's explicit three-retry fixture remains unchanged. Final exact-head
  review and remote CI are still required before merge.

## Completion Evidence

- Completion Commit: Pending. A status-only documentation commit cannot be used as completion evidence.

## Variance And Residuals

- 2026-10-10 bounded CI follow-up: run `38012251384`, Windows job `114096073540`,
  passed every validation step but exceeded the 30-minute job limit during rust-cache
  post-job compression. GitHub's annotation explicitly reports the job timeout.
  Raise only the Windows aggregate budget to 45 minutes to accommodate validation plus
  cache cleanup; individual step deadlines and all acceptance gates remain unchanged.
  This mechanical CI follow-up is kept in #694 under the existing-PR maintenance rule;
  it does not authorize toolchain or provider-scope expansion. Fresh exact-head CI and
  independent review are required; the cancelled run is not merge approval.

- NET-001 remains the owner for generic retry/circuit-breaker policy and no such architecture is
  selected here.
- The observed pre-send context-budget telemetry mismatch is a separate residual; it does not
  change provider dispatch defaults and requires an independently owned slice.

## Retrospective

- Outcome: Pending.
- Documentation: pending implementation evidence.
- Lessons: pending.

## Closeout Schedule And Startup Contract — 2026-10-10

Requested outcome: close I297, then independently claim and close I296. Standard work mode;
no deferred human rows or validation tracker. This record coordinates sequencing only and does
not establish I296 ownership. The maintainer selected these non-overlapping maintenance slices;
Search and Auto locale remain with their other sessions. Published Baseline above is unchanged.

Fresh target: main `6b4bfb8d3f20faa6e711b218d1631603da53b572`. Inventory:
I293/I294 Active, I277/I290/I291 Review, I249 Planned/deferred, I164 Paused: retain their owners
and scopes. I296 is an unmerged Planned/Unclaimed proposal in #688, scheduled after I297.
I298 Complete/Closed through #691/#692; context-budget residual has its own completed bounded
owner. No current Blocked iteration header was found; historical checkpoints in Complete owners
are not activation authority. Open #682 owns Auto locale; no provider-default overlap found.

| Item | Expected output | Dependency | Completion gate | Failure fallback |
|---|---|---|---|---|
| P1 | Effective I297 claim | #689 refreshed to target | Docs CI, validators, Agent review and CAS before merge | Correct locally; no implementation before claim |
| P2 | Default 5/300 with explicit-config compatibility | P1 | Baseline checks, full preflight and actual CLI config evidence | Batch corrections in the same candidate |
| P3 | Merged implementation and owner-first I297 closure | P2 | Exact-head CI/review/CAS; existing main Completion Commit | Keep Review while a required gate is missing |
| P4 | Independent I296 claim, implementation and closure | P3 | I296's preserved plan and own claim/checks/review/CAS | Preserve stash; report genuine blocker without claiming completion |

Artifacts: I297 and PROVIDER-007 owners first; README/configuration docs, Board, Backlog and
iteration index second. Use one worktree, separate serial branches, append a checkpoint after
each stage. Authorized actions: scoped edits, local checks, commits, candidate pushes and normal
PR merges under the recorded single-maintainer path. No tag/publication, credential access,
deployment, spending, user-data deletion, main force-push or new per-step Issue is authorized.
Preserve both stashes and other-session branches. Toolchain remains pinned 1.97.0 during I297.
Conserve disk with low-debug build environment; inspect space before expensive checks, do not
discard caches/user data without scoped authorization. Retry transient observation of the same
live CI handle; do not restart or cancel valid runs just because a poll times out.

Default decisions: explicit timeout values win; max_attempts retains retry-ordinal semantics;
no first-packet/idle change. No API widening or release-version decision here. ADR-087's
next-minor boundary prevents publishing I298 in an I297 patch. Broader resilience remains NET-001.
2026-10-10 execution update: #689 is merged; local implementation and preflight passed.
Next gate: stable implementation candidate, exact-head CI and independent review, then merge-time CAS.
