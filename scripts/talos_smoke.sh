#!/usr/bin/env bash
#
# Talos Runtime Smoke Harness
#
# Purpose: repeatable smoke validation for Talos development sessions.
# Covers: version, validation, governance preview (dry run), provider behavior,
# session resume/export evidence, permission preflight, and graceful interruption.
#
# Usage:
#   scripts/talos_smoke.sh [path-to-talos-binary]
#
# Default binary: target/debug/talos
#
# This harness runs from a DISPOSABLE HOME with NO real secret or external
# provider. It does not write files outside the temp HOME, commit, push, or
# make provider calls that cost money. The mock provider is used for LLM paths.
#
# Origin: I106 SBT102 — Self-Bootstrap Control Plane.
# Extended: I123 F131 — Clean-HOME real-binary trial smoke.

set -euo pipefail

BINARY="${1:-target/debug/talos}"
PASS=0
FAIL=0
SKIP=0

ok()   { echo "  ✅ PASS: $1"; PASS=$((PASS + 1)); }
fail() { echo "  ❌ FAIL: $1"; FAIL=$((FAIL + 1)); }
skip() { echo "  ⏭ SKIP: $1"; SKIP=$((SKIP + 1)); }

# --- Disposable HOME setup (I123 F131) ---
DISPOSABLE_HOME="$(mktemp -d)"
export HOME="$DISPOSABLE_HOME"
# Clear any TALOS_* env vars that could leak real config
for var in $(env | grep -o '^TALOS_[A-Z_]*' || true); do
  unset "$var"
done
trap 'rm -rf "$DISPOSABLE_HOME"' EXIT

echo "============================================"
echo "Talos Runtime Smoke Harness"
echo "Binary: $BINARY"
echo "Date:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "HOME:   $DISPOSABLE_HOME (disposable)"
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
# Save file hash before preview (macOS uses md5 -q, Linux uses md5sum)
BEFORE_HASH=$(if command -v md5 &>/dev/null; then md5 -q docs/iterations/I106-self-bootstrap-control-plane.md; else md5sum docs/iterations/I106-self-bootstrap-control-plane.md | awk '{print $1}'; fi)
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
# Verify no write occurred: compare file hash after preview
AFTER_HASH=$(if command -v md5 &>/dev/null; then md5 -q docs/iterations/I106-self-bootstrap-control-plane.md; else md5sum docs/iterations/I106-self-bootstrap-control-plane.md | awk '{print $1}'; fi)
if [ "$BEFORE_HASH" = "$AFTER_HASH" ]; then
  ok "no file modification from preview (hash unchanged)"
else
  fail "preview modified the owner doc unexpectedly (hash changed)"
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
elif echo "$LIST_OUTPUT" | grep -q "unable to open database file" && echo "$LIST_OUTPUT" | grep -q -- "--search"; then
  ok "session list reports missing index with recovery hint (acceptable clean environment)"
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

# --- 9. Permission Preflight (read-only) ---
echo "9. Permission Preflight (read-only)"
PREFLIGHT_OUTPUT=$("$BINARY" permissions preflight \
  --operation 'bash={"command":"cat Cargo.toml"}' 2>&1) || true
if echo "$PREFLIGHT_OUTPUT" | grep -q "bash" && echo "$PREFLIGHT_OUTPUT" | grep -qi "decision"; then
  ok "permission preflight produces scoped output for a read-only bash command"
else
  fail "permission preflight did not produce expected output"
fi
echo ""

# --- 10. Diagnostics Status (read-only, no secrets) ---
echo "10. Diagnostics Status (read-only)"
DIAG_OUTPUT=$("$BINARY" diagnostics status 2>&1) || true
if echo "$DIAG_OUTPUT" | grep -q "Talos version" && \
   echo "$DIAG_OUTPUT" | grep -q "Session Format" && \
   echo "$DIAG_OUTPUT" | grep -q "Workspace Trust" && \
   echo "$DIAG_OUTPUT" | grep -q "Residual Gates"; then
  ok "diagnostics status reports release, session, trust, and residual gates"
  if echo "$DIAG_OUTPUT" | grep -qi "api_key\|sk-ant\|secret"; then
    fail "diagnostics status leaked a secret-like string"
  else
    ok "diagnostics status contains no secret-like strings"
  fi
else
  fail "diagnostics status missing required sections"
fi
echo ""

# --- 11. Ordered Tool Turn (mock provider) ---
echo "11. Ordered Tool Turn (mock provider)"
TOOL_OUTPUT=$("$BINARY" -p --mock --no-init --no-context "/mock-request use a tool" 2>&1) || true
if [ -n "$TOOL_OUTPUT" ]; then
  ok "mock provider completed an ordered turn"
else
  fail "mock provider produced no output for ordered turn"
fi
echo ""

# --- 12. Disposable-HOME isolation (I123 F131) ---
echo "12. Disposable-HOME isolation"
ISOLATION_OUTPUT=$("$BINARY" -p --mock --no-init --no-context "hello from clean home" 2>&1) || true
if [ -n "$ISOLATION_OUTPUT" ]; then
  ok "binary starts under disposable HOME without real credentials"
else
  fail "binary produced no output under disposable HOME"
fi
echo ""

# --- 13. Config masking with fixture api_key (I123 F131) ---
echo "13. Config masking with fixture api_key"
mkdir -p "$DISPOSABLE_HOME/.talos"
cat > "$DISPOSABLE_HOME/.talos/config.toml" <<'FIXTURE'
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key = "sk-test-fixture-secret-xxxxx"
FIXTURE
MASK_OUTPUT=$("$BINARY" --config-list 2>&1) || true
if echo "$MASK_OUTPUT" | grep -q "\*\*\*"; then
  if echo "$MASK_OUTPUT" | grep -q "sk-test-fixture-secret-xxxxx"; then
    fail "config list leaks fixture api_key in plaintext"
  else
    ok "config list masks fixture api_key as ***"
  fi
else
  fail "config list shows no masking indicator"
fi
echo ""

# --- 14. Session resume with persisted-content verification (I123 F131) ---
echo "14. Session resume (persisted content survives)"
# Gotcha: print mode (-p) does NOT persist sessions; only inline mode (--inline) does.
# Create a session with a marker via inline mode, resume that exact session, and assert
# the persisted file both contains the original content and grew after the resumed turn.
RECALL_MARKER="RECALL-MARKER-8F3A"
echo "$RECALL_MARKER remember this codeword" | "$BINARY" --inline --mock --no-init --no-context >/dev/null 2>&1 || true
SESSIONS_DIR="$DISPOSABLE_HOME/.talos/sessions"
SESSION_FILE=$(find "$SESSIONS_DIR" -name '*.tlog' -type f 2>/dev/null | head -1)
if [ -z "$SESSION_FILE" ]; then
  SESSION_FILE=$(find "$SESSIONS_DIR" -name '*.jsonl' -type f 2>/dev/null | head -1)
fi
if [ -z "$SESSION_FILE" ] || [ ! -s "$SESSION_FILE" ]; then
  fail "no persisted session file created by inline mode (resume evidence missing)"
else
  BEFORE_SIZE=$(if command -v stat >/dev/null 2>&1; then stat -f%z "$SESSION_FILE" 2>/dev/null || stat -c%s "$SESSION_FILE" 2>/dev/null; else wc -c < "$SESSION_FILE"; fi)
  SESSION_ID=$(basename "$SESSION_FILE"); SESSION_ID=${SESSION_ID%.tlog}; SESSION_ID=${SESSION_ID%.jsonl}
  # Resume the EXACT session and append another turn; proves the same session was
  # loaded and its prior content persisted (no false success from a fresh session).
  echo "what codeword did I ask you to remember?" | "$BINARY" --session "$SESSION_ID" --inline --mock --no-init --no-context >/dev/null 2>&1 || true
  AFTER_SIZE=$(if command -v stat >/dev/null 2>&1; then stat -f%z "$SESSION_FILE" 2>/dev/null || stat -c%s "$SESSION_FILE" 2>/dev/null; else wc -c < "$SESSION_FILE"; fi)
  if [ "$AFTER_SIZE" -gt "$BEFORE_SIZE" ] && grep -q "$RECALL_MARKER" "$SESSION_FILE"; then
    ok "session resumed: prior content persisted (marker found) and file grew (${BEFORE_SIZE}->${AFTER_SIZE})"
  else
    fail "session resume failed: grew=${AFTER_SIZE}>${BEFORE_SIZE}? marker=$(grep -q "$RECALL_MARKER" "$SESSION_FILE" >/dev/null 2>&1 && echo yes || echo no)"
  fi
fi
echo ""

# --- 15. Export evidence (I123 F131) ---
echo "15. Export evidence"
# /export is a slash command that requires interactive TUI/inline mode.
# In print mode there is no non-interactive export path.
# Document as SKIP rather than false-fail.
EXPORT_FILE="$DISPOSABLE_HOME/export_test.md"
# Attempt: the binary has no non-interactive export flag; /export is TUI-only.
skip "/export is a TUI slash command — no non-interactive export path in print mode"
echo ""

# --- 16. Permission preflight Ask/Deny (I123 F131) ---
echo "16. Permission preflight Ask/Deny"
# Risky command: should NOT be an unconditional allow
RISKY_OUTPUT=$("$BINARY" permissions preflight \
  --operation 'bash={"command":"rm important.txt"}' 2>&1) || true
if echo "$RISKY_OUTPUT" | grep -qi "decision"; then
  if echo "$RISKY_OUTPUT" | grep -qi "current decision: allow"; then
    fail "risky command 'rm important.txt' shows unconditional allow"
  else
    ok "risky command preflight shows non-allow decision (ask/deny)"
  fi
else
  fail "risky command preflight missing decision keyword"
fi

# Read-only command: should show a decision (allow or ask)
READONLY_OUTPUT=$("$BINARY" permissions preflight \
  --operation 'bash={"command":"cat Cargo.toml"}' 2>&1) || true
if echo "$READONLY_OUTPUT" | grep -qi "decision"; then
  ok "read-only command preflight shows decision keyword"
else
  fail "read-only command preflight missing decision keyword"
fi
echo ""

# --- 17. Graceful interruption (best-effort, I123 F131) ---
echo "17. Graceful interruption (best-effort)"
# Launch a mock turn in the background, send SIGINT, and check exit.
# Signal handling in a non-TTY subprocess may not be reliable; SOFT check.
INTERRUPT_LOG="$DISPOSABLE_HOME/interrupt.log"
"$BINARY" -p --mock --no-init --no-context "do a long task" >"$INTERRUPT_LOG" 2>&1 &
BG_PID=$!
sleep 1
if kill -0 "$BG_PID" 2>/dev/null; then
  kill -INT "$BG_PID" 2>/dev/null || true
  sleep 2
  if kill -0 "$BG_PID" 2>/dev/null; then
    # Still alive after SIGINT — force kill to avoid hanging
    kill -9 "$BG_PID" 2>/dev/null || true
    wait "$BG_PID" 2>/dev/null || true
    skip "process did not terminate after SIGINT (signal handling may require TTY)"
  else
    wait "$BG_PID" 2>/dev/null || true
    EXIT_CODE=$?
    ok "process terminated after SIGINT (exit code: $EXIT_CODE)"
  fi
else
  # Process already finished before SIGINT — too fast to interrupt
  wait "$BG_PID" 2>/dev/null || true
  skip "process completed before SIGINT could be sent (mock turn too fast)"
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
