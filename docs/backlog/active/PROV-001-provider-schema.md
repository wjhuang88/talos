# PROV-001: Provider Schema Foundation

## Outcome

Talos can configure multiple OpenAI-compatible providers without recompiling, while dynamic
provider loading remains deferred.

## Status

Planned. Selected into I015; I011 S2 remains paused until this slice is activated.

## Priority

P2.

## Required Reads

- `docs/iterations/I011-open-providers.md`
- `docs/iterations/I015-provider-schema.md`
- `docs/proposals/provider-plugin-architecture.md`
- `docs/decisions/013-provider-config-schema-boundary.md`

## Acceptance Criteria

- [ ] Provider/model schema is represented as typed config data with serde + schemars.
- [ ] Secrets are referenced by env var names only; config stores no API keys.
- [ ] Model resolution behavior follows ADR-013.
- [ ] Dynamic loading, FFI, Node/Python runtimes, and package managers remain out of scope.
- [ ] Migration/import behavior is covered by tests.

## Residual Work Destination

Dynamic provider loading requires a future ADR and a new item.

