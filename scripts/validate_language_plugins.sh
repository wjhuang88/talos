#!/usr/bin/env bash
# Real optional guest delivery: complete checkout, pinned Rust, wasm target, Clang/llvm-ar.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

# Apple Clang lacks the WASM backend. Check the actual target compiler, not the
# host compiler; keep native workspace builds on their existing toolchain.
wasm_cc=${CC_wasm32_unknown_unknown:-clang}
if ! "$wasm_cc" --print-targets | grep 'wasm32' >/dev/null; then
  echo "WASM-capable Clang required: set CC_wasm32_unknown_unknown to LLVM clang (see plugins/README.md)" >&2
  exit 1
fi
export CC_wasm32_unknown_unknown="$wasm_cc"

# Prevent root workspace feature unification from bundling both language grammars.
for language in rust python; do
  cargo build --manifest-path plugins/Cargo.toml --locked --release \
    --target wasm32-unknown-unknown -p "talos-language-${language}"
done
cargo fmt --manifest-path plugins/Cargo.toml --all -- --check
cargo test --manifest-path plugins/Cargo.toml --locked --workspace
cargo clippy --manifest-path plugins/Cargo.toml --locked --workspace --all-targets -- -D warnings

bundle_test_root=$(mktemp -d "${TMPDIR:-/tmp}/talos-language-acceptance.XXXXXX")
trap 'rm -rf -- "$bundle_test_root"' EXIT
for language in rust python; do
  cargo run --manifest-path plugins/Cargo.toml --locked -p talos-plugin-package -- \
    "$language" "plugins/target/wasm32-unknown-unknown/release/talos_language_${language}.wasm" \
    "$bundle_test_root/$language"
done
export TALOS_LANGUAGE_BUNDLE_ROOT="$bundle_test_root"
cargo run --locked -p talos-plugin --features wasm --example verify_language_guests -- \
  "$bundle_test_root/rust/provider.wasm" "$bundle_test_root/python/provider.wasm"
cargo run --locked -p talos-plugin --features wasm,code-intelligence --example verify_installed_languages -- \
  "$bundle_test_root/rust" "$bundle_test_root/python"
cargo test --locked -p talos-tui --features plugin-acceptance \
  real_installed_language_plugins_drive_tui_segments
cargo test --locked -p talos-tools --features plugin-acceptance --test installed_language_plugins
cargo test --locked -p talos-cli --features plugin-acceptance --test installed_language_permissions
