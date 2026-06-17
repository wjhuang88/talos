#!/bin/sh
# Talos installer.
#
# Usage:
#   curl -fsSL https://<your-domain>/install | sh
#
# Environment overrides:
#   TALOS_REPO         GitHub <owner>/<repo>     (default: wjhuang88/talos)
#   TALOS_VERSION       release tag or "latest"  (default: latest)
#   TALOS_INSTALL_DIR   install directory         (default: ~/.talos/bin)
set -eu

owner_repo="${TALOS_REPO:-wjhuang88/talos}"
version="${TALOS_VERSION:-latest}"

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Darwin) os_part="apple-darwin" ;;
  Linux)  os_part="unknown-linux-gnu" ;;
  *) printf 'error: unsupported OS: %s\n' "$os" >&2; exit 1 ;;
esac
case "$arch" in
  x86_64|amd64)  arch_part="x86_64" ;;
  arm64|aarch64) arch_part="aarch64" ;;
  *) printf 'error: unsupported architecture: %s\n' "$arch" >&2; exit 1 ;;
esac

target="${arch_part}-${os_part}"
archive="talos-${target}.tar.gz"

if [ "$version" = "latest" ]; then
  base="https://github.com/${owner_repo}/releases/latest/download"
else
  base="https://github.com/${owner_repo}/releases/download/${version}"
fi

install_dir="${TALOS_INSTALL_DIR:-${HOME}/.talos/bin}"
tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t talos)"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

hash_tool=""
if command -v sha256sum >/dev/null 2>&1; then
  hash_tool="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
  hash_tool="shasum"
fi

compute_hash() {
  case "$hash_tool" in
    sha256sum) sha256sum "$1" | awk '{print $1}' ;;
    shasum)    shasum -a 256 "$1" | awk '{print $1}' ;;
  esac
}

printf '-> downloading talos %s (%s)\n' "$version" "$target"
curl -fsSL "${base}/${archive}" -o "${tmpdir}/${archive}"

# best-effort checksum verification
if [ -n "$hash_tool" ] \
  && curl -fsSL "${base}/checksum.sha256" -o "${tmpdir}/checksum.sha256" 2>/dev/null; then
  expected="$(grep "${archive}\$" "${tmpdir}/checksum.sha256" | awk '{print $1}' | head -n1)"
  if [ -n "$expected" ]; then
    actual="$(compute_hash "${tmpdir}/${archive}")"
    if [ "$expected" != "$actual" ]; then
      printf 'error: checksum mismatch (expected %s, got %s)\n' "$expected" "$actual" >&2
      exit 1
    fi
    printf '-> checksum verified\n'
  fi
fi

tar -xzf "${tmpdir}/${archive}" -C "${tmpdir}"
mkdir -p "$install_dir"
chmod +x "${tmpdir}/talos"
mv -f "${tmpdir}/talos" "${install_dir}/talos"

printf '-> installed talos to %s/talos\n' "$install_dir"

case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *) printf 'note: add %s to your PATH\n' "$install_dir" ;;
esac

"${install_dir}/talos" --version 2>/dev/null || true
