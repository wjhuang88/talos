#!/usr/bin/env bash
# Drives the real POSIX installer with a local, PATH-injected curl fixture.
# No network access is performed.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
installer="$root/install/install.sh"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT INT TERM

mkdir -p "$fixture/bin" "$fixture/release" "$fixture/staging"
printf '#!/bin/sh\nprintf "talos fixture 0.0.0\\n"\n' > "$fixture/staging/talos"
chmod +x "$fixture/staging/talos"

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in Darwin) os_part=darwin ;; Linux) os_part=linux ;; *) echo "unsupported test OS: $os" >&2; exit 1 ;; esac
case "$arch" in x86_64|amd64) arch_part=x86_64 ;; arm64|aarch64) arch_part=aarch64 ;; *) echo "unsupported test arch: $arch" >&2; exit 1 ;; esac
archive="talos-${arch_part}-${os_part}.tar.gz"
tar -czf "$fixture/release/$archive" -C "$fixture/staging" talos
if command -v sha256sum >/dev/null 2>&1; then
  hash="$(sha256sum "$fixture/release/$archive" | awk '{print $1}')"
else
  hash="$(shasum -a 256 "$fixture/release/$archive" | awk '{print $1}')"
fi
printf '%s  %s\n' "$hash" "$archive" > "$fixture/release/checksum.sha256"

cat > "$fixture/bin/curl" <<'EOF'
#!/bin/sh
set -eu
out=""
url=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) out="$2"; shift 2 ;;
    -*) shift ;;
    *) url="$1"; shift ;;
  esac
done
[ -n "$url" ] || exit 2
if [ "${FIXTURE_OFFLINE:-0}" = 1 ]; then exit 7; fi
case "$url" in
  *api.github.com*) payload='[{"tag_name":"v0.0.0"}]' ;;
  */checksum.sha256) payload_file="$FIXTURE_RELEASE/checksum.sha256" ;;
  *.tar.gz) payload_file="$FIXTURE_RELEASE/${url##*/}" ;;
  *) exit 22 ;;
esac
if [ -n "$out" ]; then
  if [ -n "${payload_file:-}" ]; then cp "$payload_file" "$out"; else printf '%s\n' "$payload" > "$out"; fi
else
  if [ -n "${payload_file:-}" ]; then cat "$payload_file"; else printf '%s\n' "$payload"; fi
fi
EOF
chmod +x "$fixture/bin/curl"

cat > "$fixture/bin/uname" <<'UNAME'
#!/bin/sh
for arg in "$@"; do
  case "$arg" in
    -s)
      if [ -n "${FAKE_UNAME_S:-}" ]; then printf '%s\n' "$FAKE_UNAME_S"; exit 0; fi
      ;;
    -m)
      if [ -n "${FAKE_UNAME_M:-}" ]; then printf '%s\n' "$FAKE_UNAME_M"; exit 0; fi
      ;;
  esac
done
exec /usr/bin/uname "$@"
UNAME
chmod +x "$fixture/bin/uname"

run_installer() {
  env PATH="$fixture/bin:$PATH" FIXTURE_RELEASE="$fixture/release" \
    HOME="$fixture/home" TALOS_INSTALL_DIR="$fixture/install" "$@" sh "$installer"
}

passed=0
failed=0

fail_test() {
  echo "FAIL: $1" >&2
  failed=$((failed + 1))
}

pass_test() {
  passed=$((passed + 1))
}

echo "1. real installer installs and runs a fixture archive"
output="$(run_installer TALOS_VERSION=v0.0.0 2>&1)"
test -x "$fixture/install/talos" || { fail_test "case 1: binary not installed"; }
"$fixture/install/talos" --version | grep -q 'talos fixture 0.0.0' || { fail_test "case 1: --version mismatch"; }
printf '%s\n' "$output" | grep -q 'checksum verified' || { fail_test "case 1: no checksum verified message"; }
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "2. latest version is resolved through the fixture API"
rm -rf "$fixture/install"
run_installer TALOS_VERSION=latest >/dev/null
test -x "$fixture/install/talos" || { fail_test "case 2: binary not installed"; }
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "3. checksum mismatch makes the real installer fail"
printf '%064d  %s\n' 0 "$archive" > "$fixture/release/checksum.sha256"
if run_installer TALOS_VERSION=v0.0.0 >"$fixture/mismatch.log" 2>&1; then
  fail_test "case 3: installer accepted a bad checksum"
else
  grep -q 'checksum mismatch' "$fixture/mismatch.log" || { fail_test "case 3: missing mismatch message"; }
fi
printf '%s  %s\n' "$hash" "$archive" > "$fixture/release/checksum.sha256"
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "4. offline fixture makes the real installer fail without network"
if run_installer TALOS_VERSION=latest FIXTURE_OFFLINE=1 >"$fixture/offline.log" 2>&1; then
  fail_test "case 4: installer succeeded while fixture transport was offline"
else
  grep -q 'unable to resolve latest release tag' "$fixture/offline.log" || { fail_test "case 4: missing offline message"; }
fi
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "5. unsupported OS (FreeBSD) exits non-zero with explicit message"
rm -rf "$fixture/install"
if FAKE_UNAME_S=FreeBSD run_installer TALOS_VERSION=v0.0.0 >"$fixture/unsupported_os.log" 2>&1; then
  fail_test "case 5: installer succeeded with unsupported OS"
else
  grep -q 'unsupported OS' "$fixture/unsupported_os.log" || { fail_test "case 5: missing unsupported OS message"; }
fi
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "6. unsupported architecture (riscv64) exits non-zero with explicit message"
rm -rf "$fixture/install"
if FAKE_UNAME_M=riscv64 run_installer TALOS_VERSION=v0.0.0 >"$fixture/unsupported_arch.log" 2>&1; then
  fail_test "case 6: installer succeeded with unsupported architecture"
else
  grep -q 'unsupported architecture' "$fixture/unsupported_arch.log" || { fail_test "case 6: missing unsupported arch message"; }
fi
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "7. install-dir override places binary at custom path and runs --version"
rm -rf "$fixture/install" "$fixture/custom_install"
env PATH="$fixture/bin:$PATH" FIXTURE_RELEASE="$fixture/release" \
  HOME="$fixture/home" TALOS_INSTALL_DIR="$fixture/custom_install" \
  sh "$installer" TALOS_VERSION=v0.0.0 >/dev/null
test -x "$fixture/custom_install/talos" || { fail_test "case 7: binary not at custom path"; }
"$fixture/custom_install/talos" --version | grep -q 'talos fixture 0.0.0' || { fail_test "case 7: --version mismatch at custom path"; }
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "8. temp cleanup removes installer temp directory after successful run"
rm -rf "$fixture/install" "$fixture/tmpdir"
mkdir -p "$fixture/tmpdir"
before="$(ls -A "$fixture/tmpdir" 2>/dev/null || true)"
test -z "$before" || { fail_test "case 8: tmpdir not empty before run"; }
env PATH="$fixture/bin:$PATH" FIXTURE_RELEASE="$fixture/release" \
  HOME="$fixture/home" TALOS_INSTALL_DIR="$fixture/install" TMPDIR="$fixture/tmpdir" \
  sh "$installer" TALOS_VERSION=v0.0.0 >/dev/null
leftovers="$(ls -A "$fixture/tmpdir" 2>/dev/null | grep -E '^talos' || true)"
test -z "$leftovers" || { fail_test "case 8: leftover temp dirs: $leftovers"; }
if [ "$failed" -eq 0 ]; then pass_test; fi

echo "9. corrupted-archive extraction fails with no false success"
rm -rf "$fixture/install"
dd if=/dev/urandom of="$fixture/release/$archive" bs=256 count=1 2>/dev/null
if command -v sha256sum >/dev/null 2>&1; then
  corrupt_hash="$(sha256sum "$fixture/release/$archive" | awk '{print $1}')"
else
  corrupt_hash="$(shasum -a 256 "$fixture/release/$archive" | awk '{print $1}')"
fi
printf '%s  %s\n' "$corrupt_hash" "$archive" > "$fixture/release/checksum.sha256"
if run_installer TALOS_VERSION=v0.0.0 >"$fixture/corrupt.log" 2>&1; then
  fail_test "case 9: installer succeeded with corrupted archive"
fi
tar -czf "$fixture/release/$archive" -C "$fixture/staging" talos
printf '%s  %s\n' "$hash" "$archive" > "$fixture/release/checksum.sha256"
if [ "$failed" -eq 0 ]; then pass_test; fi

total=$((passed + failed))
if [ "$failed" -gt 0 ]; then
  echo "installer fixture tests: ${passed}/${total} passed (${failed} failed)" >&2
  exit 1
fi
echo "installer fixture tests: ${total}/${total} passed"
exit 0
