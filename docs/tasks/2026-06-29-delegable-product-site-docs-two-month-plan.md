# 2026-06-29 Delegable Product Site And User Docs Two-Month Plan

**Status**: Ready for delegated assignment
**Owner area**: Public product site, user documentation, and release-accurate messaging
**Primary backlog item**: `docs/backlog/active/WEB-002-github-pages-product-site.md`
**Created**: 2026-06-29
**Target horizon**: About 8 weeks
**Architect boundary**: High-risk runtime/product architecture tasks are paused for this plan.

## Outcome

Create a release-accurate, static public product site and supporting user-documentation refresh that
can be implemented by non-architect developers without touching Talos runtime behavior. The task is
complete when `site/` can be served by GitHub Pages, README/site/release claims agree, and future
maintainers have a repeatable checklist for updating the public materials after releases.

## In Scope

- Implement `WEB-002` as a static GitHub Pages site under `site/`.
- Keep `site/` separate from internal engineering `docs/`.
- Add or update public pages for:
  - product overview;
  - install instructions;
  - safety and permission posture;
  - shipped capability overview;
  - roadmap summary;
  - release links and update checklist.
- Add `site/CNAME.example` and document how the maintainer enables `site/CNAME`.
- Add a GitHub Pages workflow or a documented repository-settings checklist.
- Keep `README.md`, `README.zh-CN.md`, and site claims synchronized.
- Add simple static validation scripts only if they do not introduce a runtime dependency.

## Out of Scope

- No `WEB-001` embedded web control surface.
- No local server that exposes approvals, logs, config, sessions, RPC, or runtime controls.
- No permission, sandbox, tool, provider, model, plugin, MCP, or remote-session changes.
- No crates.io publication, release tagging, GitHub Release publishing, or version bump.
- No DNS changes, custom-domain cutover, or repository Pages setting mutation without maintainer
  approval.
- No Node.js, bundler, framework, or package-manager dependency unless a later maintainer approves
  a separate design change. First implementation should be plain static HTML/CSS/JS.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback |
|---|---|---|---|---|---|
| D0 | Baseline inventory | Current README/release/site claim inventory and source-of-truth list. | None | Inventory recorded in this task checkpoint. | Defer unclear claims instead of inventing product promises. |
| D1 | Site information architecture | Page map, navigation, content sections, and public/private boundary. | D0 | Page map references only shipped or explicitly planned capabilities. | Keep one-page site if content is not ready. |
| D2 | Static site skeleton | `site/index.html`, `site/styles.css`, optional minimal `site/assets/`, and local preview instructions. | D1 | Opens locally without build tools and has no broken relative links. | Plain single HTML file if assets add churn. |
| D3 | Install and release content | Site install page/section mirrors README install commands and release links. | D2 | README and site install commands match byte-for-byte where practical. | Link to README install section if duplication becomes risky. |
| D4 | Safety and capability content | Site describes permissions, sandboxing, config-secret masking, and shipped capabilities conservatively. | D2 | No claim contradicts README, ADR-023, or architecture docs. | Use shorter claims and link to source docs. |
| D5 | Roadmap and non-goals content | Public roadmap distinguishes shipped, planned, and research items. | D2 | Web control, remote control, plugins, and v1.0 self-bootstrap are not presented as shipped. | Omit roadmap section if status cannot be verified. |
| D6 | GitHub Pages readiness | Pages workflow or repository-settings checklist plus `CNAME.example`. | D2 | Dry-run/checklist proves how to publish without secrets. | Use checklist only if workflow risk is unclear. |
| D7 | README synchronization | README and README.zh-CN mention the public site path/domain placeholder without conflicting claims. | D3-D6 | English and Chinese README claims match the site. | Leave README unchanged and record why if site is not enabled. |
| D8 | Validation harness | Static link/path check and documentation consistency checklist. | D2-D7 | Check command or manual checklist is recorded and repeatable. | Manual checklist is acceptable for first slice. |
| D9 | Closeout | Owner docs, Board, and this task record synchronized with validation evidence. | D0-D8 | `WEB-002` acceptance criteria are checked or residuals are explicit. | Mark Partial with exact blockers. |

## Two-Month Schedule

| Week | Focus | Deliverable |
|---|---|---|
| 1 | D0-D1 | Claim inventory and site page map. |
| 2 | D2 | Static site skeleton, local preview, visual baseline. |
| 3 | D3 | Install/release content synchronized with README. |
| 4 | D4 | Safety/capability content with conservative wording. |
| 5 | D5 | Roadmap/non-goals content and public status labels. |
| 6 | D6 | Pages workflow or settings checklist, custom-domain placeholder. |
| 7 | D7-D8 | README sync and validation harness/checklist. |
| 8 | D9 | Closeout, screenshots, validation evidence, residual-work registration. |

## Dependencies And Prerequisites

- Read first:
  - `docs/backlog/active/WEB-002-github-pages-product-site.md`
  - `README.md`
  - `README.zh-CN.md`
  - `docs/BOARD.md`
  - `docs/backlog/PRODUCT-BACKLOG.md`
  - latest release notes and release tag state
- High-risk task group stays paused unless the maintainer explicitly resumes it.
- If release state is ambiguous, use README as the public baseline and record the ambiguity.

## Artifacts And State Owners

- Primary task record: this file.
- Backlog owner: `docs/backlog/active/WEB-002-github-pages-product-site.md`.
- Derived view: `docs/BOARD.md`.
- Public docs: `README.md`, `README.zh-CN.md`, and `site/`.
- Optional CI/config: `.github/workflows/pages.yml` only if the implementation chooses workflow
  configuration instead of repository settings documentation.

## Validation And Acceptance Evidence

Minimum validation for docs-only or site-only slices:

```sh
sh scripts/validate_project_governance.sh .
git diff --check
```

Recommended validation for implementation slices:

```sh
cargo fmt --all -- --check
cargo check --workspace
sh scripts/validate_project_governance.sh .
git diff --check
```

If a Pages workflow is added, validate workflow YAML syntax through local inspection or an
approved CI dry-run. Do not push or mutate repository Pages settings without maintainer approval.

## Branch, Worktree, And Checkpoint Plan

- Preferred branch name: `docs/web-002-product-site`.
- Commit at the end of each completed week-sized slice if the maintainer has authorized commits.
- Append a checkpoint below after every task item or week.
- If interrupted, resume from the first task item whose completion gate is not satisfied.

## Allowed Permissions And External Actions

Allowed by default:

- Edit `site/`, README files, WEB-002 owner docs, this task, and derived Board entries.
- Add static assets committed to the repository when license/source is clear.
- Run local validation commands listed above.

Not allowed without explicit maintainer approval:

- Push, tag, release, deploy, change GitHub Pages settings, or change DNS.
- Add network-dependent build tools or package dependencies.
- Add telemetry, analytics, third-party scripts, forms, or external embeds.

## Destructive Or Irreversible Operations

None are authorized. Do not delete existing docs, release tags, workflow files, or internal
governance records. If cleanup seems necessary, propose it as a separate review item.

## Time, Cost, And Resource Limits

- Target duration: about 8 weeks.
- Cost: zero paid services.
- Network: avoid by default; use only maintainer-approved source checks if needed.
- Site size: keep assets small and repository-friendly; no large video or binary bundles.

## Failure, Retry, And Fallback Policy

- If a public claim cannot be verified, remove the claim or mark it as planned/research in the
  owner docs; do not guess.
- If visual polish risks schedule, ship a simple static site with accurate content first.
- If workflow configuration is uncertain, use a repository-settings checklist instead of CI.
- If README and site disagree, README/release facts win until the discrepancy is resolved.

## Default Decisions For Ambiguity

- Prefer conservative wording over marketing claims.
- Prefer plain static HTML/CSS over build tooling.
- Prefer linking to existing source docs over duplicating volatile details.
- Prefer English-first implementation with Chinese README sync; a full Chinese site can be a
  follow-up unless explicitly assigned.

## Residual-Work Destination

- Site follow-ups stay under `WEB-002`.
- Embedded/runtime web control stays under `WEB-001`, not this task.
- Release, installer, or crates.io packaging follow-ups stay under release/distribution backlog
  items.

## Programmer Handoff

You are taking a low-risk, delegated product-site and user-documentation task. Do not work on
runtime features. Your job is to make Talos easier to understand and install from public materials,
while keeping every claim tied to shipped behavior or clearly marked roadmap status.

Start by reading `WEB-002`, both README files, and this task. Build a plain static `site/` that can
be opened locally and later served by GitHub Pages. Keep internal governance docs private by not
copying task notes wholesale into the site. Do not add a web control surface, analytics, external
scripts, package managers, DNS changes, tags, releases, or runtime dependencies. When uncertain,
make the claim smaller and link to the source doc.

Each slice must update the owner docs before the Board, record validation evidence, and leave a
checkpoint here. If a requested change would touch permissions, sandboxing, provider protocol,
plugins, MCP, remote control, release publishing, or DNS, stop and escalate to the maintainer.

## Checkpoints

| Date | Completed task items | Current state and artifacts | Commands/checks and actual results | Open risks or deviations | Next task item | Recovery or resume instruction |
|---|---|---|---|---|---|---|
| 2026-06-29 | Planning only. | Delegable two-month task created for WEB-002; high-risk work remains paused. | Not run yet for implementation; run governance validation after synchronizing owner docs. | No implementation started. Domain name and Pages workflow choice remain maintainer decisions. | D0 | Resume by inventorying README/release/site claims before creating `site/`. |
| 2026-06-29 | D0 — Baseline inventory. | Source-of-truth list recorded inline below. No code changes. | Verified `git tag --list` includes `v0.2.0`; verified `Cargo.toml` `[workspace.package] version = "0.2.0"`; cross-checked README "Currently shipped" / "Not shipped yet" sections against I056 closeout and Board. | Domain name still TBD (CNAME.example only); Pages workflow choice (settings vs Actions) is a maintainer decision. | D1 | Resume by reading the D0 inventory below before designing the page map. |
| 2026-06-29 | D1 — Site information architecture. | Page map, navigation, and public/private boundary recorded inline below. No code changes. | None yet (design only). | None new. | D2 | Resume by building `site/index.html` and `site/assets/styles.css` from the D1 page map. |
| 2026-06-29 | D2 — Static site skeleton. | `site/index.html` (full home), `site/install.html` / `capabilities.html` / `safety.html` / `roadmap.html` / `releases.html` (skeletons with lead paragraphs and pointer to the GitHub source of truth), `site/404.html`, `site/CNAME.example`, `site/README.md` (maintainer notes), `site/assets/{styles.css,site.js,talos-mark.svg,favicon.svg}`. | Static link resolver (Python regex over `href` and `src`) confirms 7/7 HTML files have all relative links resolving to files in `site/`. Local `python3 -m http.server` smoke serves 200 for assets, 404 for unknown paths. No external resources, no build step, no analytics. `site/` is not in `.gitignore` and will commit cleanly. | None. Local http.server requests in this shell environment hit an HTTP_PROXY layer that occasionally returns 502, but the server log confirms the underlying server returns 200 for every page that reaches it; the static-link check is the authoritative gate. | D3 | Resume by filling `site/install.html` and `site/releases.html` from README install + release content byte-for-byte. |
| 2026-06-29 | D3 — Install and release content. | `site/install.html` now has the full install surface (shell + PowerShell installers, archive table, first-run setup, configuration management, build from source, verify-with-mock). `site/releases.html` now has the current release callout (v0.2.0), the release tag scheme, the post-release update checklist, and a section pointing at the public READMEs. | 14/14 critical install commands (shell installer, PowerShell installer, archive extraction, `--no-init`, four `--config-*` commands, two `cargo build` commands, `build.sh`, mock verify) match the README install section byte-for-byte, verified by a Python diff over `site/install.html`. | None. | D4 | Resume by filling `site/capabilities.html` and `site/safety.html` with the shipped-only capability and safety content, no roadmap overclaiming. |
| 2026-06-29 | D4 — Safety and capability content. | `site/capabilities.html` now has the built-in tools, slash commands, runtime Skills, MCP tools, and the embedding facade (all marked as shipped with the pre-1.0 caveat for `talos-runtime`). `site/safety.html` now has the three-bullet README safety model, the ADR-023 secret-masking surface, the permission posture, the sandbox boundary, and a non-goals section. | Built-in tool coverage (27/27), slash-command coverage (16/16), safety model coverage, ADR-023 surface coverage, and the no-roadmap-overclaim guard all PASS in a single Python gate. Static link resolver still passes for 7/7 files. | None. | D5 | Resume by filling `site/roadmap.html` with shipped / planned / research split. |
| 2026-06-29 | D5 — Roadmap and non-goals content. | `site/roadmap.html` now has the shipped / planned / research split for the v0.2.0 boundary: shipped capabilities are listed with the `Shipped` pill; planned items (`REL-002`, `WEBFETCH-001`, multi-resource permissions) are listed with the `Planned` pill; research items (`AGENT-002`, `WEB-001`, `PLUGIN-001`, `REMOTE-001`, `MODEL-002`) are listed with the `Research` pill. The non-goals section explicitly enumerates the things Talos is not in v0.2.0. | The D5 hard gate passes: `WEB-001`, `PLUGIN-001`, `REMOTE-001`, and `REL-002` are never inside a `Shipped` pill; every shipped item carries a `Shipped` pill. | None. | D6 | Resume by adding `site/CNAME.example` and the publishing checklist. |
| 2026-06-29 | D6 — GitHub Pages readiness. | `site/CNAME.example` carries a placeholder template with a usage note. `site/README.md` (maintainer-only) documents the repository-settings checklist: `Settings → Pages → Build and deployment → Source: "Deploy from a branch"`, branch `main`, folder `/site`, optional custom domain. No `.github/workflows/pages.yml` is added to avoid a duplicate deployment path. | Filesystem check: `site/CNAME.example` exists; `site/README.md` covers all four publishing steps including HTTPS enforcement. No CI workflow file was added. | None. Domain selection and Pages settings remain maintainer decisions. | D7 | Resume by adding the public-site row to both READMEs. |
| 2026-06-29 | D7 — README sync. | A new "Public product site" row was added to the Documentation table in `README.md` and the 文档 table in `README.zh-CN.md`. The row points at `site/` (a relative path inside the repo) rather than at a hardcoded URL, because Pages has not been enabled by the maintainer yet. | No conflicting claim introduced. Both READMEs still agree on the v0.2.0 release, the install commands, the capability list, the safety posture, and the project status. | None. | D8 | Resume by writing the validation harness. |
| 2026-06-29 | D8 — Validation harness. | `scripts/validate_public_site.sh` is a POSIX shell harness with zero runtime dependencies (uses only `find`, `awk`, `grep`, `sed`). It walks `site/`, parses every `href` and `src` in every HTML file, verifies the relative target exists, runs the four guardrails (no external scripts, no analytics, no `@import`, no external `url()`), checks that every page references the shared header assets, asserts the three critical install commands are present in `site/install.html`, and runs the D5 hard gate against the roadmap. | `sh scripts/validate_public_site.sh` exits 0 with `Errors: 0, Warnings: 0` over 7 HTML files. | None. | D9 | Resume by updating the WEB-002 owner doc and the Board. |
| 2026-06-29 | D9 — Closeout. | This row plus the rest of the closeout work. | `sh scripts/validate_project_governance.sh .` exits 0 with `0 warning(s)`. `sh scripts/validate_public_site.sh` exits 0. | None. No push, tag, release, deploy, Pages settings change, or DNS change was performed; all of those remain maintainer actions. | Done | This task is a published execution baseline. If the maintainer resumes WEB-002 follow-ups, start by re-running the two validators above and reading the "Updating the site after a release" section of `site/README.md`. |

## D0 Inventory (2026-06-29)

### Release baseline (source of truth)

- Current public release: `v0.2.0` (tag present locally, see `git tag --list`).
- Workspace metadata: `[workspace.package] version = "0.2.0"` in `Cargo.toml`.
- Prior release called out in `WEB-002`: `v0.1.2` (the v0.1.x line).
- License: `Apache-2.0` (`LICENSE`, badge in both READMEs).
- Repository: `wjhuang88/talos`.

### Public doc files (allow-list for cross-checking)

- `README.md` (English user/developer README).
- `README.zh-CN.md` (Chinese user/developer README).
- `LICENSE` (Apache-2.0 text).
- `install/install.sh` and `install/install.ps1` (user-facing release entrypoints).
- `docs/reference/ARCHITECTURE.md` (architecture facts; link to from site, do not duplicate).
- `docs/decisions/023-inline-api-key-boundary.md` (API key masking boundary; link to from site, do not duplicate).

### Internal governance files (must NOT be copied into `site/`)

- `docs/BOARD.md`, `docs/backlog/**`, `docs/iterations/**`, `docs/tasks/**`, `docs/proposals/**`,
  `docs/sop/**`, `docs/roadmap/**`, `docs/decisions/**` (except the public ADR-023 link), and any
  task checkpoints or story-detail tables. These stay private; the site may link to source-of-truth
  docs on `github.com/wjhuang88/talos` when an external visitor needs depth.

### Shipped capability claims (mirror from README "Currently shipped")

- TUI, inline, and print execution modes.
- Local provider configuration with masked secrets (per `docs/decisions/023-inline-api-key-boundary.md`).
- Built-in coding tools with permission gating.
- Session storage, search, cleanup, maintenance, memory consolidation, exploration ingestion.
- Runtime Skills from `.talos/skills/`, `~/.talos/skills/`, and inherited parent `.talos/skills/`.
- MCP stdio tools and JSON-RPC infrastructure.
- Initial `talos-runtime` embeddable facade (pre-1.0).

### Roadmap / non-goal claims (must be marked "Planned" or "Research", never "Shipped")

- Stable 1.0 SDK guarantee for `talos-runtime` → Planned, gated by `REL-002`.
- `~/.agents/skills/` dotagents shared directory discovery → Research (`AGENT-002-B`).
- Embedded browser / web control surface → Research (`WEB-001`); explicitly NOT shipped.
- WASM plugin runtime and plugin marketplace → Research (`PLUGIN-001`).
- PDF / Office document extraction beyond current web/fetch foundations → Research (`WEBFETCH-001`).
- Remote or P2P session control → Research (`REMOTE-001`).

### Release / distribution artifacts to reference (not modify)

- `install/install.sh` — macOS / Linux installer shell entrypoint.
- `install/install.ps1` — Windows PowerShell installer entrypoint.
- Archive names: `talos-x86_64-linux.tar.gz`, `talos-aarch64-linux.tar.gz`,
  `talos-x86_64-darwin.tar.gz`, `talos-aarch64-darwin.tar.gz`, `talos-x86_64-windows.zip`.
  Windows ARM64 is intentionally not published.

### Brand assets (allowed)

- `TALOS` wordmark and "⬡ The watchman never sleeps" tagline (per TUI-005, scrollback-only).
- No external image / font CDN, no analytics, no third-party scripts.

### Open decisions deferred to maintainer

- Custom domain (the `site/CNAME.example` will carry a placeholder).
- Pages publishing path: GitHub Actions Pages workflow vs repository-settings-only checklist.
- Whether to add Chinese mirror pages in a follow-up slice (default: no, sync the existing `README.zh-CN.md` only).

## D1 Site Information Architecture (2026-06-29)

### Page map (under `site/`)

| File | Audience | Purpose | Source-of-truth |
| --- | --- | --- | --- |
| `index.html` | New and evaluating users | Product positioning, current release card, install callout, link highlights. | `README.md` Highlights + Current Release Boundary sections. |
| `install.html` | New installers | All install entrypoints, archive table, first-run setup, configuration management. | `README.md` Install + Configure A Provider sections; `install/install.sh` and `install/install.ps1` (byte-for-byte). |
| `capabilities.html` | Power users | Built-in tools table, slash commands table, Skills, MCP, embedding facade. | `README.md` Built-In Capabilities, Slash Commands, Skills, MCP Tools, Embedding Talos In Rust sections. |
| `safety.html` | Operators and security reviewers | Safety model, permission posture, secret-masking boundary. | `README.md` Safety Model + `docs/decisions/023-inline-api-key-boundary.md` (link to source ADR). |
| `roadmap.html` | Anyone evaluating future work | Shipped / Planned / Research split; non-goals. | `README.md` Project Status + Board `Now` / `Next` / `Later` view (derive status only, link to owner docs). |
| `releases.html` | Maintainers and integrators | Current release tag, link to GitHub Releases, post-release update checklist. | `git tag --list`; `README.md` Contributing And Local Checks; release.yml triggers. |
| `404.html` | All | Friendly static 404 with link back to the home page. | Self-contained. |
| `assets/styles.css` | — | Single shared stylesheet, no build step. | Internal. |
| `assets/site.js` | — | Optional: copy-code buttons, current-year stamp. No analytics. | Internal. |
| `assets/talos-mark.svg` | — | Text-only "TALOS" wordmark (matches TUI-005 scrollback branding), no external font. | TUI-005 owner doc. |
| `assets/favicon.svg` | — | Small inline-friendly mark to avoid 404s on `/favicon.ico`. | Internal. |
| `CNAME.example` | Maintainer | Custom-domain placeholder. The maintainer copies it to `site/CNAME` when ready. | Maintainer decision. |
| `README.md` | Maintainer | Maintainer-only notes (publishing, updating the site). Not linked from the navigation. | Internal. |

### Navigation

Top navigation (in this order, identical on every page): Home, Install, Capabilities, Safety, Roadmap, Releases.

Footer: license link, GitHub repository link, source docs link, `last updated` stamp injected by `site.js`.

### Public / private boundary

- Public site: `site/**` is the only path exposed to GitHub Pages. Anything under `site/` is treated as public.
- Internal governance (`docs/**`, root `AGENTS.md`, `EVOLUTION.md`, task records, iteration records, Board, backlog) stays out of `site/`. The site may link to those paths on `github.com/wjhuang88/talos` when a visitor needs depth; it must not duplicate their content.
- `site/README.md` is a maintainer-only file (publishing notes). It is not in the navigation. GitHub Pages will serve it on `/README.md` if the maintainer does not exclude it; maintainer may rename to `MAINTENANCE.md` and link from the publishing notes if preferred. Default: keep `site/README.md` and add `<meta name="robots" content="noindex">` in its `<head>`.

### Style baseline

- Plain HTML5 with semantic tags (`<header>`, `<main>`, `<nav>`, `<article>`, `<footer>`).
- Single shared `assets/styles.css` with CSS custom properties for theme tokens. Light + dark via `prefers-color-scheme`.
- System font stack (no Google Fonts, no external CDN).
- No JavaScript framework, no bundler, no package manager.
- Optional `assets/site.js` for: current-year stamp, "copy" button on code blocks. No analytics, no tracking, no third-party scripts.
