#!/usr/bin/env bash
# Offline audit tests. PowerShell is required for cross-frontend parity tests only,
# not for the Unix audit command itself. Never performs registry lookups.
set -euo pipefail
export LC_ALL=C
root=$(cd "$(dirname "$0")/.." && pwd)
command -v pwsh >/dev/null || { echo 'parity tests require pwsh' >&2; exit 2; }
bash -n "$root/scripts/dependency_audit.sh"
bash "$root/scripts/test_dependency_json.sh"
awk -f "$root/scripts/lib/dependency_versions.awk" \
  -f "$root/scripts/test_dependency_versions.awk" "$root/tests/fixtures/dependency-version-cases.tsv"
bash "$root/scripts/test_dependency_registry.sh"
pwsh -NoProfile -File "$root/scripts/test_dependency_versions.ps1"
pwsh -NoProfile -File "$root/scripts/test_dependency_audit_mapping.ps1"
pwsh -NoProfile -File "$root/scripts/test_dependency_failures.ps1"
pwsh -NoProfile -File "$root/scripts/test_dependency_baseline.ps1"
pwsh -NoProfile -File "$root/scripts/test_dependency_snapshot.ps1"
pwsh -NoProfile -File "$root/scripts/test_dependency_baseline_table.ps1"
pwsh -NoProfile -File "$root/scripts/test_dependency_candidates.ps1"
bash "$root/scripts/test_dependency_audit_parity.sh"
