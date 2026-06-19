# I035: Agent Protocol Compatibility Foundation

**Status**: Complete (2026-06-19)
**Target Window**: After architecture cleanup and runtime Skill/MCP activation
**Depends On**: I030-I034 complete preferred; may proceed as research-only if implementation
dependencies slip

## Outcome

Turn AGENT-001 into a concrete compatibility plan for common Agent protocol/config conventions,
including shared configuration locations such as `~/.agents`, without coupling Talos core runtime
types to unstable external schemas.

## Selected Stories

- [x] #AGENT-001-A: Survey common Agent protocol/config conventions and record source dates
- [x] #AGENT-001-B: Write an ADR for supported config/protocol compatibility boundaries
- [x] #AGENT-001-C: Define Talos-owned DTOs for shared Agent config import
- [x] #AGENT-001-D: Specify config precedence across CLI flags, env vars, workspace config,
      `~/.talos`, and shared Agent config
- [x] #AGENT-001-E: Prototype read-only import from `~/.agents` if the survey confirms a stable
      layout
- [x] #AGENT-001-F: Update user-facing docs for supported interoperability behavior

## Acceptance Criteria

- [x] The survey distinguishes confirmed facts, assumptions, and unstable conventions.
- [x] Any external protocol dependency is captured in an ADR before implementation.
- [x] Talos keeps `~/.talos` as the Talos-owned source of state.
- [x] Shared config support is read/import-first; no silent write-back.
- [x] Secrets remain env-var based or explicit-permission gated.
- [x] Tests cover precedence and non-overwrite behavior if implementation starts.
- [x] User docs explain what is supported and what is intentionally unsupported.
- [x] Imported shared config may enable known plugins or alias policy but cannot define executable
      command bodies outside BuiltinCommand/PluginCommand registration.

## Survey Findings (2026-06-19)

### Stable / Widely Adopted Conventions

| Convention | Source | Stability | Notes |
|---|---|---|---|
| AGENTS.md hierarchical loading | Community standard | Widely adopted | Talos already loads AGENTS.md from project root |
| Claude Code layered config (`~/.claude/` + `.claude/`) | Anthropic (production) | Stable | Reference for precedence model: managed > CLI > local > project > user |
| `~/.agents/` directory | dotagentsprotocol.com (Feb 2026), agentsstandard.com | Gaining traction | Multiple efforts converging on this location |

### Emerging / Draft Conventions

| Convention | Source | Stability | Notes |
|---|---|---|---|
| `~/.agents/models.json` (JSON) | dotagentsprotocol.com (Feb 2026) | Early | Proposes a flat JSON file with `providers` → `{name, protocol, apiKeyEnv, models}` structure. Adopted by some tools as a shared provider catalog. See §Decision Record below. |
| `.agents/` repo-level spec | agentsfolder/spec (Jan 2026) | Draft | manifest.yaml, modes/, policies/, skills/ — promising but not yet widely adopted |
| MCP config in `~/.agents/mcp.json` | dotagentsprotocol.com | Early | Format not standardized across tools |

### Decision Record: `models.json` vs `~/.agents/talos/config.toml`

The `~/.agents/models.json` convention (JSON, flat shared file) was evaluated and
**rejected as Talos's primary shared config format** for these reasons:

| Factor | `models.json` | `~/.agents/talos/config.toml` |
|---|---|---|
| Format | JSON — second format to learn | TOML — same as `~/.talos/config.toml` |
| Namespace | Flat shared file — conflicts with other tools | `talos/` subdirectory — isolated |
| Schema | Custom JSON schema | Reuses existing `ProviderConfig` TOML schema |
| Adoption | One proposal, not widely adopted | N/A (Talos-owned) |

The `models.json` schema is retained here as a reference for future compatibility:
if it gains wider adoption, Talos can add a `talos config import --from agents-models-json`
command without changing the primary shared config format.

### NOT Standards (Confirmed Absence)

- No single `~/.agent` directory is universally adopted
- No cross-vendor agent protocol schema exists
- Tool-specific configs (`.claude/settings.json`, `opencode.json`) are mutually incompatible

### Config Precedence (Talos)

| Priority | Source | Description |
|---|---|---|
| 1 (highest) | CLI flags | `--model`, `--provider`, `--mock` |
| 2 | Environment variables | `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc. |
| 3 | `~/.talos/config.toml` | Talos-owned source of truth |
| 4 (lowest) | `~/.agents/` imports | Read-only shared config; provides defaults only |

## Deliverables

| Story | Deliverable | Location |
|---|---|---|
| #AGENT-001-A | Ecosystem survey with dated source evidence | This document §Survey Findings |
| #AGENT-001-B | ADR-022: Agent Config Compatibility Boundary | `docs/decisions/022-agent-config-compatibility-boundary.md` |
| #AGENT-001-C | `talos-config::agents` module with DTOs | `crates/talos-config/src/agents.rs` |
| #AGENT-001-D | Config precedence specification | ADR-022 §Decision + this document §Survey Findings |
| #AGENT-001-E | `import_agents_config()` prototype for `~/.agents/talos/config.toml` (TOML) | `crates/talos-config/src/agents.rs` |
| #AGENT-001-F | Docs: ADR-022, this iteration record, README update | `docs/decisions/022-*.md`, this file |

## Verification Log

- `cargo test -p talos-config` — 43 tests pass (36 existing + 7 new agents tests)
- `cargo clippy -p talos-config -- -D warnings` — clean
- ADR-022 follows ADR template with Constraint Decomposition, Decision, and Reversal Trigger
- `talos-config::agents` follows the existing `talos-config::opencode` one-way import pattern
- Reads from `~/.agents/talos/config.toml` (TOML, Talos namespace under shared `~/.agents/`)
- No new dependencies added
- `~/.talos` remains Talos-owned; shared config is read-only, lowest priority

## Residual Work

- Wire `import_agents_config()` into the CLI startup path (opt-in via config or flag)
- Add `~/.agents/AGENTS.md` global rules loading when Skill/MCP context-prefix grows
- Monitor `.agents/` repo-level spec for stabilization before adopting
- Monitor `~/.agents/models.json` adoption; add `talos config import --from agents-models-json` if it standardizes
- Consider a `talos config import` command for explicit migration
