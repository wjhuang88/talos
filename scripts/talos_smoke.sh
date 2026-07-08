#!/usr/bin/env bash
#
# Talos Runtime Smoke Harness
#
# Purpose: repeatable smoke validation for Talos development sessions.
# Covers: version, validation, governance preview (dry run), provider behavior,
# and session resume evidence.
#
# Usage:
#   scripts/talos_smoke.sh [path-to-talos-binary]
#
# Default binary: target/debug/talos
#
# This harness is non-mutating: it does not write files, commit, push, or make
# provider calls that cost money. The mock provider is used for LLM paths.
#
# Origin: I106 SBT102 — Self-Bootstrap Control Plane.

set -euo pipefail

BINARY="${1:-target/debug/talos}"
PASS=0
FAIL=0
SKIP=0

ok()   { echo "  ✅ PASS: $1"; PASS=$((PASS + 1)); }
fail() { echo "  ❌ FAIL: $1"; FAIL=$((FAIL + 1)); }
skip() { echo "  ⏭ SKIP: $1"; SKIP=$((SKIP + 1)); }

echo "============================================"
echo "Talos Runtime Smoke Harness"
echo "Binary: $BINARY"
echo "Date:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "============================================"
echo ""

# --- 1. Version ---
echo "1. Version"
if "$BINARY" --version 2>&1 | grep -q "^talos "; then
  VERSION=$("$BINARY" --version 2>&1)
  ok "version output: $VERSION"
else
  fail "version output not found"
fi
echo ""

# --- 2. Validation Plan (read-only) ---
echo "2. Validation Plan (read-only)"
PLAN_OUTPUT=$("$BINARY" validate plan 2>&1) || true
if echo "$PLAN_OUTPUT" | grep -q "cargo fmt" && echo "$PLAN_OUTPUT" | grep -q "cargo test --workspace"; then
  ok "validation plan lists expected workspace checks"
else
  fail "validation plan missing expected checks"
fi
echo ""

# --- 3. Validation Run (governance profile, allowlisted) ---
echo "3. Validation Run (governance profile)"
RUN_OUTPUT=$("$BINARY" validate run --profile governance 2>&1) || true
if echo "$RUN_OUTPUT" | grep -q "governance_validation" && echo "$RUN_OUTPUT" | grep -q "exit_status: 0"; then
  ok "governance validation executed and passed"
else
  fail "governance validation did not pass"
fi
echo ""

# --- 4. Governance Status (read-only) ---
echo "4. Governance Status"
GOV_OUTPUT=$("$BINARY" --governance-status 2>&1) || true
if echo "$GOV_OUTPUT" | grep -q "Manifest" && echo "$GOV_OUTPUT" | grep -q "Validation"; then
  ok "governance status output present"
else
  fail "governance status output missing"
fi
echo ""

# --- 5. Governance Iteration-Record Preview (dry run, no write) ---
echo "5. Governance Iteration-Record Preview (dry run)"
PREVIEW_OUTPUT=$("$BINARY" governance iteration-record preview \
  --iteration I106 \
  --date 2026-07-09 \
  --record-type note \
  --record "smoke harness dry-run preview" 2>&1) || true
if echo "$PREVIEW_OUTPUT" | grep -q "Mutation Preview" && echo "$PREVIEW_OUTPUT" | grep -q "I106"; then
  ok "governance preview shows mutation preview without writing"
else
  fail "governance preview did not produce expected output"
fi
# Verify no write occurred: the preview should not modify the file
if git diff --quiet docs/iterations/I106-self-bootstrap-control-plane.md 2>/dev/null; then
  ok "no file modification from preview"
else
  fail "preview modified the owner doc unexpectedly"
fi
echo ""

# --- 6. Mock Provider (print mode) ---
echo "6. Mock Provider (print mode)"
MOCK_OUTPUT=$("$BINARY" -p --mock --no-init --no-context "Say hello" 2>&1) || true
if [ -n "$MOCK_OUTPUT" ]; then
  ok "mock provider produced output in print mode"
else
  fail "mock provider produced no output"
fi
echo ""

# --- 7. Session List (resume evidence) ---
echo "7. Session List"
LIST_OUTPUT=$("$BINARY" --list --limit 3 2>&1) || true
if echo "$LIST_OUTPUT" | grep -q "session"; then
  ok "session list output present"
else
  fail "session list output missing"
fi
echo ""

# --- 8. Config List (secret masking) ---
echo "8. Config List (secret masking)"
CONFIG_OUTPUT=$("$BINARY" --config-list 2>&1) || true
if echo "$CONFIG_OUTPUT" | grep -q "\*\*\*"; then
  ok "config list masks secrets"
else
  # Config may have no secrets configured; that is also acceptable
  if echo "$CONFIG_OUTPUT" | grep -q "api_key"; then
    fail "config list shows api_key without masking"
  else
    ok "config list has no api_key to mask (acceptable)"
  fi
fi
echo ""

# --- Summary ---
echo "============================================"
echo "Smoke Harness Summary"
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
echo "  Skipped: $SKIP"
echo "============================================"

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
exit 0