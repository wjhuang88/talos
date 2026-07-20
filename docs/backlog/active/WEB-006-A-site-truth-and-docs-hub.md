# WEB-006-A: Site Truth Sync And Bilingual Documentation Hub

| Field | Value |
|---|---|
| Type | Product documentation Story |
| Parent Epic | WEB-006 |
| Status | Ready |
| Priority | P1 |

## Identity / Goal / Value

As an operator, I can open one complete Documentation page in English or Chinese
and learn how to install, configure, operate, extend, troubleshoot, and safely use
the current `v0.4.0` release without reconciling stale or scattered pages.

## Scope

- Create `site/docs.html` and `site/zh/docs.html`; add Docs/文档 to all page navs,
  language switches, 404 suggestions, home “next” links, and footers where useful.
- Use a documentation IA with anchored sections: Quick start; configuration and
  credentials; providers/models/variants; TUI/inline/print/RPC modes; composer and
  commands; tools and permissions; sessions/storage/memory; Skills/MCP/plugins;
  safety boundaries; troubleshooting; release/support links.
- Update all existing EN/ZH pages from `v0.2.2` to current `v0.4.0` truth, including
  shipped/planned/research classification and REL-002 NO-GO.
- Prefer concise summaries plus canonical links. Do not copy internal backlog or
  iteration prose into the public site.

## Acceptance

- Given either locale, when a visitor opens Documentation, then all listed sections
  are reachable by semantic headings and a local table of contents.
- Given any public page, when the visitor uses primary navigation, then Documentation
  is reachable in one action and EN/zh-CN counterparts link to each other.
- Given `v0.4.0`, when version and capability claims are compared with README,
  registries/config reference and release notes, then no `v0.2.2` current-release
  claim or shipped/research overclaim remains.
- Given a narrow viewport, when navigation and docs content render, then no horizontal
  page overflow is introduced; code blocks may scroll internally.
- All images/links have appropriate text, headings are ordered, and pages work without
  JavaScript except optional copy buttons.

## Validation

- `sh scripts/validate_public_site.sh`
- `sh scripts/validate_installers.sh`
- Browser screenshots/manual inspection at desktop and mobile widths for both locales.
- Recorded EN/zh-CN section-parity and source-of-truth matrix.

## Required Reads

- Parent WEB-006 and every Required Read listed there.
- `crates/talos-conversation/src/command_registry.rs`
- `crates/talos-config/src/models.toml`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/decisions/048-model-variant-representation.md`
