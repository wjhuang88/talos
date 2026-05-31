# Iteration I007: Skilled Agent

## Scope

Skills system with TUI sidebar, SKILL.md parsing, progressive disclosure, OpenAI provider, and system prompt assembly.

## Selected Stories

- [x] #I007-S1: TUI skill index sidebar
- [x] #I007-S2: SKILL.md parser and loader
- [x] #I007-S3: Progressive disclosure (3 levels)
- [x] #I007-S4: OpenAI provider
- [x] #I007-S5: System prompt assembly

## Execution Plan

1. S1 (TUI sidebar) + S2 (SKILL.md parser) + S4 (OpenAI provider) — parallel, no dependencies
2. S3 (Progressive disclosure) + S5 (System prompt assembly) — parallel, depend on S2

## Acceptance Criteria

- [x] TUI sidebar shows loaded skills
- [x] SKILL.md files discovered and parsed from 3 locations
- [x] Progressive disclosure: 3 levels of skill loading
- [x] OpenAI provider works with streaming and tool calls
- [x] System prompt assembled from 5 sources with caching optimization
- [x] `cargo test --workspace` exits 0
- [x] `cargo clippy --workspace` has no warnings

## Execution Results

### I007-S1: TUI skill index sidebar
- `SkillInfo` struct with name, description, active status
- `SkillSidebar` widget with toggle visibility, collapsed mode
- Ctrl+K keybinding to toggle sidebar
- 10 unit tests for sidebar rendering and state management

### I007-S2: SKILL.md parser and loader
- `talos-skill` crate with YAML frontmatter + Markdown body parsing
- `SkillLoader` discovers skills from 3 locations: project, user home, parent dirs
- `Skill` struct with name, description, triggers, body
- 25 unit tests for parsing, discovery, and error handling

### I007-S3: Progressive disclosure (3 levels)
- `SkillManager` with Level 0 (index), Level 1 (full body), Level 2 (reference files)
- Token estimation for skill index (<3000 tokens for 20 skills)
- Skill matching based on trigger patterns
- 22 unit tests for disclosure levels and token estimation

### I007-S4: OpenAI provider
- `OpenAIProvider` implementing `LanguageModel` trait
- SSE streaming from `/chat/completions` endpoint
- Tool call format conversion (OpenAI → internal)
- Error handling: 401, 429, 5xx with retry logic
- 38 unit tests for streaming, tool calls, and error handling

### I007-S5: System prompt assembly
- `SystemPromptBuilder` with 5 sources: identity, tools, skills, context, preferences
- Optimal ordering for prompt caching (stable sections first)
- Cache marker generation for Anthropic API
- Custom prompt and append prompt support
- 21 unit tests for prompt assembly and caching

### Summary
- **Total tests**: 403 (up from 352 in I006, +51)
- **New crates**: `talos-skill`
- **New modules**: `prompt.rs` in `talos-agent`
- **Key achievement**: Skills system with progressive disclosure reduces system prompt size while maintaining agent awareness of available skills
