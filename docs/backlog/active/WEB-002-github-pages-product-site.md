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

## Progress Log

- 2026-06-29 — D0 baseline inventory recorded in the two-month plan task
  (`docs/tasks/2026-06-29-delegable-product-site-docs-two-month-plan.md`).
  Shipped / Planned / Research split is mapped to README and to the
  in-progress `site/roadmap.html` skeleton.
- 2026-06-29 — D1 site information architecture recorded in the same task
  file. Page map: `index`, `install`, `capabilities`, `safety`, `roadmap`,
  `releases`, plus `404`. Public / private boundary: `site/**` is the only
  public surface; `docs/**` stays out and is only linked to.
- 2026-06-29 — D2 static site skeleton landed. `site/` exists with 7 HTML
  pages, a shared stylesheet, a small dependency-free script, a
  text-only wordmark, a favicon, a `CNAME.example` placeholder, and a
  maintainer-only `README.md`. All relative links resolve (Python static
  resolver verified). The site can be opened via `file://` or any
  static file server. No analytics, no external scripts, no build tools.
- 2026-06-29 — D3 install and release content. `site/install.html` and
  `site/releases.html` populated. 14/14 install commands (shell
  installer, PowerShell installer, archive extraction, `--no-init`, four
  `--config-*` commands, two `cargo build` commands, `build.sh`, mock
  verify) match the README install section byte-for-byte, verified by a
  Python diff. The releases page cross-references the GitHub release
  history and records the post-release update checklist that keeps
  `site/`, `README.md`, and `README.zh-CN.md` in sync.
- 2026-06-29 — D4 safety and capability content. `site/capabilities.html`
  carries the shipped-only built-in tools, slash commands, runtime
  Skills, MCP tools, and embedding facade (with the pre-1.0 caveat for
  `talos-runtime` and a link to ADR-024). `site/safety.html` carries the
  three-bullet README safety model, the ADR-023 secret-masking surface,
  the permission posture, the sandbox boundary, and the explicit
  pre-1.0 limits. Coverage check: 27/27 built-in tools, 16/16 slash
  commands, all five ADR-023 surface tokens, no roadmap overclaims.
- 2026-06-29 — D5 roadmap and non-goals content. `site/roadmap.html`
  has the shipped / planned / research split. `WEB-001`, `PLUGIN-001`,
  `REMOTE-001`, and `REL-002` are never presented as shipped; every
  shipped item carries the `Shipped` pill. The non-goals section
  enumerates the things Talos is not in v0.2.0.
- 2026-06-29 — D6 GitHub Pages readiness. `site/CNAME.example` carries
  a placeholder template. `site/README.md` (maintainer-only) documents
  the repository-settings checklist. No `.github/workflows/pages.yml`
  was added; a workflow would create a duplicate deployment path. Domain
  selection, repository Pages settings, DNS, and HTTPS enforcement
  remain maintainer actions.
- 2026-06-29 — D7 README sync. A new "Public product site" row was
  added to the Documentation table in `README.md` and the 文档 table in
  `README.zh-CN.md`. Both READMEs reference the relative `site/` path
  rather than a hardcoded URL, because Pages has not been enabled yet.
- 2026-06-29 — D8 validation harness. `scripts/validate_public_site.sh`
  is a POSIX shell harness with zero runtime dependencies. It walks
  `site/`, parses every `href` and `src`, runs four hard guardrails
  (no external scripts, no analytics, no `@import`, no external
  `url()`), asserts the three critical install commands are present in
  `site/install.html`, and re-runs the D5 roadmap gate. Current run:
  `Errors: 0, Warnings: 0` over 7 HTML files.
- 2026-06-29 — D9 closeout. `sh scripts/validate_project_governance.sh .`
  exits 0 with `0 warning(s)`. `sh scripts/validate_public_site.sh`
  exits 0. No push, tag, release, deploy, Pages settings change, or DNS
  change was performed; all of those remain maintainer actions. The
  closeout is a published execution baseline; future follow-ups start
  by re-running both validators and reading the "Updating the site
  after a release" section of `site/README.md`.

## Validation

- Static link/path check for `site/`.
- GitHub Pages workflow dry-run or repository settings checklist.
- Manual review against `README.md`, `README.zh-CN.md`, and the latest release tag.

## Delegation Boundary

This item is suitable for non-architect implementation when scoped to static site and
documentation work. Do not implement `WEB-001`, runtime web control, session/RPC surfaces,
permission UI, release publishing, DNS changes, or GitHub Pages settings changes without explicit
maintainer approval.
