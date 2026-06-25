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
  Darwin) os_part="darwin" ;;
  Linux)  os_part="linux" ;;
  *) printf 'error: unsupported OS: %s\n' "$os" >&2; exit 1 ;;
esac
case "$arch" in
  x86_64|amd64)  arch_part="x86_64" ;;
  arm64|aarch64) arch_part="aarch64" ;;
  *) printf 'error: unsupported architecture: %s\n' "$arch" >&2; exit 1 ;;
esac

archive="talos-${arch_part}-${os_part}.tar.gz"

# GitHub's /releases/latest excludes prereleases, so for a prerelease-only
# project the "latest/download" shortcut 404s. Resolve the newest release tag
# (prereleases included) via the API instead.
resolve_latest_tag() {
  curl -fsSL "https://api.github.com/repos/${1}/releases?per_page=1" \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n1
}

if [ "$version" = "latest" ]; then
  version="$(resolve_latest_tag "$owner_repo")"
  if [ -z "$version" ]; then
    printf 'error: unable to resolve latest release tag for %s\n' "$owner_repo" >&2
    exit 1
  fi
fi
base="https://github.com/${owner_repo}/releases/download/${version}"

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

printf '%s\n' "-> downloading talos ${version} (${archive})"
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
    printf '%s\n' '-> checksum verified'
  fi
fi

tar -xzf "${tmpdir}/${archive}" -C "${tmpdir}"
mkdir -p "$install_dir"
chmod +x "${tmpdir}/talos"
mv -f "${tmpdir}/talos" "${install_dir}/talos"

printf '%s\n' "-> installed talos to ${install_dir}/talos"

case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *) printf 'note: add %s to your PATH\n' "$install_dir" ;;
esac

"${install_dir}/talos" --version 2>/dev/null || true
