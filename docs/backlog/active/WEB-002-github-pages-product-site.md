# WEB-002: GitHub Pages Product Site And Custom Domain

**Status**: Ready for delegated assignment
**Priority**: P3
**Created**: 2026-06-27
**Source**: User request to publish a separate GitHub Pages site under a personal domain
**Depends on**: v0.1.2 release baseline; README positioning update
**Execution plan**: `docs/tasks/2026-06-29-delegable-product-site-docs-two-month-plan.md`

## Problem

Talos now has a public release baseline, but the repository README files must remain developer- and
user-facing project documents. A separate public product site can present the released product,
installation path, roadmap, and safety posture without mixing marketing/site assets into internal
engineering docs.

## Scope

- Add a standalone GitHub Pages directory, preferably `site/`, that is independent from `docs/`.
- Publish static pages from release-accurate content: product positioning, install instructions,
  safety boundaries, capability overview, release notes link, and roadmap summary.
- Support a custom domain through a committed `CNAME` file when the domain is selected.
- Add a GitHub Actions Pages workflow or equivalent repository Pages configuration.
- Keep generated/public site assets out of the governance source-of-truth documents.
- Mirror public claims with `README.md` and `README.zh-CN.md` so users do not see conflicting
  capability descriptions.

## Non-Goals

- No embedded Talos web control surface; that remains `WEB-001`.
- No in-app RPC, approvals, logs, or config UI.
- No Node.js runtime requirement in the product binary. A static-site build tool may be considered
  separately, but the preferred first slice is plain static HTML/CSS.
- No automatic deployment of secrets or private docs.

## Acceptance Criteria

- [ ] A `site/` directory can be served by GitHub Pages without depending on local developer tools.
- [ ] A custom-domain `CNAME` path is documented and can be enabled by changing one file.
- [ ] The site does not expose internal task notes, private governance records, or unpublished claims.
- [ ] README install and positioning claims match the site.
- [ ] Pages deployment is documented and recoverable by future agents.

## Validation

- Static link/path check for `site/`.
- GitHub Pages workflow dry-run or repository settings checklist.
- Manual review against `README.md`, `README.zh-CN.md`, and the latest release tag.

## Delegation Boundary

This item is suitable for non-architect implementation when scoped to static site and
documentation work. Do not implement `WEB-001`, runtime web control, session/RPC surfaces,
permission UI, release publishing, DNS changes, or GitHub Pages settings changes without explicit
maintainer approval.
