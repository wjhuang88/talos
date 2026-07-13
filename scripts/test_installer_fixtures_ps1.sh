#!/usr/bin/env bash
# Bash wrapper for PowerShell installer fixture tests.
# Checks for pwsh availability and delegates to install_fixtures.ps1.
set -euo pipefail

if ! command -v pwsh >/dev/null 2>&1; then
  echo "SKIP: PowerShell (pwsh) not installed on this platform; installer fixture tests for install.ps1 are untested here. CI on Windows must run them."
  exit 0
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
pwsh -NoProfile -NonInteractive -File "$root/scripts/install_fixtures.ps1"
