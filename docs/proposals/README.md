# Proposals

## Purpose

Capture ideas and directions that are not yet ready for implementation. Proposals are NOT tasks —
Agents must not start coding from a proposal without going through requirement intake.

## Naming Convention

```
docs/proposals/
├── README.md           (this file)
├── <slug>.md           (proposal document)
└── ...
```

## Template

```markdown
# [Proposal Title]

## Problem
[What problem does this solve?]

## Proposed Approach
[High-level description]

## Alternatives Considered
[What else could work?]

## Open Questions
[What needs validation before this becomes a story?]

## Dependencies
[What must exist first?]
```

## Rules

- Proposals are explicitly NOT committed work.
- To move a proposal to implementation, follow `docs/sop/REQUIREMENT-INTAKE.md`.
- Update the proposal status when it becomes a backlog story.

## Current Proposals

- [Built-in Workspace Search Tools](builtin-workspace-search-tools.md) — detailed design for
  I012 `find_files` / `grep` tools and fff-inspired ranking/indexing follow-ups.
- [Provider Plugin Architecture](provider-plugin-architecture.md) — long-term provider config
  and plugin direction behind I011-S2.
- [Reasoning / Thinking Field](reasoning-thinking-field.md) — provider-specific reasoning field
  handling.
- [Session Context Contamination](session-context-contamination.md) — context contamination
  investigation and mitigation direction.
- [Standard Agent Protocol Support](standard-agent-protocol-support.md) — medium-term
  compatibility direction for common Agent protocol/config conventions such as shared `~/.agent`
  configuration.
- [TUI Stream Markdown Rendering](tui-stream-markdown-rendering.md) — single-line and block
  Markdown recognition/rendering direction for the inline TUI stream renderer.
- [Tree-sitter Code Analysis](tree-sitter-code-analysis.md) — research proposal for adding
  parser-backed code analysis after dependency and ADR review.
- [Talos Crate Distribution Architecture](talos-crate-distribution-architecture.md) — proposal for
  making Talos-owned capabilities independently publishable as crates while keeping
  `talos-runtime` as the SDK facade.
- [Unified Event Stream](unified-event-stream.md) — event stream proposal retained as reference.
- [Remote Session Protocol](remote-session-protocol.md) — far-future research proposal for remote
  session query and command protocol (mobile app, web dashboard, cross-device continuity).
- [WASM Runtime Plugin Protocol](wasm-runtime-plugin-protocol.md) — long-term protocol-first
  design for WASM plugins that can provide tools, commands, hooks, filters, and future capabilities.
- [Plugin Encapsulation Format](plugin-encapsulation-format.md) — **DRAFT 2026-06-30, awaiting
  decision.** Four-entity model: skill/mcp/hook are config-introduced atomic components; plugin is a
  packaging format bundling any subset of them plus tools, carried by WASM/Lua/dylib. Blocks
  PLUGIN-001, CMD-002, HOOK-001, TOOL-008 Phase 3.
