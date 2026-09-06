# Iteration I248: Dependency Governance Closed Loop

> Document status: Active
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
| Authorization Evidence | Draft PR #496 proposes atomic claim+activation; ineffective until target-main merge; independent review pending |
| Implementation PR | Not started |
| Last Updated | 2026-09-06 |
| Handoff / Release Condition | #496 merged as `7c3f6421`; local convergence and one stable candidate |

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

## Verification Evidence

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

- Completion Commit: pending

## Variance And Residuals

- I247 local branch is rejected evidence and remains preserved for audit only.

## Retrospective

- Outcome: pending
