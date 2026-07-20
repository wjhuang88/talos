# WEB-006-C: Public Site Drift Prevention And Release Gate

| Field | Value |
|---|---|
| Type | Technical documentation Story |
| Parent Epic | WEB-006 |
| Status | Complete — I143 (maintainer acceptance 2026-07-20) |
| Priority | P2 |

## Goal

Make future release/site synchronization repeatable and fail visibly when the
documentation hub, locale mirror, release version, navigation, install commands,
or accessibility contract drifts.

## Scope

- Extend `validate_public_site.sh` to require both docs pages, nav coverage on every
  page, locale counterpart parity, current workspace release string where applicable,
  and no stale current-release markers.
- Preserve existing no-analytics/no-external-assets, link, install-command and roadmap
  hard gates.
- Update `DOCS-SYNC-CHECKLIST.md` and `site/README.md` with named source owners,
  release order, browser QA matrix and rollback.
- Treat deployment as asynchronous Pages workflow handoff; never equate a local pass
  with a successful production deployment.

## Acceptance

- Deliberately removing a docs page/nav link, changing one locale section, restoring
  `v0.2.2` as current, or breaking CTA component color causes a deterministic failure.
- `sh scripts/validate_public_site.sh` reports all EN/ZH pages with 0 errors/warnings.
- `sh scripts/validate_installers.sh`, governance validation and `git diff --check` pass.
- The iteration records the Pages run URL/status if deployment is authorized.

## Required Reads

- Parent WEB-006 and WEB-006-A/B
- `scripts/validate_public_site.sh`
- `scripts/validate_installers.sh`
- `.github/workflows/pages.yml`
- `docs/reference/DOCS-SYNC-CHECKLIST.md`
- `site/README.md`
