# 024: Embeddable Runtime API Boundary

## Status

Accepted

## Context

Talos now has a working in-process agent runtime centered on `talos-agent::Agent` and the
`AppServerSession` seam. The CLI and TUI no longer own the core turn loop, which makes reuse
possible. A new product requirement, `RUNTIME-001`, asks for other Rust projects to embed Talos's
agent runtime while supplying their own UI, commands, persistence, and product features.

The current internal API is usable by Talos itself, but it is not yet a clean dependency surface:

- `talos-agent` exposes low-level construction (`Agent::with_security`) instead of a stable
  runtime facade.
- `talos-agent` depends on product subsystems such as tools, permissions, sandboxing, skills,
  plugins, and memory. That is acceptable for Talos internals but too broad as the only public SDK
  entrypoint.
- `Agent::new` still exists as a deprecated no-permission/no-sandbox constructor for tests.
- `SessionConfig` contains a CLI-shaped `print_mode` field.
- `SessionEvent::AgentEvent` currently cannot round-trip through serde because nested tagged enums
  both use `type`.
- `/mock-request` is implemented as a magic user-message prefix inside the core turn loop.

A decision is needed before adding facade code because the first public embedding API will become
semver-bound and will shape later `REMOTE-001`, `WEB-001`, and `PLUGIN-001` work.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Other Rust projects need to depend on Talos runtime without inheriting Talos CLI/TUI behavior. | Hard | User request / RUNTIME-001 | No |
| All write-capable tools must go through the permission pipeline. | Hard | AGENTS.md Hard Constraint #4 | No |
| No speculative features beyond current scope. | Hard | AGENTS.md Hard Constraint #7 | No |
| Crate public APIs are semver-bound. | Hard | AGENTS.md Hard Constraint #6 | No |
| `talos-core` remains dependency-minimal and owns protocol types. | Hard | AGENTS.md Rust crate rules | No |
| `talos-agent` owns the turn loop. | Soft | ARCHITECTURE.md / existing implementation | No unless a second turn-loop owner appears |
| `AppServerSession` SQ/EQ seam is the canonical UI/core boundary. | Soft | ADR-005 / ADR-006 | No |
| Runtime facade should be easy to use safely. | Soft | RUNTIME-001 | Yes, but unsafe defaults are rejected |

## Reasoning

There are two viable placement choices:

1. Put the facade under `talos-agent::runtime`.
2. Create a new `talos-runtime` crate that depends on `talos-agent` and exposes the embedding API.

Placing the facade inside `talos-agent` is smaller in the short term, but it expands the public
surface of the turn-loop crate and makes it harder to distinguish "core internals Talos may change"
from "SDK surface external projects may depend on." It also encourages embedders to reach around
the facade and call lower-level constructors directly.

A new `talos-runtime` crate is a real abstraction, but it is justified by the user-visible need:
other projects need a stable, safe dependency boundary. The facade crate can intentionally re-export
only a curated set of types and leave `talos-agent` as the implementation crate for the turn loop.
This keeps the turn-loop internals available to Talos without promising every `talos-agent` public
method as SDK surface.

`talos-core` should continue to own protocol and trait types. The runtime crate should not invent
parallel `Message`, `ToolCall`, `AgentEvent`, or provider/tool traits. It can define SDK-friendly
command/event wrappers only where the existing session protocol is not safe to expose yet, such as
the nested tagged event serialization problem.

The first stable target should be in-process Rust embedding, not network transport. Remote control
is still `REMOTE-001`; mirroring the future remote protocol too early would either freeze a weak
wire contract or create speculative complexity. The runtime API should, however, avoid choices that
make remote wrapping difficult later: events must be serializable, bounded, and product-neutral.

Safety defaults matter more than minimal constructor friction. The SDK happy path must make
permission-aware execution the documented default. Test-only or deliberately unsafe construction
can exist behind explicit names, but it must not be the primary runtime builder.

## Decision

1. **Create a dedicated `talos-runtime` facade crate for RUNTIME-001 implementation.**
   - `talos-runtime` will be the public embeddable SDK entrypoint.
   - It may depend on `talos-core`, `talos-agent`, `talos-permission`, `talos-sandbox`,
     `talos-skill`, `talos-plugin`, and optional built-in tool/provider crates as needed.
   - It must not depend on `talos-cli` or `talos-tui`.

2. **Keep `talos-agent` as the turn-loop implementation crate, not the SDK facade.**
   - `Agent`, `AppServerSession`, and lower-level configuration methods remain implementation
     surfaces.
   - Future docs must distinguish supported SDK surface from internal/transitional surfaces.

3. **Keep protocol and trait foundations in `talos-core`.**
   - `Message`, `AgentEvent`, `ToolCall`, `LanguageModel`, `AgentTool`, `ToolRegistry`, and
     provider/tool definitions remain canonical.
   - Runtime-specific wrappers may be added only to repair or stabilize SDK-facing command/event
     shape.

4. **Define the first facade around safe runtime construction and a handle.**
   - `RuntimeBuilder` composes provider, tools, workspace root, permission policy, sandbox policy,
     prompt configuration, skills, memory, and hooks without forcing callers through every
     low-level `Agent` mutator.
   - `RuntimeHandle` submits turns, streams events, interrupts active turns, and shuts down.
   - The happy path defaults to permission-aware behavior; no-permission execution must be
     explicitly named as test-only or unsafe-by-policy.

5. **Do not expose CLI/TUI/product command behavior through the core runtime.**
   - `/mock-request` must move out of magic user-message handling before the SDK is considered
     stable.
   - Request preview becomes an explicit diagnostic method or caller-owned command.
   - `SessionConfig::print_mode` is replaced or wrapped by product-neutral runtime policy.

6. **Do not make the first SDK API a remote/wire protocol.**
   - RUNTIME-001 v1 is in-process Rust.
   - Events and commands must still be serializable so `REMOTE-001` and `WEB-001` can wrap the same
     semantics later without redesigning the runtime.

7. **Fix SDK-blocking protocol defects before declaring the facade stable.**
   - `SessionEvent::AgentEvent` nested tagged enum serialization must round-trip or be hidden
     behind a stable runtime event wrapper.
   - Runtime policy and event types must avoid CLI-only names and host-path leakage.

## Rejected Alternatives

- **Expose `talos-agent` directly as the SDK.** Rejected because it makes all current public
  constructors and mutators look semver-supported and leaves safe construction too easy to bypass.
- **Put `RuntimeBuilder` in `talos-agent::runtime`.** Rejected for now because it blurs facade and
  implementation ownership. Revisit only if the new crate creates measurable dependency or API
  churn without reducing coupling.
- **Use `talos-rpc` as the SDK boundary.** Rejected because the first target is in-process Rust
  embedding, not transport.
- **Add a global event bus for embedder observation.** Rejected by ADR-006; runtime observation
  must remain typed, explicit, and auditable.
- **Auto-enable built-in tools without a permission policy.** Rejected by the permission hard
  constraint.

## Implementation Guardrails

- First implementation slice creates the crate/facade and one embedder proof; it must not refactor
  CLI/TUI behavior opportunistically.
- Any public type exported by `talos-runtime` must be considered SDK surface unless explicitly
  marked experimental.
- `talos-runtime` must not introduce a runtime/native dependency merely to create the facade.
- Existing CLI/TUI run paths should adapt to the facade only after the facade is proven with a
  minimal non-CLI example or integration test.
- README and architecture docs must describe the distinction between Talos product crates and
  embeddable runtime API once code lands.

## Reversal Trigger

Revisit this decision if:

- The facade crate becomes only a thin re-export with no meaningful safety or boundary value.
- A second turn-loop implementation appears and requires moving more runtime logic out of
  `talos-agent`.
- The first two external embedding consumers need incompatible runtime APIs that the facade cannot
  express without leaking internals.
- Remote/web control becomes the primary embedding surface before any in-process consumer exists.

## Related

- [RUNTIME-001: Embeddable Agent Runtime API](../backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md)
- [ADR-005: Canonical TUI Event Architecture](005-tui-event-architecture.md)
- [ADR-006: Event Architecture Boundary](006-event-architecture-boundary.md)
- [ADR-009: Tool Provenance Tracking](009-tool-provenance.md)
- [ADR-021: Tool Call Protocol Architecture](021-tool-call-protocol-architecture.md)
- [REMOTE-001: Remote Session Protocol](../backlog/active/REMOTE-001-remote-session-protocol.md)
- [PLUGIN-001: WASM Runtime Plugin Protocol](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md)
