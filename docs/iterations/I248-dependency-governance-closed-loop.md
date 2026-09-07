# Iteration I248: Dependency Governance Closed Loop

> Document status: Review
> Published plan date: 2026-09-06
> Planned objective: Deliver the complete #474 dependency baseline, audit, SOP, and live handoff loop.
> Baseline rule: preserve this scope; a different outcome requires a new iteration.
> MVP deliverable: Bash and PowerShell produce equivalent actionable reports from the same committed baseline and a live registry snapshot, with a generated bounded upgrade candidate and no automatic dependency mutation.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session |
| Work Slice | Full #474 governance mechanism: baseline, cross-platform audit, SOP, live handoff; no dependency upgrade |
| Claimed At | 2026-09-06 |
| Source Issue | #474 |
| Governance Claim PR | #496 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim PR #496 merged as `7c3f6421a073aad62e6a4aaf8f6e1c7df9a674bd`; claim effective. This records the actual merge, not missing historical CAS/review evidence. Implementation requires its own stable-candidate gates. |
| Implementation PR | #497; acceptance follow-up #498 |
| Last Updated | 2026-09-07 |
| Handoff / Release Condition | Complete platform audit acceptance in #498 before closeout; full dependency-version upgrade remains separately governed |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| DEPENDENCY-001 | #474 | Refinement / Unclaimed | release, testing, collaboration and doc-check SOPs | One usable dependency-governance workflow |

### Scope

- Script-owned accepted baseline with source commit and timestamp.
- Direct external dependency collection distinguishing manifest requirements, every lock resolution, baseline, latest stable, users, and policy signals.
- Bash and PowerShell parity for human, JSON and Markdown output, including offline/registry-unavailable behavior.
- Generic aggressive-latest-stable upgrade SOP with isolation, validation, blocker, rollback and baseline-advancement rules.
- Live audit evidence and one generated bounded candidate proposal; no automatic upgrades.

### Non-Goals

- No Cargo.toml/Cargo.lock/version changes, workspace dependency centralization, helper runtime, public API change, or concrete dependency upgrade.

### Acceptance

- Same fixture input yields semantically identical ordered Bash/PowerShell JSON and golden human output.
- Duplicate resolved versions, pre-1.0 minor risk, prerelease, yanked, deprecated, security, baseline drift, upstream drift, malformed input and unavailable registry are distinct outcomes.
- Live report records source SHA/time and produces a candidate Story without mutating the baseline or repository dependencies.

### Planned Validation

- Fixture/golden and mutation tests; Bash/PowerShell parity on supported platforms.
- Explicit live audit with recorded network result.
- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`.
- Both governance validators and `git diff --check`.

### Documentation To Update

- `docs/reference/DEPENDENCY-BASELINE.md`, `docs/sop/DEPENDENCY-UPGRADE.md`, README usage for audit commands, backlog/Board/Issue state.

### Risks And Rollback

- Risk: parser or registry semantics produce false freshness or hide a resolution.
- Rollback: revert the implementation candidate; accepted baseline advances only after a merged governed upgrade.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-06 | Planning | I248 replaces the invalidly widened local I247 attempt; claim remains ineffective until target-main merge. |
| 2026-09-06 | Activation | #496 merged as `7c3f6421`; claim and Active state are effective on target main. |
| 2026-09-07 | Completion | Implementation PR #497 merged as `7b4e21ce6514cc4ce7e79e0a2b2491ffe497be27`; source implementation head `dbd847ec5092d8097d985582c3289692745cf681`, exact-head CI `34043207621`, and independent review `5563452959` were bound before merge. The governance mechanism is complete; the requested full dependency upgrade is owned by a separate iteration. |

## Verification Evidence

### 2026-09-06 Implementation Checkpoint

- Implementation commit `0806f5f1` adds the Bash and PowerShell frontends from effective
  `main@7c3f6421`; no Cargo or dependency files changed.
- `bash -n scripts/dependency_audit.sh`: passed.
- `scripts/dependency_audit.sh --format json`: passed and emits non-empty resolved package entries
  with explicit `registry-unavailable`/partial status.
- `pwsh -NoProfile -File scripts/dependency_audit.ps1 -Format json`: passed against locked offline
  all-feature metadata, preserving manifest requirements, users and unique resolved versions.
- `scripts/test_dependency_audit_parity.sh`: added; compares schema, status and sorted dependency
  identities from both frontends without registry access. It must pass before stable candidate push.
- This is not completion evidence: baseline entries, fixture mutation/parity, live latest comparison,
  and generated candidate handoff remain open.

### 2026-09-06 Change-Control Decision

The request to include one actual dependency upgrade is a scope addition, not an in-scope correction:
the published I248 baseline and Issue #474 explicitly exclude dependency changes. It is assigned to
new planned I249, which depends on I248's live report and owns one candidate only. I248 remains
complete only when its mechanism works without performing an upgrade; I249 cannot activate before
the candidate is selected with fresh evidence.

### 2026-09-06 Inventory And Evidence Correction

Inventory base: `main@e336e438208eebca95576db9fbf245783651d0c1`; candidate before this
checkpoint: `94910d545dbc3dc3ae8615c39e2f0fe4ed22fd32`. Reviewed header variants include
`> Document status`, `**Status**` and plain `Status`, not substring matches against historical
timeline text. The I001-I024 legacy files have no current status header and are not activated by
this proposal; preserve them without inferring a new execution authority.

| Set | Observed disposition | Action |
|---|---|---|
| Active / In Progress / Review / Planned / Blocked on target main | No current explicit iteration header declares these states | No competing iteration is activated or closed by #496 |
| I164 | Paused, explicitly superseded by I165 | Keep paused; do not resume |
| I028, I081-I083, I086-I089 | Superseded before implementation/activation | Preserve terminal disposition |
| Other explicit current headers through I246 | Terminal (some retain dated review wording or old Claimed fields) | Do not reopen; stale metadata is not implementation authority |
| Local I247 | Unpublished invalid activation/scope extension on retained branch | No claim transfer; preserve diagnostic branch; do not merge/replay implementation |
| I248 | Atomic Active / Claimed proposal in #496 only | Not effective before merge; only selected implementation candidate |

PR inventory at the checkpoint: #496 is the only open PR, governance-only; no overlapping
implementation PR. Recheck this and the main SHA during merge-time CAS. This inventory does not
claim a repository-wide closeout-evidence audit of every historical Complete document.

CI run `34016411779` at `94910d54` succeeded. Despite its job name, the Unix job ran
**Reduced documentation validation** and skipped Rust toolchain/preflight steps. It is not Rust
compilation or implementation acceptance evidence and will not transfer to a substantive new head.

### Required Readiness And Behavioral Tests

- Before converting #496 from Draft, resolve the recovery ledger's parser/platform readiness
  question with disposable uncommitted experiments; record actual commands and outcomes here.
  No experiment is accepted implementation or authorization.
- Test actual frontends against Cargo/registry inputs, not grep strings in their expected output.
  Golden output must change when input requirements, users, resolutions or registry data mutate.
- Cover aliases, optional/target/build/dev dependencies, registry identity, every direct resolution,
  malformed/truncated/escaped JSON and unsupported sources with explicit failure/unknown evidence.
- Distinguish baseline and upstream drift; preserve prerelease/build metadata and 0.x compatibility
  semantics; unknown advisory/deprecation coverage must not be reported as no risk.
- Baseline includes generated accepted entries, timestamp, source SHA and prior validation evidence.
  Audit commands cannot bless a candidate automatically. Initial baseline bootstrap must cite
  existing target-main validation; later advancements cite an already-merged upgrade owner.
- Publish schema and exit-code contract; retain deterministic offline tests and explicit partial
  live results. Bash/PowerShell parity needs executable evidence from both paths.
- Before first implementation push, run `./scripts/release_preflight.sh`, focused audit tests,
  explicit-base governance validators and staged-diff review. Live upstream availability is never
  a requirement of ordinary compile/test validation. No change to Cargo run/build default scope.

- Pending claim and implementation.

### 2026-09-06 Read-Only Cargo Metadata Probe

Probe tree: `41209ca7fd2803abcbaeb704bb8b7eeb52782669`, with Cargo files identical to
`main@e336e438208eebca95576db9fbf245783651d0c1`. On the local Unix development host,
PowerShell is available at `/opt/homebrew/bin/pwsh`, awk at `/usr/bin/awk`, curl at `/usr/bin/curl`;
Cargo reports 1.97.0. No new dependency or audit helper was installed.

`cargo metadata --locked --offline --all-features --format-version 1`, parsed with PowerShell
`ConvertFrom-Json`, exited 0. Filtering packages by `workspace_members` produced 22 members and
22 member resolve nodes, with 238 registry dependency declarations. This supersedes the Issue's
historical 21-member snapshot for implementation sizing only, not its historical record.

Following only those member nodes' `deps[].pkg` IDs into `packages[]`, then filtering registry
sources, produced 56 distinct external package IDs and 56 names, with no repeated direct-resolved
names on this tree. This does not remove the multi-resolution acceptance requirement: fixtures
must introduce duplicate versions and prove neither is dropped. The member declarations include
23 optional entries, 5 target-conditioned entries and 1 renamed entry (counts across all sources).

Reproduce the mapping by retaining two independent collections: member `packages[].dependencies`
for requirements/kind/target/optional/rename, and member `resolve.nodes[].deps` for exact package
IDs and dependency kinds. A flat lockfile package-name join is not equivalent. `--no-deps` cannot
prove resolved versions. No Cargo manifest or Cargo.lock changes resulted from the probe.

Readiness still outstanding: bounded Unix JSON parsing (escapes/nesting/malformed input), alias and
kind/target joining, and cross-platform fixture semantics. Availability of PowerShell and a passing
metadata command alone do not establish these. #496 stays Draft until those assumptions are resolved;
this section records a disposable investigation, not implementation or an effective claim.

## Completion Evidence

### 2026-09-06 Local Convergence And Scope Correction

Current implementation remains local on `impl/i248-dependency-governance`; no implementation PR
has been submitted. The dated pre-merge readiness text above is historical, not the current claim
state. Local fixture tests now exercise actual Bash/PowerShell entrypoints for collection,
SemVer, registry failures, baseline comparison, snapshot generation and JSON/table/Markdown parity.
`bash scripts/test_dependency_audit.sh` passed on the local Unix host with PowerShell 7.6.2;
this is not Windows execution evidence or a release-preflight result. Both governance validators
passed with zero warnings, including explicit `COLLABORATION_VALIDATION_BASE=origin/main`, and
`git diff --check` passed after adding the generic dependency SOP and routes.

The user's request is a full dependency upgrade across the long task, not one pilot package.
The earlier change-control interpretation assigning exactly one upgrade to I249 was too narrow.
Preserve that published plan as history; do not count its single-package outcome as satisfaction
of the full request. I248 still owns the mechanism, without Cargo mutations. After its fresh
full audit, allocate the full-upgrade owner/slices under CHANGE-CONTROL with every direct dependency
and relevant transitive update disposed as current, upgraded or explicitly blocked/excepted.
Latest stable includes majors; preserve existing default features and run/build scope.

Outstanding I248 acceptance includes actual accepted-baseline bootstrap with verified provenance,
full live audit and generated candidate handoff, supported-platform evidence, schema/exit contract,
final local preflight and stable-candidate review/CI/merge/closeout. Security/deprecation signals
currently remain explicitly unknown; registry version lookup is not an advisory audit.

### 2026-09-07 Acceptance Audit — Supersedes Earlier Completion Proposal

The preceding dated local checkpoint is preserved as historical evidence. The completion proposal
in this branch was premature: #497's Windows Rust job did not execute the dependency audit scripts.
I248 remains Review/Claimed until its PowerShell frontend has passed on Windows. #498 is the
in-scope acceptance follow-up under the existing #496 claim and collaboration SOP reviewer-follow-up
rule. No new dependency-version authority is established by this correction.

| Required result | Evidence and current disposition |
|---|---|
| Accepted baseline | `docs/reference/dependency-baseline.json`: 67 identities, source `ce3d4cb948f0f5f1a346f87630bad282a03a1704`, acceptance time and CI `33979958221`; Cargo/crates/toolchain diff to #497 merge is empty |
| Live handoff | `dependency-audit-2026-09-06.json` records source `7c3f6421`, command, observation date and all 67 rows; `dependency-upgrade-candidates-2026-09-06.json` contains 34 proposals; neither advances the baseline |
| Schema, errors and parity | `scripts/test_dependency_audit.sh` exercises actual Bash/PowerShell entrypoints, JSON/table/Markdown, malformed metadata, registry failures, snapshots and candidate grouping; #497 independent review `5563452959` records passing local execution |
| Workspace validation | #497 CI `34043207621` passed Unix preflight and Windows Rust checks at `dbd847ec`; this proves Rust compatibility but not Windows audit execution |
| Windows audit acceptance | Pending #498 native PowerShell frontend fixture and workspace collection checks; must record successful run/head before Complete |
| Independent review and merge | #497 review `5563452959`, merge `7b4e21ce`; #498 requires fresh review after correcting finding `5563583120` |

Deprecated/security signals remain unknown as required by the audit contract; ordinary registry
version lookup is not an advisory clearance. The full upgrade remains under planned I250.

- Completion Commit: `dbd847ec5092d8097d985582c3289692745cf681`

## Variance And Residuals

- I247 local branch is rejected evidence and remains preserved for audit only.

## Retrospective

- Outcome: Mechanism merged; Windows audit acceptance remains in Review. Full upgrade remains planned under I250.
