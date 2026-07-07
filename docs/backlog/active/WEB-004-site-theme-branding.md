# WEB-004: Product Site Theme & Branding Optimization

**Status**: Complete (2026-07-06, F15 of the frontline four-month execution plan — verified already-shipped work)
**Priority**: P3
**Created**: 2026-06-29
**Source**: User request after WEB-002 site deployment
**Depends on**: WEB-002 (site structure on `main`)

## Problem

The Talos product site (`site/`) uses a generic color palette — blue accent (`#2b6cb0` light, `#6aa9e8` dark), plain monospace text logo, and a simple hexagon favicon — that does not reflect the project's own brand identity. The Talos TUI uses the **Nord** color palette (Frost cyan/blue-green gradients, Polar Night dark backgrounds, Snow Storm light foregrounds) and a hexagon-motif mark. The public site should visually cohere with the product it represents.

Current gap:

- **Colors**: Generic blue accent, no connection to Nord's Frost (`#8fbcbb`, `#88c0d0`, `#81a1c1`, `#5e81ac`) or Polar Night (`#2e3440`, `#3b4252`, `#434c5e`, `#4c566a`)
- **Logo** (`talos-mark.svg`): Plain monospace text "TALOS" with `currentColor` — no brand mark, no gradient, no visual weight
- **Favicon** (`favicon.svg`): Minimal hexagon outline + "T" — functional but not distinctive
- **No brand-asset system**: No reusable brand colors, no inline SVG symbol system, no mark–wordmark pairing

## Identity / Goal / Value

Align the product site's visuals with the Talos brand — Nord color palette, hexagon brand mark, cohesive dark/light themes — without introducing build tools, JS frameworks, or external assets.

## Scope

- **Color palette**: Replace generic blue accent with Nord-based tokens:
  - Frost (`--talos-accent`: `#88c0d0` dark, `#5e81ac` light or similar) as primary accent
  - Polar Night (`--talos-bg`: `#2e3440` dark) for dark background
  - Aurora accent colors for status pills (shipped/planned/research) to match TUI semantic colors
  - Maintain light/dark `prefers-color-scheme` support, but with Nord-aligned values
- **Logo redesign** (`talos-mark.svg`):
  - Add hexagon mark element (matching the `&#x2B21;` / TUI-005 hexagon motif)
  - Optionally add Nord Frost gradient to the wordmark
  - Keep it as pure inline SVG (no external deps)
- **Favicon redesign** (`favicon.svg`):
  - Update to match the new logo — hexagon mark with Nord accent color
- **Header brand area**: Style the `.talos-brand__mark` hexagon character with Nord gradient or accent
- **Conservative approach**: All changes in `site/assets/` only (CSS + SVGs); no HTML structure changes unless needed for new brand elements

## Exclusions

- No build tool, bundler, or CSS preprocessor
- No external fonts, icons, or third-party brand assets
- No JS framework or runtime brand injection
- No animation system or canvas-based logo rendering
- No HTML restructuring beyond what's needed for brand mark integration (minimal)
- No TUI/Terminal brand changes — site only
- No changes to the product's internal TUI Nord theme (already done)

## Acceptance Criteria

1. [x] Dark mode uses Polar Night (`#2e3440`-ish) background with Frost accent
       — `--talos-bg: #2e3440` and `--talos-accent: #88c0d0` in the dark `@media (prefers-color-scheme: dark)` block (`site/assets/styles.css:38,42`).
2. [x] Light mode uses Snow Storm-inspired background with deeper Frost accent
       — `--talos-bg: #eceff4` (Snow Storm) and `--talos-accent: #5e81ac` (Frost dark blue) at `site/assets/styles.css:12,16`.
3. [x] Logo SVG shows a hexagon mark + "TALOS" wordmark, visually cohesive with TUI splash brand
       — `site/assets/talos-mark.svg` renders a hexagon polygon with Nord Frost gradient (`#88c0d0` → `#81a1c1` → `#5e81ac`) alongside the TALOS monospace wordmark.
4. [x] Favicon SVG matches the hexagon mark
       — `site/assets/favicon.svg` renders a hexagon polygon with `#88c0d0` stroke and a "T" inside.
5. [x] Status pills (`--talos-shipped`, `--talos-planned`, `--talos-research`) use Aurora-inspired colors (green, yellow, purple) consistent with TUI
       — `--talos-shipped: #a3be8c` (Aurora green), `--talos-planned: #ebcb8b` (Aurora yellow), `--talos-research: #b48ead` (Aurora purple); applied via `.talos-pill--shipped/planned/research` classes used on roadmap pages.
6. [x] All 7 site pages render correctly with the new theme (no visual regressions)
       — all 7 EN pages reference `assets/styles.css` + `assets/talos-mark.svg`/`favicon.svg`; verified via `rg styles\.css|talos-mark|favicon` (2 matches each).
7. [x] `scripts/validate_public_site.sh` still passes (0 errors, 0 warnings)
       — verified 2026-07-06: 14 HTML files checked, 0 errors, 0 warnings.
8. [x] Light/dark `prefers-color-scheme` switching works correctly for all new color tokens
       — `@media (prefers-color-scheme: dark)` block (`site/assets/styles.css:34`) overrides every token.

## Required Reads

- `docs/backlog/active/WEB-002-github-pages-product-site.md`
- `crates/talos-tui/src/theme.rs` (Nord palette constants)
- `docs/backlog/active/TUI-005-logo-splash.md` (brand design decisions)
- `site/assets/styles.css` (current tokens)
- `site/assets/talos-mark.svg` (current logo)
- `site/assets/favicon.svg` (current favicon)
- All 7 `site/*.html` files (header brand area)
- `scripts/validate_public_site.sh`

## Design Notes

- Nord palette reference: https://www.nordtheme.com/docs/colors-and-palettes
- The TUI's Nord theme uses: NORD0–NORD3 (Polar Night), NORD4–NORD6 (Snow Storm), NORD7–NORD10 (Frost), NORD11–NORD15 (Aurora)
- For CSS (no RGB color functions), approximate with hex: `#2e3440` (dark bg), `#88c0d0` (Frost cyan accent), `#81a1c1` (Frost blue accent), `#5e81ac` (Frost dark blue)
- Logo should reference the TUI's hexagon motif (`&#x2B21;` character, hexagon geometry in favicon)
- Keep SVG inline and pure — no raster images, no base64
