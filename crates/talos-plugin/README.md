# Plugin host

This crate owns Plugin manifests, verified offline Bundle installation, explicit lifecycle,
capability registration and bounded WASM execution. Concrete optional language sources live in
[root plugins](../../plugins/README.md), outside the default host dependency graph.

`install_bundle(source, destination)` verifies the Bundle digest and copies the package. It does
not load, activate, register capabilities or grant permissions. The existing explicit loader uses
`talos-plugin.toml`; root language packages supply that activation descriptor alongside the
versioned `manifest.toml`, with matching identity and artifact.

Language guests accept caller-supplied source only. ABI v2 adds a checked guest-owned allocation;
ABI v1 remains supported. Language execution has a fresh store per request, no host imports,
64 MiB memory / 16,384 table-element ceilings, a 500 ms deadline and 2 billion fuel by default.
Explicit smaller limits remain effective. Generic Tool runtime fuel is separate and unchanged.
Timeout, malformed output and lifecycle stop retain the existing domain fallback behavior.

The language default covers cold Tree-sitter query initialization: on the development macOS
host, Rust's tiny-source highlight used approximately 897 million fuel / 56 ms, and 13 KiB used
1.054 billion / 73 ms. Python's 22 KiB sample used 260 million / 25 ms. These are diagnostic
measurements, not portable latency guarantees. Near-limit inputs can safely exhaust fuel or
the response budget; callers must keep handling unavailable/plain-text results.

Focused host checks:

```sh
cargo test --locked -p talos-plugin --features wasm,code-intelligence --lib
```
