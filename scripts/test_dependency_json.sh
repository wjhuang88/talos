#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
root=$(cd "$(dirname "$0")/.." && pwd)
parser="$root/scripts/lib/dependency_json.awk"
valid=(
  '{"packages":[{"name":"alias","optional":true}],"resolve":null}'
  '{"a":"quote\" and slash\\","b":"\u4e2d\ud83d\ude00","n":-1.25e+9}'
  '[null,false,true,0,1.2,-3E2,{},[]]'
)
invalid=(
  '' '{' '[1,]' '{"x":1,}' '{"x":01}' 'true false'
  '{"a":1,"\u0061":2}' '"\ud800"' '"\udc00"' '"\q"'
  '"\u0000"' '1.' '[1 2]' '{"x" 1}'
)
for input in "${valid[@]}"; do
  printf '%s\n' "$input" | awk -v json_mode=validate -f "$parser" >/dev/null
done
for input in "${invalid[@]}"; do
  if printf '%s\n' "$input" | awk -v json_mode=validate -f "$parser" >/dev/null 2>&1; then
    printf 'unexpected acceptance: %s\n' "$input" >&2
    exit 1
  fi
done
printf 'dependency JSON parser: PASS (%s valid, %s rejected)\n' "${#valid[@]}" "${#invalid[@]}"
