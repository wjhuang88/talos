# WEB-006: Public Documentation Hub And Release-Synchronized Site

| Field | Value |
|---|---|
| Type | Product documentation Epic |
| Status | In Progress — I143 |
| Priority | P1 |
| Parent | None |
| Selected Iteration | I143 (Planned) |
| User | New and existing Talos operators |

## Goal And Value

Bring `talos.hwj.zone` from its stale `v0.2.2` snapshot to the current `v0.4.0`
product truth, add a complete bilingual Documentation entry point, consolidate
scattered install/configuration/usage/capability/safety material, and repair the
unreadable primary install button. A release should have one discoverable public
documentation surface rather than requiring visitors to reconstruct behavior from
marketing pages and repository files.

## Child Stories And Order

1. [WEB-006-A](WEB-006-A-site-truth-and-docs-hub.md): release-truth matrix,
   `docs.html`/`zh/docs.html`, bilingual navigation and content consolidation.
2. [WEB-006-B](WEB-006-B-site-accessibility-and-button-contrast.md): primary-button
   color fix and theme/state accessibility matrix. May land independently.
3. [WEB-006-C](WEB-006-C-site-drift-prevention-and-release-gate.md): validator,
   source-of-truth and formal publication gates. Depends on A and B.

## Scope

- Sync every public version/capability/non-goal claim against the `v0.4.0` release,
  README pair, runtime help/registries, config reference, and accepted ADR boundaries.
- Add a first-class `Documentation`/`文档` navigation item and canonical hub pages.
- Consolidate getting started, configuration, models/variants, interaction modes,
  commands, tools, sessions, Skills/MCP/plugins, safety, troubleshooting, and links
  to deeper references.
- Preserve focused Install, Capabilities, Safety, Roadmap, and Releases pages; the
  docs hub organizes and links them instead of copying internal governance records.
- Keep EN and zh-CN structure and claims in parity.
- Fix primary CTA contrast in light/dark and interactive states.

## Explicit Exclusions

- No framework, package manager, generated-doc platform, analytics, CDN, external
  font, search service, or network dependency.
- No exposure of `docs/backlog`, Board, iteration/task notes, secrets, credentials,
  or internal-only operational records.
- No product behavior, installer, release tag, DNS, Pages settings, or v1.0 claim.
- No promise that pre-1.0 APIs or formats are stable.

## Completion

All child stories are Complete; the static and installer validators pass; browser
review covers desktop/mobile, EN/zh-CN, light/dark and keyboard focus; every public
claim is traceable to a named source; Pages workflow is ready to deploy from `main`.

## Required Reads

- `README.md`, `README.zh-CN.md`
- `docs/reference/DOCS-SYNC-CHECKLIST.md`
- `docs/reference/config.reference.toml`
- `docs/backlog/active/WEB-002-github-pages-product-site.md`
- `docs/backlog/active/WEB-003-site-internationalization.md`
- `docs/backlog/active/WEB-004-site-theme-branding.md`
- `site/README.md`, `site/*.html`, `site/zh/*.html`, `site/assets/styles.css`
- `scripts/validate_public_site.sh`, `scripts/validate_installers.sh`
