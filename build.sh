#!/bin/bash
set -euo pipefail

ulimit -n 65536 2>/dev/null || true

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="$SCRIPT_DIR/dist"
mkdir -p "$DIST_DIR"

PLATFORMS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
)

ZIG_PLATFORMS=(
  "aarch64-unknown-linux-gnu"
  "x86_64-unknown-linux-gnu"
)

XWIN_PLATFORMS=(
  "x86_64-pc-windows-msvc"
  "aarch64-pc-windows-msvc"
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
  echo ">>> Building ${target} (cargo-xwin) ..."
  cargo xwin build --release --target "$target" -p talos-cli
}

copy_binary() {
  local target="$1"
  local src="$SCRIPT_DIR/target/${target}/release/talos"
  local dst_name

  case "$target" in
    *-windows-*) dst_name="talos-${target}.exe"; src="${src}.exe" ;;
    *)           dst_name="talos-${target}" ;;
  esac

  cp "$src" "${DIST_DIR}/${dst_name}"
  echo "    → ${DIST_DIR}/${dst_name}"
}

for p in "${PLATFORMS[@]}"; do
  build_native "$p"
  copy_binary "$p"
done

for p in "${ZIG_PLATFORMS[@]}"; do
  build_zig "$p"
  copy_binary "$p"
done

for p in "${XWIN_PLATFORMS[@]}"; do
  build_xwin "$p"
  copy_binary "$p"
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
