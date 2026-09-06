#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
root=$(cd "$(dirname "$0")/.." && pwd)
select_version() {
  awk -f "$root/scripts/lib/dependency_json.awk" \
    -f "$root/scripts/lib/dependency_versions.awk" \
    -f "$root/scripts/lib/dependency_registry.awk"
}
input='{"crate":{"newest_version":"9.0.0-rc.1"},"versions":[{"num":"1.9.0","yanked":false},{"num":"1.10.0","yanked":false},{"num":"9.0.0-rc.1","yanked":false},{"num":"3.0.0","yanked":true}]}'
actual=$(printf '%s\n' "$input" | select_version)
[[ "$actual" == '{"latest_stable":"1.10.0","yanked_versions":["3.0.0"]}' ]]
actual=$(printf '%s\n' '{"versions":[{"num":"2.0.0-alpha.1","yanked":false}]}' | select_version)
[[ "$actual" == '{"latest_stable":null,"yanked_versions":[]}' ]]
if printf '%s\n' '{"versions":[{"num":"1.0.0"}]}' | select_version >/dev/null 2>&1; then
  echo 'missing yanked state was accepted' >&2; exit 1
fi
echo 'awk registry selection: PASS (numeric stable selection, yanked, prerelease-only, malformed)'
