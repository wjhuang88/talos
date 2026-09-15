# Language Provider Distribution Matrix

This matrix records the first post-Rust provider distribution path required by LANG-003.
Providers are verified offline Bundles and are never downloaded or activated implicitly.

| Language | Bundle identity | ABI | Compatibility | Missing/corrupt/incompatible fallback | Consumers |
|---|---|---:|---|---|---|
| Python | `offline-python-language-provider` `0.1.0` | WASM language ABI `1` | `talos-plugin` Bundle manifest + `talos_text::LanguageId::python` | Plain text; provider remains unavailable | Highlighting and shared symbol-provider context |

The fixture under `crates/talos-plugin/tests/fixtures/language-provider-python` is deliberately
offline and has no network or startup activation path. Its zero-result provider exercises the
safe plain-text fallback while validating the same Bundle and ABI boundary used by a real asset.
Static parser dependencies remain part of the existing default build; the provider fixture is a
separately distributed asset and must not be counted as a default parser footprint.
