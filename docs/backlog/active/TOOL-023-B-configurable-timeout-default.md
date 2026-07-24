# TOOL-023-B: Configurable Execution Timeout with 300s Default

**Status**: Ready (2026-07-24)
**Priority**: P2
**Parent Epic**: TOOL-023
**Type**: Product/State Story
**Depends on**: TOOL-023-A (bash timeout must actually fire first)

## Problem

Both `bash` and `exec` accept a per-call `timeout_secs` (clamped `[1, 600]`) but the
default is 120s and there is no global config knob. The requester wants a 300s
default that a tool call may override and that an operator may set globally. 300s
matches the existing sandbox precedent (`talos-sandbox/src/hardening.rs`
`RLIMIT_CPU = 300`).

## Goal / Value

Operators get a single, documented default execution timeout (300s), overridable per
call and via config, without editing code.

## Scope

- Change the `bash` and `exec` built-in default timeout from 120s to 300s.
- Add a global config field, e.g. a new `[tools]` table in
  `crates/talos-config/src/types.rs` `Config` with `default_timeout_secs: u64`
  (serde default 300, schemars-validated). Wire it through tool construction in
  `crates/talos-agent/src/configuration.rs` so `BashTool`/`ExecTool` receive it.
- Precedence: per-call `timeout_secs` (if present) > global `[tools].default_timeout_secs`
  (if set) > built-in 300s default.
- Document the key in `docs/reference/config.reference.toml` and the README tool section.

## Exclusions

- Max clamp stays at 600s (raising it is deferred at the Epic level; if a per-call
  value exceeds the clamp it is still clamped — record this interaction in the story
  so it is not a surprise).
- No per-tool distinct defaults (`bash` and `exec` share the same default) unless a
  concrete need appears.

## Decision Links And Constraints

- ADR-023 (config boundary): the new field is plain config, no secret handling.
- Config types use `serde` + `schemars` with JSON Schema validation on load
  (workspace rule); the new field must follow that.

## Uncertainty And Validation Path

Confirm precedence with tests: (a) per-call value wins over config; (b) config wins
over built-in; (c) absent both yields 300s; (d) a per-call value above the max clamp
is clamped to 600s.

## State/Status Owners

This story file; parent `TOOL-023`; Board mirror.

## User-Facing Documentation

- `docs/reference/config.reference.toml`: document `[tools].default_timeout_secs`.
- `README.md` / `README.zh-CN.md`: update the bash/exec timeout wording to state the
  300s default and the config key.

## Required Reads

- `crates/talos-tools/src/bash_tool.rs` (default + `timeout_secs` input)
- `crates/talos-tools/src/exec_tool.rs` (default + `timeout_secs` input)
- `crates/talos-config/src/types.rs` (`Config` struct)
- `crates/talos-agent/src/configuration.rs` (tool construction wiring)

## Acceptance for behavior

- Given no per-call timeout and no config override
  When `bash`/`exec` runs a command that exceeds 300s
  Then it is killed at ~300s with a timeout result.

- Given `[tools].default_timeout_secs = 60` in config and no per-call timeout
  When `bash`/`exec` runs a command exceeding 60s
  Then it is killed at ~60s.

- Given a per-call `timeout_secs = 10` and a config default of 300
  When the command exceeds 10s
  Then it is killed at ~10s (per-call wins).

## Acceptance for technical work

- [ ] Precedence tests (per-call > config > built-in 300s) pass in `talos-tools`/`talos-agent`.
- [ ] Clamp-interaction test: per-call value > 600 is clamped to 600.
- [ ] `config.reference.toml` and README (EN/zh-CN) updated.
- [ ] `cargo test --workspace --locked` + clippy clean.
- [ ] Parent `TOOL-023` and Board synchronized.
