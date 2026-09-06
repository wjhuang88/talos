#!/usr/bin/env bash
# Render only the generated Markdown block; never overwrite acceptance or policy.
set -euo pipefail
export LC_ALL=C
root=$(cd "$(dirname "$0")/.." && pwd)
awk -f "$root/scripts/lib/dependency_json.awk" \
  -f "$root/scripts/lib/dependency_emit.awk" \
  -f "$root/scripts/lib/dependency_baseline_table.awk" "${1:-$root/docs/reference/dependency-baseline.json}"
