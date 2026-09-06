#!/usr/bin/env bash
set -euo pipefail
format=table; live=0
while (($#)); do case "$1" in --format) format="${2:?}"; shift 2;; --live) live=1; shift;; *) echo "unknown argument: $1" >&2; exit 2;; esac; done
root=$(cd "$(dirname "$0")/.." && pwd)
tree=$(cargo tree --workspace --depth 1 --prefix none 2>/dev/null)
if ((live)); then
  command -v curl >/dev/null 2>&1 || { echo 'registry-unavailable' >&2; exit 3; }
  failed=0
  while read -r name version; do
    [[ -n "$name" ]] || continue
    response=$(curl --fail --silent --show-error --max-time 10 "https://crates.io/api/v1/crates/$name" 2>/dev/null) || { failed=1; continue; }
    latest=$(printf '%s' "$response" | sed -n 's/.*"newest_version":"\([0-9][^"]*\)".*/\1/p')
    [[ -n "$latest" ]] || failed=1
  done < <(printf '%s\n' "$tree" | awk 'BEGIN{inroot=0} /^[A-Za-z0-9_-]+ v[0-9].*\/crates\//{inroot=1;next} inroot && /^[A-Za-z0-9_-]+ v[0-9]/{print $1, $2}')
  ((failed == 0)) || { echo 'registry-unavailable' >&2; exit 3; }
fi
case "$format" in
  json)
    printf '%s\n' "$tree" | awk '
      BEGIN{printf "{\"schema\":\"talos.dependency-audit/v1\",\"status\":\"local-only\",\"registry\":\"not-queried\",\"dependencies\":["; first=1; inroot=0}
      /^[A-Za-z0-9_-]+ v[0-9].*\/crates\//{inroot=1; next}
      inroot && /^[A-Za-z0-9_-]+ v[0-9]/{if(!first)printf ",";first=0;printf "{\"name\":\"%s\",\"resolved_versions\":[\"%s\"],\"classification\":[\"registry-unavailable\"]}",$1,substr($2,2)}
      END{printf "],\"partial\":true}\n"}' ;;
  table|markdown) printf 'schema: talos.dependency-audit/v1\nstatus: local-only\nregistry: not-queried\nclassification: registry-unavailable\n'; printf '%s\n' "$tree" | awk 'BEGIN{inroot=0} /^[A-Za-z0-9_-]+ v[0-9].*\/crates\//{inroot=1;next} inroot && /^[A-Za-z0-9_-]+ v[0-9]/{print $1 "\t" substr($2,2) "\tregistry-unavailable"}' ;;
  *) echo 'format must be table, json, or markdown' >&2; exit 2;;
esac
