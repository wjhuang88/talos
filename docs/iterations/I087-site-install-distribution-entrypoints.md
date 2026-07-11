# Iteration I087: Site Install Distribution Entrypoints

> Document status: Superseded before activation (2026-07-12)
> Published plan date: 2026-07-03
> Planned objective: Execute weeks 5-8 of the 2026-07-03 four-month hardening plan: decide and
> implement verified install script entrypoints under `talos.hwj.zone` if they pass release
> distribution checks.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: installer entrypoints are either safely hosted on the product site with validation
> or explicitly deferred with a documented blocker.
> Supersession: the materially revised distribution/productization target is replanned under I118;
> this baseline remains historical.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| H110 | REL-001/WEB-002 | Planned/Complete | v0.2.2 release posture | Decide site-hosted install entrypoint shape |
| H111 | REL-001/WEB-002 | Planned | H110 | Add generated/synchronized site installer files without divergence from `install/` |
| H112 | REL-001/WEB-002 | Planned | H111 | Update README/site install commands only after validation |
| H113 | REL-001/ARCH-031 | Planned | H112 | Release docs matrix: assets, checksums, install commands, no crates.io claim |
| H114 | Hardening plan | Planned | H110-H113 | Month-2 release/distribution closeout |

### Scope

- Evaluate `https://talos.hwj.zone/install.sh` and a PowerShell equivalent as stable user-facing
  install entrypoints.
- Keep GitHub Releases as the binary artifact source of truth.
- Keep repository `install/` scripts canonical and prove any site-hosted copies are synchronized.
- Preserve checksum verification and unsupported-platform behavior.

### Non-Goals

- No crate publish.
- No package-manager installation channel.
- No automatic online asset download beyond the explicit installer action.
- No manual GitHub Release mutation in this iteration unless separately approved.

### Acceptance

- Given a site-hosted installer exists, when validation runs, then its content matches the canonical
  installer source or is generated from it.
- Given README install commands change, when reviewed, then the site endpoint has already been
  deployed or locally validated with a documented deployment path.
- Given release docs are updated, then they distinguish GitHub Release assets, site entrypoints, and
  blocked crates.io install.

### Planned Validation

- `cargo fmt --all -- --check` if Rust files change
- `scripts/validate_public_site.sh`
- installer dry-run or URL construction test
- `scripts/check_publish_guard.sh .` when release/publish docs change
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- README and README.zh-CN if install commands change
- `site/install.html` and `site/zh/install.html`
- `docs/backlog/active/REL-001-release-installer-readiness.md`
- `docs/tasks/2026-07-03-four-month-product-hardening-plan.md`
- `docs/BOARD.md` after owner docs

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-03 | Planning | Created as the I087 shell for site-hosted installer entrypoint discussion and implementation. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
