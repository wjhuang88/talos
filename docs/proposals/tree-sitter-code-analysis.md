# Tree-sitter Code Analysis Proposal

> Status: Research proposal
> Created: 2026-06-12
> Backlog: CODE-001

## Context

The requested capability is to introduce `tree-sitter` for code analysis, but the concrete behavior
must be researched before implementation. Upstream describes tree-sitter as a parser generator and
incremental parsing library that builds concrete syntax trees, updates them efficiently as source
changes, and remains robust in the presence of syntax errors.

For Talos, the important question is not whether tree-sitter is useful; it is where the dependency
belongs and what narrow user-facing behavior justifies it.

## Constraints

- Talos is Rust-first. New C/C++ runtime or generated parser dependencies need explicit decision
  coverage before implementation.
- Code analysis must be read-only until a separate permission-gated refactoring story is accepted.
- Workspace file reads must stay bounded to the workspace root.
- Parsing must not block an agent turn indefinitely; large inputs need limits and clear fallback.
- Unsupported language or parse failure must degrade to text-only behavior.

## Research Plan

1. Inspect the current Rust binding and language grammar crates.
2. Prototype only outside mainline code if needed, using a temporary branch or scratch spike.
3. Compare three first-slice candidates:
   - TUI code block syntax highlighting.
   - AST-aware symbol extraction for workspace files.
   - Shared parser service crate used by both UI and tools.
4. Decide whether a new ADR is required for tree-sitter's C runtime/generated grammar code.
5. Write the follow-up implementation story only after the dependency and first-slice choice are
   explicit.

## Expected Output

- Recommended dependency set and version pinning approach.
- Supported-language list for the first slice.
- Parser API boundary proposal.
- Test plan covering unsupported language, syntax error, large input, timeout, and workspace path
  constraints.
- ADR recommendation.

## Open Questions

- Should Talos prefer parser-backed syntax highlighting first because it is low risk, or tool-facing
  symbol extraction first because it creates more agent value?
- Should grammar crates be optional features to control binary size?
- Should parser cache live only inside a request, inside a session, or in a future workspace index?
