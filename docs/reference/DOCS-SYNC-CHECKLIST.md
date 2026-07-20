# Docs Sync Checklist

Created: 2026-06-30 (T05 of the four-month self-bootstrap plan)

This checklist defines the surfaces that must stay in sync whenever user-visible behavior,
installation, tools, commands, or SDK contracts change. It is a reference artifact; update it when
a new docs surface is added, not on every behavior change.

## When To Run This Checklist

Run the relevant rows before marking any implementation slice complete:

- A tool was added, removed, renamed, or its permission/summary changed.
- A slash command was added, removed, or renamed.
- The install path or package metadata changed.
- The runtime SDK surface (`talos-runtime`) changed.
- A release tag was created or install instructions changed.
- Configuration keys or defaults changed.

## Surfaces

| Surface | Path / Location | Owner | Sync Trigger |
|---|---|---|---|
| Root README (en) | `README.md` | Product docs | Install path, tool list, slash commands, capabilities, SDK boundary, config schema |
| Root README (zh-CN) | `README.zh-CN.md` | Product docs | Mirror every user-visible change in the English README |
| Public site (en) | `site/index.html` + `site/*.html`, especially `site/docs.html` | Product docs | Release version, install, capabilities, configuration, commands, safety, and support links |
| Public site (zh-CN) | `site/zh/*.html`, especially `site/zh/docs.html` | Product docs | Mirror English public claims and documentation section structure |
| Site assets | `site/assets/` | Product docs | Branding/theme changes (WEB-004) |
| AGENTS.md | `AGENTS.md` | Governance | Task router, hard constraints, current traps, session checklist |
| Crate docs (`//!`) | `crates/*/src/lib.rs` | Crate owner | Public API, support boundary, safety notes — required before publish |
| Crates.io metadata | `crates/*/Cargo.toml` `[package]` | Crate owner | description, keywords, categories, readme — required before publish |
| Release notes | GitHub Release body / `CHANGELOG` (if created) | Release owner | Every tagged release: new features, fixes, breaking changes, known issues |
| Architecture reference | `docs/reference/ARCHITECTURE.md` | Architecture | Crate structure, data flow, trait boundaries — when architecture changes, not when work is planned |
| Publication matrix | `docs/reference/CRATE-PUBLICATION-MATRIX.md` | Distribution | Publish readiness state, dry-run evidence, gate status |
| Config reference | `docs/reference/config.reference.toml` | Config owner | When config keys, defaults, or provider schemas change |
| Backlog | `docs/backlog/PRODUCT-BACKLOG.md` | Story owners | Story status, decision context — before BOARD sync |
| Board | `docs/BOARD.md` | Derived view | After owner docs change state (Never before owner docs) |

## Tool-Surface Sync

When a built-in tool changes (name, permission, summary, presentation), update **all** of:

1. `README.md` "Built-In Capabilities" section — the bullet list of tools.
2. `README.zh-CN.md` equivalent section.
3. `site/docs.html`, `site/zh/docs.html`, and any relevant site capability page.
4. `docs/backlog/active/TOOL-007-tool-set-design-audit.md` if the audit roster changes.
5. Agent system prompt assets (`crates/talos-agent/src/prompt/assets/`) if tool identity text
   references the changed tool.
6. TUI command/help text if the tool has a slash-command peer.

Current baseline (T04 audit, 2026-06-30): **30 native tools** across 8 families, 29 presented by
default, 1 hidden (`http_request` in `AdvancedNetwork`). always_on set: `read`, `write`, `edit`,
`ls`, `grep`, `glob`, `document_extract`.

## Command-Surface Sync

When a slash command changes, update:

1. `README.md` "Slash Commands" table.
2. `README.zh-CN.md` equivalent.
3. Both site documentation hubs and the command/capability page.
4. `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md` registry.
5. TUI `/help` output (driven by the registry, but verify rendering).

## Install-Surface Sync

When the install path changes, update **in lockstep**:

1. `README.md` "Install" section (release archive, `install.sh`, `cargo install` path).
2. `README.zh-CN.md` equivalent.
3. `site/index.html`, `site/zh/index.html`, and both documentation hubs' getting-started sections.
4. `docs/reference/CRATE-PUBLICATION-MATRIX.md` if crate publish state changed.
5. `scripts/check_publish_guard.sh` expectations if `publish` flags changed.

## SDK-Surface Sync

When `talos-runtime` public API changes, update:

1. `README.md` "Embedding Talos In Rust" section.
2. `crates/talos-runtime/src/lib.rs` `//!` docs.
3. `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`.
4. `docs/backlog/active/ARCH-031-crate-publication-boundary.md` support contract.
5. SDK examples (T12 deliverable) when they exist.

## Validation

- `sh scripts/validate_public_site.sh` when any `site/` file changes. It checks
  locale pairs, Documentation navigation, current-release drift, and the CTA
  component selector/focus contract in addition to links and external-resource
  guardrails.
- `sh scripts/validate_installers.sh` when the public install copy changes.
- `scripts/validate_project_governance.sh .` when backlog/board/iteration docs change.
- Manual for a public-site release: inspect desktop and narrow mobile widths in EN and zh-CN,
  light and dark themes, and keyboard focus. Confirm that a local static pass is not reported
  as a successful Pages deployment.
- Manual: diff the English and zh-CN READMEs after every user-visible change to catch drift.

## Public Site Release Order

1. Establish the release truth from the tagged release, README pair, command registry,
   configuration reference, and accepted public-boundary decisions.
2. Update all eight EN/ZH page pairs, including the two documentation hubs; classify
   shipped, planned, and research work conservatively.
3. Run the static and installer validators, then complete the browser QA matrix above.
4. Hand deployment to the Pages workflow separately. Record its run URL/status only when
   deployment observation is authorized; a local validator pass is never deployment evidence.
