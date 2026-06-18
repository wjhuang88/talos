# Standard Agent Protocol Support

## Problem

Talos is becoming capable enough that it should interoperate with broader Agent ecosystem
conventions instead of only exposing Talos-specific configuration and protocols. Users may expect
standard Agent configuration paths such as `~/.agent`, shared protocol descriptors, or
tool/permission/session metadata that other Agent-aware clients can understand.

The risk is two-sided:

- If Talos ignores common conventions, users must duplicate configuration across tools.
- If Talos adopts unstable conventions too eagerly, Talos core types and config behavior become
coupled to external churn.

## Proposed Approach

Use a compatibility-layer design:

1. **Survey before implementation**: identify currently common Agent protocol/config conventions,
   their owners, stability, and license implications.
2. **Talos-owned source of truth**: keep `~/.talos` for Talos state and Talos-specific config.
3. **Shared config import/read layer**: support `~/.agent` or other common locations through
   explicit import/read adapters.
4. **Clear precedence**: define order across CLI flags, environment variables, workspace config,
   `~/.talos`, and shared Agent config.
5. **No silent writes**: writing to shared config requires an explicit command and approval.
6. **DTO boundary**: external protocol/config shapes convert into Talos-owned DTOs at the edge.
7. **Documentation first**: publish supported surfaces, unsupported surfaces, and migration rules.

## Candidate Capability Areas

- Shared model/provider configuration.
- Shared tool registry or tool manifest metadata.
- Permission policy import/export.
- Session or workspace identity metadata.
- Agent capability descriptors for IDEs, launchers, or orchestrators.

## Alternatives Considered

- **Replace `~/.talos` with `~/.agent`**: rejected. Talos needs a stable owner directory for
  sessions, cache, logs, and implementation-specific state.
- **Write directly to shared config by default**: rejected. Shared config may be used by other
  tools and may contain secrets.
- **Wait for a single official standard**: too conservative. A read/import compatibility layer can
  support stable conventions without making them core dependencies.

## Open Questions

1. Which Agent protocol/config conventions are stable enough to support first?
2. Is `~/.agent` a directory convention, a file convention, or a family of tool-specific layouts?
3. Should Talos support only read/import initially, or also export?
4. How should conflicts be shown when `~/.talos` and shared Agent config disagree?
5. What minimum capability descriptor is useful to external clients without exposing unsafe
   execution authority?

## Dependencies

- Provider schema boundary remains stable enough for config import.
- Permission pipeline remains the only authority for write/execute-capable actions.
- Protocol adapters stay outside core runtime types.
- Any accepted external protocol dependency must be captured in an ADR before implementation.
