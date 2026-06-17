# CONF-002: First-Run Model Configuration Onboarding

| Field | Value |
|-------|-------|
| Story ID | CONF-002 |
| Priority | P1 |
| Status | Planned |
| Depends On | CONF-001 (shares the config write path) |
| Estimate | M |
| Origin | User request 2026-06-17 — Talos enters the TUI with no model configured, then fails on first message |

## Problem

When no usable model configuration exists (no enabled provider, or provider missing `api_key` /
`model`), Talos still drops the user straight into the TUI. The first message then fails with a
provider/credential error, which is a confusing and hostile first-run experience.

## Proposed

Add a startup **pre-flight check**: if no model can be resolved, do NOT enter the TUI. Instead run
an interactive guided setup, then proceed:

1. Choose a provider (OpenAI, Anthropic, OpenAI-compatible/Bailian, DashScope, …).
2. Enter credentials (masked input), or point to an `${ENV_VAR}`.
3. Pick a model.
4. Optional connectivity test (a minimal models list / ping).
5. Persist via `talos-config` and continue into the TUI / CLI.

The wizard must be re-runnable (e.g. `talos init`) and must not block non-interactive / CI
environments.

## Acceptance Criteria

- [ ] Startup pre-flight detects "no usable model" (no enabled provider with valid `api_key` +
      `model`) before the TUI/CLI run loop starts.
- [ ] Interactive wizard guides provider → credentials → model → optional connectivity test with
      masked credential input, and writes config through `talos-config`.
- [ ] On success the session proceeds normally; on cancel it exits with a helpful, actionable
      message (never a silent hang or raw stack trace).
- [ ] Non-interactive environments degrade gracefully — env-var-driven config and a `--no-init`
      style escape hatch still start without prompting.
- [ ] `talos init` re-runs the wizard to reconfigure at any time.
- [ ] Reuses CONF-001 config primitives; no second config-write path.

## Required Reads

- `docs/backlog/active/CONF-001-config-editing.md`
- `crates/talos-config/src/lib.rs` (provider/model config, env substitution)
- `crates/talos-cli/` (startup / run-loop entry)
- `docs/backlog/active/PROV-001-provider-schema.md` (provider schema alignment)
