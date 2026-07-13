#!/usr/bin/env bash
#
# Talos Second-Operator Replay Packet (I123 F132)
#
# Runs the installer fixture matrix (F130) and the clean-HOME trial smoke (F131)
# in sequence, records platform/environment, and writes a machine-comparable
# JSON result record so a second operator can diff two runs for variance.
#
# Usage:
#   scripts/replay_trial.sh [path-to-talos-binary]
#
# Default binary: target/debug/talos
#
# Exit code: non-zero only if a required step FAILED (exit != 0). An intentional
# SKIP (e.g. PowerShell wrapper exiting 0 when pwsh is absent) does NOT fail.
#
# No network access, no real credentials, mock provider only.

set -euo pipefail

BINARY="${1:-target/debug/talos}"
root="$(cd "$(dirname "$0")/.." && pwd)"
script_dir="$root/scripts"

# Result directory: prefer target/trial-replay; fall back to a temp dir.
if [ -d "$root/target" ]; then
  out_dir="$root/target/trial-replay"
  mkdir -p "$out_dir"
else
  out_dir="$(mktemp -d)"
fi
ts="$(date -u +%Y%m%dT%H%M%SZ)"
record="$out_dir/trial-replay-$ts.json"

os="$(uname -s)"
arch="$(uname -m)"
rustc_v="$(rustc --version 2>/dev/null || echo 'unavailable')"
if command -v pwsh >/dev/null 2>&1; then
  pwsh_v="$(pwsh --version 2>/dev/null || echo 'present')"
else
  pwsh_v="absent"
fi

steps_json=""
overall=0
notes=""

sanitize() {
  # strip double quotes / backslashes so the value is safe inside JSON
  printf '%s' "$1" | tr -d '"' | tr -d '\\'
}

add_step() {
  local name="$1" rc="$2" summary="$3"
  local entry
  entry="{\"name\":\"$(sanitize "$name")\",\"exit_code\":$rc,\"summary\":\"$(sanitize "$summary")\"}"
  if [ -z "$steps_json" ]; then
    steps_json="$entry"
  else
    steps_json="$steps_json,$entry"
  fi
  if [ "$rc" -ne 0 ]; then
    overall=1
  fi
}

run_step() {
  local name="$1"; shift
  echo "=== $name ==="
  set +e
  local out
  out="$("$@" 2>&1)"
  local rc=$?
  set -e
  printf '%s\n' "$out" | tail -3
  local summary
  summary="$(printf '%s\n' "$out" | grep -E 'passed|Summary|SKIP|skipped' | tail -1)"
  add_step "$name" "$rc" "$summary"
  echo "--- $name: exit $rc ---"
  echo ""
}

echo "============================================"
echo "Talos Second-Operator Replay Packet (F132)"
echo "Generated: $ts"
echo "Platform:  $os / $arch"
echo "Rust:      $rustc_v"
echo "pwsh:      $pwsh_v"
echo "Binary:    $BINARY"
echo "============================================"
echo ""

if [ ! -x "$BINARY" ]; then
  echo "binary $BINARY not found; attempting local build"
  set +e
  cargo build -p talos-cli --bin talos --locked 2>&1 | tail -5
  rc=$?
  set -e
  if [ $rc -ne 0 ] || [ ! -x "$BINARY" ]; then
    echo "build failed; cannot replay trial" >&2
    notes="binary build failed; replay could not run"
    overall=1
  fi
fi

if [ "$overall" -eq 0 ]; then
  run_step "installer_fixtures_posix" bash "$script_dir/test_installer_fixtures.sh"
  run_step "installer_fixtures_powershell" bash "$script_dir/test_installer_fixtures_ps1.sh"
  run_step "clean_home_trial_smoke" bash "$script_dir/talos_smoke.sh" "$BINARY"
fi

if [ "$pwsh_v" = "absent" ]; then
  notes="powershell SKIP: pwsh absent on this platform (Windows CI must run the PowerShell fixture); Windows ARM64 installer untested here (not published)."
else
  notes="Windows ARM64 installer untested on this platform (not published); PowerShell fixture executed."
fi

cat > "$record" <<EOF
{
  "generated_utc": "$ts",
  "platform": { "os": "$os", "arch": "$arch" },
  "rustc": "$rustc_v",
  "pwsh": "$pwsh_v",
  "binary": "$BINARY",
  "steps": [$steps_json],
  "overall_exit": $overall,
  "notes": "$notes"
}
EOF

echo "============================================"
echo "Replay Packet Summary"
echo "  Record: $record"
echo "  Overall exit: $overall"
if [ -n "$notes" ]; then echo "  Notes: $notes"; fi
echo "============================================"

exit "$overall"
