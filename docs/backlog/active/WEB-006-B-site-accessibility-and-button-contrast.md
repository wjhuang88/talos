# WEB-006-B: Site Accessibility And Primary Button Contrast

| Field | Value |
|---|---|
| Type | Product/UX Story |
| Parent Epic | WEB-006 |
| Status | Ready |
| Priority | P1 |

## Problem And Root Cause

The home-page “Install Talos” text is unreadable. The general
`.talos-main a` selector has greater specificity than
`.talos-button--primary`, so it overrides the intended white CTA text with the
normal link color.

## Scope

- Fix the cascade using a component-scoped selector/token design; do not use
  `!important`.
- Verify primary/secondary buttons and normal links in light/dark plus
  normal/hover/focus-visible/visited states.
- Provide a visible keyboard focus treatment and preserve underline behavior for
  prose links without adding it to buttons.
- Check text/background contrast against WCAG AA: at least 4.5:1 for normal text
  and 3:1 for large text/UI boundaries.

## Acceptance

- Given the home page in light or dark mode, when primary CTA state changes, then
  its label remains readable and computed color is the component foreground token.
- Given keyboard navigation, when a button receives focus, then a non-color-only
  focus indicator is visible.
- Given ordinary prose links, when style changes land, then their existing semantic
  link appearance is preserved.
- A regression check fails if `.talos-main a` can again override primary/secondary
  button foregrounds.

## Validation

- Static CSS selector/token guard in `scripts/validate_public_site.sh`.
- Browser computed-style and screenshot matrix for two themes and interactive states.
- `git diff --check`.

## Required Reads

- Parent WEB-006
- `site/index.html`, `site/zh/index.html`
- `site/assets/styles.css`
- `docs/backlog/active/WEB-004-site-theme-branding.md`
