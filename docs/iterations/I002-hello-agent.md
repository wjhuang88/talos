# Iteration I002: Hello Agent (MVP)

## Scope

第一个能对话的 Agent。实现配置系统、Anthropic streaming provider、基础 turn loop 和 CLI print 模式。
完成后用户可以运行 `talos "hello" -p` 获得流式 LLM 响应。

## Selected Stories

- [x] #I002-S1: Minimal configuration system
- [x] #I002-S2: Anthropic streaming provider
- [x] #I002-S3: Basic turn loop (no tools)
- [x] #I002-S4: CLI print mode and stdin pipe

## Acceptance Criteria

- [x] `ANTHROPIC_API_KEY=sk-... cargo run -p talos-cli -- "Explain Rust ownership" -p` streams response
- [x] `echo "What is 2+2?" | cargo run -p talos-cli -- -p` returns "4"
- [x] Missing API key prints actionable error (not a panic)
- [x] `cargo test --workspace` exits 0 (32 tests passed)
- [x] `cargo clippy --workspace` has no warnings

## Risks

- **Anthropic API 变更**: SSE 格式可能随 API 版本变化。Mitigation: 锁定 API 版本 header。
- **tokio 复杂度**: SQ/EQ 模式对简单 turn loop 可能过度设计。Mitigation: 用最简实现，不过早抽象。

## Execution Results

### I002-S1: Minimal configuration system
- `Config` struct with `provider`, `model`, `api_key` fields
- `Provider` enum: Anthropic (default), OpenAI
- `Config::load()` reads `~/.talos/config.toml`, performs `${ENV_VAR}` substitution, validates
- `Config::api_key()` checks config → env vars (`ANTHROPIC_API_KEY` / `OPENAI_API_KEY`)
- `ConfigError` with clear messages: MissingApiKey, InvalidConfig, IoError, ParseError
- 12 unit tests passing

### I002-S2: Anthropic streaming provider
- `LanguageModel` trait in `talos-core` with `async fn stream()` returning `Receiver<AgentEvent>`
- `ProviderError` enum: AuthenticationFailed, RateLimited, ServerError, NetworkError, InvalidResponse
- `AnthropicProvider` with reqwest SSE streaming, exponential backoff retry (3 retries on 429/5xx)
- SSE parser handles: message_start, content_block_delta, message_delta, error events
- `cache_control` headers for prompt caching
- 4 integration tests via mockito (streaming, 401, 429, 500)

### I002-S3: Basic turn loop
- `Agent` struct with `Arc<dyn LanguageModel>` provider
- `Agent::run()` collects TextDelta events into response string
- `Agent::run_streaming()` forwards all events to broadcast channel
- `AgentError` enum: ProviderError, Cancelled, UnexpectedEvent
- 6 unit tests with MockModel (success, error, channel close, streaming, tool events ignored)

### I002-S4: CLI print mode and stdin pipe
- Clap CLI: positional prompt, `-p/--print`, `-m/--model`, `--provider`
- Prompt resolution: positional arg → stdin pipe → error
- Streaming output to stdout with flush after each delta
- Clear error messages for missing config/API key (exit 1)
- `std::io::IsTerminal` for stdin pipe detection

### Retrospective
- Subagent delegation worked well for S1-S3 but timed out on S4 (30min limit)
- Manual fixes needed: missing `serde_json` dev-dep, missing `use std::io::Write`, unused variable warning
- Total: 32 tests passing, 0 clippy warnings, ~1470 lines of new code
