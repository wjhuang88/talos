# ADR-004: Production-Grade Event Loop Architecture

## Status

Accepted

## Context

I005 的交互式模式经历了多次 Ctrl+C 和退出问题的修复，暴露出事件循环设计的根本缺陷：
- stdin 阻塞导致无法干净退出
- 多个异步任务生命周期管理混乱
- 缺乏统一的状态机
- 取消传播不明确

需要一个生产级的事件循环架构，能够优雅处理：
1. 用户输入（stdin）
2. Agent 执行（异步 LLM 调用）
3. 工具调用（可能有多个并发）
4. 流式输出（实时渲染）
5. 信号处理（Ctrl+C, SIGTERM）
6. 优雅关闭（按顺序清理资源）

## Decision

### 核心原则

1. **单一事件通道**：所有事件通过一个 `mpsc::unbounded` 通道流入主循环
2. **显式状态机**：应用状态用 enum 表示，所有转换显式定义
3. **分层取消**：CancellationToken 树形结构，父取消自动传播到子
4. **事件生产与消费分离**：事件源只负责发送，主循环只负责处理
5. **渲染与逻辑分离**：状态更新后统一调用 render 函数

### 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                    Event Sources                         │
├─────────────────────────────────────────────────────────┤
│ stdin thread  │ agent task  │ tool tasks  │ signals     │
└───────┬───────┴──────┬──────┴──────┬──────┴──────┬──────┘
        │              │              │              │
        └──────────────┴──────────────┴──────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   Event Channel     │
                    │  (mpsc::unbounded)  │
                    └──────────┬──────────┘
                               │
                    ┌──────────▼──────────┐
                    │    Main Loop        │
                    │  - Update state     │
                    │  - Dispatch actions │
                    │  - Render UI        │
                    └──────────┬──────────┘
                               │
                    ┌──────────▼──────────┐
                    │   State Machine     │
                    │  - Idle             │
                    │  - WaitingForInput  │
                    │  - AgentRunning     │
                    │  - ToolExecuting    │
                    │  - ShuttingDown     │
                    └─────────────────────┘
```

### 事件类型

```rust
enum AppEvent {
    // User input
    UserInput(String),
    UserInterrupt,  // Ctrl+C
    
    // Agent lifecycle
    AgentStarted,
    AgentTextDelta(String),
    AgentToolCall(ToolCall),
    AgentToolResult(ToolResult),
    AgentCompleted,
    AgentError(String),
    
    // Tool execution
    ToolStarted(String),
    ToolCompleted(String, ToolResult),
    
    // System
    ShutdownRequested,
    ShutdownComplete,
}
```

### 状态机

```rust
enum AppState {
    Idle,
    WaitingForInput,
    AgentRunning {
        cancel_token: CancellationToken,
        task_handle: JoinHandle<()>,
    },
    ToolExecuting {
        agent_cancel: CancellationToken,
        tool_cancels: Vec<CancellationToken>,
        tool_handles: Vec<JoinHandle<()>>,
    },
    ShuttingDown {
        shutdown_token: CancellationToken,
    },
}
```

### 取消层次

```
app_cancel (root)
  └── session_cancel
       └── turn_cancel
            ├── agent_cancel
            └── tool_cancel_1
            └── tool_cancel_2
            └── tool_cancel_N
```

取消传播：
- Ctrl+C 第一次：取消当前 turn（turn_cancel）
- Ctrl+C 第二次：取消整个 session（session_cancel）
- SIGTERM：取消整个 app（app_cancel）

### 主循环伪代码

```rust
async fn main_loop(mut event_rx: mpsc::Receiver<AppEvent>) {
    let mut state = AppState::Idle;
    
    loop {
        render(&state);
        
        let event = match event_rx.recv().await {
            Some(e) => e,
            None => break,  // channel closed
        };
        
        state = match (state, event) {
            // Idle transitions
            (AppState::Idle, AppEvent::UserInput(input)) => {
                start_agent_turn(input)
            }
            
            // WaitingForInput transitions
            (AppState::WaitingForInput, AppEvent::UserInput(input)) => {
                start_agent_turn(input)
            }
            (AppState::WaitingForInput, AppEvent::UserInterrupt) => {
                // First Ctrl+C: just show hint
                show_hint("Press Ctrl+C again to exit");
                AppState::WaitingForInput
            }
            
            // AgentRunning transitions
            (AppState::AgentRunning { cancel_token, task_handle }, AppEvent::AgentTextDelta(text)) => {
                render_text(&text);
                AppState::AgentRunning { cancel_token, task_handle }
            }
            (AppState::AgentRunning { cancel_token, task_handle }, AppEvent::AgentToolCall(call)) => {
                let (tool_state, handles) = execute_tools(call);
                AppState::ToolExecuting {
                    agent_cancel: cancel_token,
                    tool_cancels: vec![],
                    tool_handles: handles,
                }
            }
            (AppState::AgentRunning { cancel_token, task_handle }, AppEvent::UserInterrupt) => {
                cancel_token.cancel();
                task_handle.abort();
                show_message("Turn cancelled");
                AppState::WaitingForInput
            }
            (AppState::AgentRunning { .. }, AppEvent::AgentCompleted) => {
                AppState::WaitingForInput
            }
            
            // ToolExecuting transitions
            (AppState::ToolExecuting { .. }, AppEvent::ToolCompleted(name, result)) => {
                // Check if all tools done
                if all_tools_complete() {
                    AppState::AgentRunning { ... }  // resume agent
                } else {
                    state  // still executing
                }
            }
            
            // Shutdown transitions
            (state, AppEvent::ShutdownRequested) => {
                cancel_all();
                AppState::ShuttingDown { shutdown_token: CancellationToken::new() }
            }
            (AppState::ShuttingDown { .. }, AppEvent::ShutdownComplete) => {
                break;  // exit loop
            }
            
            // Default: ignore unexpected events
            (state, _) => state,
        };
    }
    
    cleanup();
}
```

### stdin 处理

使用 `std::thread::spawn`（不是 tokio::spawn）读取 stdin，通过 `mpsc::Sender::blocking_send` 发送事件：

```rust
fn spawn_stdin_reader(event_tx: mpsc::Sender<AppEvent>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            match stdin.read_line(&mut line) {
                Ok(0) => break,  // EOF
                Ok(_) => {
                    let trimmed = line.trim().to_string();
                    if event_tx.blocking_send(AppEvent::UserInput(trimmed)).is_err() {
                        break;  // channel closed
                    }
                }
                Err(_) => break,
            }
        }
    });
}
```

**为什么用 std::thread 而不是 tokio::spawn：**
- stdin 读取是阻塞操作，会一直等待用户输入
- tokio runtime 会等待所有 spawned task 完成才退出
- std::thread 不被 tokio 追踪，进程退出时自动终止
- 这是 tokio 官方推荐的处理阻塞 I/O 的方式

### 信号处理

```rust
fn spawn_signal_handler(event_tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c().await.ok();
            if event_tx.send(AppEvent::UserInterrupt).await.is_err() {
                break;
            }
        }
    });
}
```

### 关闭序列

```rust
async fn shutdown(state: AppState) {
    match state {
        AppState::ToolExecuting { tool_cancels, tool_handles, .. } => {
            // 1. Cancel all tools
            for cancel in tool_cancels {
                cancel.cancel();
            }
            // 2. Wait for tools to finish (with timeout)
            for handle in tool_handles {
                tokio::time::timeout(Duration::from_secs(2), handle).await.ok();
            }
        }
        AppState::AgentRunning { cancel_token, task_handle } => {
            // 1. Cancel agent
            cancel_token.cancel();
            // 2. Wait for agent to finish (with timeout)
            tokio::time::timeout(Duration::from_secs(2), task_handle).await.ok();
        }
        _ => {}
    }
    
    // 3. Close event channel
    drop(event_tx);
    
    // 4. tokio runtime 自动关闭（没有活跃的 tokio task）
    // 5. 进程退出，OS 清理 std::thread
}
```

### 渲染策略

每次状态转换后调用 `render(&state)`：
- 清除当前行
- 根据状态渲染提示符/进度/输出
- flush stdout

```rust
fn render(state: &AppState) {
    print!("\r\x1b[K");  // clear line
    match state {
        AppState::Idle | AppState::WaitingForInput => {
            print!("> ");
        }
        AppState::AgentRunning { .. } => {
            // 流式文本已经在 AgentTextDelta 事件中实时渲染
        }
        AppState::ToolExecuting { .. } => {
            print!("[executing tools...]");
        }
        AppState::ShuttingDown { .. } => {
            print!("[shutting down...]");
        }
    }
    io::stdout().flush().ok();
}
```

## Consequences

### Positive

1. **可预测性**：所有状态转换显式定义，易于推理
2. **可测试性**：可以模拟事件序列测试状态机
3. **可维护性**：新增功能只需添加新事件和状态转换
4. **优雅关闭**：分层取消确保资源按顺序清理
5. **无阻塞问题**：stdin 用 std::thread，不阻塞 tokio runtime

### Negative

1. **复杂度增加**：比简单的 select! 循环更复杂
2. **样板代码**：需要定义事件类型、状态枚举、转换逻辑
3. **学习曲线**：新贡献者需要理解状态机设计

### Neutral

1. **性能**：unbounded channel 可能有内存压力，但对 CLI 应用不是问题
2. **调试**：状态机转换需要良好的日志记录

## Related

- ADR-003: TUI 渐进式演进策略
- I005: Smart Agent（当前迭代）
- I010: Polished Agent（将实现完整 TUI）

## Implementation Plan

1. **I005-S7**: 实现基础事件循环架构（本 ADR）
   - 定义 AppEvent 和 AppState
   - 实现主循环
   - 迁移现有功能到新架构
   
2. **I006-I009**: 在新架构上增量添加功能
   - 每个新功能 = 新事件类型 + 新状态转换
   
3. **I010**: TUI 集成
   - 替换 render 函数为 ratatui 渲染
   - 事件循环保持不变

## References

- [Tokio 官方文档：处理阻塞操作](https://tokio.rs/tokio/tutorial/bridging)
- [Elm Architecture](https://guide.elm-lang.org/architecture/)（本设计的灵感来源）
- [Redux 模式](https://redux.js.org/introduction/core-concepts)（类似的事件驱动状态管理）
