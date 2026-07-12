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

run_installer() {
  env PATH="$fixture/bin:$PATH" FIXTURE_RELEASE="$fixture/release" \
    HOME="$fixture/home" TALOS_INSTALL_DIR="$fixture/install" "$@" sh "$installer"
}

echo "1. real installer installs and runs a fixture archive"
output="$(run_installer TALOS_VERSION=v0.0.0 2>&1)"
test -x "$fixture/install/talos"
"$fixture/install/talos" --version | grep -q 'talos fixture 0.0.0'
printf '%s\n' "$output" | grep -q 'checksum verified'

echo "2. latest version is resolved through the fixture API"
rm -rf "$fixture/install"
run_installer TALOS_VERSION=latest >/dev/null
test -x "$fixture/install/talos"

echo "3. checksum mismatch makes the real installer fail"
printf '%064d  %s\n' 0 "$archive" > "$fixture/release/checksum.sha256"
if run_installer TALOS_VERSION=v0.0.0 >"$fixture/mismatch.log" 2>&1; then
  echo "installer accepted a bad checksum" >&2
  exit 1
fi
grep -q 'checksum mismatch' "$fixture/mismatch.log"
printf '%s  %s\n' "$hash" "$archive" > "$fixture/release/checksum.sha256"

echo "4. offline fixture makes the real installer fail without network"
if run_installer TALOS_VERSION=latest FIXTURE_OFFLINE=1 >"$fixture/offline.log" 2>&1; then
  echo "installer succeeded while fixture transport was offline" >&2
  exit 1
fi
grep -q 'unable to resolve latest release tag' "$fixture/offline.log"

echo "installer fixture tests: 4/4 passed"
