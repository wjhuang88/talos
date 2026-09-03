# Iteration I245: v0.9.0 GitHub And Crates.io Publication

> Document status: Active / Claimed (proposed; ineffective until claim PR merge)
> Published plan date: 2026-09-03
> Planned objective: publish one validated v0.9.0 GitHub Release before publishing the Cargo
> package closure, then prove external CLI installation and runtime SDK consumption.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: users can download v0.9.0, install the `talos` binary from crates.io, and build
> the documented `talos-runtime` fixture without workspace paths.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | `@wjhuang88` |
| Executing Agent | `Codex / GPT-5 mainline release session` |
| Work Slice | REL-005 only: v0.9.0 version/release surfaces, reviewed candidate, GitHub-first tag/Release, dependency-ordered Cargo publication, external CLI/runtime verification and owner-first closeout. |
| Claimed At | 2026-09-03 |
| Source Issue | None |
| Governance Claim PR | #479 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly requested the complete GitHub and crates.io release. Protected release execution still requires independent exact-head review before irreversible actions. |
| Implementation PR | Not started |
| Last Updated | 2026-09-03 |
| Handoff / Release Condition | Claim and activation are ineffective until the finalized governance PR merges to `main`; immutable publication additionally requires the I245 release gates. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| REL-005 | ARCH-031 | Ready | v0.8.0 release closed; clean current main; registry credentials available | GitHub v0.9.0 Release precedes the Cargo publication and external acceptance. |

### Non-Terminal Iteration Inventory At Selection

| Iteration | State | Disposition |
|---|---|---|
| I164 | Paused / superseded | Preserve; do not resume. |
| I207 | Planned / Unclaimed | Preserve; do not activate or modify its steering scope. |
| I208 | Planned / Unclaimed | Preserve; do not activate or modify its steering scope. |
| I245 | Proposed Active / Claimed | Selected release slice; ineffective until claim PR merge. |

### Ordered Delivery

1. Confirm main, registry state, package closure, credentials and v0.9.0 availability.
2. Synchronize versions and public release surfaces locally; run versioned release preflight.
3. Push one stable candidate for exact-head CI and independent release review.
4. Merge after CAS, rerun versioned preflight on the reviewed main commit, and push one annotated tag.
5. Wait for the GitHub Release and all assets to complete.
6. Publish Cargo packages in metadata-derived dependency order, verifying visibility between waves.
7. Run external CLI/runtime acceptance and close owners with pre-existing evidence.

### Scope And Non-Goals

- Scope is exactly the REL-005 owner.
- No product feature, API redesign, dependency upgrade, Desktop/Dashboard implementation,
  RUNTIME-006 completion, REL-002 qualification or `talos-models` publication.

### Planned Validation

- `./scripts/release_preflight.sh v0.9.0`
- `bash scripts/check_publish_guard.sh .`
- exact-head CI and independent release review
- GitHub Release asset/checksum verification
- dependency-ordered registry visibility checks
- isolated `cargo install` and registry-only runtime fixture

### Documentation To Update

- `README.md`, `README.zh-CN.md`
- paired `site/` release and Documentation surfaces
- `docs/releases/v0.9.0.md`
- REL-005, I245, release task, Board, backlog, iteration index and manifest

### Risks And Rollback

- Risk: immutable tag or partial registry publication fails.
- Rollback: stop at the failed gate; never move a tag or overwrite a crate; source changes use a
  new patch version and separately recorded recovery owner.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-03 | Atomic claim+activation proposal | Based on `main@336c65e08c49c0e95334a8d967bb8a1648aa3a8f`; no Active/Review iteration or overlapping PR exists. This record has no effect until its finalized PR reaches `main`. |

## Verification Evidence

- Pending claim-stage validators and exact-head CI.

## Completion Evidence

- Completion Commit: Pending.
- Status-only documentation commits cannot self-certify completion.

## Variance And Residuals

- None at selection. RUNTIME-006 and REL-002 remain separate owners.
