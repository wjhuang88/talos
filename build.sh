#!/bin/bash
set -euo pipefail

ulimit -n 65536 2>/dev/null || true

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="$SCRIPT_DIR/dist"
mkdir -p "$DIST_DIR"

XWIN_PLATFORMS=(
  "x86_64-pc-windows-msvc"
)

ZIG_PLATFORMS=(
  "aarch64-unknown-linux-gnu"
  "x86_64-unknown-linux-gnu"
)

PLATFORMS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
)

build_native() {
  local target="$1"
  echo ">>> Building ${target} ..."
  cargo build --release --target "$target" -p talos-cli
}

build_zig() {
  local target="$1"
  echo ">>> Building ${target} (zigbuild) ..."
  cargo zigbuild --release --target "$target" -p talos-cli
}

build_xwin() {
  local target="$1"
  local target_env
  target_env="$(echo "$target" | tr '-' '_')"
  echo ">>> Building ${target} (cargo-xwin) ..."
  # ring's build script reads bare CC/AR (not CC_<target>), so set both.
  # clang-cl is the MSVC-compatible driver that understands /imsvc flags
  # set by cargo-xwin; plain clang treats them as file paths and fails.
  env \
    CC=clang-cl \
    AR=llvm-lib \
    "CC_${target_env}=clang-cl" \
    "AR_${target_env}=llvm-lib" \
    cargo xwin build --release --target "$target" -p talos-cli
}

package_binary() {
  local target="$1"
  local release_dir="$SCRIPT_DIR/target/${target}/release"
  local archive

  case "$target" in
    *-windows-*)
      archive="$DIST_DIR/talos-${target}.zip"
      ( cd "$release_dir" && zip -q "$archive" talos.exe )
      ;;
    *)
      archive="$DIST_DIR/talos-${target}.tar.gz"
      tar -czf "$archive" -C "$release_dir" talos
      ;;
  esac

  echo "    → ${archive}"
}

for p in "${XWIN_PLATFORMS[@]}"; do
  build_xwin "$p"
  package_binary "$p"
done

for p in "${ZIG_PLATFORMS[@]}"; do
  build_zig "$p"
  package_binary "$p"
done

for p in "${PLATFORMS[@]}"; do
  build_native "$p"
  package_binary "$p"
done

echo ""
echo "========================================="
echo " Build complete. dist/:"
ls -lh "$DIST_DIR"

CHECKSUM_FILE="$DIST_DIR/checksum.sha256"
cd "$DIST_DIR"
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum talos-* > "$CHECKSUM_FILE"
else
  shasum -a 256 talos-* > "$CHECKSUM_FILE"
fi
echo ""
echo "Checksums → ${CHECKSUM_FILE}"
cat "$CHECKSUM_FILE"
echo "========================================="
