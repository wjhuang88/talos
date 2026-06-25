# CONF-001: Interactive Configuration Editing (CLI + TUI)

| Field | Value |
|-------|-------|
| Story ID | CONF-001 |
| Priority | P2 |
| Status | Planned |
| Depends On | None |
| Estimate | M |
| Origin | User request 2026-06-17 — no in-app way to edit config; users must hand-edit `~/.talos/config.toml` |

## Problem

Talos stores all settings in `~/.talos/config.toml` (managed by `talos-config`), but there is no
built-in way to view or edit configuration from inside the product. Users have to know the file
location, understand the schema, and edit TOML by hand. This is error-prone and a poor experience,
especially for provider/model/API-key setup.

## Proposed

Provide two complementary configuration surfaces that both round-trip through `talos-config`
(never writing raw TOML behind its back):

- **CLI subcommand** `talos config`:
  - `talos config list` — print all settings.
  - `talos config get <key>` — print a single value.
  - `talos config set <key> <value>` — validate + persist.
  - `talos config edit` — open the file in `$EDITOR` (optional convenience).
- **TUI slash command** `/config` — view and edit the most important settings (at minimum model +
  provider) inline, without leaving the TUI.

## Acceptance Criteria

- [ ] `talos config get/set/list` read and write through the `talos-config` API and JSON-Schema
      validation rejects invalid values with a clear error.
- [ ] Changes persist to `~/.talos/config.toml` and `${ENV_VAR}` substitution semantics survive a
      set/get round-trip.
- [ ] Secret fields (e.g. persisted inline `api_key`) are never echoed in plaintext by
      `get`/`list`; `set` accepts them but masks on redisplay.
- [ ] TUI `/config` can view and edit model + provider settings inline.
- [ ] No regression for env-var-driven config or for existing config files on load.

## Required Reads

- `crates/talos-config/src/lib.rs` (config struct, persisted inline api_key, `${ENV}` substitution)
- `crates/talos-cli/` (CLI subcommand surface)
- `crates/talos-tui/` (slash-command framework, gated by TUI-002 sub-slice D)
