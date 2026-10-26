# Iteration I309: Upgrade gix To Latest Stable 0.90.0

> Document status: Active (proposed in #774; ineffective until merge)
> Published plan date: 2026-10-26
> Planned objective: upgrade gix 0.87.1 to the latest observed stable, non-yanked 0.90.0 while preserving existing Git behavior and permission boundaries.
> Baseline rule: preserve this target; append execution facts rather than rewriting acceptance.
> MVP deliverable: existing embedded Git status/diff/revision/worktree operations run against gix 0.90.0 with validated locked dependency resolution.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | I309: gix 0.87.1 to 0.90.0 dependency graph, necessary consumer migrations, Git regression validation and dependency documentation; excludes Auto policy and search/provider implementation. |
| Claimed At | 2026-10-26 |
| Source Issue | None; explicit maintainer request in handoff session |
| Governance Claim PR | #774 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer explicitly requested latest gix upgrade; claim requires exact-head review, applicable CI, both validators and merge-time CAS. |
| Implementation PR | Not started |
| Last Updated | 2026-10-26 |
| Handoff / Release Condition | Verified upgrade and existing implementation commit evidence; no release authorized. |

Claim and activation are ineffective until the finalized governance-only record merges to main.
No dependency or source implementation belongs in this candidate.

## Selection Inventory And Disposition

Inventory at main `881a27b890e5310055380e6e3f5b0da78f9fdd3c`:

| Owner | State | Disposition |
|---|---|---|
| I249 | Planned | Retain broad dependency-pilot deferral; I309 selects only the newly requested gix upgrade, not that pilot. |
| I277 | Review | Retain deferred human/device acceptance. |
| I293 / I294 | Active | Retain search transport/provider evidence ownership; no search changes. |
| I302 | Review | Retain Windows native-resource residual #717. |
| I306 | Active | Retain provider protocol ownership; no provider changes. |
| I308 | Active / Partial | Retain T8 scope and stable Runtime candidate; dependency upgrades are excluded from its baseline. I309 runs as the explicitly requested independent dependency stage, without changing I308 policy. |
| I164 | Paused | Retain superseded startup target. |
| I300 | Complete | Preserve T7 completion and historical gix observations; do not reopen it. |

Open PRs #767 (search), #749 (provider), #705 (Windows diagnostics) do not own the gix graph.
Refresh this check before merge. Sequence source work after the Runtime candidate is stable;
do not mix changes or claim its old validation covers upgraded dependencies.

## Published Baseline

### Selected Work

Standalone maintainer-requested dependency upgrade; no new product feature or host-Git replacement.

### Scope

- Upgrade the tools optional gix declaration and affected root/fixture locks without deleting locks.
- Preserve `default-features = false` and existing basic/status/revision/blob-diff/index/sha1/sha256 feature intent.
- Migrate only APIs required by the new version; inspect feature/native/license/advisory/duplicate changes.
- Verify crate MSRV (workspace 1.95) independently from pinned development compiler 1.99.0.
- Update operator-facing Git/dependency documentation with actual results and remaining host-Git boundary.

### Non-Goals

- No removal date or claim that gix replaces host Git, new unsafe code, new direct dependency, Auto-policy redesign, search/provider changes, release or tag.
- No upgrade of unrelated direct dependencies or advancement of an accepted audit baseline before implementation merge.

### Acceptance

1. Manifest and locked resolution use observed latest stable gix 0.90.0; all consumer and SDK fixture locks remain coherent.
2. Existing status, diff, revision and linked-worktree fixtures pass; SHA-1/SHA-256 coverage and structured Git permission behavior remain intact.
3. Default/minimal/git feature combinations, required workspace/all-feature checks and regression suites pass. Windows cross-build uses cargo xwin; native CI is distinct evidence.
4. Exact graph, MSRV, feature/native/license/advisory findings and rollback are recorded honestly; unknown advisory coverage stays unknown.
5. Independent security/API review, exact-head CI and merge CAS pass; owner completion cites an already-existing implementation SHA.

### Planned Validation

- `bash scripts/dependency_audit.sh --format json` before/after; `bash scripts/test_dependency_audit.sh`.
- `./scripts/release_preflight.sh` with locked workspace checks/test contracts.
- `cargo check --workspace --all-targets --all-features --locked`.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- `cargo test --workspace --all-features --locked`.
- Affected tools no-default/default/git feature checks/tests; runtime SDK fixture; cargo xwin Windows check and native platform CI.
- Both governance validators and staged-diff inspection; domain regression coverage above.

### Documentation Targets

- This owner, `docs/iterations/README.md`, `docs/BOARD.md` and the existing Git user/reference documentation identified during implementation.

### Risk And Rollback

Pre-1.0 minor releases may break APIs and expand transitive graph. Keep dependency resolution
and necessary source migrations separately reviewable/revertible; revert the upgrade stage to
restore 0.87.1 without altering T7/T8 policy. Do not conceal a failed validation by narrowing scope.

## Evidence And Execution

- Initial observation: clean main source `881a27b890e5310055380e6e3f5b0da78f9fdd3c`; current manifest/lock gix 0.87.1.
- crates.io sparse index `https://index.crates.io/3/g/gix`, fetched 2026-10-26: latest stable non-yanked 0.90.0, declared Rust 1.88. This is registry evidence, not a validated graph/MSRV result.
- Upstream release: https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.90.0 (2026-10-22).
- Dependency graph migration, runtime validation and completion remain unimplemented.
