#!/usr/bin/env bash
set -uo pipefail

TARGET_SHA="${TARGET_SHA:-b5fcaaf3eb98f6f2e6734c1ad7be92701ce026fe}"
CONTROL_ROOT="${CONTROL_ROOT:?CONTROL_ROOT is required}"
EVIDENCE_DIR="${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
mkdir -p "$EVIDENCE_DIR/logs" "$EVIDENCE_DIR/fixture"
summary="$EVIDENCE_DIR/summary.tsv"
printf 'name\texit\tpassed\tfailed\tignored\twarnings\n' > "$summary"
overall=0

metrics() {
  python3 - "$1" <<'PY'
import re, sys
text = open(sys.argv[1], encoding='utf-8', errors='replace').read()
rows = re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', text)
passed = sum(int(row[0]) for row in rows)
failed = sum(int(row[1]) for row in rows)
ignored = sum(int(row[2]) for row in rows)
warnings = len(re.findall(r'(?m)^warning(?:\[|:)', text))
print(f'{passed}\t{failed}\t{ignored}\t{warnings}')
PY
}

run_cmd() {
  local name="$1"
  local command="$2"
  local log="$EVIDENCE_DIR/logs/${name}.log"
  printf '\n===== %s =====\n%s\n' "$name" "$command" | tee "$log"
  set +e
  bash -lc "$command" >> "$log" 2>&1
  local status=$?
  set -e
  local values
  values="$(metrics "$log")"
  printf '%s\t%s\t%s\n' "$name" "$status" "$values" | tee -a "$summary"
  cat "$log"
  if [[ "$status" -ne 0 ]]; then
    overall=1
  fi
}

set -e
printf 'target_sha=%s\nactual_sha=%s\n' "$TARGET_SHA" "$(git rev-parse HEAD)" | tee "$EVIDENCE_DIR/baseline.txt"
git status --short | tee -a "$EVIDENCE_DIR/baseline.txt"
git log -8 --oneline | tee -a "$EVIDENCE_DIR/baseline.txt"
if [[ "$(git rev-parse HEAD)" != "$TARGET_SHA" ]]; then
  overall=1
fi

run_cmd provider_terminal 'cargo test --locked -p talos-provider terminal'
run_cmd provider_eof 'cargo test --locked -p talos-provider eof'
run_cmd agent_terminal 'cargo test --locked -p talos-agent terminal'
run_cmd session_diagnostic 'cargo test --locked -p talos-session diagnostic'
run_cmd conversation_terminal 'cargo test --locked -p talos-conversation terminal'
run_cmd cli_terminal 'cargo test --locked -p talos-cli terminal'
run_cmd tui_terminal 'cargo test --locked -p talos-tui terminal'
run_cmd fmt 'cargo fmt --all -- --check'
run_cmd workspace_check 'cargo check --workspace --locked'
run_cmd workspace_clippy 'cargo clippy --workspace --locked -- -D warnings'
run_cmd workspace_test 'cargo test --workspace --locked'
run_cmd governance 'scripts/validate_project_governance.sh .'
run_cmd diff_check 'git diff --check'
run_cmd cli_build 'cargo build --locked -p talos-cli'
run_cmd source_scan "rg -n 'unknown|finish_reason|stop_reason|StopReason::EndTurn' crates/talos-provider/src/openai_sse.rs crates/talos-provider/src/anthropic_stream.rs"

fixture_root="$EVIDENCE_DIR/fixture"
python3 "$CONTROL_ROOT/scripts/verification/i168_fixture_server.py" \
  --port-file "$fixture_root/port" \
  --log-file "$fixture_root/requests.jsonl" &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 100); do
  [[ -s "$fixture_root/port" ]] && break
  sleep 0.05
done
port="$(cat "$fixture_root/port")"
fixture_failures=0
printf 'protocol\tmode\texit\tstdout_match\tstderr_match\n' > "$fixture_root/results.tsv"

run_case() {
  local protocol="$1"
  local mode="$2"
  local expected_exit="$3"
  local stdout_needle="$4"
  local stderr_needle="$5"
  local case_dir="$fixture_root/${protocol}-${mode}"
  local home="$case_dir/home"
  mkdir -p "$home/.talos"
  local endpoint proto
  if [[ "$protocol" == openai ]]; then
    endpoint="http://127.0.0.1:$port/v1"
    proto="openai-chat"
  else
    endpoint="http://127.0.0.1:$port/v1/messages"
    proto="anthropic-messages"
  fi
  cat > "$home/.talos/config.toml" <<EOF
provider = "fixture"
model = "${protocol}-${mode}"

[providers.fixture]
protocol = "$proto"
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
  HOME="$home" "$PWD/target/debug/talos" --print --provider fixture \
    --model "${protocol}-${mode}" --no-context 'I168 fixture' \
    > "$case_dir/stdout" 2> "$case_dir/stderr"
  local status=$?
  set -e
  local out_ok=1 err_ok=1
  [[ -z "$stdout_needle" ]] || grep -Fqi "$stdout_needle" "$case_dir/stdout" || out_ok=0
  [[ -z "$stderr_needle" ]] || grep -Fqi "$stderr_needle" "$case_dir/stderr" || err_ok=0
  [[ "$status" -eq "$expected_exit" ]] || fixture_failures=$((fixture_failures + 1))
  [[ "$out_ok" -eq 1 ]] || fixture_failures=$((fixture_failures + 1))
  [[ "$err_ok" -eq 1 ]] || fixture_failures=$((fixture_failures + 1))
  printf '%s\t%s\t%s\t%s\t%s\n' "$protocol" "$mode" "$status" "$out_ok" "$err_ok" | tee -a "$fixture_root/results.tsv"
  printf -- '--- %s/%s stdout ---\n' "$protocol" "$mode"
  cat "$case_dir/stdout"
  printf -- '--- %s/%s stderr ---\n' "$protocol" "$mode"
  cat "$case_dir/stderr"
}

for protocol in openai anthropic; do
  run_case "$protocol" normal 0 'fixture normal' ''
  run_case "$protocol" max-tokens 0 'fixture partial' 'truncated'
  if [[ "$protocol" == openai ]]; then
    run_case "$protocol" unknown 1 'fixture partial' 'content_filter'
  else
    run_case "$protocol" unknown 1 'fixture partial' 'pause_turn'
  fi
  run_case "$protocol" eof 1 'fixture partial' 'explicit terminal'
  run_case "$protocol" decode-error 1 '' 'decode error'
  run_case "$protocol" transport-error 1 'fixture partial' 'transport read error'
  run_case "$protocol" tool-continuation 1 'fixture partial' 'explicit terminal'
done

set +e
python3 - "$fixture_root/requests.jsonl" <<'PY'
import json, sys
records = [json.loads(line) for line in open(sys.argv[1], encoding='utf-8') if line.strip()]
failed = False
for protocol in ('openai', 'anthropic'):
    model = f'{protocol}-tool-continuation'
    continuation = [r for r in records if r['model'] == model and r['ordinal'] >= 2]
    ok = bool(continuation) and any(r['has_tool_result'] for r in continuation)
    print(f'{model}\tcompleted_tool_result_seen={str(ok).lower()}')
    failed |= not ok
raise SystemExit(1 if failed else 0)
PY
request_status=$?
set -e
[[ "$request_status" -eq 0 ]] || fixture_failures=$((fixture_failures + 1))
printf 'binary=%s\ncommit=%s\nfailures=%s\n' \
  "$PWD/target/debug/talos" "$(git rev-parse HEAD)" "$fixture_failures" \
  | tee "$fixture_root/metadata.txt"
if [[ "$fixture_failures" -ne 0 ]]; then
  overall=1
  fixture_status=1
else
  fixture_status=0
fi
printf 'rebuilt_binary_fixture\t%s\t0\t0\t0\t0\n' "$fixture_status" | tee -a "$summary"

cat > "$EVIDENCE_DIR/owner-gate.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
i168=docs/iterations/I168-provider-terminal-outcome-integrity.md
story=docs/backlog/active/RUNTIME-003-provider-terminal-outcome-integrity.md
! grep -Fq 'Planning/governance validation: pending.' "$i168"
! grep -Fq 'Implementation and runtime evidence: pending.' "$i168"
! grep -Fq 'Completion Commit: pending.' "$i168"
grep -Eq '^> Document status: .*Complete' "$i168"
grep -Eq '^\| Status \| Complete' "$story"
! grep -Eq '^- \[ \]' "$story"
SH
chmod +x "$EVIDENCE_DIR/owner-gate.sh"
run_cmd owner_evidence_gate "$EVIDENCE_DIR/owner-gate.sh"

printf '\nOVERALL_EXIT=%s\n' "$overall" | tee -a "$summary"
cat "$summary"
exit "$overall"
