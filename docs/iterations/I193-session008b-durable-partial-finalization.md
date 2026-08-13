# Iteration I193: SESSION-008-B Durable Partial-Turn Finalization

> Document status: Planned
> Published plan date: 2026-08-13
> Planned objective: implement the Accepted ADR-058 contract as one atomic, idempotent durable
> Success/Error/Cancelled finalization path with display-safe partial replay.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: after a completed display-safe tool result is followed by interruption or an
> eligible provider error, a real durable Session restart shows that result exactly once with an
> explicit incomplete outcome and reconstructs the same safe model context.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline session 2026-08-13 |
| Work Slice | Implement only SESSION-008-B / I193 under ADR-058: one atomic/idempotent Success/Error/Cancelled finalizer, session-owned closed-prefix handoff, durable outcome projection and restart integration. Preserve successful-turn compatibility and TLOG v1; exclude RUNTIME-005, SESSION-009, TOOL-024, permissions, UI redesign and unrelated session cleanup. |
| Claimed At | 2026-08-13 |
| Source Issue | #45 |
| Governance Claim PR | #210 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent exact-head review is required because this slice changes durable session/storage and cancellation behavior. This proposal has no ownership effect before the finalized claim reaches `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-13 |
| Handoff / Release Condition | Finalize the claim with its actual PR number, pass exact-head CI and both governance validators, obtain independent natural-person approval, pass merge-time CAS and merge the claim before creating an implementation branch. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. This proposed `Claimed` record is not
an effective claim until PR #210 is merged into `main`.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SESSION-008-B | SESSION-008 / Issue #45 | Ready / Unclaimed | SESSION-008-A Complete at `e288afb5`; ADR-058 Accepted | One durable finalization and replay path for admitted Success/Error/Cancelled turns |

### Scope

- Add the ADR-058 atomic finalization operation in `talos-session`, retaining `commit_turn` as a
  source-compatible Success projection and preserving TLOG schema version 1.
- Enforce first-terminal-outcome-wins semantics: identical retries return original entry IDs and
  conflicting outcome or canonical payload retries fail without mutation.
- Bind every admitted partial entry to its `turn_id`, store the hidden outcome marker atomically,
  and expose normalized outcome records without placing markers into visible transcript or model
  context.
- Publish the latest structurally closed, persistence-policy-filtered prefix to session-owned state
  so cancellation can finalize stable facts without depending on an aborted future's return value.
- Route eligible provider Error and user Cancelled paths through the same finalizer while preserving
  current successful-turn, entry-ID, event-ordering, resume, fork and pending-journal behavior.
- Update user-facing durable-session documentation to distinguish the currently released behavior
  characterized by I187 from the ADR-058 target until the implementation merge lands.

### Non-Goals

- No RUNTIME-005 bounded shutdown, SESSION-009 multi-client work, TOOL-024 background jobs,
  permission-policy change, TUI redesign, remote protocol, schema-version bump or data rewrite.
- No raw reasoning, credentials, tool arguments, provider-private payloads, unfinished assistant
  fragments or incomplete tool-call batches in durable state.
- No automatic retry or undo of tool side effects, no inference of terminal outcome from ordinary
  entries and no claim that an interrupted turn completed successfully.
- No historical empty-artifact cleanup or SESSION-010 behavior change.

### Acceptance

- Given a completed display-safe tool exchange before interruption, when the durable Session is
  restarted through the real runtime path, then its admitted messages appear exactly once, the
  normalized outcome is Cancelled, and rebuilt model context contains the same safe facts.
- Given an eligible provider Error after a completed tool exchange, when durable and legacy paths
  resume, then they expose the same canonical safe prefix and an Error outcome without a trailing
  unfinished assistant fragment.
- Given duplicate finalization with the same outcome and canonical filtered payload, when it is
  retried, then original entry IDs are returned and no entry or marker is duplicated.
- Given a different outcome or payload for an already finalized `turn_id`, when it is retried, then
  a structured conflict is returned and the original durable state remains byte-for-byte
  authoritative.
- Given cancellation before any persistable fact, when finalization runs, then only the hidden
  Cancelled evidence is stored; the visible transcript contains no fabricated empty turn.
- Given reasoning, secrets, raw/private tool data, unfinished assistant text or an unclosed
  tool-call batch, when the prefix is finalized, then the existing policy and structural closure
  boundary excludes it from durable transcript and model context.
- Given legacy TLOG v1 files, marker-only Error/Cancelled turns, successful commits, resume, fork
  and pending journals, when read after the change, then compatibility and existing ordering remain
  unchanged.
- SESSION-008-R1: documentation and owner links identify
  `docs/reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md` as the current released-behavior
  truth source and ADR-058 as the target contract until this implementation reaches `main`; no
  target behavior is described as shipped early.
- SESSION-008-R2: no concurrency defect or ENOSPC root cause is claimed without reproduction. If
  the seven transient `talos-session` failures recur, evidence records free disk bytes, inode
  availability, temporary paths, complete stderr and the default-parallel result before diagnosis.

### Planned Validation

- `cargo fmt --all --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --locked -p talos-session`
- `cargo test --locked -p talos-agent session`
- `cargo test --locked -p talos-runtime`
- `cargo test --workspace --locked`
- real durable-runtime interruption and provider-error restart fixtures that assert transcript,
  normalized outcome and rebuilt model context parity
- legacy TLOG, successful-turn, resume, fork, filtering, pending-journal and workspace regressions
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- independent exact-head review of storage atomicity, race semantics, redaction, compatibility and
  runtime reachability

### Documentation To Update

- `docs/backlog/active/SESSION-008-interrupted-turn-partial-persistence.md`
- `docs/decisions/058-partial-turn-durable-finalization.md` only if implementation evidence needs
  an additive status note; the Accepted decision itself is not rewritten
- `docs/reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md` with an explicit supersession
  boundary only after implementation reaches `main`
- affected durable-session/runtime public documentation and README EN/zh-CN behavior claims
- `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md`

### Risks And Rollback

- Risk: finalization races can duplicate entries, overwrite an earlier terminal winner or report
  completion inconsistent with persisted data.
- Risk: the stable-prefix handoff can retain private/raw data or admit a structurally incomplete
  tool exchange.
- Risk: additive public outcome APIs can accidentally break pre-1.0 embedders or TLOG compatibility.
- Rollback: revert the isolated finalizer and runtime integration while retaining the existing
  success-only compatibility path and TLOG v1 data; keep SESSION-008 Partial and RUNTIME-005
  blocked until corrected evidence exists.

## Non-Terminal Coordination Record

- I159-I162 remain Blocked under their existing dependency gates.
- I164 remains Paused and is not resumed.
- I188 and I189 remain Planned / Claimed and unactivated; neither scope overlaps this Session slice.
- ARCH-034-R04 remains Partial and every remaining child, including AG-11, stays independently
  claimable.
- Dashboard and Desktop lanes remain independently governed; this shared contract lands only
  through `main` and no product lane may copy it.
- SESSION-008 remains Ready / Released and B remains unclaimed until this claim reaches `main`;
  RUNTIME-005 remains blocked on B.
- Issues #45, #49 and #59 remain open. Archival PRs #120/#121 remain immutable.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-13 | Selection proposal | I193 publishes the SESSION-008-B target in claim PR #210 and proposes an independent-review claim. No implementation branch or production change is authorized before claim merge. |

## Verification Evidence

- Claim PR #210 pending finalized exact-head validation and independent review.

## Completion Evidence

- No completion evidence. A status-only commit cannot certify this iteration.

## Variance And Residuals

- RUNTIME-005-A/B/C remain blocked until I193 has pre-existing completion evidence.
- SESSION-008-R1/R2 are mandatory implementation and closeout evidence, not optional notes.

## Retrospective

- Pending claim, activation and execution.
