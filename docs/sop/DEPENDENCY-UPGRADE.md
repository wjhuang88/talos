# SOP: Dependency Upgrade

## Purpose And Authority

Maintain the latest stable dependency set through repeatable evidence, not a manually curated
version list. This procedure applies to future upgrades generally; a historical audit is not a
permanent upgrade plan. An audit or generated snapshot grants no implementation authority.

Use [requirement intake](REQUIREMENT-INTAKE.md), [iteration selection](START-ITERATION.md),
[collaboration](AGENT-COLLABORATION.md), [implementation](ITERATION-WORKFLOW.md),
[change control](CHANGE-CONTROL.md), [testing](TESTING.md), and
[documentation synchronization](DOC-CHECK.md). Those procedures remain authoritative; do not
create a separate dependency claim database. Release/tag/publication remains separately governed
by [release routing](RELEASE.md).

## 1. Capture Evidence

Record the source commit, clean/dirty state, UTC observation time, pinned development toolchain,
package MSRV and audit command. Cargo manifests are requirement/build truth; Cargo.lock is exact
resolution truth; an accepted baseline is a previously validated snapshot. Latest registry data
is an observation, never accepted build truth. Capture all four independently.

Run local collection without network:

```bash
bash scripts/dependency_audit.sh --format json
```

```powershell
pwsh -NoProfile -File scripts/dependency_audit.ps1 -Format json
```

For upstream evidence, add `--live` or `-Live`. For reproducible replay use `--registry FILE` or
`-RegistryPath FILE`, containing a package-name-to-crates.io-response JSON object. A missing/null
response means unavailable; malformed version evidence means unknown. Do not reinterpret either
as current. `--metadata FILE` / `-MetadataPath FILE` supplies captured complete Cargo metadata;
without it, the command uses locked, offline, all-feature Cargo metadata.

Compare an accepted JSON snapshot with `--baseline FILE` / `-BaselinePath FILE`. The audit never
writes that file. Baseline drift means requirements, consumers or resolutions changed relative to
accepted evidence; upstream drift means a newer stable release exists. Both may be present.

The direct-external-dependency inventory is the default surface, including optional, build, dev,
target-specific, renamed and multiple-resolution declarations. Transitive packages are not a
second manually maintained baseline. Inspect their changes during upgrade validation, particularly
duplicates, advisories, native components and feature closure. Git/unsupported registry sources
require explicit investigation rather than silent omission.

## 2. Choose Latest Stable, Then Determine Isolation

Latest stable is the default target, including major releases. Risk determines isolation and
validation, not whether to attempt an upgrade. Exclude yanked releases from the target. Treat
prereleases as explicit exceptions. A pre-1.0 minor boundary, and a patch boundary on `0.0.x`, can
be breaking. A `current` version classification says nothing about security or deprecation:
unknown signals require investigation, not a claim of no advisories.

Compatible changes may be grouped when validation surfaces overlap, failure attribution remains
clear, and the group can be reverted. Isolate changes involving breaking versions, public API,
protocol/provider behavior, runtime lifecycle, persistence, permissions/sandbox, native/unsafe
code, substantial source migration, large graph/binary changes or independently required rollback.
Do not create one PR per editing attempt: converge each authorized slice locally before pushing
a stable candidate. Nor should a full upgrade request be silently reduced to one pilot package.

Record every audited item as current, selected, or explicitly blocked/excepted. The upgrade owner
records old/new manifest and all resolved versions, source SHA/time, registry evidence, affected
consumers, expected graph changes, exclusions, validation and rollback. Allocate the existing
Story/iteration/claim records before implementation. Dependency centralization is a separate
refactor unless specifically authorized.

## 3. Implement And Validate Locally

Preserve default features, default workspace members and normal `cargo run`/`cargo build` scope
unless a separately accepted change authorizes them. Update all affected declarations consistently;
do not delete Cargo.lock or relax constraints merely to get a green build. Compare package MSRV
against the new dependency graph independently of the pinned development compiler.

Use `./scripts/release_preflight.sh` and the current testing contract. For a dependency candidate,
also exercise all-target/all-feature locked check and Clippy, all-feature workspace tests, and
affected minimal/default/optional feature combinations. Record exact commands/results rather than
claiming a broad validation from one package test. Run deterministic audit tests with
`bash scripts/test_dependency_audit.sh`; these tests must not query crates.io. Ordinary compilation
CI must not depend on live latest-version discovery.

Derive domain checks from real consumers and changed dependency APIs:

| Surface | Evidence to derive |
|---|---|
| Protocol/provider/network | Client/server/transport fixtures, streaming, timeout, retry and failure paths |
| Runtime/plugin/native | Lifecycle, traps/panics, cancellation, resource limits and safe fallback |
| Persistence | Migration, restart, compatibility, corruption and failure recovery |
| Git/repository | Status, diff, revision and worktree fixtures |
| TUI/rendering | Rendering fixtures and real-terminal acceptance when visible behavior changes |
| Permission/sandbox | Existing mandatory independent security/adversarial review |
| Published APIs | Semver assessment, ADR/migration plan for breaking Talos APIs, consumer/package checks |

Inspect lockfile graph changes, duplicate versions, licensing/security evidence, feature closure,
generated residues and staged diff. New native/unsafe boundaries still require existing ADR gates.
Unknown advisory coverage must remain visibly unknown. Obtain the authorization-specific review
and exact-head CI for the stable candidate; repeat merge-time CAS before merging.

## 4. Blockers, Exceptions And Rollback

A blocked latest-stable upgrade records an owner, exact versions, failed command or upstream
incompatibility, impact, temporary accepted version, mitigation and concrete revisit trigger.
Never silently convert an old version into permanent policy. A prerelease exception additionally
explains why stable is unsuitable and how to return to stable. Keep these records with the upgrade
owner and link them from the baseline's human-maintained exception section.

Keep upgrades independently revertible. On failure, repair locally or revert the complete
candidate's manifests, lockfile and necessary migrations together. Persistence changes may require
data recovery rather than source-only rollback; document that before implementation. Never move a
release tag, force-push main or imply crates.io publication can be undone by reverting Git.

## 5. Advance The Accepted Baseline Only After Merge

Verify that the upgrade implementation already exists in target-main history and that its required
validation/review passed. Generate from that exact dependency tree, not an unmerged candidate.
The initial bootstrap instead cites prior target-main validation explicitly; it is not evidence of
an upgrade that never occurred.

Snapshot generation uses `--snapshot --format json --source-commit SHA --accepted-at UTC
--validation-evidence TEXT`, or PowerShell `-Snapshot -Format json -SourceCommit SHA -AcceptedAt
UTC -ValidationEvidence TEXT`. UTC uses `YYYY-MM-DDTHH:MM:SSZ`. It emits JSON only, rejects live
lookup, and does not verify the supplied attestation or write the accepted file. The accepting
maintainer must verify provenance, exact source-tree correspondence and evidence before committing.

Keep one script-owned machine snapshot; derive Markdown tables from it. Humans maintain domain
notes and exception links, not duplicate version cells. Audit the resulting snapshot against the
accepted tree: no unexplained baseline drift. Close owner-first using already-existing implementation
SHAs, then synchronize indexes and Issue. Future scans use that new baseline but refresh upstream
observations independently. Neither snapshot generation nor its status-only commit self-certifies
completion.
