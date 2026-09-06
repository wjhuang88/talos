#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
format=table
metadata_path=
registry_path=
baseline_path=
live=0
snapshot=0
export TALOS_SNAPSHOT_SOURCE= TALOS_SNAPSHOT_TIME= TALOS_SNAPSHOT_EVIDENCE=
audit_tmp=
while (($#)); do
  case "$1" in
    --format) format="${2:?missing format}"; shift 2 ;;
    --metadata) metadata_path="${2:?missing metadata path}"; shift 2 ;;
    --registry) registry_path="${2:?missing registry path}"; shift 2 ;;
    --baseline) baseline_path="${2:?missing baseline path}"; shift 2 ;;
    --live) live=1; shift ;;
    --snapshot) snapshot=1; shift ;;
    --source-commit) TALOS_SNAPSHOT_SOURCE="${2:?missing source commit}"; shift 2 ;;
    --accepted-at) TALOS_SNAPSHOT_TIME="${2:?missing acceptance timestamp}"; shift 2 ;;
    --validation-evidence) TALOS_SNAPSHOT_EVIDENCE="${2:?missing validation evidence}"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
case "$format" in json|table|markdown) ;; *) echo 'format must be json, table or markdown' >&2; exit 2;; esac
if ((live)) && [[ -n "$registry_path" ]]; then echo '--live and --registry are mutually exclusive' >&2; exit 2; fi
if ((snapshot)) && { ((live)) || [[ -n "$registry_path" || -n "$baseline_path" || "$format" != json ]]; }; then
  echo 'snapshot requires JSON format and local collection only' >&2; exit 2
fi
if ((!snapshot)) && [[ -n "$TALOS_SNAPSHOT_SOURCE$TALOS_SNAPSHOT_TIME$TALOS_SNAPSHOT_EVIDENCE" ]]; then
  echo 'snapshot provenance requires --snapshot' >&2; exit 2
fi
root=$(cd "$(dirname "$0")/.." && pwd)
collect() {
  awk -f "$root/scripts/lib/dependency_json.awk" -f "$root/scripts/lib/dependency_collect.awk"
}
if [[ -n "$metadata_path" ]]; then
  report=$(collect < "$metadata_path")
else
  report=$(cargo metadata --locked --offline --all-features --format-version 1 --manifest-path "$root/Cargo.toml" | collect)
fi
if ((snapshot)); then
  printf '%s\n' "$report" | awk -f "$root/scripts/lib/dependency_json.awk" \
    -f "$root/scripts/lib/dependency_emit.awk" -f "$root/scripts/lib/dependency_snapshot.awk"
  exit
fi
registry_mode=fixture
if ((live)); then
  command -v curl >/dev/null || { echo 'live audit requires curl' >&2; exit 2; }
  audit_tmp=$(mktemp -d)
  trap 'rm -f "$audit_tmp/response.json" "$audit_tmp/registry.json"; rmdir "$audit_tmp"' EXIT
  names=$(printf '%s\n' "$report" | awk -f "$root/scripts/lib/dependency_json.awk" \
    -f "$root/scripts/lib/dependency_registry_names.awk")
  registry_path="$audit_tmp/registry.json"
  registry_mode=queried
  {
    printf '{'
    separator=
    while IFS= read -r name; do
      [[ -n "$name" ]] || continue
      printf '%s"%s":' "$separator" "$name"
      separator=,
      if curl --fail --silent --show-error --max-time 10 --max-filesize 8388608 \
        --user-agent 'talos-dependency-audit/1' \
        --output "$audit_tmp/response.json" "https://crates.io/api/v1/crates/$name"; then
        if awk -f "$root/scripts/lib/dependency_json.awk" "$audit_tmp/response.json"; then
          cat "$audit_tmp/response.json"
        else
          # Valid transport but malformed evidence is unknown, not unavailable.
          printf '{}'
        fi
      else
        printf 'null'
      fi
    done <<< "$names"
    printf '}\n'
  } > "$registry_path"
fi
if [[ -n "$registry_path" ]]; then
  report=$({ printf '{"audit":%s,"registry":' "$report"; cat "$registry_path"; printf '}\n'; } |
    awk -v registry_mode="$registry_mode" -f "$root/scripts/lib/dependency_json.awk" -f "$root/scripts/lib/dependency_versions.awk" \
      -f "$root/scripts/lib/dependency_emit.awk" -f "$root/scripts/lib/dependency_enrich.awk")
fi
if [[ -n "$baseline_path" ]]; then
  report=$({ printf '{"audit":%s,"baseline":' "$report"; cat "$baseline_path"; printf '}\n'; } |
    awk -f "$root/scripts/lib/dependency_json.awk" -f "$root/scripts/lib/dependency_emit.awk" \
      -f "$root/scripts/lib/dependency_baseline.awk")
fi
if [[ "$format" == json ]]; then
  printf '%s\n' "$report"
else
  printf '%s\n' "$report" | awk -v output_format="$format" -f "$root/scripts/lib/dependency_json.awk" -f "$root/scripts/lib/dependency_render.awk"
fi
