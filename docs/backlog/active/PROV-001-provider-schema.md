# PROV-001: Provider Schema Foundation

## Outcome

Talos can configure multiple OpenAI-compatible providers without recompiling, while dynamic
provider loading remains deferred.

## Status

Complete. Schema slice and opencode import landed on 2026-06-08: named provider/model config,
env-var-only secrets, built-in defaults, local config migration, and one-way opencode import with
9 unit tests. `cargo test -p talos-config -p talos-provider -p talos-cli` passes.

## Priority

P2.

## Required Reads

- `docs/iterations/I011-open-providers.md`
- `docs/iterations/I015-provider-schema.md`
- `docs/proposals/provider-plugin-architecture.md`
- `docs/decisions/013-provider-config-schema-boundary.md`

## Acceptance Criteria

- [x] Provider/model schema is represented as typed config data with serde + schemars.
- [x] Secrets are referenced by env var names only; config stores no API keys.
- [x] Model resolution behavior follows ADR-013 for active provider/model selection.
- [x] Dynamic loading, FFI, Node/Python runtimes, and package managers remain out of scope.
- [x] Migration/import behavior is covered by tests.

## Residual Work Destination

Dynamic provider loading requires a future ADR and a new item.
