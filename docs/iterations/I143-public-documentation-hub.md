# Iteration I143: Public Documentation Hub And v0.4.0 Site Sync

> Document status: In Progress
> Published plan date: 2026-07-20
> Planned objective: deliver WEB-006 as a release-grade bilingual documentation site.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `site/docs.html` and `site/zh/docs.html` provide complete current
> documentation, the unreadable install CTA is fixed, and all public-site gates pass.

## Published Baseline

- Selected: WEB-006-A, WEB-006-B, WEB-006-C under WEB-006.
- Order: B may proceed independently; A establishes content/IA; C closes drift gates.
- User docs: all `site/*.html`, `site/zh/*.html`, `site/README.md`,
  `DOCS-SYNC-CHECKLIST.md`.
- Validation: public-site and installer validators, EN/ZH truth/parity matrix,
  browser desktop/mobile + light/dark + keyboard review, governance, diff check.
- Non-goals: no runtime feature, release/tag, Pages/DNS mutation, framework, analytics,
  external asset or v1.0 claim.
- Rollback: revert docs/nav/CSS as one site release; current focused pages remain usable.

## Acceptance

- WEB-006 child acceptance is satisfied with `v0.4.0` truth and no stale current claim.
- 16 public HTML pages (8 per locale) resolve links and share navigation/assets.
- The primary CTA passes computed-style/contrast/focus checks in both themes.
- Pages workflow deployment is observed only if separately authorized.

## Iteration Inventory Disposition

- I018 is fulfilled by I047 and reconciled as Complete/superseded.
- I139 is Complete; its stale Review inventory row is corrected.
- I144 is Planned but explicitly deferred until I143 closes.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Planning | Baseline published; not activated and no site implementation performed. |
| 2026-07-20 | Activation | Maintainer selected I143 as the active iteration. WEB-006-A and WEB-006-B may proceed in parallel within the static-site boundary; WEB-006-C follows their completed content and CSS contracts. |
| 2026-07-20 | Implementation | Frontline delivery added paired EN/zh-CN Documentation hubs, synchronized the 16 public pages to v0.4.0, repaired CTA cascade/focus styling, and added static drift checks. |
| 2026-07-20 | Independent static review | `validate_public_site.sh`, installer validation, governance validation, and `git diff --check` passed. Browser automation could not start because the controlled environment omitted its required execution-policy metadata; browser QA remains outstanding, so this iteration stays In Progress. |
| 2026-07-20 | Release-gate follow-up | `release_preflight.sh` now derives the version from `Cargo.toml` and runs public-site plus installer validation before Rust checks. It requires the README pair and all release surfaces to name that version, rejects a retained immediate prior release after a bump, and was verified with `v0.4.0` plus a deliberately mismatched version argument. |

## Planning Verification

- `sh scripts/validate_public_site.sh`: planning baseline passed over 14 current HTML
  files with 0 errors and 0 warnings; this does not satisfy future WEB-006 acceptance.
- `git diff --check`: passed for the published planning changes.
- `scripts/validate_project_governance.sh .`: passed with 0 warnings after final
  owner/backlog/Board synchronization.
