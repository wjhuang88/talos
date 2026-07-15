#!/usr/bin/env bash
#
# I127 scheduler independent replay packet.
#
# Runs without real credentials or a pre-existing Talos home. The CLI command
# checks clean-HOME composition; the three deterministic fixture tests execute
# the register/fire/list/cancel/shutdown lifecycle through the real
# Agent/session path.
#
# Usage: scripts/replay_i127_scheduler.sh [path-to-talos-binary]

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
binary="${1:-$root/target/debug/talos}"
clean_home="$(mktemp -d "${TMPDIR:-/tmp}/talos-i127-home.XXXXXX")"
operator_home="$HOME"

cleanup() {
  rm -rf "$clean_home"
}
trap cleanup EXIT

cd "$root"
# Keep the pinned toolchain available while isolating only Talos state. Rustup
# and Cargo default beneath HOME, so preserve their original roots explicitly.
export RUSTUP_HOME="${RUSTUP_HOME:-$operator_home/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$operator_home/.cargo}"
for variable in $(env | sed -n 's/^\(TALOS_[A-Z_]*\)=.*/\1/p'); do
  unset "$variable"
done
export HOME="$clean_home"
export TALOS_HOME="$clean_home/.talos"

if [[ ! -x "$binary" ]]; then
  cargo build -p talos-cli --bin talos --locked
fi

echo "I127 scheduler replay"
echo "HOME=$HOME (disposable)"
echo "TALOS_HOME=$TALOS_HOME (disposable)"

# This is a composition smoke only: --mock prints the request shape and does
# not synthesize a tool invocation. The lifecycle evidence is the fixture
# matrix below.
"$binary" -p --mock --no-init --no-context \
  "/mock-request schedule a one-shot follow-up after 1 second"

cargo test -p talos-agent fixture_provider_delay_fires_and_follow_up_gets_fresh_deny \
  --locked -- --test-threads=1
cargo test -p talos-agent fixture_provider_list_cancel_full_lifecycle \
  --locked -- --test-threads=1
cargo test -p talos-agent sf130_shutdown_leaves_no_leaked_tasks \
  --locked -- --test-threads=1

echo "I127 scheduler replay passed"
