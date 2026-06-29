# site/

This directory is the public Talos product site. It is published via
[GitHub Pages](https://docs.github.com/en/pages) and is intentionally kept
separate from the internal engineering docs in `docs/`.

## Layout

| Path | Purpose |
| --- | --- |
| `index.html` | Product overview and current release callout. |
| `install.html` | Install instructions and the release archive table. |
| `capabilities.html` | Built-in tools, slash commands, Skills, MCP. |
| `safety.html` | Safety model, permission posture, secret masking. |
| `roadmap.html` | Shipped / Planned / Research split. |
| `releases.html` | Current release tag, update checklist, release-history link. |
| `404.html` | Static 404. |
| `assets/styles.css` | Single shared stylesheet. No build step. |
| `assets/site.js` | Minimal JS: footer year stamp, copy-code buttons. No analytics. |
| `assets/talos-mark.svg` | Branded wordmark (matches TUI-005 scrollback mark). |
| `assets/favicon.svg` | Inline-friendly favicon. |
| `CNAME.example` | Custom-domain template. Copy to `CNAME` when a domain is ready. |

## Local preview

The site is plain static HTML/CSS/JS, so any of these work without a build
step:

```sh
# macOS, using the system Python
cd site && python3 -m http.server 8000

# macOS, using the system Ruby
cd site && ruby -run -e httpd . -p 8000

# or just open index.html in a browser
open site/index.html
```

## Publishing

GitHub Pages does not support `/site` as a branch-deployment folder (only `/`
and `/docs` are available). This repository uses a lightweight
[GitHub Actions workflow](../.github/workflows/pages.yml) to deploy the site
instead.

### One-time setup

1. Repository settings &rarr; Pages &rarr; Build and deployment &rarr; Source:
   "GitHub Actions". The workflow is already committed; it takes effect after
   one push to `main` that touches `site/`.
2. (Optional, when a domain is selected) Custom domain: enter the bare domain
   in the Pages settings UI. The `site/CNAME` file must also contain the same
   domain on a single line.
3. (Optional) Enforce HTTPS once the certificate is provisioned.

After setup, every push to `main` that changes files under `site/` triggers
`pages.yml`, which uploads `site/` as an artifact and deploys it.

## What does not belong here

- Internal governance: `docs/BOARD.md`, `docs/backlog/**`, `docs/iterations/**`,
  `docs/tasks/**`, `docs/proposals/**`, `docs/sop/**`, `docs/roadmap/**`,
  `docs/decisions/**` (except when a decision is part of the public boundary,
  such as ADR-023 for the API-key masking story).
- Internal task notes and per-iteration checkpoints.
- Anything that depends on a network call (analytics, fonts, CDNs,
  third-party scripts).
- Anything that requires a build tool, package manager, or framework.

## Updating the site after a release

After the maintainer tags a new release (e.g. `v0.2.1`), the public
materials must be updated in this order:

1. Update `README.md` and `README.zh-CN.md` with the new tag, install
   changes, and capability boundary.
2. Update the version string on the home page (`index.html`) and the current
   release card on the releases page (`releases.html`).
3. Run the D8 static-validation harness to confirm internal links and
   relative paths still resolve.
4. Open a PR titled `docs(site): sync vX.Y.Z release notes` referencing the
   iteration or release that produced the change.
5. After the PR merges, GitHub Pages publishes on its normal schedule;
   no separate release action is required for the site.

## Owned by

- Backlog item: `docs/backlog/active/WEB-002-github-pages-product-site.md`
- Two-month plan: `docs/tasks/2026-06-29-delegable-product-site-docs-two-month-plan.md`
- Brand: TUI-005 (`docs/backlog/active/TUI-005-logo-splash.md`).
