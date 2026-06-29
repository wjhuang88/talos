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
