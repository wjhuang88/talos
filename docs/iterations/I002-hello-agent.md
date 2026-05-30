# Iteration I002: Hello Agent (MVP)

## Scope

第一个能对话的 Agent。实现配置系统、Anthropic streaming provider、基础 turn loop 和 CLI print 模式。
完成后用户可以运行 `talos "hello" -p` 获得流式 LLM 响应。

## Selected Stories

- [ ] #I002-S1: Minimal configuration system
- [ ] #I002-S2: Anthropic streaming provider
- [ ] #I002-S3: Basic turn loop (no tools)
- [ ] #I002-S4: CLI print mode and stdin pipe

## Acceptance Criteria

- [ ] `ANTHROPIC_API_KEY=sk-... cargo run -p talos-cli -- "Explain Rust ownership" -p` streams response
- [ ] `echo "What is 2+2?" | cargo run -p talos-cli -- -p` returns "4"
- [ ] Missing API key prints actionable error (not a panic)
- [ ] `cargo test --workspace` exits 0
- [ ] `cargo clippy --workspace` has no warnings

## Risks

- **Anthropic API 变更**: SSE 格式可能随 API 版本变化。Mitigation: 锁定 API 版本 header。
- **tokio 复杂度**: SQ/EQ 模式对简单 turn loop 可能过度设计。Mitigation: 用最简实现，不过早抽象。

## Execution Results

(To be filled after completion)
