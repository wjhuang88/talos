#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="${1:-$repo_root/target/debug/talos}"
server="$repo_root/scripts/fixtures/i168_provider_terminal_fixture.py"
tmp="$(mktemp -d)"
port_file="$tmp/port"
request_log="$tmp/requests.jsonl"
server_pid=""

cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf "$tmp"
}
trap cleanup EXIT

python3 "$server" --port-file "$port_file" --log-file "$request_log" &
server_pid=$!
for _ in $(seq 1 100); do
  [[ -s "$port_file" ]] && break
  sleep 0.05
done
[[ -s "$port_file" ]] || { echo "fixture server failed to publish port" >&2; exit 1; }
port="$(cat "$port_file")"

printf 'protocol\tmode\texit\tstdout_match\tstderr_match\n'
case_index=0
run_case() {
  local protocol="$1"
  local mode="$2"
  local expected_exit="$3"
  local stdout_needle="$4"
  local stderr_needle="$5"
  local model="${protocol}-${mode}"
  local case_dir="$tmp/case-$case_index-$model"
  local home="$case_dir/home"
  local out="$case_dir/stdout"
  local err="$case_dir/stderr"
  case_index=$((case_index + 1))
  mkdir -p "$home/.talos"

  local endpoint provider_protocol
  if [[ "$protocol" == "openai" ]]; then
    endpoint="http://127.0.0.1:$port/v1"
    provider_protocol="openai-chat"
  else
    endpoint="http://127.0.0.1:$port/v1/messages"
    provider_protocol="anthropic-messages"
  fi

  cat > "$home/.talos/config.toml" <<EOF
provider = "fixture"
model = "$model"

[providers.fixture]
protocol = "$provider_protocol"
base_url = "$endpoint"
api_key = "fixture-only"

[providers.fixture.timeout]
dispatch_timeout_secs = 5
first_packet_timeout_secs = 5
stream_idle_timeout_secs = 5
max_attempts = 1
backoff_base_ms = 1
backoff_max_ms = 1
EOF

  set +e
  HOME="$home" "$binary" --print --provider fixture --model "$model" --no-context \
    "I168 deterministic terminal fixture" >"$out" 2>"$err"
  local status=$?
  set -e

  local out_ok=1
  local err_ok=1
  [[ -z "$stdout_needle" ]] || grep -Fqi "$stdout_needle" "$out" || out_ok=0
  [[ -z "$stderr_needle" ]] || grep -Fqi "$stderr_needle" "$err" || err_ok=0
  printf '%s\t%s\t%s\t%s\t%s\n' "$protocol" "$mode" "$status" "$out_ok" "$err_ok"

  if [[ "$status" -ne "$expected_exit" || "$out_ok" -ne 1 || "$err_ok" -ne 1 ]]; then
    echo "fixture $model failed" >&2
    echo "--- stdout ---" >&2
    cat "$out" >&2 || true
    echo "--- stderr ---" >&2
    cat "$err" >&2 || true
    exit 1
  fi
}

for protocol in openai anthropic; do
  run_case "$protocol" normal 0 "fixture normal" ""
  run_case "$protocol" max-tokens 0 "fixture partial" "response truncated"
  if [[ "$protocol" == "openai" ]]; then
    run_case "$protocol" unknown 1 "fixture partial" "content_filter"
  else
    run_case "$protocol" unknown 1 "fixture partial" "pause_turn"
  fi
  run_case "$protocol" eof 1 "fixture partial" "explicit terminal signal"
  run_case "$protocol" decode-error 1 "fixture partial" "decode error"
  run_case "$protocol" transport-error 1 "fixture partial" "transport read error"
  run_case "$protocol" tool-continuation 1 "fixture partial" "explicit terminal signal"
done

python3 - "$request_log" <<'PY'
import json
import sys

records = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
for protocol in ("openai", "anthropic"):
    model = f"{protocol}-tool-continuation"
    continuation = [
        record
        for record in records
        if record["model"] == model and record["response_ordinal"] >= 2
    ]
    if not continuation or not any(record["has_tool_result"] for record in continuation):
        raise SystemExit(f"{model}: continuation request did not preserve completed tool result")
    print(
        f"{protocol}\ttool-continuation-prefix\tcompleted_tool_result_seen=true"
    )
PY

printf 'binary\t%s\n' "$binary"
printf 'commit\t%s\n' "$(git -C "$repo_root" rev-parse HEAD)"
