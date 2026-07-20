#!/bin/sh
# Public site static validator.
#
# Walks `site/`, verifies that every HTML page's internal links resolve
# to files inside `site/`, and checks for a few hard guardrails the
# public site commits to: no external scripts, no analytics, no
# third-party font or stylesheet imports. Also re-checks the byte-for-byte
# alignment of the install commands against the root README.
#
# Usage:
#   sh scripts/validate_public_site.sh
#
# Exit code 0 on success, 1 on any check failure.

set -u

script_dir=$(cd "$(dirname "$0")" && pwd)
project_root=$(cd "$script_dir/.." && pwd)
site_root="$project_root/site"
readme="$project_root/README.md"
readme_zh="$project_root/README.zh-CN.md"

errors=0
warnings=0

log_error() {
  printf 'ERROR: %s\n' "$1"
  errors=$((errors + 1))
}

log_warn() {
  printf 'WARNING: %s\n' "$1"
  warnings=$((warnings + 1))
}

log_info() {
  printf '  %s\n' "$1"
}

# 1. Site directory must exist
if [ ! -d "$site_root" ]; then
  log_error "site/ directory not found at $site_root"
  exit 1
fi

# 2. Required files
for required in index.html install.html docs.html capabilities.html safety.html roadmap.html releases.html 404.html assets/styles.css assets/site.js CNAME.example README.md zh/index.html zh/install.html zh/docs.html zh/capabilities.html zh/safety.html zh/roadmap.html zh/releases.html zh/404.html; do
  if [ ! -f "$site_root/$required" ]; then
    log_error "site/$required is missing"
  fi
done

# 3. Walk every HTML page, parse href and src attributes, check that
#    relative references resolve to a real file inside site/.
html_count=0
for html in $(find "$site_root" -name '*.html' -type f | sort); do
  rel=${html#"$site_root"/}
  html_count=$((html_count + 1))
done

for html in $(find "$site_root" -name '*.html' -type f | sort); do
  rel=${html#"$site_root"/}
  awk '
    match($0, /href[[:space:]]*=[[:space:]]*"[^"]+"/) {
      s = substr($0, RSTART, RLENGTH)
      sub(/^href[[:space:]]*=[[:space:]]*"/, "", s)
      sub(/"$/, "", s)
      print s
    }
    match($0, /src[[:space:]]*=[[:space:]]*"[^"]+"/) {
      s = substr($0, RSTART, RLENGTH)
      sub(/^src[[:space:]]*=[[:space:]]*"/, "", s)
      sub(/"$/, "", s)
      print s
    }
  ' "$html" | while IFS= read -r ref; do
    case "$ref" in
      http://*|https://*|mailto:*|"#"|""|data:*|javascript:*)
        continue
        ;;
    esac
    ref_path=$(printf '%s' "$ref" | sed -e 's/#.*$//' -e 's/?.*$//')
    if [ -z "$ref_path" ]; then
      continue
    fi
    html_dir=$(dirname "$html")
    target="$html_dir/$ref_path"
    if [ ! -e "$target" ]; then
      printf 'site/%s: %s\n' "$rel" "$ref"
    fi
  done
done > "$site_root/.broken-links.tmp" 2>/dev/null
if [ -s "$site_root/.broken-links.tmp" ]; then
  while IFS= read -r line; do
    log_error "broken link: $line"
  done < "$site_root/.broken-links.tmp"
fi

# 2b. The public IA is mirrored: every page must expose the documentation
# hub and each English page has an equivalent zh-CN page.
for page in index install docs capabilities safety roadmap releases 404; do
  if [ ! -f "$site_root/$page.html" ] || [ ! -f "$site_root/zh/$page.html" ]; then
    log_error "site locale pair is incomplete for $page"
    continue
  fi
  if ! grep -F 'href="docs.html"' "$site_root/$page.html" >/dev/null 2>&1; then
    log_error "site/$page.html is missing the Documentation navigation link"
  fi
  if ! grep -F 'href="docs.html"' "$site_root/zh/$page.html" >/dev/null 2>&1; then
    log_error "site/zh/$page.html is missing the 文档 navigation link"
  fi
done

# The documentation hubs must retain the same release-grade section IA.
for section in quick-start configuration models modes tools sessions extensions safety troubleshooting; do
  if ! grep -F "id=\"$section\"" "$site_root/docs.html" >/dev/null 2>&1; then
    log_error "site/docs.html is missing #$section"
  fi
  if ! grep -F "id=\"$section\"" "$site_root/zh/docs.html" >/dev/null 2>&1; then
    log_error "site/zh/docs.html is missing #$section"
  fi
done

# v0.4.0 is the current public release. Historical GitHub URLs are allowed,
# but stale v0.2.2 copy is not.
if grep -rEn 'v?0\.2\.2' "$site_root" --include='*.html' >/dev/null 2>&1; then
  log_error "site/ still claims stale v0.2.2 release content"
fi
if ! grep -F 'v0.4.0' "$site_root/index.html" >/dev/null 2>&1 || ! grep -F 'v0.4.0' "$site_root/zh/index.html" >/dev/null 2>&1; then
  log_error "home pages must name the current v0.4.0 release"
fi
if ! grep -F 'v0.4.0' "$readme" >/dev/null 2>&1 || ! grep -F 'v0.4.0' "$readme_zh" >/dev/null 2>&1; then
  log_error "English and zh-CN READMEs must name the current v0.4.0 release"
fi

# CTA regression guard: prose-link styling must not override button foregrounds.
if ! grep -F '.talos-main a:not(.talos-button)' "$site_root/assets/styles.css" >/dev/null 2>&1; then
  log_error "stylesheet lacks the scoped prose-link selector"
fi
if ! grep -F '.talos-main .talos-button' "$site_root/assets/styles.css" >/dev/null 2>&1 || ! grep -F 'color: #fff;' "$site_root/assets/styles.css" >/dev/null 2>&1 || ! grep -F '.talos-button:focus-visible' "$site_root/assets/styles.css" >/dev/null 2>&1; then
  log_error "stylesheet lacks the CTA foreground or keyboard-focus contract"
fi
rm -f "$site_root/.broken-links.tmp"

# 4. Guardrail: no external scripts, no analytics
if grep -rEn '<script[^>]+src[[:space:]]*=' "$site_root" --include='*.html' 2>/dev/null | grep -E 'https?://' >/dev/null; then
  log_error "site/ has an external <script src=...> reference; not allowed"
fi
for needle in google-analytics googletagmanager gtag plausible umami fathom segment.com hotjar; do
  if grep -rEin "$needle" "$site_root" 2>/dev/null >/dev/null; then
    log_error "site/ contains analytics token: $needle"
  fi
done

# 5. Guardrail: no external stylesheets or fonts
if grep -rEn '@import' "$site_root/assets" 2>/dev/null >/dev/null; then
  log_error "site/assets has @import; not allowed (no external font/style imports)"
fi
if grep -rEn 'url\([\"'\'']?https?://' "$site_root/assets" 2>/dev/null >/dev/null; then
  log_error "site/assets has url() pointing at an external host; not allowed"
fi

# 6. Guardrail: every page must include the shared header assets
for html in $(find "$site_root" -name '*.html' -type f | sort); do
  rel=${html#"$site_root"/}
  if ! grep -q 'assets/styles.css' "$html" && ! grep -q '../assets/styles.css' "$html"; then
    log_error "site/$rel is missing shared stylesheet reference"
  fi
  if ! grep -q 'assets/site.js' "$html" && ! grep -q '../assets/site.js' "$html"; then
    log_warn "site/$rel is missing shared site.js reference (allowed only on 404)"
  fi
done

# 7. Byte-for-byte install command alignment with README
if [ -f "$readme" ]; then
  install_html="$site_root/install.html"
  if [ -f "$install_html" ]; then
    expected='curl -fsSL https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.sh | sh'
    if ! grep -F "$expected" "$install_html" >/dev/null 2>&1; then
      log_error "site/install.html is missing the shell installer command"
    fi
    expected='iex (irm https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.ps1)'
    if ! grep -F "$expected" "$install_html" >/dev/null 2>&1; then
      log_error "site/install.html is missing the PowerShell installer command"
    fi
    expected='talos -p --mock "/mock-request summarize this repository"'
    if ! grep -F "$expected" "$install_html" >/dev/null 2>&1; then
      log_error "site/install.html is missing the mock verify command"
    fi
  else
    log_error "site/install.html not found"
  fi
fi

# 8. Roadmap hard gate: WEB-001 / PLUGIN-001 / REMOTE-001 / REL-002 must
#    never appear under a Shipped pill.
roadmap_html="$site_root/roadmap.html"
if [ -f "$roadmap_html" ]; then
  for token in WEB-001 PLUGIN-001 REMOTE-001 REL-002; do
    # Find the <li> that contains the token, then check it does not also
    # contain the Shipped pill. We do this with a small awk pass.
    awk -v tok="$token" '
      /<li>/ { buf = ""; capturing = 1 }
      capturing { buf = buf ORS $0 }
      /<\/li>/ {
        if (capturing && index(buf, tok) > 0) {
          if (index(buf, "talos-pill--shipped") > 0) {
            printf "site/roadmap.html: %s is marked Shipped (not allowed)\n", tok
            exit 1
          }
        }
        capturing = 0
      }
    ' "$roadmap_html" || log_error "site/roadmap.html marks $token as Shipped (not allowed)"
  done
fi

# Summary
printf '\n'
printf 'Public site validation summary\n'
printf '  HTML files checked:   %d\n' "$html_count"
printf '  Errors:               %d\n' "$errors"
printf '  Warnings:             %d\n' "$warnings"
printf '\n'

if [ "$errors" -gt 0 ]; then
  exit 1
fi
exit 0
