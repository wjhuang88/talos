# I205 PR Workflow Throughput Audit

## Closure Ledger

| Field | Record |
|---|---|
| Requested outcome | Measure recent PR/review overhead, retain evidence-bearing gates, and select the smallest separately claimable workflow simplification. |
| Artifacts | This decision report, `I205-PR-WORKFLOW-EVIDENCE.json`, and `scripts/audit_pr_workflow.py`. |
| Existing assets to preserve | Collaboration claims, protected-scope independent review, exact-head evidence, merge-time CAS, owner-first truth, pre-existing Completion Commit evidence, immutable release tags and GitHub-before-Cargo ordering. |
| State owners | GOV-007 and I205 remain Active/Claimed until this audit PR is independently reviewed, merged and closed owner-first. |
| Validation | Regenerate the JSON through GitHub REST, validate both governance contracts, compile-check the audit script, and run `git diff --check`. |
| Evidence and uncertainty | GitHub PR/comment facts are confirmed by REST. Cause classification and future savings are reasoned interpretations declared below. GraphQL outage observations are local execution evidence and are not recoverable from GitHub REST. |
| Residual destination | A new bounded governance child must claim and implement the selected atomic claim/activation rule. I205 changes no executable rule. |

## Executive Decision

The current workflow is over-segmented and under-automated. The defect is not independent review
itself: several reviews stopped real architecture, permission/sandbox and publication failures. The
throughput failure is that ordinary claim, activation, implementation and closeout state are split
across separate PRs while consistency rules remain manual. Reviewers consequently spend repeated
exact-head rounds finding malformed or contradictory governance text that deterministic checks
should reject before review.

Select **atomic claim activation** as the first separately claimable implementation slice:

- a governance-only claim PR may propose both an effective `Claimed` record and one iteration's
  `Active` delivery state;
- neither has target-branch effect before merge;
- implementation still starts only from the claim merge commit or later current `main`;
- implementation and closure remain separate, and protected implementation keeps independent
  exact-head review.

This changes the normal four-PR chain
`claim -> activation -> implementation -> closeout` to three PRs
`claim+activation -> implementation -> closeout`. It removes one state-only CI/merge boundary
without weakening the ownership boundary that exists before code starts.

## Reproducible Evidence

Run from repository root with authenticated read access to GitHub:

```bash
python3 scripts/audit_pr_workflow.py \
  --output docs/reference/I205-PR-WORKFLOW-EVIDENCE.json
```

The script uses an explicit PR population rather than title/date inference. It reads each PR,
Issue comments and formal review objects through GitHub REST, recognizes explicit review records,
extracts reviewed-head bindings and verifies every classified correction comment still exists.
The generated snapshot is the machine-readable source for the tables below.

### Population And Totals

Snapshot period: 2026-08-14 through 2026-08-18. The population contains ten I159-I205-era
delivery chains, including the I188 and I209 urgent paths and the I203/I204 release path.

| Measure | Result |
|---|---:|
| PRs | 42 |
| Merged | 40 |
| Closed without merge | 2 |
| Implementation or delivery-evidence PRs | 10 |
| Coordination, state, review-record or correction PRs | 32 |
| Explicit review rounds | 37 |
| `REQUEST CHANGES` rounds | 11 |
| Approval rounds | 26 |
| Reviewed-head changes between review rounds | 10 |
| Files reported changed across PRs | 407 |

The implementation/evidence classification is deliberately narrow and disclosed in the script.
It does not claim that every other PR was useless; it shows that 32 of 42 PRs primarily carried
coordination or state rather than the deliverable itself.

### Delivery Chain Table

| Chain | PRs | Delivery PRs | Review rounds | Requests for change | Disposition |
|---|---:|---:|---:|---:|---|
| I159 | 3 | 1 | 5 | 2 | Architecture fact and exact-base validator failures required correction. |
| I160 | 6 | 1 | 8 | 4 | Claim/derived-state work caused repeated baseline, owner and YAML drift. |
| I161 | 8 | 2 | 6 | 3 | Protected implementation review found real sandbox/permission defects; one activation PR was abandoned. |
| I162 | 4 | 1 | 4 | 0 | Four clean claim/activation/implementation/closeout PRs. |
| I188 | 2 | 1 | 1 | 0 | Long-lived decision branch required current-main refresh, then separate closeout. |
| I202 | 3 | 1 | 2 | 0 | Claim, implementation and closeout; no separate activation PR. |
| I203 | 4 | 1 | 5 | 1 | Publication review caught a stale publish guard after the reviewed head changed. |
| I204 | 5 | 1 | 5 | 1 | A pre-activation candidate was rejected and replaced by readiness-only evidence. |
| I205 | 2 | 0 | 0 | 0 | Claim and activation PRs precede this audit implementation PR. |
| I209 | 5 | 1 | 1 | 0 | Planning, claim, activation, urgent implementation and closeout. |

Seven separate activation PRs appear in the sample. They changed 51 files in aggregate, produced
four explicit approval rounds, and produced no blocking review finding. This does not prove an
activation check can disappear; it shows the check can move into claim merge and implementation
ancestry verification without needing its own PR.

## Correction Cause Analysis

The evidence snapshot contains thirteen reviewed cause records. Nine are classified as
mechanically preventable, one as substantive architecture, two as substantive security and one as
substantive release. These are interpretations, not GitHub-native fields; each cites an immutable
comment ID and summary for independent challenge.

### Mechanically Preventable

| Evidence | Failure | Automation that should catch it |
|---|---|---|
| #226 / `5290893479` | Exact-base inventory omitted newly merged I195. | Generate or compare non-terminal inventory against target base. |
| #236 / `5292595210` | Unbound `HEAD^` validation missed an earlier branch edit and falsely reported preflight success. | Resolve the PR merge base by default or require an explicit base; never treat `HEAD^` as branch validation. |
| #238 / `5294558043` | Published baseline mutation, section displacement and stale owner state. | Baseline hashes plus structural owner validation. |
| #241 / `5296769853` | Derived views moved to Review before owners. | Owner/derived-state consistency validation. |
| #241 / `5296887355` | Manifest was invalid YAML and a dated checkpoint was rewritten. | Parse YAML and enforce append-only checkpoint/baseline regions. |
| #247 / `5300311670` | Activation PR used the wrong stale branch ref and was abandoned. | Explicit head ref and base/parent assertion when creating PRs. |
| #258 / `5305824849` | Release candidate work began before the activated readiness boundary. | Branch ancestry and Work Slice path classifier. |
| #273 / `5313751610` | A hand-expanded exact SHA in a comment was wrong. | Generate evidence comments from API values. |
| #279 / `5315276523` | Duplicate Issue row and remote-owner reconciliation forced a docs-only head change. | Generate Issue inventory and make remote reconciliation idempotent. |

### Substantive Review Value

| Evidence | Failure stopped | Gate retained |
|---|---|---|
| #235 / `5291877133` | A Ready decision denied the real `scraper` dependency and would have broken the intended default feature boundary. | Architecture/dependency review against source. |
| #244 / `5300016186` | The security review scope omitted permission Deny precedence and owner invariants. | Independent protected-scope review against the normative owner matrix. |
| #250 / `5301049998` and later rounds | Sandbox fallback permitted unresolved execution and preset composition overwrote caller hardening; a first fix then broke CLI Ask delegation. | Independent exact-head security review and adversarial tests. |
| #264 / `5307808009` | A head change exposed a publish guard that still encoded the old crate boundary. | Exact-head release review and standardized release preflight. |

Removing these reviews would trade visible ceremony for security and release regressions. The
correct optimization is to ensure reviewers receive structurally valid, current-base evidence and
spend their attention on these semantic boundaries.

## Other Churn Sources

### Exact-Head Changes

Six reviewed PRs account for ten distinct reviewed-head changes: #235, #236, #238, #241, #250 and
#264. Four of those PRs mixed real correction with documents that could have been validated before
the first review. #250 and #264 demonstrate why a content-changing head must still invalidate the
old review.

Evidence reuse is therefore allowed only when both Git tree and head SHA are unchanged. A remote
status refresh, CI retry or corrected prose comment that does not move the head may reuse review;
any commit requires rebinding.

### Validator Base Failure

EVOLUTION 49 and #236 confirm the collaboration validator's local fallback compares `HEAD^` when
no target base is supplied. That is not branch validation for a multi-commit PR. During I188
closeout, a multi-commit history also exposed historical source commit `c88c1d1a` as not being an
ancestor reachable from target `main`; PR #283 was transparently reduced to one source commit before final review rather than
using filler commits to make the validator pass. The audit does not change the validator, but the
follow-up automation backlog must remove `HEAD^` as an evidentiary default.

### GitHub Transport Failures

Creating #286 and #287 through `gh pr create` returned GraphQL 503 while the REST endpoint remained
available; both PRs were created through `POST /repos/wjhuang88/talos/pulls`. This is confirmed
local execution evidence, not recoverable from the resulting PR REST objects. It caused retries but
does not justify a governance gate. Operational tooling should use a bounded REST fallback and
record the resulting PR number once.

### Remote Issue Reconciliation

New Issues and status comments can appear while a PR is open. #279 required a docs-only head move
to remove a duplicate row and restore owner reconciliation. I205 itself registered PROMPT-001 for
Issue #285 while activation was in flight. Issue discovery should be generated/idempotent and
should not require unrelated implementation heads to move solely to refresh a derived matrix.

## Gates Retained And Why

| Gate | Evidence / Hard constraint | Decision |
|---|---|---|
| Effective target-branch claim before implementation | Collaboration ownership and AGENTS Goal-Driven/Git rules; prevents duplicate or premature work. | Retain. Claim may activate atomically, but implementation still begins after merge. |
| Independent review for sandbox, permission and process-hardening | AGENTS Hard Constraints 4-5; #250 found three real protected-path failures after CI was green. | Retain unchanged. |
| Exact-head CI/review after content changes | #250 and #264 fixes changed semantics; old conclusions could not carry forward. | Retain; allow reuse only when head is unchanged. |
| Merge-time CAS | #226/#228 show target truth changed while branches were open. | Retain, automate API inputs. |
| Owner-first truth | #238/#241 demonstrate derived views can contradict owners while validators remain green. | Retain and mechanize. |
| Pre-existing Completion Commit | Prevents a status commit from self-certifying behavior; required by AGENTS. | Retain. Closeout automation may cite an existing merge/implementation SHA later. |
| Release preflight, immutable tags, GitHub-before-Cargo | AGENTS Hard Constraint 10 and release contract; #258/#264 show boundary drift. | Retain unchanged. |
| Full Rust checks for Rust/Cargo changes | #236 failed before Cargo ran while prose claimed full validation. | Retain; CI classifier should skip them only for proven text-only paths. |

## Target Scenario Matrix

| Scenario | Target flow | Required review | Completion |
|---|---|---|---|
| Ordinary product work | `claim+activate` -> implementation -> closeout | Existing authorization path; exact-head implementation review/CI. | Separate owner-first closeout cites existing implementation SHA. |
| Protected security work | `claim+activate` -> protected implementation -> closeout | Independent review remains mandatory on the claim boundary when security scope is established and on the exact implementation head. | Security evidence/matrix verified before closeout. |
| Release/publication | `claim+activate` only after readiness gates -> release implementation/execution -> closeout | Exact-head release review, preflight and publication authorization remain explicit. | GitHub Release before Cargo publish; immutable tag and external publish evidence recorded. |
| Bounded maintenance | One PR under existing exception; no iteration activation. | Review proportional to the bounded change; no owner status or protected behavior change allowed. | Merge itself closes the bounded task; widening scope stops and requires a claim. |

## Selected Follow-Up Slice

Create a new governance child and runnable iteration for **atomic claim activation only**. The Work
Slice must be limited to:

1. Amend `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/START-ITERATION.md` and
   `docs/sop/GIT-WORKFLOW.md` so a governance-only claim PR may set one selected iteration Active
   with an explicit "effective on merge" record.
2. Extend `scripts/validate_collaboration_claims.sh` fixtures/checks to require:
   - one complete Claimed record in the same owner;
   - no implementation/Cargo/runtime files in the claim PR;
   - no conflicting Active iteration in the target-base inventory;
   - dependencies and activation gates recorded as satisfied;
   - implementation branch ancestry at or after the eventual claim merge.
3. Add scenario fixtures for ordinary, protected, release and bounded-maintenance paths. The rule
   must not make protected independent review optional or apply iteration activation to bounded
   maintenance.
4. Leave closeout automation, generated Board/index views, YAML parsing, baseline hashing and the
   validator merge-base default to separate later children.

Expected reduction: one PR and one CI/CAS cycle per newly claimed iteration, a 25% reduction from
the current normal four-PR chain. Applied retrospectively to this sample's seven standalone
activation PRs, it would remove 7 of 42 PRs (16.7%) and 4 of 37 explicit review rounds (10.8%) while
retaining all implementation and closeout reviews. This is a counterfactual estimate, not measured
post-change performance.

## Migration And Rollback

### Migration

1. Land the new child claim and its executable SOP/validator change independently of I205.
2. Pilot atomic claim activation on one ordinary, non-release, non-protected iteration.
3. Confirm the implementation branch parent/ancestor is the combined claim merge and no concurrent
   Active owner was introduced.
4. Use the flow for protected/release work only after fixture coverage and at least one clean pilot.
5. Do not rewrite existing owner history. Already effective Planned/Claimed owners may use the
   existing activation flow until a separate migration rule is accepted.

### New Risks And Compensation

| Risk | Compensation |
|---|---|
| An open claim PR displays Active before target ownership is effective. | Mandatory "effective on merge" wording; validators and Board treat target branch as authoritative. |
| Claim merge races another Active owner. | Exact target-base inventory plus merge-time CAS immediately before merge. |
| Implementation starts from the pre-claim base. | Verify implementation branch ancestry against the claim merge before accepting the implementation PR. |
| Protected review is mistaken as satisfied by claim review alone. | SOP and fixtures explicitly require independent exact-head implementation review. |
| Existing Planned/Claimed owners are silently migrated. | Grandfather them; no history rewrite and no automatic status transition. |

### Rollback

Revert the atomic-activation SOP and validator change and return to a separate activation PR. No
product data, release artifact or persistent runtime state requires migration. Owners activated
under the combined rule remain valid because their Claimed and Active records already exist on
`main`; only future transitions revert to the old sequence.

## Deferred Improvements

The evidence supports additional children, but combining them with atomic activation would recreate
the large governance slice I205 was designed to avoid:

- default collaboration validation to the actual target merge base instead of `HEAD^`;
- parse `.agent-governance/manifest.yaml` and validate owner/derived-view state consistency;
- preserve published-baseline hashes and append-only checkpoint regions mechanically;
- generate Board, iteration index and Issue matrices from owner records;
- automate post-merge closeout preparation while retaining existing Completion Commit evidence;
- provide bounded REST fallback for GitHub GraphQL PR creation failures.

These remain recommendations until separately owned and claimed. I205 authorizes none of them.
