#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="${1:-$repo_root/target/debug/talos}"
server="$repo_root/scripts/fixtures/i168_provider_terminal_fixture.py"
tmp="$(mktemp -d)"
port_file="$tmp/port"
request_log="$tmp/requests.jsonl"
server_pid=""
warning='Warning: response truncated because the provider reached the output token limit; partial response preserved.'

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

assert_exact() {
  local path="$1"
  local expected="$2"
  python3 - "$path" "$expected" <<'PY'
from pathlib import Path
import sys
actual = Path(sys.argv[1]).read_text(encoding="utf-8")
expected = sys.argv[2]
if actual != expected:
    raise SystemExit(f"{sys.argv[1]} mismatch: expected {expected!r}, got {actual!r}")
PY
}

assert_empty() {
  [[ ! -s "$1" ]] || { echo "$1 expected empty" >&2; cat "$1" >&2; exit 1; }
}

assert_contains() {
  grep -Fqi "$2" "$1" || { echo "$1 missing: $2" >&2; cat "$1" >&2; exit 1; }
}

assert_not_contains() {
  if grep -Fqi "$2" "$1"; then
    echo "$1 unexpectedly contains: $2" >&2
    cat "$1" >&2
    exit 1
  fi
}

write_config() {
  local protocol="$1"
  local model="$2"
  local home="$3"
  local endpoint provider_protocol
  mkdir -p "$home/.talos"
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
}

printf 'protocol\tmode\texit\n'
case_index=0
run_case() {
  local protocol="$1"
  local mode="$2"
  local expected_exit="$3"
  local model="${protocol}-${mode}"
  local case_dir="$tmp/case-$case_index-$model"
  local home="$case_dir/home"
  LAST_OUT="$case_dir/stdout"
  LAST_ERR="$case_dir/stderr"
  case_index=$((case_index + 1))
  mkdir -p "$case_dir"
  write_config "$protocol" "$model" "$home"

  set +e
  HOME="$home" "$binary" --print --provider fixture --model "$model" --no-context \
    "I168 deterministic terminal fixture" >"$LAST_OUT" 2>"$LAST_ERR"
  local status=$?
  set -e
  printf '%s\t%s\t%s\n' "$protocol" "$mode" "$status"
  if [[ "$status" -ne "$expected_exit" ]]; then
    echo "fixture $model exit mismatch: expected $expected_exit, got $status" >&2
    cat "$LAST_OUT" >&2 || true
    cat "$LAST_ERR" >&2 || true
    exit 1
  fi
}

run_combined_max_tokens() {
  local protocol="$1"
  local model="${protocol}-max-tokens"
  local case_dir="$tmp/combined-$protocol"
  local home="$case_dir/home"
  local combined="$case_dir/combined"
  mkdir -p "$case_dir"
  write_config "$protocol" "$model" "$home"
  HOME="$home" "$binary" --print --provider fixture --model "$model" --no-context \
    "I168 combined stream fixture" >"$combined" 2>&1
  assert_exact "$combined" $'fixture partial\n'"$warning"$'\n'
}

run_case openai normal 0
assert_exact "$LAST_OUT" $'fixture normal\n'
assert_empty "$LAST_ERR"

run_case openai max-tokens 0
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_exact "$LAST_ERR" "$warning"$'\n'
[[ "$(grep -Foc "$warning" "$LAST_ERR")" -eq 1 ]]
assert_not_contains "$LAST_ERR" "Error:"
run_combined_max_tokens openai

run_case openai content-filter 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "filtered"
assert_contains "$LAST_ERR" "content_filter"
assert_not_contains "$LAST_ERR" "unsupported provider finish_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case openai legacy-function-call 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "legacy function_call"
assert_contains "$LAST_ERR" "tool_calls"
assert_not_contains "$LAST_ERR" "unsupported provider finish_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case openai unknown 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "unsupported provider finish_reason"
assert_contains "$LAST_ERR" "fixture_unknown_reason"
assert_not_contains "$LAST_ERR" "content_filter"
assert_not_contains "$LAST_ERR" "response truncated"

for mode in eof decode-error transport-error tool-continuation; do
  run_case openai "$mode" 1
  assert_exact "$LAST_OUT" $'fixture partial\n'
  assert_not_contains "$LAST_ERR" "response truncated"
done
assert_contains "$tmp/case-$((case_index-4))-openai-eof/stderr" "explicit terminal signal"
assert_contains "$tmp/case-$((case_index-3))-openai-decode-error/stderr" "decode error"
assert_contains "$tmp/case-$((case_index-2))-openai-transport-error/stderr" "transport read error"
assert_contains "$tmp/case-$((case_index-1))-openai-tool-continuation/stderr" "explicit terminal signal"

run_case anthropic normal 0
assert_exact "$LAST_OUT" $'fixture normal\n'
assert_empty "$LAST_ERR"

run_case anthropic stop-sequence 0
assert_exact "$LAST_OUT" $'fixture stop sequence\n'
assert_empty "$LAST_ERR"

run_case anthropic max-tokens 0
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_exact "$LAST_ERR" "$warning"$'\n'
[[ "$(grep -Foc "$warning" "$LAST_ERR")" -eq 1 ]]
assert_not_contains "$LAST_ERR" "Error:"
run_combined_max_tokens anthropic

run_case anthropic pause-turn 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "paused"
assert_contains "$LAST_ERR" "pause_turn"
assert_not_contains "$LAST_ERR" "unsupported provider stop_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case anthropic refusal 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "refused"
assert_contains "$LAST_ERR" "refusal"
assert_not_contains "$LAST_ERR" "unsupported provider stop_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case anthropic unknown 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "unsupported provider stop_reason"
assert_contains "$LAST_ERR" "fixture_unknown_reason"
assert_not_contains "$LAST_ERR" "pause_turn"
assert_not_contains "$LAST_ERR" "refusal"
assert_not_contains "$LAST_ERR" "response truncated"

for mode in eof decode-error transport-error tool-continuation; do
  run_case anthropic "$mode" 1
  assert_exact "$LAST_OUT" $'fixture partial\n'
  assert_not_contains "$LAST_ERR" "response truncated"
done
assert_contains "$tmp/case-$((case_index-4))-anthropic-eof/stderr" "explicit terminal signal"
assert_contains "$tmp/case-$((case_index-3))-anthropic-decode-error/stderr" "decode error"
assert_contains "$tmp/case-$((case_index-2))-anthropic-transport-error/stderr" "transport read error"
assert_contains "$tmp/case-$((case_index-1))-anthropic-tool-continuation/stderr" "explicit terminal signal"

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
    print(f"{protocol}\ttool-continuation-prefix\tcompleted_tool_result_seen=true")
PY

printf 'binary\t%s\n' "$binary"
printf 'commit\t%s\n' "$(git -C "$repo_root" rev-parse HEAD)"
