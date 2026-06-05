# ADR-013: Provider Config Schema Boundary

- **Status**: Accepted
- **Date**: 2026-06-05
- **Backlog**: #I011-S2

## Context

`#I011-S2` is the paused provider plugin architecture foundation. The intended
first slice is config/schema support for user-declared OpenAI-compatible
providers, not dynamic loading of external provider code.

This is a public configuration boundary. If it evolves ad hoc, every future
provider integration, migration, CLI override, and troubleshooting flow becomes
harder to keep stable.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| No secrets in code/config | Hard | AGENTS.md hard constraint #3 | No |
| Public config schema is compatibility-sensitive | Hard | AGENTS.md semver/public API rule | Only with migration plan |
| Provider openness without recompilation is desired | Soft | I011 product direction | Yes |
| External dynamic provider loading is unvalidated | Assumption | Proposal state | Defer |

## Reasoning

Talos already supports one pragmatic compatibility path: `provider = "openai"`
with `base_url`. The next step should make that configurable for multiple
named providers while avoiding dynamic runtime execution or language plugins.
That keeps the first slice self-contained and testable.

## Decision

- `#I011-S2` implements a schema/config foundation only. It must not load
  provider code from another language or dynamic library.
- The first provider config schema supports named providers, models, base URL,
  protocol, context/output limits, and `api_key_env`.
- Secrets are referenced by environment variable name only. Provider config must
  not store API keys.
- Initial protocol scope is `openai-chat` unless another protocol receives a
  separate ADR or story-level decision.
- Model resolution must be deterministic. Prefer `provider/model` syntax for
  ambiguous names; bare model names may resolve only within the configured
  provider.
- A one-way import from opencode-style config is allowed, but Talos config
  remains the source of truth after import.
- Dynamic loading, external provider processes, FFI, Node/Python runtimes, and
  plugin package managers are out of scope for `#I011-S2`.
- Any future provider plugin execution boundary must get a separate ADR that
  covers process isolation, protocol, lifecycle, versioning, and secrets.

## Reversal Trigger

Revisit this decision after schema-only provider config has shipped and there is
clear demand for provider code outside the Talos binary.
