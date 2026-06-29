#!/bin/sh
# Check that product-only crates cannot be accidentally published.
# Product-only crates must carry publish = false in their Cargo.toml.
# Gate-before-publish crates must NOT carry publish = false (they are manifest-ready,
# gated by review, not hard-blocked).
#
# Usage: scripts/check_publish_guard.sh [workspace-root]
# Exit 0 = all guards pass; exit 1 = a guard violation was found.

set -u

root="${1:-.}"

errors=0

# Product-only crates that MUST have publish = false.
product_only_crates="talos-cli talos-tui talos-evolution"

# Gate-before-publish crates that must NOT have publish = false
# (they are manifest-ready; the gate is the review process, not a manifest flag).
gate_crates="talos-sandbox talos-tools talos-agent talos-runtime talos-mcp"

check_publish_false() {
  crate="$1"
  toml="${root}/crates/${crate}/Cargo.toml"
  if [ ! -f "$toml" ]; then
    printf 'ERROR: %s/Cargo.toml not found\n' "$crate" >&2
    errors=$((errors + 1))
    return
  fi
  # Look for publish = false (with optional whitespace)
  if grep -qE '^[[:space:]]*publish[[:space:]]*=[[:space:]]*false' "$toml"; then
    printf '  OK  %s: publish = false (product-only guard active)\n' "$crate"
  else
    printf 'ERROR: %s: missing publish = false — product-only crate can be published accidentally\n' "$crate" >&2
    errors=$((errors + 1))
  fi
}

check_no_publish_false() {
  crate="$1"
  toml="${root}/crates/${crate}/Cargo.toml"
  if [ ! -f "$toml" ]; then
    printf 'WARN  %s/Cargo.toml not found (skipped)\n' "$crate" >&2
    return
  fi
  if grep -qE '^[[:space:]]*publish[[:space:]]*=[[:space:]]*false' "$toml"; then
    printf 'ERROR: %s: has publish = false — gate crate should be manifest-ready, not hard-blocked\n' "$crate" >&2
    errors=$((errors + 1))
  else
    printf '  OK  %s: no publish = false (manifest-ready, gated by review)\n' "$crate"
  fi
}

printf '=== Publication Guard Check ===\n\n'
printf 'Product-only crates (must have publish = false):\n'
for crate in $product_only_crates; do
  check_publish_false "$crate"
done

printf '\nGate-before-publish crates (must NOT have publish = false):\n'
for crate in $gate_crates; do
  check_no_publish_false "$crate"
done

printf '\n'
if [ "$errors" -gt 0 ]; then
  printf 'FAILED: %d guard violation(s) found.\n' "$errors" >&2
  exit 1
fi
printf 'PASSED: all publication guards verified.\n'
exit 0
