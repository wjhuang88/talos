#!/usr/bin/env bash
# Installer validation script.
#
# Validates install/install.sh and install/install.ps1 against:
# - Canonical GitHub release URLs
# - Platform archive naming (matches README table)
# - Offline/error handling behavior (graceful failure, not silent crash)
# - Site install.html references the same install commands
#
# Usage:
#   scripts/validate_installers.sh
#
# Origin: I118 LT032 — bounded local productization installer validation.
# Exit code 0 on success, 1 on any check failure.

set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
project_root=$(cd "$script_dir/.." && pwd)
install_dir="$project_root/install"
site_dir="$project_root/site"
readme="$project_root/README.md"

errors=0

log_error() {
  printf 'ERROR: %s\n' "$1"
  errors=$((errors + 1))
}

log_ok() {
  printf '  OK: %s\n' "$1"
}

echo "============================================"
echo "Installer Validation"
echo "============================================"
echo ""

# 1. Install scripts must exist
echo "1. Install script existence"
for f in install.sh install.ps1; do
  if [ -f "$install_dir/$f" ]; then
    log_ok "$f exists"
  else
    log_error "install/$f is missing"
  fi
done
echo ""

# 2. install.sh must reference the correct GitHub release URLs
echo "2. Canonical release URLs"
if grep -q 'github.com.*releases/download\|api.github.com/repos' "$install_dir/install.sh"; then
  log_ok "install.sh references GitHub releases"
else
  log_error "install.sh does not reference GitHub releases"
fi

if grep -q 'github.com.*releases/download\|api.github.com/repos' "$install_dir/install.ps1"; then
  log_ok "install.ps1 references GitHub releases"
else
  log_error "install.ps1 does not reference GitHub releases"
fi
echo ""

# 3. Platform archive names must match README
echo "3. Platform archive naming"
for archive in talos-x86_64-linux.tar.gz talos-aarch64-linux.tar.gz \
  talos-x86_64-darwin.tar.gz talos-aarch64-darwin.tar.gz \
  talos-x86_64-windows.zip; do
  if grep -q "$archive" "$install_dir/install.sh" || grep -q "$archive" "$install_dir/install.ps1"; then
    log_ok "$archive referenced in installer"
  else
    # Some archives are platform-specific to one script; check README instead
    if grep -q "$archive" "$readme"; then
      log_ok "$archive documented in README"
    else
      log_error "$archive not found in installers or README"
    fi
  fi
done
echo ""

# 4. Offline/error handling: scripts must exit non-zero on failure
echo "4. Error handling"
if grep -q 'set -e\|set -eu\|set -euo\|exit 1\|exit\b.*[1-9]' "$install_dir/install.sh"; then
  log_ok "install.sh has explicit error exit"
else
  log_error "install.sh lacks explicit error exit"
fi

if grep -q 'exit 1\|throw\|Write-Error' "$install_dir/install.ps1"; then
  log_ok "install.ps1 has explicit error exit"
else
  log_error "install.ps1 lacks explicit error exit"
fi
echo ""

# 5. Install commands must match site/install.html
echo "5. Site/install.html alignment"
install_html="$site_dir/install.html"
if [ -f "$install_html" ]; then
  expected_sh='curl -fsSL https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.sh | sh'
  if grep -F "$expected_sh" "$install_html" >/dev/null 2>&1; then
    log_ok "site/install.html has correct shell install command"
  else
    log_error "site/install.html missing correct shell install command"
  fi

  expected_ps='iex (irm https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.ps1)'
  if grep -F "$expected_ps" "$install_html" >/dev/null 2>&1; then
    log_ok "site/install.html has correct PowerShell install command"
  else
    log_error "site/install.html missing correct PowerShell install command"
  fi
else
  log_error "site/install.html not found"
fi
echo ""

# 6. No credential or secret references in install scripts
echo "6. Credential safety"
if grep -qiE 'api_key|secret|password|token' "$install_dir/install.sh" 2>/dev/null; then
  log_error "install.sh contains credential-like string"
else
  log_ok "install.sh has no credential-like strings"
fi

if grep -qiE 'api_key|secret|password|token' "$install_dir/install.ps1" 2>/dev/null; then
  log_error "install.ps1 contains credential-like string"
else
  log_ok "install.ps1 has no credential-like strings"
fi
echo ""

# Summary
echo "============================================"
echo "Installer Validation Summary"
echo "  Errors: $errors"
echo "============================================"

if [ "$errors" -gt 0 ]; then
  exit 1
fi
exit 0
