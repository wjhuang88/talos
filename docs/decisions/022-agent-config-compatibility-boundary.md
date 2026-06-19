# ADR-022: Agent Config Compatibility Boundary

- **Status**: Accepted
- **Date**: 2026-06-19
- **Backlog**: #AGENT-001, I035

## Context

The Agent ecosystem is converging on shared configuration conventions.
`~/.agents/` and `.agents/` directories are proposed by multiple efforts
(agentsfolder/spec, agentsstandard.com, dotagentsprotocol.com) as
vendor-neutral locations for agent rules, MCP config, skills, and model
settings. Claude Code uses a layered model (`~/.claude/` + `.claude/`) with
clear precedence. The AGENTS.md standard is widely adopted for hierarchical
project instructions.

Talos currently reads only from `~/.talos/config.toml`. Without compatibility,
users must duplicate configuration across tools.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|---|---|---|---|
| No secrets in code/config | Hard | AGENTS.md HC #3 | No |
| `~/.talos` is Talos-owned state | Hard | AGENT-001 scope | No |
| Public config schema is semver-bound | Hard | AGENTS.md HC #6 | Only with migration plan |
| No silent write-back to shared config | Soft | I035 acceptance | Yes |
| Shared config import is read-only | Soft | I035 acceptance | Yes |
| `~/.agents/` convention gains ecosystem traction | Assumption | Survey (2026-06-19) | Monitor |

## Survey Findings (2026-06-19)

### Emerging Conventions

| Convention | Source | Stability | Layout |
|---|---|---|---|
| `~/.agents/` | dotagentsprotocol.com (Feb 2026) | Early, gaining traction | agents.md, mcp.json, models.json, skills/, agents/ |
| `~/.agents/AGENTS.md` | agentsstandard.com | Moderate | Global behavior rules + symlinks to tool-specific paths |
| `.agents/` (repo root) | agentsfolder/spec (Jan 2026) | Draft spec | manifest.yaml, modes/, policies/, skills/, profiles/ |
| `.claude/` + `~/.claude/` | Claude Code (production) | Stable | settings.json, CLAUDE.md, skills/, agents/, rules/ |
| AGENTS.md | Community standard | Widely adopted | Hierarchical, root-to-cwd loading |

### What is NOT a standard

- No single `~/.agent` directory is universally adopted
- No shared agent protocol schema exists across vendors
- Tool-specific config formats (`.claude/settings.json`, OpenCode `opencode.json`) are
  incompatible with each other

### Claude Code Precedence Model (reference)

| Priority | Source | Location |
|---|---|---|
| 1 (highest) | Managed policy | OS/MDM-level |
| 2 | CLI flags | `--model`, `--permission-mode` |
| 3 | Local overrides | `.claude/settings.local.json` |
| 4 | Project settings | `.claude/settings.json` |
| 5 (lowest) | User settings | `~/.claude/settings.json` |

All levels are additive for instructions (CLAUDE.md) but overridden for settings.

## Reasoning

### Namespace under `~/.agents/talos/`

`~/.agents/` is a shared directory that other agent tools may also read from or write to.
To avoid configuration conflicts between tools, Talos uses its own namespace:
`~/.agents/talos/`.  This follows the same principle as `~/.config/<app>/` on Linux and
`~/Library/Application Support/<app>/` on macOS — each tool gets a subdirectory.

```
~/.agents/
├── talos/
│   └── config.toml      # Talos provider/model shared config
├── AGENTS.md            # Global behavior rules (all agents)
├── mcp.json             # Shared MCP config (future standard)
└── skills/              # Shared skill definitions (future standard)
```

### What to support

1. **`~/.agents/talos/config.toml` read support**: The `~/.agents/` convention is gaining
   traction. Talos reads from its own namespace under it, leaving the rest of `~/.agents/`
   available for other tools and shared standards to evolve independently.

2. **TOML format**: Consistent with `~/.talos/config.toml`. Users editing Talos config
   already know TOML; no second format to learn.

3. **Talos-owned config precedence**: CLI flags > env vars > `~/.talos/config.toml` >
   `~/.agents/talos/config.toml`. Shared config is the lowest priority — it provides
   defaults that Talos-specific config can override.

### What NOT to support now

1. **`.agents/` repo-level**: The spec is still draft. Wait for stabilization before importing
   project-level agent config.

2. **Tool-specific imports (`.claude/`, OpenCode)**: Already have `opencode` import for provider
   config. Don't add general-purpose import of vendor-specific formats — the ecosystem is
   converging on `~/.agents/`.

3. **Write-back to shared config**: Always explicit, never automatic. An import/migration command
   may be added later.

4. **Agent protocol schemas**: No stable cross-vendor protocol exists. This is adjacent to
   REMOTE-001 and not in scope.

## Decision

- **Talos reads from `~/.agents/talos/config.toml`** (TOML format) as a read-only
  compatibility layer. The `talos/` subdirectory namespaces Talos config within the
  shared `~/.agents/` directory, preventing conflicts with other tools.
- **Import is opt-in**: Enabled via `config.toml` (`[agents]` section) or explicit
  `talos config import` command. Not automatic.
- **Precedence order** (highest first):
  1. CLI flags (`--model`, `--provider`)
  2. Environment variables (`ANTHROPIC_API_KEY`, etc.)
  3. `~/.talos/config.toml` (Talos-owned source of truth)
  4. Workspace-local `.talos/config.toml` (future)
  5. `~/.agents/talos/config.toml` (read-only, lowest priority)
- **Secrets remain env-var based**: `api_key_env` references only. No API keys in
  shared config.
- **DTO boundary**: External config shapes convert into Talos-owned types at the import edge.
  `talos-config::agents` module owns the import logic, following the existing
  `talos-config::opencode` pattern.
- **TOML format**: Same format as `~/.talos/config.toml` — users don't need to learn
  a second config language.

## Reversal Trigger

- If `~/.agents/` is abandoned by the ecosystem within 12 months, deprecate `~/.agents/talos/` import.
- If another tool's config format under `~/.agents/` standardizes and conflicts with
  Talos's namespace, re-evaluate the subdirectory name.
- If a cross-vendor agent protocol schema standardizes (e.g., RFC or W3C), replace
  custom DTOs with the standard schema.

## Required Reads

- `docs/proposals/standard-agent-protocol-support.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/backlog/active/AGENT-001-standard-agent-protocol-support.md`
- `docs/iterations/I035-agent-protocol-compatibility-foundation.md`
