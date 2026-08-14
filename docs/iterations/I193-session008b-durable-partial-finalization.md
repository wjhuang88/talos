# Iteration I193: SESSION-008-B Durable Partial-Turn Finalization

> Document status: Complete
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
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No separate natural-person reviewer is available. The maintainer may use the Single-maintainer merge path only after exact-head CI, both governance validators, dependency/overlap CAS, and a documented non-authorizing technical audit show no unresolved blocking feedback. Role separation (author/executor, technical auditor, merge authority) must be disclosed, but roles do not impersonate distinct natural persons. |
| Implementation PR | #216 |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Merged through the disclosed Single-maintainer path; retain RUNTIME-005 and I188/I189 boundaries. |

The `Claimed` record became effective when PR #210 merged into `main` as
`fb5a1f62aed7d86657473fa766876045724f6419`. I193 was activated from `main@f778543c` and its
implementation merged through PR #216 as `1b5461cdcb03c7a896b814ccad2d93aa44010fc6`.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SESSION-008-B | SESSION-008 / Issue #45 | Complete | SESSION-008-A Complete at `e288afb5`; ADR-058 Accepted | One durable finalization and replay path for admitted Success/Error/Cancelled turns |

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
- SESSION-008 and B are complete with the implementation evidence below; RUNTIME-005 retains its
  owner-defined A/B/C gates and only its B dependency on SESSION-008-B is now satisfied.
- Issues #45, #49 and #59 remain open. Archival PRs #120/#121 remain immutable.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-13 | Claim merge | PR #210 merged as `fb5a1f62aed7d86657473fa766876045724f6419` through the documented Single-maintainer merge path. The claim is effective; I193 remains Planned and no implementation branch has been created. |
| 2026-08-13 | Activation | Activated from exact `main@f778543c7ceeb2a099eb3863fc8259da68d02195` in independent worktree `/private/tmp/talos-i193` on `feat/session-i193-partial-finalization`. I194 remains separately Planned/Claimed after PR #211; I188/I189 remain Planned/Claimed and unactivated; I159-I162 remain Blocked; I164 remains Paused. Dashboard PR #212 is separate and must refresh its own target-branch base. |
| 2026-08-14 | Implementation merge | PR #216 merged at `1b5461cdcb03c7a896b814ccad2d93aa44010fc6`; source implementation commit `404d7a4bf5b9c7dedeae479fe91fa5400b42d411` pre-existed this status record. |

## Verification Evidence

- Claim PR #210 exact head `f7199120` passed CI and the documented Single-maintainer merge CAS;
  claim merge `fb5a1f62` is now on `main`.
- Implementation worktree validation on 2026-08-13 passed `cargo check --workspace --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --locked -p talos-session`, `cargo test --locked -p talos-agent session`,
  `cargo test --locked -p talos-runtime`, and `cargo test --workspace --locked`.
- Real actor/durable-session fixtures cover provider Error and user Cancelled after a completed tool
  exchange, reopen the Session, and assert the closed prefix plus explicit incomplete outcome.
- Durable finalizer fixtures cover identical retry, conflicting outcome/payload with byte-identical
  preservation, ambiguous legacy-entry rejection, empty-prefix marker-only cancellation, and
  reasoning/secret/tool-output filtering.
- SESSION-008-R1 remains explicit: I187 describes the pre-I193 released behavior and ADR-058 is the
  target contract until this implementation reaches `main`. SESSION-008-R2 did not trigger: the
  seven transient failures did not recur during default-parallel workspace validation, so no
  concurrency or ENOSPC diagnosis is asserted.
- Final exact-head CI run `31691761892` passed all five checks, including Windows workspace;
  role-separated non-authorizing technical audits are PR comments `5287961007` and `5287989820`.
  R1 now preserves I187 as the pre-I193 baseline and recognizes ADR-058 as implemented on `main`.

## Completion Evidence

- Completion Commit: `404d7a4bf5b9c7dedeae479fe91fa5400b42d411` (pre-existing implementation source
  commit; the status commit is not used as its own evidence).
- Merge evidence: PR #216 squash merge `1b5461cdcb03c7a896b814ccad2d93aa44010fc6`, exact final
  head `c8bc33ab5c6a4a72a14a0a4f402488fa386a5b67`, CI `31691761892`.

## Variance And Residuals

- RUNTIME-005 remains Refinement / Unclaimed with no selected iteration. RUNTIME-005-A remains
  Ready / not selected and depends on SESSION-008-A decision output plus the completed RUNTIME-001
  API; RUNTIME-005-B remains Blocked on RUNTIME-005-A Accepted plus SESSION-008-B Complete; and
  RUNTIME-005-C remains Blocked on RUNTIME-005-B Complete.
- SESSION-008-R1/R2 are mandatory implementation and closeout evidence, not optional notes.

## Retrospective

- I193 closed the ADR-058 durable finalization gap without changing TLOG v1, activating I188/I189,
  or advancing unrelated runtime/permission/product lanes. I187 remains historical evidence.
