# Iteration I181: Native And Panic-Boundary Security Review

> Document status: Planned - corrected exact-head independent approval pending
> Published plan date: 2026-08-08
> Planned objective: independently review every recorded native, panic-capable, and subprocess boundary in ARCH-034-R04, reconcile governing ADR facts, and convert only proven gaps into bounded follow-up owners without changing production behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an independently approved call-site/failure-mode/containment/test disposition that is mechanically traceable to current source and names a safe fallback and separate implementation owner for every accepted gap.
> Infrastructure-only exception: this iteration produces a security-review artifact and runnable controlled-failure validation plan; it makes no user-behavior or remediation claim and changes no production/test/dependency/security-policy code.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-08 |
| Work Slice | Independently review the ARCH-034-R04 native/panic boundary matrix covering ADR-007 libc and process hardening, subprocess families, arborium/tree-sitter, `gix`, and bundled SQLite; classify gaps and create bounded follow-up owners only; preserve all runtime/API/dependency/permission/sandbox/process-hardening/unsafe/storage behavior. |
| Claimed At | 2026-08-08 |
| Source Issue | None |
| Governance Claim PR | #174 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #174 review `PRR_kwDOSrj_LM8AAAABI2KjFw` analyzed commit `24694b88` but was submitted as `COMMENTED` through @wjhuang88 and explicitly disclaims authorization; a different GitHub identity must approve the corrected exact head. |
| Implementation PR | Not started |
| Last Updated | 2026-08-08 |
| Handoff / Release Condition | Remain Planned and unmerged until an independent security reviewer approves the exact claim; rejected gaps are recorded and released, while accepted gaps receive separate bounded implementation owners/claims. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. This protected claim cannot use
single-maintainer authorization. A proposed `Claimed` record has no effect until independently
approved and merged into `main`.

## Closure Ledger

| Dimension | Record |
|---|---|
| Requested outcome | Advance the remaining ARCH-034 security finding without bypassing independent review or pre-authorizing unknown implementation. |
| Artifacts to create/update | I181, ARCH-034-R04 matrix/claim, ARCH-034 parent, iteration index, Product Backlog, Board, August audit register, and governance manifest. |
| Existing assets to preserve | ADR-007/008/020 text and semantics, current runtime/public behavior, permission gates, process limits, storage formats, I159-I162 blockers, and recovery PRs #120/#121. |
| State/status owners | ARCH-034-R04 and I181 first; ARCH-034 parent and derived indexes/views second. |
| Validation required | Source/call-site trace, both governance validators, architecture audit, scale assessment, `git diff --check`, claim-only diff review, and exact-head CI. |
| Evidence and uncertainty | Source and failure-mode facts now include the non-authorizing review's independently reproduced F-A/F-B/F-C evidence; its dispositions remain provisional because GitHub records the submission under the PR author's identity. |
| Residual-work destination | One new bounded owner/claim per independently accepted implementation gap; ARCH-034-C remains after R04 disposition. |

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Unchanged. I161 separately requires an independent security-review plan and cannot be activated through I181. |
| I164 | Paused | Historical superseded target; no activation. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable; no implementation or review authority. |
| I180 / ARCH-034-R11 | Closed with evidence | Existing completion commit `10cceec6a`; no overlap with R04 security semantics. |
| ARCH-034-R04 | Refinement at selection | Selected for review-only planning; no protected implementation is selected. |
| ARCH-034-C | Refinement | Remains gated on B/R04 disposition and is not selected. |
| Other Ready/Refinement backlog items | Unselected | Existing owners remain unchanged; none overlaps this review-only Work Slice. |

No other Active, Review, or Planned iteration exists. I181 is selected after I180 and the
ARCH-034 owner-truth repair merged because R04 is the only remaining remediation finding, but its
hard security gate prohibits implementation before independent review.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R04 review phase | ARCH-034 | Refinement | I171 audit, R02/R03/R05-R11 closure, ADR-007/008/020, independent reviewer | One approved/rejected/needs-evidence disposition per boundary family and one bounded follow-up owner per accepted gap. |

### Scope

- Verify the R04 call-site/failure-mode/containment/test matrix against current `main`.
- Review escape vectors and failure modes for parent hardening, Unix `pre_exec`, direct exec,
  other subprocess families, both arborium/tree-sitter consumers, both `gix` consumers, SQLite,
  and existing panic adapters.
- Reconcile factual drift in ADR-007 and ADR-008 through a proposed decision update only when the
  independent reviewer accepts the change; ADR semantics are not changed by the claim PR.
- Record each row as Accepted gap, Rejected gap, Adequately contained, or Needs evidence.
- For every accepted gap, define one safe fallback, controlled-failure fixture, affected platform,
  protected surface, rollback, and separate owner/claim gate.

### Non-Goals

- No production or test code, Cargo manifest/lockfile, dependency, permission, sandbox,
  process-hardening, `unsafe`, storage schema/data, public API, runtime behavior, or security-policy
  change.
- No single-maintainer merge, self-approval, speculative catch-all `catch_unwind`, silent fallback,
  or broad “fix all native code” implementation claim.
- No I159-I162 activation, ARCH-034-C implementation, release/tag/publish action, or recovery PR
  modification.

### Acceptance

- An independent security reviewer validates the exact source matrix and records escape-vector and
  failure-mode analysis on the claim/review PR.
- Every matrix row has confirmed call sites, current containment, controlled-failure evidence, and
  an explicit disposition without overstating unknown severity.
- Every accepted gap names a safe fallback and a separately claimable implementation/test slice;
  rejected or contained rows record evidence.
- ADR-007/008/020 facts are either confirmed current or assigned a separately reviewed decision
  amendment; no policy meaning changes implicitly.
- Both governance validators, architecture audit, scale assessment, diff checks, and exact-head CI
  pass for the review artifacts.

### Planned Validation

- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `scripts/audit_architecture.py .`
- `scripts/assess_project_scale.sh .`
- `git diff --check`
- Claim-only path/diff inspection proving no Rust, Cargo, dependency, test, ADR semantic, or
  protected implementation change.
- Independent review of controlled-failure fixtures proposed for process, git, symbol, SQLite,
  permission, and crash containment.
- Exact-head Unix/Windows CI and remote owner reconciliation.

### Documentation To Update

- Update ARCH-034-R04 and I181 as owners, then synchronize ARCH-034, iteration/backlog indexes,
  Board, August audit register, and governance manifest.
- No product/user documentation change is expected because I181 changes no user behavior.

### Risks And Rollback

- Risk: a planning artifact could accidentally be treated as security approval, broaden ADR
  exceptions, or authorize protected code before review.
- Rollback: close/release the unapproved claim and restore R04 to Refinement. Retain the factual
  matrix as audit evidence only; never infer approval from CI or document existence.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-08 | Planning | I181 selected after I180/PR #171 and owner-truth PR #173 closure, non-terminal iteration inventory, open-PR overlap check, and read-only native/panic call-site tracing. Claim and independent approval remain pending. |
| 2026-08-08 | Review follow-up | Review `PRR_kwDOSrj_LM8AAAABI2KjFw` recommended approval analytically but GitHub rejected self-approval and stored it as `COMMENTED`. F-A (unused public parent-mutation API), F-B (post-timeout pipe wait can be unbounded), F-C (symlink-cycle stack overflow and uncapped file reads), and stale ADR-007 site-5 facts were independently reproduced and added to R04. The authorization gate remains open. |

## Verification Evidence

- Review `PRR_kwDOSrj_LM8AAAABI2KjFw`: useful analytical evidence, but not authorization because its
  GitHub author is @wjhuang88 and its state is `COMMENTED` rather than `APPROVED`.
- Corrected local review-follow-up diff: `git diff --check`, both governance validators (0
  warnings), `python3 scripts/audit_architecture.py .` (21 crates, 0 cycles, 5 unsafe lexical
  candidates), and `scripts/assess_project_scale.sh .` (high-risk / release-managed / on-demand)
  passed on 2026-08-08.
- Corrected exact-head CI and approval by a different GitHub identity remain pending.
- Runtime evidence is intentionally not applicable before accepted implementation slices exist.

## Completion Evidence

- Not applicable while I181 is Planned. A future Complete status requires an already-existing
  independently reviewed evidence commit; this planning record does not claim completion.

## Variance And Residuals

- ARCH-034-C remains Refinement until R04/B has a truthful disposition.
- Any accepted implementation gap requires a new bounded owner/claim after I181; it cannot be
  implemented on the claim/review branch.

## Retrospective

- Outcome: mandatory factual review corrections incorporated; independent approval still pending.
- Documentation: owner-first synchronization pending claim PR.
- Lessons: pending; promote a rule only if the review exposes a recurring governance failure.
