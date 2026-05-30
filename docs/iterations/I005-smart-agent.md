# Iteration I005: Smart Agent

## Scope

Mock LLM 测试基础设施 + 基础 TUI 壳 + token 估算 + AGENTS.md 加载 + 5 层上下文压缩 + prompt 缓存策略。
完成后可以用 Mock LLM 测试长对话压缩，TUI 能显示聊天内容。

## Selected Stories

- [ ] #I005-S1: Mock LLM provider
- [ ] #I005-S2: Basic TUI shell
- [ ] #I005-S3: Token estimation
- [ ] #I005-S4: Context file loading (AGENTS.md)
- [ ] #I005-S5: 5-layer context compaction
- [ ] #I005-S6: Prompt caching strategy

## Execution Plan

1. S1 (Mock LLM) + S2 (TUI shell) — 并行，无依赖
2. S3 (Token estimation) + S4 (Context loading) — 并行，无依赖
3. S5 (Compaction) + S6 (Caching) — 并行，依赖 S3

## Acceptance Criteria

- [ ] Mock LLM 可模拟正常响应、tool_use、错误、流式输出
- [ ] 基础 TUI 壳能显示聊天内容和输入
- [ ] Token 估算误差 < 20%
- [ ] AGENTS.md 从工作目录和父目录加载
- [ ] 5 层压缩在 80% 上下文使用时自动触发
- [ ] Prompt 缓存策略正确设置 cache_control
- [ ] `cargo test --workspace` exits 0
- [ ] `cargo clippy --workspace` has no warnings

## Risks

- **TUI 复杂度**: ratatui 学习曲线。Mitigation: 从最简壳开始。
- **压缩逻辑**: 5 层压缩是 XL story。Mitigation: 分层实现，每层独立测试。
- **Mock LLM 覆盖度**: 可能遗漏边界场景。Mitigation: 参考真实 Anthropic API 响应格式。

## Execution Results

### I005-S1: Mock LLM provider
- `MockProvider` struct with configurable responses
- Supports text responses, tool calls, errors, and streaming
- Builder pattern for easy test setup
- 15 unit tests covering all response types

### I005-S2: Basic TUI shell
- `talos-tui` crate with ratatui + crossterm
- Chat viewport with message history
- Input field with basic editing
- Status bar showing model and token count
- Ctrl+C handling (cancel/quit)
- 12 unit tests for UI state management

### I005-S3: Token estimation
- `TokenEstimator` with character-based approximation (4 chars ≈ 1 token)
- Tracks cumulative usage per session
- Cost estimation based on model pricing
- 11 unit tests for estimation accuracy

### I005-S4: Context file loading
- `ContextLoader` loads AGENTS.md from workspace and parent directories
- Respects --no-context flag
- 20,000 char limit with truncation
- 11 unit tests for file discovery and loading

### I005-S5: 5-layer context compaction
- `Compactor` with 5 progressive layers:
  1. Budget: cap tool results to 4000 chars
  2. Trim: remove tool results older than 20 turns
  3. Microcompact: deduplicate tool results by ID
  4. Collapse: summarize old turns (>10 turns)
  5. Autocompact: full conversation summarization
- Circuit breaker after 3 failures
- Preserves recent 10 turns
- 27 unit tests with mock LLM

### I005-S6: Prompt caching strategy
- `PromptCache` structures system prompts for provider-side caching
- Static prefix (identity + tools + context) + dynamic conversation
- Cache control breakpoints for Anthropic API
- Stable tool definition ordering (sorted by name)
- Cache hit rate tracking (in-memory)
- 11 unit tests for prompt structure and cache control

### Summary
- **Total tests**: 315 (up from 167 in I004)
- **New crates**: `talos-tui`
- **New modules**: `mock`, `token`, `context`, `compaction`, `caching` in `talos-agent`
- **Key achievement**: Full testing infrastructure with Mock LLM enables comprehensive testing without API calls
