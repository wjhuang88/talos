# I015: Provider Schema

**User can**: Configure multiple OpenAI-compatible providers without recompiling Talos, while
keeping provider configuration self-contained and secrets out of files.

## Status: PLANNED

## Decision Gate

Follow ADR-013. Any dynamic provider loading, external language runtime, FFI, or provider process
boundary requires a new ADR and is out of scope for this iteration.

## Selected Stories

- [ ] #I011-S2: Provider plugin architecture foundation
- [ ] Reasoning/thinking field follow-up only if it stays within the schema contract and receives
      its own ADR before public protocol/event changes.

## Scope

- Add provider/model schema types for named OpenAI-compatible providers.
- Add deterministic provider/model resolution.
- Add one-way import from opencode-style provider config.
- Keep existing `base_url` behavior compatible.

## Non-Goals

- No dynamic provider code loading.
- No Node/Python runtimes.
- No provider package manager.
- No persisted reasoning stream fields without a separate ADR.

## Acceptance Criteria

- [ ] Config schema supports named providers and models.
- [ ] API keys remain env-var references only.
- [ ] Ambiguous model names resolve deterministically or return clear errors.
- [ ] Migration/import path is one-way and tested.
- [ ] `cargo test -p talos-config -p talos-provider -p talos-cli` passes.
