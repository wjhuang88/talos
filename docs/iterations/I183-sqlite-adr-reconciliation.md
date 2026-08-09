# Iteration I183: Bundled SQLite ADR Reconciliation

> Document status: Planned
> Published plan date: 2026-08-09
> Planned objective: reconcile ADR-008 with all five existing direct workspace consumers of bundled SQLite—four runtime crates plus quarantined non-runtime `talos-models`—and add a repository validator that rejects an unapproved sixth direct consumer, without changing dependencies or runtime behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: maintainers can run one validator that proves the resolved native-consumer set exactly matches the accepted ADR inventory, `rusqlite`/`libsqlite3-sys` each resolve to one version, and `talos-models` remains isolated, while the decision and architecture references explain the purpose, ownership and runtime/quarantine classification of every accepted consumer.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 architecture session 2026-08-09 |
| Work Slice | Implement only ARCH-034-R04-AG7: inventory the five current direct workspace consumers of bundled SQLite (four runtime consumers plus quarantined non-runtime `talos-models`), reconcile ADR-008 and architecture/decision references to an exact accepted allowlist with that distinction, and add a cross-platform validator over parsed locked Cargo metadata that counts a workspace package only when it has a direct edge to a non-workspace package that transitively reaches `libsqlite3-sys`, rejects an unexpected or missing accepted consumer, rejects multiple resolved `rusqlite`/`libsqlite3-sys` versions, and rejects any workspace package that depends on `talos-models`; wire it into standard governance validation and synchronize owner/iteration/Board evidence; do not change Cargo manifests, Cargo.lock, dependencies, Rust source, schemas, migrations, database behavior, the `talos-models` quarantine or any other R04 child. |
| Claimed At | 2026-08-09 |
| Source Issue | None |
| Governance Claim PR | #183 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent review `5231879125` approved exact head `360576c9c32f5335c36185368051152432ad6e5a` with no blockers and disclosed that a distinct natural-person reviewer used the shared `@wjhuang88` account. The review independently confirmed the five-consumer 4+1 classification, diff purity, inventory, validators and CI `31316076588`, then requested four non-blocking baseline clarifications. Review `5231992214` bound to amended head `0284f0f334a3f9dd85e251edb9d04e19e05936af` found one blocking consumer-boundary contradiction while reconfirming scope, inventory and governance; this claim adopts its boundary and edge-kind corrections, and the next exact head still requires independent approval before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Proposed claim is ineffective until PR #183 merges after independent exact-head architecture approval and merge-time CAS. Start implementation only from that merge or later `main`. |

This proposed `Claimed` record has no ownership effect until it is merged to `main`.

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Unchanged under their published TUI-037, sequential dependency, security-review and release-readiness gates. |
| I164 | Paused / superseded | Remains historical; I165 owns its replacement layout target and is already terminal. |
| ARCH-034-R04 | Partial | Select only AG-7; completed AG-4 remains closed and AG-1/2/3/5/6/8/9/10 remain unclaimed. |
| I181 / I182 | Terminal | Preserve review evidence `aea26ad0` and AG-4 implementation evidence `ae31242b`; neither grants AG-7 authority. |
| Recovery PRs #120/#121 | Open archival evidence | Keep immutable and unmerged; both are non-overlapping. |
| Other Active/Review/Planned/Blocked iterations | No competing selectable implementation | Preserve published owner states; do not activate or supersede another owner through I183. |

No existing branch, open claim PR, implementation PR or effective claimant was found for AG-7/I183.
The only open PRs at selection were archival recovery PRs #120/#121. Remote branches not merged to
`main` are historical provenance and do not override target-branch owner truth.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R04-AG7 | ARCH-034-R04 | Ready | I181 disposition, bounded AG-7 owner and independently reviewed effective claim | ADR-008 and repository validation name exactly five accepted direct workspace consumers: four runtime crates plus quarantined non-runtime `talos-models`. |

### Scope

- Inventory runtime consumers `talos-session`, `talos-evolution`, `talos-exploration`, and
  `talos-memory`, plus quarantined non-runtime `talos-models`: purpose, owning module,
  runtime/quarantine classification, schema/migration surface, and existing corrupt, busy/locked
  and fallback coverage.
- Amend or supersede ADR-008 so each existing consumer is explicitly accepted or rejected and any
  future consumer still requires an ADR update rather than inheriting broad SQLite authority.
- Add one cross-platform repository validator over parsed `cargo metadata --locked` output. A
  workspace package is a consumer if and only if it has a direct dependency edge to a non-workspace
  package that transitively reaches `libsqlite3-sys`; reachability only through other workspace
  packages is layering, not an additional consumer. Do not infer the native boundary from literal
  `rusqlite` or `bundled` manifest text.
- Include resolved normal, build, development and target-specific dependency edges in that rule;
  the native-dependency policy applies to workspace source, build and test surfaces on every target.
- Fail when the resolved boundary-consumer set differs from the accepted five, when more than one
  `rusqlite` or `libsqlite3-sys` version is resolved, or when any workspace package has a direct or
  transitive dependency on quarantined `talos-models`.
- Wire the validator into the standard project-governance path and synchronize the decision index,
  architecture/dependency references, AG-7, R04, iteration index, backlog, Board and governance
  manifest.

### Non-Goals

- No Cargo manifest, dependency resolution, schema, migration, database file, timeout, retry,
  busy policy, panic handling, storage path or runtime code change.
- No assertion that the existing SQLite panic/deadline/corrupt/busy behavior is uniformly safe.
- No sixth consumer, SQLite replacement, system-SQLite mode, `talos-models` runtime activation,
  crate publication or release action.
- Any containment inconsistency found by the inventory becomes a separately owned residual.

### Acceptance

- Given the locked workspace, when the inverse `cargo tree --locked -i libsqlite3-sys` is inspected
  at each workspace-to-non-workspace boundary and the new validator runs, then both identify exactly
  the same five accepted direct consumer crates, while documentation separately identifies four
  runtime consumers and quarantined non-runtime `talos-models`.
- Given a sixth workspace package has a direct edge to a non-workspace `rusqlite`, `sqlx`, `libsql`
  or other dependency that transitively reaches `libsqlite3-sys`, when the validator runs against a
  controlled metadata fixture, then it fails with the unexpected crate and directs the maintainer
  to update the accepted ADR first.
- Given one accepted consumer is removed or loses the bundled dependency, when the validator runs,
  then it fails as policy drift rather than silently narrowing the recorded exception.
- Given multiple resolved `rusqlite` or `libsqlite3-sys` versions, when the validator runs, then it
  fails ADR-008 clause 4 and reports every conflicting version.
- Given any workspace package gains a direct or transitive dependency on `talos-models`, when the
  validator runs, then it fails the quarantined non-runtime classification even if the SQLite
  consumer count remains five.
- Given the five current consumers, when ADR-008 and the inventory are reviewed, then every crate's
  purpose, ownership, schema/migration surface and known failure coverage are explicit without
  claiming runtime remediation.
- Given the implementation diff, then no manifest, lockfile or Rust source changes are present and
  runtime behavior remains unchanged.

### Planned Validation

- New SQLite-consumer validator positive and controlled negative metadata fixtures on Unix and
  Windows: unexpected boundary crossing by alternate normal/build/development or target-specific
  edge, workspace-only layered reachability, missing accepted consumer, duplicate native versions,
  and a workspace dependent of `talos-models`.
- `cargo tree --locked -i libsqlite3-sys`
- ADR/document-link checks and `git diff --check`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `./scripts/release_preflight.sh`
- Exact-head Unix/Windows CI, independent architecture review and merge-time CAS.

### Documentation To Update

- `docs/decisions/008-sqlite-bundled-storage.md` and `docs/decisions/README.md`
- Preserve the existing correct 4+1 classification in `docs/reference/ARCHITECTURE.md`; add only
  the accepted allowlist/validator pointer and the focused five-consumer inventory produced by I183
- AG-7/R04 owner chain, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`,
  `docs/BOARD.md` and `.agent-governance/manifest.yaml`

No user-facing feature guide changes are required because I183 changes policy truth and validation,
not product behavior. The decision and architecture references are the affected user-facing
maintainer documentation.

### Risks And Rollback

- Risk: a manifest-text scan could miss alternate dependency paths or accept commented,
  target-specific or dev-only declarations.
  Rollback: traverse the resolved graph from parsed locked Cargo metadata and reject any fixture
  whose native reachability, versions, or quarantine edges differ; do not merge a regex-only or
  direct-`rusqlite` approximation.
- Risk: broad ADR wording could authorize arbitrary future consumers.
  Rollback: keep an exact allowlist and require an accepted ADR change before the validator list
  changes.
- Risk: inventory findings could expand into runtime remediation.
  Rollback: register each such finding as a separate owner and keep I183 documentation/validation
  only.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-09 | Selection | Current `main@20a09a47`; I159-I162 remain Blocked, I164 remains Paused/superseded, no Active/Review implementation iteration exists, and open PRs #120/#121 are immutable archival evidence. PR #183 proposes the only AG-7/I183 claim; it remains ineffective until independently approved and merged. |

## Verification Evidence

- Initial draft passed both governance validators and `git diff --check`; finalized exact-head CI
  and independent architecture review remain pending. No implementation has started.

## Completion Evidence

- Not applicable while Planned. A later closure must cite an already-existing implementation merge
  SHA and must not use its own status-only commit as evidence.

## Variance And Residuals

- None at publication. Runtime SQLite containment remains outside I183.

## Retrospective

- Pending execution.
