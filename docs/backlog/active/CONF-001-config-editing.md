# CONF-001: Interactive Configuration Editing (CLI + TUI)

| Field | Value |
|-------|-------|
| Story ID | CONF-001 |
| Priority | P2 |
| Status | Partial (CLI surface complete via I045+F102; TUI `/config` deferred per F103) |
| Depends On | None |
| Estimate | M |
| Origin | User request 2026-06-17 — no in-app way to edit config; users must hand-edit `~/.talos/config.toml` |

## Problem

Talos stores all settings in `~/.talos/config.toml` (managed by `talos-config`). I045 added a
basic CLI flag surface for viewing and editing config: `--config-list`, `--config-get`, and
`--config-set`. The original product-facing target remains only partially satisfied because there
is no `talos config ...` subcommand UX, env/schema behavior still needs explicit evidence in this
story, and TUI `/config` remains deferred.

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

## Planning Link

Selected into
`docs/tasks/2026-06-29-crate-distribution-hardening-two-month-plan.md` as a reconciled C1-C3
feature track. The existing `--config-*` flags are baseline behavior; remaining work is
subcommand/compatibility design, validation evidence, UX hardening, and TUI readiness decision.

## Acceptance Criteria

- [x] `talos config get/set/list` read and write through the `talos-config` API and JSON-Schema
      validation rejects invalid values with a clear error. (I045 shipped subcommands; I080/F102
      added `Config::validate()` before save.)
- [x] Changes persist to `~/.talos/config.toml` and `${ENV_VAR}` substitution semantics survive a
      set/get round-trip. (I080/F102 evidence tests.)
- [x] Secret fields (e.g. persisted inline `api_key`) are never echoed in plaintext by
      `get`/`list`; `set` accepts them but masks on redisplay. (I045 + I080/F102 masking tests.)
- [ ] TUI `/config` can view and edit model + provider settings inline. (Deferred per I080/F103;
      CLI config commands and `/model` picker cover current needs.)
- [x] No regression for env-var-driven config or for existing config files on load. (I080/F102.)

## Execution Baseline

- I045 closed on 2026-06-24 and records CONF-001-S as complete for the flag-based CLI surface.
- Current code evidence:
  - `crates/talos-cli/src/main.rs` defines `--config-list`, `--config-get`, and `--config-set`.
  - `crates/talos-cli/src/main.rs` routes reads/writes through `Config::load()`,
    `config_get_dotted()`, `config_set_dotted()`, `Config::validate()`, and `Config::save()`.
  - `crates/talos-cli/src/main.rs` masks `api_key` values for list/get display.
  - `crates/talos-config/src/tests.rs` covers inline `api_key` serialization, env resolution, and
    config save/load behavior from the I045 fix.
- Residual implementation:
  - Decide whether to add `talos config get/list/set` subcommands or formally keep/document the
    existing flags.
  - Add/confirm tests that `${ENV_VAR}` substitution and JSON-Schema validation survive the chosen
    get/set/list path.
  - Improve user-facing errors/docs and decide whether TUI `/config` is ready or remains residual.

## Required Reads

- `crates/talos-config/src/lib.rs` (config struct, persisted inline api_key, `${ENV}` substitution)
- `crates/talos-cli/` (CLI subcommand surface)
- `crates/talos-tui/` (slash-command framework, gated by TUI-002 sub-slice D)
