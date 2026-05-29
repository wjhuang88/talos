# Iteration I001: Project Scaffold

## Scope

建立 Talos 项目的 Cargo workspace 结构和核心类型系统。本迭代不包含任何业务逻辑，
只产出可编译的项目骨架和共享的类型定义。

## Selected Stories

- [x] #I001-S1: Initialize Cargo workspace
- [x] #I001-S2: Core message types and event protocol

## Acceptance Criteria

- [x] `cargo check --workspace` exits 0
- [x] `cargo build -p talos-cli` produces a binary
- [x] Binary runs and prints version/help text
- [x] Workspace uses Rust edition 2024
- [x] All message types compile and are importable from other crates
- [x] `serde` round-trip test passes: `Message` -> JSON -> `Message`
- [x] No circular dependencies: `talos-core` depends on nothing
- [x] Doc comments on all public types

## Risks

- **Edition 2024 稳定性**: Rust edition 2024 较新，某些工具链版本可能不完全支持。Mitigation: 使用最新 stable Rust。
- **类型设计过度**: 容易在第一次设计时就加入过多字段。Mitigation: 只定义当前已知的最小类型集。

## Execution Results

### I001-S1: Initialize Cargo workspace
- Cargo workspace with 5 crates: talos-core, talos-config, talos-provider, talos-agent, talos-cli
- Edition 2024, rust-version 1.85
- `talos` binary with clap-based --version and --help
- `cargo check --workspace` ✅, `cargo build -p talos-cli` ✅

### I001-S2: Core message types and event protocol
- `Message` enum: User, Assistant, Tool variants with serde tag-based serialization
- `AgentEvent` enum: TurnStart, TextDelta, ToolCall, ToolResult, TurnEnd, Error
- `StopReason` enum: EndTurn, ToolUse, MaxTokens
- `Usage` struct: input/output/cache_read/cache_write tokens
- `ToolCall` and `ToolResult` structs
- 4 serde round-trip tests passing
- `cargo test --workspace` ✅ (4 passed), `cargo clippy --workspace` ✅ (0 warnings)

### Retrospective
- Edition 2024 works smoothly with Rust 1.95
- Clap integration for CLI is straightforward
- Serde tagged enums provide clean JSON serialization for the message protocol
