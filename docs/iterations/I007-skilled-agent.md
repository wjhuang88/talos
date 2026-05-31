# Iteration I007: Skilled Agent

## Scope

Skills system with TUI sidebar, SKILL.md parsing, progressive disclosure, OpenAI provider, and system prompt assembly.

## Selected Stories

- [ ] #I007-S1: TUI skill index sidebar
- [ ] #I007-S2: SKILL.md parser and loader
- [ ] #I007-S3: Progressive disclosure (3 levels)
- [ ] #I007-S4: OpenAI provider
- [ ] #I007-S5: System prompt assembly

## Execution Plan

1. S1 (TUI sidebar) + S2 (SKILL.md parser) + S4 (OpenAI provider) — parallel, no dependencies
2. S3 (Progressive disclosure) + S5 (System prompt assembly) — parallel, depend on S2

## Acceptance Criteria

- [ ] TUI sidebar shows loaded skills
- [ ] SKILL.md files discovered and parsed from 3 locations
- [ ] Progressive disclosure: 3 levels of skill loading
- [ ] OpenAI provider works with streaming and tool calls
- [ ] System prompt assembled from 5 sources with caching optimization
- [ ] `cargo test --workspace` exits 0
- [ ] `cargo clippy --workspace` has no warnings

## Execution Results

(To be filled after completion)
