# I035: Agent Protocol Compatibility Foundation

**Status**: Planned
**Target Window**: After architecture cleanup and runtime Skill/MCP activation
**Depends On**: I030-I034 complete preferred; may proceed as research-only if implementation
dependencies slip

## Outcome

Turn AGENT-001 into a concrete compatibility plan for common Agent protocol/config conventions,
including shared configuration locations such as `~/.agent`, without coupling Talos core runtime
types to unstable external schemas.

## Selected Stories

- [ ] #AGENT-001-A: Survey common Agent protocol/config conventions and record source dates
- [ ] #AGENT-001-B: Write an ADR for supported config/protocol compatibility boundaries
- [ ] #AGENT-001-C: Define Talos-owned DTOs for shared Agent config import
- [ ] #AGENT-001-D: Specify config precedence across CLI flags, env vars, workspace config,
      `~/.talos`, and shared Agent config
- [ ] #AGENT-001-E: Prototype read-only import from `~/.agent` if the survey confirms a stable
      layout
- [ ] #AGENT-001-F: Update user-facing docs for supported interoperability behavior

## Acceptance Criteria

- [ ] The survey distinguishes confirmed facts, assumptions, and unstable conventions.
- [ ] Any external protocol dependency is captured in an ADR before implementation.
- [ ] Talos keeps `~/.talos` as the Talos-owned source of state.
- [ ] Shared config support is read/import-first; no silent write-back.
- [ ] Secrets remain env-var based or explicit-permission gated.
- [ ] Tests cover precedence and non-overwrite behavior if implementation starts.
- [ ] User docs explain what is supported and what is intentionally unsupported.

## Risks

- The Agent protocol ecosystem may change quickly; do not hard-code a convention without dated
  source evidence.
- `~/.agent` must not become a dumping ground for Talos private state.
- Runtime Skill activation, MCP session integration, remote session control, WASM plugins, and
  provider plugins are adjacent but separate work.

## Verification Log

(to be filled as stories land)
