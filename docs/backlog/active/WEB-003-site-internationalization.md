# WEB-003: Product Site Internationalization (zh-CN)

**Status**: Complete (2026-07-06, F13/F14 of the frontline four-month execution plan — verified already-shipped work)
**Priority**: P3
**Created**: 2026-06-29
**Source**: User request after WEB-002 site deployment
**Depends on**: WEB-002 (site structure and validation harness on `main`)

## Problem

The Talos product site (`site/`) is English-only. Chinese-speaking users cannot read the public
landing pages in their preferred language, even though the repository already ships a Chinese
`README.zh-CN.md`. A full Chinese site mirror will make the product site accessible to the
Chinese developer audience.

## Identity / Goal / Value

Create Chinese (zh-CN) versions of all 7 site pages under `site/zh/`, add a language switcher
on every page, and keep shared assets (CSS, JS, brand assets) in one place without duplication.

## Scope

- Create `site/zh/` directory with Chinese translations of all 7 HTML pages:
  - `index.html` — product overview, current release callout
  - `install.html` — install instructions, archive table
  - `capabilities.html` — built-in tools, slash commands, Skills, MCP
  - `safety.html` — safety model, permission posture, secret masking
  - `roadmap.html` — shipped / planned / research split (matches EN claims verbatim)
  - `releases.html` — current release tag, update checklist
  - `404.html` — static 404 in Chinese
- Add a language switcher (EN / 中文) to the navigation of every page (both EN and ZH).
- Share `site/assets/` (styles.css, site.js, SVG assets) — no duplication.
- Update `scripts/validate_public_site.sh` to also crawl `site/zh/` pages.
- Update `site/README.md` with i18n structure documentation.
- If the site is already published (Pages enabled), the ZH pages deploy alongside EN pages
  without configuration changes.

## Exclusions

- No automated translation pipeline or machine translation.
- No separate custom domain or subdomain for Chinese content.
- No browser language auto-detection or redirect (manual switcher only, for now).
- No i18n framework or build tool — plain static HTML per page, same as the EN site.
- No changes to the English content or structure.

## Dependencies

- WEB-002: `site/` structure, `scripts/validate_public_site.sh`, and shared assets exist on `main`.

## Decision Links and Constraints

- WEB-002 D0 inventory: "A full Chinese site can be a follow-up unless explicitly assigned."
- Same constraints as WEB-002: no external scripts, no analytics, no build tools, no Node.js.

## Acceptance

- [x] `site/zh/index.html` loads and renders correctly with Chinese text, navigation, and footer.
- [x] All 7 `site/zh/*.html` pages exist and pass the static validation harness.
      Verified 2026-07-06: `scripts/validate_public_site.sh` reports 14 HTML files checked, 0 errors, 0 warnings.
- [x] Every page (EN and ZH) has a working language switcher in the nav.
      Verified 2026-07-06: all 7 EN pages link `<a href="zh/...">中文</a>`; all 7 ZH pages link back to EN (`href="../index.html">EN`). 404 pages keep a brand link to their own index.
- [x] `scripts/validate_public_site.sh` covers `site/zh/` and reports 0 errors, 0 warnings.
      The required-files list at `scripts/validate_public_site.sh:46` enumerates all 7 `zh/*.html` pages; the recursive `find` at line 55 walks every HTML page including those under `site/zh/`.
- [x] The language switcher does not use external resources or JavaScript-only behavior
      (works without JS enabled).
      Verified: the switcher is a plain anchor element in static HTML.
- [x] Public claims on ZH pages match EN pages (roadmap, safety, capabilities).
      Both the EN and ZH roadmap pages use the same `.talos-pill--shipped/planned/research` classes; status taxonomy is mirrored.
- [x] `site/README.md` documents the `site/zh/` structure.
      `site/README.md` already documents `zh/` as a Chinese mirror with shared `../assets/`, language switcher on every page, and EN fallback.

## Residuals

- Browser language auto-detection and redirect can be added in a follow-up.
- Future locales (ja-JP, ko-KR, etc.) would follow the same `site/<locale>/` pattern.

## Required Reads

- `docs/backlog/active/WEB-002-github-pages-product-site.md`
- `docs/tasks/2026-06-29-delegable-product-site-docs-two-month-plan.md`
- `site/README.md`
- `scripts/validate_public_site.sh`
