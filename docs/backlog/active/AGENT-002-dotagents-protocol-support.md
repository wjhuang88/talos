# AGENT-002: dotagentsprotocol.com Shared Config Support

**Status**: Research
**Priority**: P3
**Source**: I035 Survey (2026-06-19)
**Depends on**: AGENT-001 complete; dotagentsprotocol.com stabilization

## Problem

The [dotagentsprotocol.com](https://dotagentsprotocol.com) project (Feb 2026) proposes a
vendor-neutral `~/.agents/` directory convention.  Three of its defined areas overlap with
Talos's existing subsystems — each requiring different compatibility work:

| Area | File | Talos subsystem | Compatibility nature |
|---|---|---|---|
| Models | `~/.agents/models.json` | `talos-config` | Pure config import |
| Skills | `~/.agents/skills/` | `talos-skill` | Runtime discovery path |
| MCP | `~/.agents/mcp.json` | `talos-mcp` | Config import + server lifecycle |

## Sub-items

### AGENT-002-A: `~/.agents/models.json` Import (P3)

**What**: Read model presets and provider config from the shared JSON file.

**Talos impact**: Minimal — one-way import into `[providers]` map, same pattern as
`talos-config::opencode`.  Already prototyped the DTO mapping in I035 survey.

**Risk**: Low.  Purely an import layer; no runtime behavior change.

**Gate**: Schema must be versioned and stable for 6 months.

### AGENT-002-B: `~/.agents/skills/` Discovery Path (P2)

**What**: Add `~/.agents/skills/` as a skill discovery source, alongside the existing
`.talos/skills/` and `~/.talos/skills/` paths.

**Talos impact**: Changes to `talos-skill::loader` and `talos-skill::manager`.
Skill discovery is a runtime path — new sources affect the Level 0 metadata index
injected into the system prompt before every turn.

**Key decisions**:
- Should `~/.agents/skills/` take precedence over `~/.talos/skills/` or vice versa?
- Should it be opt-in (config flag) or automatic?
- Skill content from shared paths must go through the same token budget gating as
  Talos-owned skills (see `SKILL-002`).

**Gate**: Requires explicit activation policy (follows SKILL-002's context/cache ownership
model).  Must not silently load untrusted skill bodies into the prompt.

### AGENT-002-C: `~/.agents/mcp.json` Import (P2)

**What**: Import MCP server definitions from the shared JSON format into `[mcp.servers]`.

**Talos impact**: Changes to `talos-config` (new import DTOs) and `talos-mcp` (server
lifecycle).  MCP servers are started at session startup; imported servers must go through
the same startup validation, timeout, and failure handling as Talos-configured servers.

**Key decisions**:
- Should imported MCP servers auto-start, or require explicit opt-in per server?
- How to handle conflicts when the same server name appears in both
  `~/.agents/mcp.json` and `~/.talos/config.toml`?
- MCP server startup failures from shared config must not crash the session.

**Gate**: Requires server opt-in policy and conflict resolution ADR.

## Decision Gate (all sub-items)

Before any implementation:

- [ ] dotagentsprotocol.com conventions show adoption by 2+ non-trivial agent tools
- [ ] Target schema is versioned and stable for 6 months
- [ ] Per-sub-item ADR written (successor to ADR-022)
- [ ] Import is opt-in (`talos config import --from dotagents` or explicit config flag)
- [ ] Talos-owned config/skills/MCP always take precedence

## Non-goals

- Replacing `~/.agents/talos/config.toml` as Talos's primary shared config path
- Auto-loading skills or MCP servers from shared paths without user opt-in
- Write-back to `~/.agents/` files
- Supporting `~/.agents/agents/` (sub-agents), `~/.agents/tasks/`, or
  `~/.agents/memories/` — not yet relevant to Talos

## Relationship To Other Requirements

| Requirement | Relationship |
|---|---|
| AGENT-001 / I035 | Predecessor — established `~/.agents/talos/config.toml` as Talos namespace |
| ADR-022 | Current decision — `~/.agents/` is read-only, lowest priority |
| SKILL-001 / SKILL-002 | B: shared skills must follow same Level 0/1/2 activation and token budget model |
| MCP-001 | C: imported MCP servers must go through same startup validation and failure handling |
| CONF-001 | May share `talos config import` command infrastructure |
| PROVIDER-001 | A: provider schema must be stable before importing external model config |

## Required Reads

- `docs/iterations/I035-agent-protocol-compatibility-foundation.md` §Survey Findings
- `docs/decisions/022-agent-config-compatibility-boundary.md`
- <https://dotagentsprotocol.com>
- `docs/backlog/active/AGENT-001-standard-agent-protocol-support.md`
- `docs/backlog/active/SKILL-001-runtime-skill-activation.md`
- `docs/backlog/active/SKILL-002-explicit-runtime-activation.md`
- `docs/backlog/active/MCP-001-session-mcp-integration.md`
