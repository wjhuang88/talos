# Optional Plugin sources

`languages/rust` and `languages/python` are real source-only Tree-sitter language Providers.
They reuse Arborium 2.18.2 grammars and the existing Talos symbol queries; Python source is parsed,
never executed. `languages/guest` owns the shared WASM wire adapter. `package` is an offline
Bundle packager. The host remains in `crates/talos-plugin`.

This is an independent Cargo workspace with its own committed lockfile. Build from a complete
Talos checkout: the guests compile the source-only `symbol.rs` and `symbol_queries.rs` modules
from `crates/talos-text/src`. Copying only `plugins/` is insufficient. No optional guest becomes
a static dependency of the default host. Build each language separately to avoid workspace
feature unification including both grammars in an individual artifact.

## Build and package

Use the repository-pinned Rust toolchain, the `wasm32-unknown-unknown` standard library, and a
Clang toolchain capable of targeting WASM. Arborium compiles its reviewed C grammar/runtime
internals into the isolated WASM artifact; it does not load native grammar libraries into Talos.
On macOS, `llvm-ar` must be available (the bundled Rust LLVM tools or LLVM on PATH); BSD `ar`
cannot archive WASM objects. Missing tools fail the build, without a host-execution fallback.

Run from the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo build --manifest-path plugins/Cargo.toml --locked --release --target wasm32-unknown-unknown -p talos-language-rust
cargo build --manifest-path plugins/Cargo.toml --locked --release --target wasm32-unknown-unknown -p talos-language-python
mkdir -p plugins/bundles
cargo run --manifest-path plugins/Cargo.toml --locked -p talos-plugin-package -- rust plugins/target/wasm32-unknown-unknown/release/talos_language_rust.wasm plugins/bundles/rust
cargo run --manifest-path plugins/Cargo.toml --locked -p talos-plugin-package -- python plugins/target/wasm32-unknown-unknown/release/talos_language_python.wasm plugins/bundles/python
```

Output directories must not already exist: packaging refuses to overwrite them. For a rebuild,
choose a new output directory. Generated target/Bundle directories are not source artifacts and
are ignored. The packager validates WASM, rejects imports/start functions, requires ABI exports
and a constant version-2 probe, and writes an actual SHA-256 digest. Packaging neither installs
nor activates. ABI v1-only hosts reject these new guests; an I278 ABI-v2 host is required.

Each package supplies versioned `manifest.toml`, `provider.wasm`, and a matching
`talos-plugin.toml` for the existing explicit lifecycle/`--plugin` loader. Use the host SDK's
`talos_plugin::install_bundle` for digest-verified installation, then explicitly select the
installed directory with `--plugin`. There is no implicit download, activation or permission
grant. Do not confuse repository source `plugins/` with installed `.talos/plugins/` directories.

CLI language-provider activation currently reaches TUI/shared-context consumers. Print and inline
paths do not retain that language context; passing `--plugin` there does not enable these language
providers. This delivery preserves that existing limitation rather than claiming all-mode support.

## Checks and limits

Full optional-plugin acceptance (including both actual TUI and symbol-tool consumers) runs with
`bash scripts/validate_language_plugins.sh`. It builds each language separately, creates temporary
verified Bundles and cleans those packages after the checks. CI runs this stage only on the full
code-validation route; changes to root plugin sources or reused talos-text modules take that route.
The opt-in `plugin-acceptance` features require `TALOS_LANGUAGE_BUNDLE_ROOT` pointing to a directory
containing real `rust/` and `python/` packages; missing artifacts fail instead of silently skipping.

```sh
cargo test --manifest-path plugins/Cargo.toml --locked --workspace
cargo run --locked -p talos-plugin --features wasm --example verify_language_guests -- plugins/target/wasm32-unknown-unknown/release/talos_language_rust.wasm plugins/target/wasm32-unknown-unknown/release/talos_language_python.wasm
cargo run --locked -p talos-plugin --features wasm,code-intelligence --example verify_installed_languages -- plugins/bundles/rust plugins/bundles/python
```

The diagnostic command validates real artifact admission, source-dependent highlights and four
symbol operations, including Unicode/comments, repeated declarations and near-limit fallback.
The second command installs into a uniquely owned temporary directory, checks explicit activation,
shared-context behavior and revocation, then cleans its installation. These diagnostics do not
replace TUI-renderer and AgentTool integration acceptance. Guest source and result caps
are each 256 KiB. Resource or malformed-source failures remain unavailable/plain text. Highlight
captures use Arborium's own flat-token priority/overlap resolver and canonical theme names.
Symbols retain built-in syntactic behavior: references are not semantic name resolution, and
definition lookup can return no result after parse failure. Guests never read caller paths.

## Layout dispositions

Rust/Python are delivered here. TypeScript, Browser/CDP and tool examples in #466's illustrative
tree are not implemented by I278; no empty placeholders imply those capabilities have shipped.
Existing no-op language WAT fixtures and constant tool WAT fixtures stay under host tests as
negative/compatibility fixtures. They are not these production source implementations. The
built-in language provider and default distribution are preserved.
