# I015: Provider Schema

**User can**: Configure multiple OpenAI-compatible providers without recompiling Talos, while
keeping provider configuration self-contained and secrets out of files.

## Status: COMPLETE

## Decision Gate

Follow ADR-013. Any dynamic provider loading, external language runtime, FFI, or provider process
boundary requires a new ADR and is out of scope for this iteration.

## Selected Stories

- [x] #I011-S2: Provider plugin architecture foundation
- [ ] Reasoning/thinking field follow-up only if it stays within the schema contract and receives
      its own ADR before public protocol/event changes.

## Scope

- Add provider/model schema types for named built-in and OpenAI-compatible providers.
- Add deterministic active provider/model resolution.
- Add one-way import from opencode-style provider config.
- Migrate away from top-level `base_url`/`api_key` fields; secrets are env-var references only.

## Non-Goals

- No dynamic provider code loading.
- No Node/Python runtimes.
- No provider package manager.
- No persisted reasoning stream fields without a separate ADR.
- No top-level provider credential or base URL fields.

## Acceptance Criteria

- [x] Config schema supports named providers and models.
- [x] API keys remain env-var references only.
- [x] Active provider/model resolution is deterministic.
- [x] Migration/import path is one-way and tested.
- [x] `cargo test -p talos-config -p talos-provider -p talos-cli` passes.

## 2026-06-06 Progress

- `talos-config` now has `ProviderConfig`, `ModelConfig`, and `ProviderProtocol`.
- `~/.talos/config.toml` uses:
  - top-level `provider = "<name>"` and `model = "<model>"`;
  - `[providers.<name>]` for protocol, base URL, and `api_key_env`;
  - `[providers.<name>.models.<model>]` for context/output limits.
- Built-in defaults exist for `anthropic` and `openai`.
- Local Bailian test config was migrated to `provider = "bailian"` with `glm-5` limits
  `context_limit = 202752` and `output_limit = 4096`.

## 2026-06-08 Progress

- Added `opencode` module to `talos-config` with `import_opencode_providers()`.
- Translates opencode JSON provider blocks into Talos `ProviderConfig`/`ModelConfig`.
- Supports full config (`{ "provider": { ... } }`) or bare provider object.
- Maps `npm` field to `ProviderProtocol` (anthropic → `AnthropicMessages`, default → `OpenAIChat`).
- Merges imported providers into existing `Config` via `Config::import_opencode_providers()`.
- 9 unit tests cover: single/multiple provider import, bare object, missing fields, unknown npm,
  invalid JSON/schema, and partial model limits.
- 36 tests pass in `talos-config` (was 27; +9 from opencode import), 37 in `talos-provider`,
  24 in `talos-cli`.

## Residual Work

- Consider exposing configured `context_limit` to the future compaction trigger after the system
  prompt and conversation history are separated enough to compact safely.
