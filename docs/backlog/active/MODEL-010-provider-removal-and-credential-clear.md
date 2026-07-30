# MODEL-010: Provider Removal And Credential Clear

| Field | Value |
| --- | --- |
| Story ID | MODEL-010 |
| Type | Product / Configuration Story |
| Priority | P2 |
| Status | In Progress — Finalization Durability (I157 reopened round 11) |
| Source | Maintainer requirement recorded 2026-07-24: "现在是不是没有删除已连接 provider 的能力" → confirmed absent, requested backlog entry |
| Parent Epic | None (peer to MODEL-008 provider lifecycle work) |
| Depends on | MODEL-008-A `/connect` wizard (I147); ADR-013 provider config schema; ADR-023 inline api_key boundary; TUI-033 parameterless commands (I146) |
| Blocks | None |
| Selected Iteration | I157 (Complete; Phase 1 `84e7a6a3`, Phase 2 `46c919ee`) |





> Completion Commit: `7153bfaa` + `911887f4` — sanitized TOML parse, ambiguous finalize protection, FsOperation enum, FaultPlan with ordered failures, checkpoint() semantic hook, all numeric FaultyFs migrated, corrupt-credentials secret-safe error.

## Problem

The provider configuration lifecycle is currently additive-only. `/connect`
(`command_registry.rs:262`, `session_handlers.rs::handle_connect_with_credential`)
and the custom-provider wizard (MODEL-008-A) can **add or update** a provider,
but no user-facing surface can **remove** one:

- No `/disconnect` (or equivalent) slash command exists.
- `ConfigCommand` (`crates/talos-cli/src/main.rs`) exposes only `List` / `Get` /
  `Set` — there is no `unset` / `delete` / `remove`.
- `config_set_dotted` (`main.rs`) only ever assigns; there is no
  `providers.remove(name)` path anywhere in the codebase.
- The `/connect` ConnectPicker (`crates/talos-tui/src/panel_state.rs`
  `open_connect_picker`) offers only `ConnectSelect` / `OpenWizard`; there is no
  delete action.

The only current workaround is hand-editing `~/.talos/config.toml` to delete the
`[providers.<name>]` table — the exact TOML-editing burden the MODEL-008 line of
work set out to eliminate. This story closes the missing symmetric capability.

## Goal / Value

Let a user remove a previously connected provider — clearing its persisted
credential and its `[providers.<name>]` entry — without hand-editing TOML, using
a cancel-safe, confirmation-gated flow that never touches unrelated providers,
models, or the active-model selection except where removal makes it invalid.

## Scope

### Removal semantics (the hard design decision)

`self.providers` only holds providers the user has explicitly configured;
built-in providers are injected on demand via `builtin_provider_config`
(`config.rs:520-529`). Therefore removal has two distinct meanings that MUST be
made explicit in behavior and UI copy:

1. **Custom (non-builtin) provider** — removing the `[providers.<name>]` entry
   deletes it entirely; it disappears from `/connect` "Connected" and does not
   reappear as "Available" (it was never a builtin).
2. **Builtin-backed provider** (e.g. `anthropic`, `openai`) — removing the entry
   clears the persisted credential and user overrides, but the provider name
   still appears under `/connect` "Available" because the builtin catalog is
   hardcoded. The UI MUST communicate this as "credential cleared / disconnected"
   rather than implying the provider itself was destroyed.

### Surfaces (decided: CLI-only for this story)

- **CLI (in scope)**: add `talos config unset <dotted-key>` (a new `ConfigCommand::Unset`
  variant) with a required `--confirm` flag:
  - `config unset providers.<name> --confirm` → removes the entire
    `[providers.<name>]` entry.
  - `config unset providers.<name>.api_key --confirm` → clears only that field to
    `None` (field omitted from TOML, never `""`).
  - Without `--confirm`, return an instructive error and make no change (mirrors
    `/todo delete` gating in `todo_view.rs::handle_todo_delete`).
- **TUI (out of scope, deferred)**: a `/connect` ConnectPicker delete action is
  deferred to a follow-up story. It requires threading a new
  `PanelItemAction`/`PanelAction`/`UserInput`/`SessionLifecycleRequest` variant
  through six layers (`panel_state.rs` → `state.rs` → app loop → `tui_bridge.rs`
  → `session_handlers.rs`) plus a destructive-row confirm step, which is a
  materially larger surface than the config-mutation core this story delivers.

### Active-model safety (decided: proceed, rely on existing picker fallback)

Removal is **not refused** when it targets the active provider. The
"no active model → re-open model picker" path already exists and is tested:
startup detects the invalid/unauthenticated active model
(`mode_runners.rs:232` `needs_model_setup` / `needs_api_key`) and
`handle_session_model` opens the picker when `model_id` is empty
(`session_handlers.rs:488`); the picker omits unauthenticated providers
(`model_lifecycle.rs:51`). After removing the active provider,
`provider_authenticated(config.provider)` returns `false` and the next session
start / `/model` re-prompts selection rather than crashing. This story MUST add a
regression test proving that removing the active provider leaves the config in a
picker-recoverable state and does not panic in `active_provider_config()` /
`config.api_key()` when `config.provider` points at the removed entry.

### Persistence

- Exactly one atomic config write on success. No partial write on cancel, field
  error, "provider not found", active-model conflict, or I/O failure.
- Clearing a single credential field writes `None` (key omitted from TOML), never
  a dangling `api_key = ""` / `api_key_env = ""`.

## Explicit Exclusions

- Bulk / wildcard removal ("remove all providers").
- Removing individual model overrides within a provider
  (`providers.<name>.models.<model>`) — credential/entry removal only.
- Any change to the builtin catalog itself (a builtin provider name cannot be
  "deleted" from the catalog; only its user config entry can be cleared).
- New `unsafe`, native dependencies, or a second credential store.
- Relaxing the ADR-023 masking/persistence boundary.
- Environment-variable secret deletion (a user-owned `${ENV_VAR}` is not touched;
  only the config entry / inline key is removed).

## Design / Security Constraints

- Reuse `ProviderConfig`, endpoint/config save logic, and the ADR-023 masking
  boundary. Do not introduce a parallel mutation path.
- Determine and document public API / semver impact before changing any public
  config-mutation Rust API. A new `ConfigCommand` variant is an additive CLI
  surface; a breaking public API change requires an ADR with migration guidance
  before this story is Ready.
- Removal must be confirmation-gated (CLI: explicit `--confirm` or interactive
  y/N; TUI: a confirm step) to match the destructive-action pattern used by
  session `/delete`.
- Never render, log, or serialize the cleared credential value during or after
  removal; diagnostics show `***` or absence only.

## Acceptance

Behavior:

- Given a connected **custom** provider, when the user runs
  `config unset providers.<name> --confirm`, then its `[providers.<name>]` entry
  is gone from `~/.talos/config.toml`, unrelated providers/models are unchanged,
  and it no longer appears under `/connect` Connected or Available.
- Given a connected **builtin-backed** provider (e.g. `anthropic`), when the user
  runs `config unset providers.<name> --confirm`, then its persisted
  credential/overrides are cleared and the entry removed, the provider still
  appears under `/connect` Available (builtin catalog is hardcoded), and the
  output states the credential was cleared / provider disconnected (not
  destroyed).
- Given `config unset providers.<name>` **without** `--confirm`, when run, then an
  instructive error is returned and config is exactly unchanged.
- Given a removal of the provider that owns the active model, when confirmed,
  then config is written with the provider gone, `active_provider_config()` /
  `config.api_key()` do not panic, and the next session start / `/model` re-opens
  the model picker (no dangling reference to a deleted provider).
- Given a single-credential clear (`config unset providers.<name>.api_key
  --confirm`), when applied, then that field is written as `None` (absent from
  TOML) while the rest of the `[providers.<name>]` entry is preserved.
- Given any not-found key, validation error, or I/O failure, when it occurs, then
  config is exactly unchanged (no partial write).
- Given logs, panel labels, `talos config list`, or Debug output after removal,
  when inspected, then no cleared credential value is present.

Technical / governance:

- [x] `cargo test --workspace --locked` proves entry removal, single-field clear,
      `--confirm` gating, no-partial-write, and active-provider-removal recovery.
- [x] `cargo fmt --all`, `cargo clippy --workspace --locked -- -D warnings`,
      `scripts/validate_project_governance.sh .`, and `git diff --check` are clean.
- [x] Owner status here and the Board mirror are synchronized.
- [x] README EN/zh-CN and `docs/reference/config.reference.toml` document
      `config unset`, or a documentation residual is registered.

## Completion Evidence

- Phase 1 (config mutation core): `84e7a6a3` — `ConfigUnsetOutcome` enum +
  `Config::unset_dotted` method + 10 talos-config unit tests.
- Phase 2 (CLI wiring + docs): `46c919ee` — `ConfigCommand::Unset` variant +
  `run_config_unset` + `--confirm` gating + 11 talos-cli tests + README/zh-CN/
  config.reference.toml documentation.
- Workspace: `cargo test --workspace --locked` 2566 passed, 0 failed.
- Clippy: `cargo clippy --workspace --locked -- -D warnings` clean.
- Governance: `scripts/validate_project_governance.sh .` 0 warnings.
- Runtime fixture: isolated HOME; missing-`--confirm` byte-identical; custom
  removal; builtin-backed disconnection semantics; api_key clear (no empty
  string in TOML); active-provider removal → picker recovery, no panic;
  credential scan clean in `config list`/`config get`.
- Security scan: no credential in stdout/stderr; no `unsafe`; no new
  dependencies; no bulk/wildcard; single mutation path via `config.unset_dotted`.

## Resolved Decisions (2026-07-24)

1. **Surface** — CLI `config unset <dotted-key> --confirm` only. TUI `/connect`
   delete deferred to a follow-up story (six-layer plumbing + destructive-row
   confirm is out of proportion to the config-mutation core).
2. **Command shape** — `config unset providers.<name>` (extends the existing
   dotted-key `config set` grammar in `config_set_dotted`), not a separate
   `provider remove`. This reuses one config-mutation vocabulary.
3. **Confirmation** — required `--confirm` flag, matching the destructive
   `/todo delete <id> --confirm` pattern; picker-select is reserved for
   reversible actions (`/detach`, session `/delete`).
4. **Active-model conflict** — proceed and rely on the existing tested
   "no active model → picker" fallback; do not refuse. A regression test for the
   active-provider-removal recovery path is mandatory (see Acceptance).
5. **Model overrides** — removing `providers.<name>` drops that provider's
   `models.*` overrides with it (they are meaningless without the provider);
   clearing only `providers.<name>.api_key` preserves the rest of the entry
   including model overrides.

## Required Reads

- `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
- `docs/iterations/I157-provider-removal-credential-clear.md`
- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `docs/backlog/active/MODEL-008-A-interactive-custom-provider-wizard.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `crates/talos-config/src/types.rs`
- `crates/talos-config/src/config.rs`
- `crates/talos-config/src/builtin.rs`
- `crates/talos-cli/src/main.rs`
- `crates/talos-cli/src/session_handlers.rs`
- `crates/talos-tui/src/panel_state.rs`
- `crates/talos-tui/src/state.rs`

## Minimum Validation

- Config mutation unit tests: remove custom entry; clear builtin-backed
  credential; single-field clear writes `None`; unrelated providers preserved.
- No-partial-write property tests across cancel / not-found / active-model /
  I/O-failure paths.
- Active-model-safety test for the committed policy.
- Credential-absence / masking assertions after removal.
- Locked fmt / check / clippy / test and
  `scripts/validate_project_governance.sh .`; `git diff --check`.
